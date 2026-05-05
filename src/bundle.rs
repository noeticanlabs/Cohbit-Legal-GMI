use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Write, BufRead, BufReader};
use std::path::{Path, PathBuf};
use anyhow::{Context, Result, anyhow};
use crate::persistence::{PersistenceManager, ChainManifest};
use crate::receipt::stable_hash;
use crate::adapter::{LegalSourceAdapter, AdapterTrace};

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
                        // Note: In a real scenario, we might want to fail or warn if a source is missing.
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

        // 5. Generate Bundle Hash
        // For simplicity, we hash the bundle manifest and the core manifest together.
        let terminal_hash = stable_hash(&format!("{},{}", stable_hash(&bundle_manifest), manifest.root_commitment));
        fs::write(self.bundle_dir.join("bundle_hash.txt"), terminal_hash)?;

        Ok(())
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
        let expected_hash = stable_hash(&format!("{},{}", stable_hash(&bundle_manifest), manifest.root_commitment));
        if terminal_hash != expected_hash {
            return Err(anyhow!("Bundle terminal hash mismatch"));
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

        // 3. Persistence Integrity Check
        let pm = PersistenceManager { base_dir: self.bundle_dir.clone() };
        pm.verify_integrity()?;

        Ok(())
    }
}

use chrono::Utc;
