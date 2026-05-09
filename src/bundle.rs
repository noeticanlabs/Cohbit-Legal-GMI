use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};
use crate::persistence::{PersistenceManager, ChainManifest};
use crate::receipt::stable_hash;
use crate::adapter::LegalSourceAdapter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceManifestEntry {
    pub citation: String,
    pub source_id: String,
    pub text_hash: String,
    pub file_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleManifest {
    pub bundle_id: String,
    pub chain_id: String,
    pub root_commitment: String,
    pub exported_at: String,
    pub source_entries: Vec<SourceManifestEntry>,
}

pub struct BundleManager {
    pub bundle_dir: PathBuf,
}

impl BundleManager {
    pub fn new(dir: impl AsRef<Path>) -> Self {
        Self { bundle_dir: dir.as_ref().to_path_buf() }
    }

    pub fn export_bundle(
        &self,
        pm: &PersistenceManager,
        adapters: &[&dyn LegalSourceAdapter],
    ) -> Result<()> {
        if self.bundle_dir.exists() {
            fs::remove_dir_all(&self.bundle_dir)?;
        }
        fs::create_dir_all(&self.bundle_dir)?;
        fs::create_dir_all(self.bundle_dir.join("sources"))?;

        // 1. Copy Core Files
        fs::copy(pm.base_dir.join("manifest.json"), self.bundle_dir.join("manifest.json"))?;
        
        let receipts_path = pm.base_dir.join("receipts.ndjson");
        if receipts_path.exists() {
            fs::copy(&receipts_path, self.bundle_dir.join("receipts.ndjson"))?;
        }

        let traces_path = pm.base_dir.join("traces.ndjson");
        if traces_path.exists() {
            fs::copy(&traces_path, self.bundle_dir.join("traces.ndjson"))?;
        }

        let anchors_path = pm.base_dir.join("anchor_receipts.ndjson");
        if anchors_path.exists() {
            fs::copy(&anchors_path, self.bundle_dir.join("anchor_receipts.ndjson"))?;
        }

        // 2. Extract and Bind Sources
        let mut source_entries = Vec::new();
        if traces_path.exists() {
            let file = File::open(&traces_path)?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line?;
                let trans_trace: crate::receipt::TransitionTrace = serde_json::from_str(&line)?;
                
                if let Some(trace) = &trans_trace.adapter_trace {
                    // Find a matching source from adapters
                    let mut source_found = false;
                    for adapter in adapters {
                        if let Ok(resolved) = adapter.resolve_reference(&trace.citation) {
                            if let Ok(text) = adapter.extract_source_text(&resolved) {
                                let text_hash = stable_hash(&text);
                                if text_hash == trace.extracted_text_hash {
                                    let file_name = format!("{}.txt", stable_hash(&trace.citation));
                                    let source_path = self.bundle_dir.join("sources").join(&file_name);
                                    fs::write(source_path, text)?;

                                    source_entries.push(SourceManifestEntry {
                                        citation: trace.citation.clone(),
                                        source_id: resolved.id.0,
                                        text_hash,
                                        file_name,
                                    });
                                    source_found = true;
                                    break;
                                }
                            }
                        }
                    }
                    if !source_found {
                        return Err(anyhow!("Missing source evidence for citation: {}", trace.citation));
                    }
                }
            }
        }

        // 3. Create Bundle Manifest
        let manifest = pm.load_manifest()?;
        let bundle_manifest = BundleManifest {
            bundle_id: stable_hash(&format!("bundle_{}", Utc::now().to_rfc3339())),
            chain_id: manifest.chain_id.clone(),
            root_commitment: manifest.root_commitment.clone(),
            exported_at: Utc::now().to_rfc3339(),
            source_entries,
        };
        fs::write(
            self.bundle_dir.join("source_manifest.json"),
            serde_json::to_string_pretty(&bundle_manifest)?,
        )?;

        // 4. Generate Audit Report
        let report = format!(
            "# CohBit Legal GMI Audit Report\n\n- **Bundle ID**: {}\n- **Chain ID**: {}\n- **Root Commitment**: {}\n- **Exported At**: {}\n",
            bundle_manifest.bundle_id,
            bundle_manifest.chain_id,
            bundle_manifest.root_commitment,
            bundle_manifest.exported_at
        );
        fs::write(self.bundle_dir.join("audit_report.md"), report)?;

        // 5. Generate Bundle Hash (Binds all files)
        let bundle_hash = self.compute_terminal_bundle_hash()?;
        fs::write(self.bundle_dir.join("bundle_hash.txt"), bundle_hash)?;

        Ok(())
    }

    fn compute_terminal_bundle_hash(&self) -> Result<String> {
        let mut hasher = blake3::Hasher::new();
        let files = ["manifest.json", "receipts.ndjson", "traces.ndjson", "anchor_receipts.ndjson", "source_manifest.json", "audit_report.md"];
        for f in files {
            let p = self.bundle_dir.join(f);
            if p.exists() {
                let data = fs::read(&p)?;
                hasher.update(&data);
            }
        }
        // Also bind the source files
        let sources_dir = self.bundle_dir.join("sources");
        if sources_dir.exists() {
            let mut entries: Vec<_> = fs::read_dir(sources_dir)?.collect::<Result<Vec<_>, _>>()?;
            entries.sort_by_key(|e| e.file_name()); // Canonical order
            for entry in entries {
                let data = fs::read(entry.path())?;
                hasher.update(&data);
            }
        }
        Ok(hasher.finalize().to_hex().to_string())
    }

    pub fn verify_bundle(&self) -> Result<()> {
        let manifest_path = self.bundle_dir.join("manifest.json");
        let source_manifest_path = self.bundle_dir.join("source_manifest.json");
        let bundle_hash_path = self.bundle_dir.join("bundle_hash.txt");

        if !manifest_path.exists() || !source_manifest_path.exists() || !bundle_hash_path.exists() {
            return Err(anyhow!("Missing core bundle files"));
        }

        let manifest_data = fs::read_to_string(manifest_path)?;
        let manifest: ChainManifest = serde_json::from_str(&manifest_data)?;

        let source_manifest_data = fs::read_to_string(source_manifest_path)?;
        let bundle_manifest: BundleManifest = serde_json::from_str(&source_manifest_data)?;

        // 1. Verify Terminal Hash
        let terminal_hash = fs::read_to_string(bundle_hash_path)?;
        let expected_hash = self.compute_terminal_bundle_hash()?;
        if terminal_hash != expected_hash {
            return Err(anyhow!("Bundle terminal hash mismatch (Bundle Tampered)"));
        }

        // 2. Verify Source Integrity
        for entry in &bundle_manifest.source_entries {
            let source_path = self.bundle_dir.join("sources").join(&entry.file_name);
            if !source_path.exists() {
                return Err(anyhow!("Missing source file: {}", entry.file_name));
            }
            let text = fs::read_to_string(source_path)?;
            if stable_hash(&text) != entry.text_hash {
                return Err(anyhow!("Source integrity failure: {}", entry.citation));
            }
        }

        // 3. Verify Anchors
        let anchors_path = self.bundle_dir.join("anchor_receipts.ndjson");
        if manifest.latest_step_index > 0 {
            if !anchors_path.exists() {
                return Err(anyhow!("Anchoring Violation: Bundle contains logic (step_index {}) but no external anchors. Production-grade bundles must be anchored if not at genesis.", manifest.latest_step_index));
            }

            let file = File::open(anchors_path)?;
            let reader = BufReader::new(file);
            let mut last_anchor = None;
            for line in reader.lines() {
                let line = line?;
                let anchor: crate::anchoring::AnchorReceipt = serde_json::from_str(&line)?;
                last_anchor = Some(anchor);
            }
            if let Some(anchor) = last_anchor {
                if anchor.root_commitment != manifest.root_commitment {
                    return Err(anyhow!("Anchor mismatch: Root commitment does not match anchor log"));
                }
                if anchor.manifest_hash != stable_hash(&manifest) {
                    return Err(anyhow!("Anchor mismatch: Manifest hash does not match anchor log"));
                }
            } else {
                 return Err(anyhow!("Anchor log exists but contains no receipts"));
            }
        }

        // 4. Persistence Integrity Check
        let pm = PersistenceManager { base_dir: self.bundle_dir.clone() };
        pm.verify_integrity()?;

        Ok(())
    }
}

use chrono::Utc;
