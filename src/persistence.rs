use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{Write, BufRead, BufReader};
use std::path::{Path, PathBuf};
use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use crate::receipt::{LegalReceipt, stable_hash};
use crate::{ReceiptHash, StateHash};

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainManifest {
    pub version: String,
    pub chain_id: String,
    pub latest_step_index: u64,
    pub latest_receipt_hash: Option<String>,
    pub latest_state_hash: String,
    pub root_commitment: String,
}

pub struct PersistenceManager {
    pub base_dir: PathBuf,
}

impl PersistenceManager {
    pub fn new(base_dir: impl AsRef<Path>) -> Result<Self> {
        let path = base_dir.as_ref().to_path_buf();
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        Ok(Self { base_dir: path })
    }

    pub fn initialize_chain(&self, chain_id: &str, initial_state_hash: StateHash) -> Result<ChainManifest> {
        let manifest_path = self.base_dir.join("manifest.json");
        if manifest_path.exists() {
            return self.load_manifest();
        }

        let manifest = ChainManifest {
            version: "1.0.0".into(),
            chain_id: chain_id.into(),
            latest_step_index: 0,
            latest_receipt_hash: None,
            latest_state_hash: initial_state_hash.0,
            root_commitment: "genesis".into(),
        };

        self.save_manifest(&manifest)?;
        Ok(manifest)
    }

    pub fn load_manifest(&self) -> Result<ChainManifest> {
        let manifest_path = self.base_dir.join("manifest.json");
        let data = fs::read_to_string(manifest_path).context("Failed to read manifest")?;
        serde_json::from_str(&data).context("Failed to parse manifest")
    }

    pub fn save_manifest(&self, manifest: &ChainManifest) -> Result<()> {
        let manifest_path = self.base_dir.join("manifest.json");
        let data = serde_json::to_string_pretty(manifest)?;
        fs::write(manifest_path, data).context("Failed to write manifest")
    }

    pub fn append_receipt(&self, receipt: &LegalReceipt) -> Result<ChainManifest> {
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

        if let Some(prev_rho) = &receipt.last_receipt_hash {
            if Some(prev_rho.0.clone()) != manifest.latest_receipt_hash {
                return Err(anyhow!(
                    "Receipt Chain Discontinuity: expected {:?}, got {:?}",
                    manifest.latest_receipt_hash,
                    prev_rho.0
                ));
            }
        }

        // Compute new root commitment: root_n = H(root_{n-1}, rho_n)
        let root_input = format!("{},{}", manifest.root_commitment, receipt.receipt_hash.0);
        let new_root = stable_hash(&root_input);

        let trace_hash = receipt.trace.as_ref().map(|t| stable_hash(t));

        let record = ReceiptLogRecord {
            record_type: "receipt".into(),
            version: "1.0.0".into(),
            step_index: receipt.step_index,
            receipt_hash: receipt.receipt_hash.0.clone(),
            previous_receipt_hash: receipt.last_receipt_hash.as_ref().map(|h| h.0.clone()),
            pre_state_hash: receipt.pre_state_hash.0.clone(),
            post_state_hash: receipt.post_state_hash.0.clone(),
            trace_hash,
            written_at: Utc::now().to_rfc3339(),
            root_commitment: new_root.clone(),
        };

        // Write to receipts.ndjson
        let log_path = self.base_dir.join("receipts.ndjson");
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;
        
        let record_json = serde_json::to_string(&record)?;
        writeln!(file, "{}", record_json)?;

        // If trace exists, write to traces.ndjson
        if let Some(trace) = &receipt.trace {
            let trace_log_path = self.base_dir.join("traces.ndjson");
            let mut trace_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(trace_log_path)?;
            writeln!(trace_file, "{}", serde_json::to_string(trace)?)?;
        }

        // Update Manifest
        manifest.latest_step_index = receipt.step_index;
        manifest.latest_receipt_hash = Some(receipt.receipt_hash.0.clone());
        manifest.latest_state_hash = receipt.post_state_hash.0.clone();
        manifest.root_commitment = new_root;

        self.save_manifest(&manifest)?;

        Ok(manifest)
    }

    pub fn verify_integrity(&self) -> Result<String> {
        let log_path = self.base_dir.join("receipts.ndjson");
        if !log_path.exists() {
            return Ok("genesis".into());
        }

        let file = fs::File::open(log_path)?;
        let reader = BufReader::new(file);

        let mut current_root = "genesis".to_string();
        let mut last_step = 0;
        let mut last_rho: Option<String> = None;
        let mut last_state = String::new();

        for (i, line) in reader.lines().enumerate() {
            let line = line?;
            let record: ReceiptLogRecord = serde_json::from_str(&line)
                .with_context(|| format!("Failed to parse record at line {}", i + 1))?;

            // 1. Root Recomputation
            let expected_root_input = format!("{},{}", current_root, record.receipt_hash);
            let recomputed_root = stable_hash(&expected_root_input);
            if recomputed_root != record.root_commitment {
                return Err(anyhow!("Tamper detected at step {}: root commitment mismatch", record.step_index));
            }

            // 2. Continuity Checks
            if i > 0 {
                if record.step_index != last_step + 1 {
                    return Err(anyhow!("Step index gap at step {}", record.step_index));
                }
                if record.previous_receipt_hash != last_rho {
                    return Err(anyhow!("Receipt chain broken at step {}", record.step_index));
                }
                if record.pre_state_hash != last_state {
                    return Err(anyhow!("State continuity broken at step {}", record.step_index));
                }
            }

            current_root = record.root_commitment;
            last_step = record.step_index;
            last_rho = Some(record.receipt_hash.clone());
            last_state = record.post_state_hash.clone();
        }

        // Verify against manifest
        let manifest = self.load_manifest()?;
        if manifest.root_commitment != current_root {
            return Err(anyhow!("Manifest root commitment mismatch with log"));
        }

        Ok(current_root)
    }
}
