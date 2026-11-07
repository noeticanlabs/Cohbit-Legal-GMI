use crate::receipt::{canonical_receipt_hash, stable_hash, LegalReceipt, TransitionTrace};
use crate::verifier::VerificationDecision;
use crate::LegalManifold;
use crate::StateHash;
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptLogRecord {
    pub record_type: String,
    pub version: String,
    pub step_index: u64,
    pub receipt_hash: String,
    pub previous_receipt_hash: Option<String>,
    pub pre_state_hash: String,
    pub post_state_hash: String,
    pub trace_hash: Option<String>,
    pub written_at: String,
    pub root_commitment: String,
    pub decision: VerificationDecision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainManifest {
    pub version: String,
    pub chain_id: String,
    pub genesis_state_hash: String,
    pub latest_step_index: u64,
    pub latest_receipt_hash: Option<String>,
    pub latest_state_hash: String,
    pub latest_snapshot_hash: Option<String>,
    pub root_commitment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofLifecycleRecord {
    pub step_index: u64,
    pub receipt_hash: String,
    pub claim_id: String,
    pub dependencies: std::collections::BTreeMap<String, String>,
    pub original_status: crate::proof::ProofStatus,
    pub current_status: crate::proof::ProofStatus,
}

pub struct PersistenceManager {
    pub base_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PendingTransaction {
    record: ReceiptLogRecord,
    trace: crate::receipt::TransitionTrace,
    next_manifest: ChainManifest,
    next_state: Option<LegalManifold>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailurePoint {
    AfterPending,
    AfterReceipt,
    AfterTrace,
    AfterState,
}

impl PersistenceManager {
    /// Recompute current proof usability without changing immutable receipts.
    /// Revocations, changed dependency hashes and expiry yield ReevaluationRequired.
    pub fn assess_proof_lifecycle(
        &self,
        policy: &crate::proof::TrustPolicy,
        now: &str,
    ) -> Result<Vec<ProofLifecycleRecord>> {
        let path = self.base_dir.join("traces.ndjson");
        if !path.exists() {
            return Ok(vec![]);
        }
        let mut records = Vec::new();
        for line in BufReader::new(File::open(path)?).lines() {
            let trace: TransitionTrace = serde_json::from_str(&line?)?;
            // A rejected attempt is historical evidence, never an admitted
            // claim and therefore never part of the current proof index.
            if trace.decision != VerificationDecision::Accept
                || trace.policy_version != crate::proof::PROOF_POLICY
            {
                continue;
            }
            let (Some(artifact), Some(original_status)) = (trace.legal_proof, trace.proof_status)
            else {
                continue;
            };
            let current_status = match artifact.check(policy, now) {
                Ok(checked) => checked.status().clone(),
                Err(_) => match artifact.usability(policy, now) {
                    crate::proof::ProofStatus::Qualified => crate::proof::ProofStatus::Rejected,
                    status => status,
                },
            };
            records.push(ProofLifecycleRecord {
                step_index: trace.step_index,
                receipt_hash: trace.receipt_hash.0,
                claim_id: artifact.context.claim_id,
                dependencies: artifact.context.dependencies,
                original_status,
                current_status,
            });
        }
        Ok(records)
    }

    /// Persist a replaceable current-status index. Historical receipts remain
    /// append-only; this index is always reproducible from them and trust state.
    pub fn refresh_proof_lifecycle(
        &self,
        policy: &crate::proof::TrustPolicy,
        now: &str,
    ) -> Result<Vec<ProofLifecycleRecord>> {
        let records = self.assess_proof_lifecycle(policy, now)?;
        let path = self.base_dir.join("proof-lifecycle.json");
        let temporary = self.base_dir.join("proof-lifecycle.json.tmp");
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temporary)?;
        file.write_all(serde_json::to_vec_pretty(&records)?.as_slice())?;
        file.sync_all()?;
        fs::rename(temporary, path)?;
        Ok(records)
    }

    pub fn new(base_dir: impl AsRef<Path>) -> Result<Self> {
        let path = base_dir.as_ref().to_path_buf();
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        Ok(Self { base_dir: path })
    }

    pub fn initialize_chain(
        &self,
        chain_id: &str,
        initial_state_hash: StateHash,
    ) -> Result<ChainManifest> {
        let manifest_path = self.base_dir.join("manifest.json");
        if manifest_path.exists() {
            return self.load_manifest();
        }

        let manifest = ChainManifest {
            version: "2.0.0".into(),
            chain_id: chain_id.into(),
            genesis_state_hash: initial_state_hash.0.clone(),
            latest_step_index: 0,
            latest_receipt_hash: None,
            latest_state_hash: initial_state_hash.0,
            latest_snapshot_hash: None,
            root_commitment: "genesis".into(),
        };

        self.save_manifest(&manifest)?;
        Ok(manifest)
    }

    pub fn initialize_from_manifold(
        &self,
        chain_id: &str,
        manifold: &LegalManifold,
    ) -> Result<ChainManifest> {
        let mut manifest = self.initialize_chain(chain_id, manifold.get_state_hash())?;
        if manifest.latest_step_index == 0 {
            self.save_state(manifold)?;
            manifest.latest_snapshot_hash = Some(stable_hash(manifold));
            self.save_manifest(&manifest)?;
        }
        Ok(manifest)
    }

    pub fn load_state(&self) -> Result<LegalManifold> {
        let manifest = self.load_manifest()?;
        let path = self.base_dir.join("state.json");
        let data = fs::read_to_string(path).context("Failed to read state snapshot")?;
        let state: LegalManifold =
            serde_json::from_str(&data).context("Failed to parse state snapshot")?;
        if state.get_state_hash().0 != manifest.latest_state_hash
            || state.step_index != manifest.latest_step_index
            || state.last_receipt_hash.as_ref().map(|hash| hash.0.clone())
                != manifest.latest_receipt_hash
            || manifest.latest_snapshot_hash.as_ref() != Some(&stable_hash(&state))
        {
            return Err(anyhow!("State snapshot does not match the chain manifest"));
        }
        Ok(state)
    }

    fn save_state(&self, manifold: &LegalManifold) -> Result<()> {
        let path = self.base_dir.join("state.json");
        let temporary_path = self.base_dir.join("state.json.tmp");
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temporary_path)?;
        file.write_all(serde_json::to_string_pretty(manifold)?.as_bytes())?;
        file.sync_all()?;
        fs::rename(temporary_path, path)?;
        Ok(())
    }

    pub fn load_manifest(&self) -> Result<ChainManifest> {
        let manifest_path = self.base_dir.join("manifest.json");
        let data = fs::read_to_string(manifest_path).context("Failed to read manifest")?;
        serde_json::from_str(&data).context("Failed to parse manifest")
    }

    pub fn save_manifest(&self, manifest: &ChainManifest) -> Result<()> {
        let manifest_path = self.base_dir.join("manifest.json");
        let temporary_path = self.base_dir.join("manifest.json.tmp");
        let data = serde_json::to_string_pretty(manifest)?;
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temporary_path)?;
        file.write_all(data.as_bytes())?;
        file.sync_all()?;
        fs::rename(&temporary_path, &manifest_path).context("Failed to atomically replace manifest")
    }

    pub fn append_receipt(&self, receipt: &LegalReceipt) -> Result<ChainManifest> {
        self.append_internal(receipt, None, None)
    }

    pub fn append_transition(
        &self,
        receipt: &LegalReceipt,
        next_state: &LegalManifold,
    ) -> Result<ChainManifest> {
        self.append_internal(receipt, Some(next_state), None)
    }

    pub fn append_receipt_with_failure(
        &self,
        receipt: &LegalReceipt,
        failure: Option<FailurePoint>,
    ) -> Result<ChainManifest> {
        self.append_internal(receipt, None, failure)
    }

    pub fn append_transition_with_failure(
        &self,
        receipt: &LegalReceipt,
        next_state: &LegalManifold,
        failure: Option<FailurePoint>,
    ) -> Result<ChainManifest> {
        self.append_internal(receipt, Some(next_state), failure)
    }

    fn append_internal(
        &self,
        receipt: &LegalReceipt,
        next_state: Option<&LegalManifold>,
        failure: Option<FailurePoint>,
    ) -> Result<ChainManifest> {
        let _writer_lock = self.acquire_writer_lock()?;
        self.recover_pending_locked()?;
        receipt
            .verify_canonical()
            .map_err(|e| anyhow!("Canonical receipt verification failed: {}", e))?;
        let mut manifest = self.load_manifest()?;

        // Continuity Checks
        if receipt.step_index != manifest.latest_step_index + 1 {
            return Err(anyhow!(
                "Step Index Discontinuity: expected {}, got {}",
                manifest.latest_step_index + 1,
                receipt.step_index
            ));
        }

        if receipt.pre_state_hash.0 != manifest.latest_state_hash {
            return Err(anyhow!(
                "State Hash Discontinuity: expected {}, got {}",
                manifest.latest_state_hash,
                receipt.pre_state_hash.0
            ));
        }

        if let Some(state) = next_state {
            if state.get_state_hash() != receipt.post_state_hash
                || state.step_index != receipt.step_index
                || state.last_receipt_hash.as_ref() != Some(&receipt.receipt_hash)
            {
                return Err(anyhow!(
                    "Resulting state does not correspond to the receipt"
                ));
            }
        }

        let receipt_previous = receipt
            .last_receipt_hash
            .as_ref()
            .map(|hash| hash.0.clone());
        if receipt_previous != manifest.latest_receipt_hash {
            return Err(anyhow!(
                "Receipt Chain Discontinuity: expected {:?}, got {:?}",
                manifest.latest_receipt_hash,
                receipt_previous
            ));
        }

        // Compute new root commitment: root_n = H(root_{n-1}, rho_n)
        let root_input = format!("{},{}", manifest.root_commitment, receipt.receipt_hash.0);
        let new_root = stable_hash(&root_input);

        let trace_hash = receipt.trace.as_ref().map(stable_hash);

        let record = ReceiptLogRecord {
            record_type: "receipt".into(),
            version: receipt.schema_version.clone(),
            step_index: receipt.step_index,
            receipt_hash: receipt.receipt_hash.0.clone(),
            previous_receipt_hash: receipt.last_receipt_hash.as_ref().map(|h| h.0.clone()),
            pre_state_hash: receipt.pre_state_hash.0.clone(),
            post_state_hash: receipt.post_state_hash.0.clone(),
            trace_hash,
            written_at: Utc::now().to_rfc3339(),
            root_commitment: new_root.clone(),
            decision: receipt.decision.clone(),
        };

        manifest.latest_step_index = receipt.step_index;
        manifest.latest_receipt_hash = Some(receipt.receipt_hash.0.clone());
        manifest.latest_state_hash = receipt.post_state_hash.0.clone();
        manifest.latest_snapshot_hash = next_state.map(stable_hash);
        manifest.root_commitment = new_root;

        let pending = PendingTransaction {
            record,
            trace: receipt
                .trace
                .clone()
                .expect("canonical receipt requires trace"),
            next_manifest: manifest.clone(),
            next_state: next_state.cloned(),
        };
        self.write_pending(&pending)?;
        self.finish_pending(&pending, failure)?;
        Ok(manifest)
    }

    pub fn recover(&self) -> Result<()> {
        let _writer_lock = self.acquire_writer_lock()?;
        self.recover_pending_locked()
    }

    fn acquire_writer_lock(&self) -> Result<File> {
        let lock_path = self.base_dir.join("writer.lock");
        let lock_file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(lock_path)?;
        lock_file
            .try_lock()
            .context("Another writer holds the persistence lock")?;
        Ok(lock_file)
    }

    fn pending_path(&self) -> PathBuf {
        self.base_dir.join("pending-transaction.json")
    }

    fn write_pending(&self, pending: &PendingTransaction) -> Result<()> {
        let pending_path = self.pending_path();
        let temporary_path = self.base_dir.join("pending-transaction.json.tmp");
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temporary_path)?;
        file.write_all(serde_json::to_string(pending)?.as_bytes())?;
        file.sync_all()?;
        fs::rename(temporary_path, pending_path)?;
        Ok(())
    }

    fn recover_pending_locked(&self) -> Result<()> {
        let pending_path = self.pending_path();
        if !pending_path.exists() {
            return Ok(());
        }
        let pending: PendingTransaction = serde_json::from_str(&fs::read_to_string(&pending_path)?)
            .context("Failed to parse pending transaction")?;
        self.finish_pending(&pending, None)
    }

    fn finish_pending(
        &self,
        pending: &PendingTransaction,
        failure: Option<FailurePoint>,
    ) -> Result<()> {
        if failure == Some(FailurePoint::AfterPending) {
            return Err(anyhow!("Injected failure after pending transaction"));
        }
        self.append_record_once(&pending.record)?;
        if failure == Some(FailurePoint::AfterReceipt) {
            return Err(anyhow!("Injected failure after receipt append"));
        }
        self.append_trace_once(&pending.trace)?;
        if failure == Some(FailurePoint::AfterTrace) {
            return Err(anyhow!("Injected failure after trace append"));
        }
        if let Some(state) = &pending.next_state {
            self.save_state(state)?;
        }
        if failure == Some(FailurePoint::AfterState) {
            return Err(anyhow!("Injected failure after state snapshot"));
        }
        self.save_manifest(&pending.next_manifest)?;
        let pending_path = self.pending_path();
        if pending_path.exists() {
            fs::remove_file(pending_path)?;
        }
        Ok(())
    }

    fn append_record_once(&self, record: &ReceiptLogRecord) -> Result<()> {
        let path = self.base_dir.join("receipts.ndjson");
        if let Some(last) = Self::last_json_line::<ReceiptLogRecord>(&path)? {
            if last.step_index == record.step_index {
                if stable_hash(&last) == stable_hash(record) {
                    return Ok(());
                }
                return Err(anyhow!(
                    "Conflicting receipt already exists at step {}",
                    record.step_index
                ));
            }
            if last.step_index > record.step_index {
                return Err(anyhow!("Receipt log is ahead of pending transaction"));
            }
        }
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        writeln!(file, "{}", serde_json::to_string(record)?)?;
        file.sync_all()?;
        Ok(())
    }

    fn append_trace_once(&self, trace: &TransitionTrace) -> Result<()> {
        let path = self.base_dir.join("traces.ndjson");
        if let Some(last) = Self::last_json_line::<TransitionTrace>(&path)? {
            if last.step_index == trace.step_index {
                if stable_hash(&last) == stable_hash(trace) {
                    return Ok(());
                }
                return Err(anyhow!(
                    "Conflicting trace already exists at step {}",
                    trace.step_index
                ));
            }
            if last.step_index > trace.step_index {
                return Err(anyhow!("Trace log is ahead of pending transaction"));
            }
        }
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        writeln!(file, "{}", serde_json::to_string(trace)?)?;
        file.sync_all()?;
        Ok(())
    }

    fn last_json_line<T>(path: &Path) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        if !path.exists() {
            return Ok(None);
        }
        let last = BufReader::new(File::open(path)?)
            .lines()
            .collect::<std::io::Result<Vec<_>>>()?
            .into_iter()
            .rev()
            .find(|line| !line.trim().is_empty());
        last.map(|line| serde_json::from_str(&line).map_err(Into::into))
            .transpose()
    }

    pub fn verify_integrity(&self) -> Result<String> {
        let log_path = self.base_dir.join("receipts.ndjson");
        if !log_path.exists() {
            return Ok("genesis".into());
        }

        let trace_log_path = self.base_dir.join("traces.ndjson");
        if !trace_log_path.exists() {
            return Err(anyhow!("Receipt log exists but trace log is missing"));
        }
        let mut trace_reader = BufReader::new(fs::File::open(trace_log_path)?).lines();

        let file = fs::File::open(&log_path)?;
        let reader = BufReader::new(file);

        let mut current_root = "genesis".to_string();
        let mut last_step = 0;
        let mut last_rho: Option<String> = None;
        let mut last_state = String::new();
        let mut last_record: Option<ReceiptLogRecord> = None;

        for (i, line) in reader.lines().enumerate() {
            let line = line?;
            let record: ReceiptLogRecord = serde_json::from_str(&line)
                .with_context(|| format!("Failed to parse record at line {}", i + 1))?;

            // 1. Root Recomputation
            let expected_root_input = format!("{},{}", current_root, record.receipt_hash);
            let recomputed_root = stable_hash(&expected_root_input);
            if recomputed_root != record.root_commitment {
                return Err(anyhow!(
                    "Tamper detected at step {}: root commitment mismatch",
                    record.step_index
                ));
            }

            // 2. Continuity Checks
            if i == 0 {
                if record.step_index != 1 || record.previous_receipt_hash.is_some() {
                    return Err(anyhow!("Invalid genesis successor receipt"));
                }
            } else {
                if record.step_index != last_step + 1 {
                    return Err(anyhow!("Step index gap at step {}", record.step_index));
                }
                if record.previous_receipt_hash != last_rho {
                    return Err(anyhow!(
                        "Receipt chain broken at step {}",
                        record.step_index
                    ));
                }
                if record.pre_state_hash != last_state {
                    return Err(anyhow!(
                        "State continuity broken at step {}",
                        record.step_index
                    ));
                }
            }

            // 3. Full trace and canonical receipt verification
            let trace_hash = record.trace_hash.as_ref().ok_or_else(|| {
                anyhow!("Receipt at step {} has no trace hash", record.step_index)
            })?;
            let trace_line = trace_reader
                .next()
                .ok_or_else(|| anyhow!("Missing trace entry for step {}", record.step_index))??;
            let trace: TransitionTrace = serde_json::from_str(&trace_line)
                .with_context(|| format!("Invalid trace at step {}", record.step_index))?;
            if stable_hash(&trace) != *trace_hash {
                return Err(anyhow!("Trace hash mismatch at step {}", record.step_index));
            }
            let canonical_hash = canonical_receipt_hash(&trace);
            if canonical_hash.0 != record.receipt_hash
                || trace.receipt_hash != canonical_hash
                || trace.step_index != record.step_index
                || trace.decision != record.decision
                || trace.prev_receipt_hash.as_ref().map(|hash| hash.0.clone())
                    != record.previous_receipt_hash
                || trace.pre_state_hash.0 != record.pre_state_hash
                || trace.post_state_hash.0 != record.post_state_hash
            {
                return Err(anyhow!(
                    "Canonical receipt or trace correspondence failed at step {}",
                    record.step_index
                ));
            }

            current_root = record.root_commitment.clone();
            last_step = record.step_index;
            last_rho = Some(record.receipt_hash.clone());
            last_state = record.post_state_hash.clone();
            last_record = Some(record);
        }

        if trace_reader.next().transpose()?.is_some() {
            return Err(anyhow!("Trace log contains unaccounted tail entries"));
        }

        // Verify against manifest
        let manifest = self.load_manifest()?;
        if let Some(first_line) = BufReader::new(fs::File::open(&log_path)?).lines().next() {
            let first: ReceiptLogRecord = serde_json::from_str(&first_line?)?;
            if first.pre_state_hash != manifest.genesis_state_hash {
                return Err(anyhow!(
                    "Genesis state hash does not match the first receipt"
                ));
            }
        }
        if manifest.root_commitment != current_root {
            return Err(anyhow!("Manifest root commitment mismatch with log"));
        }

        if let Some(last) = last_record {
            if manifest.latest_step_index != last.step_index
                || manifest.latest_receipt_hash != Some(last.receipt_hash)
                || manifest.latest_state_hash != last.post_state_hash
            {
                return Err(anyhow!(
                    "Manifest terminal fields do not match last log record"
                ));
            }
        }

        if manifest.latest_snapshot_hash.is_some() {
            self.load_state()?;
        }

        Ok(current_root)
    }
}
