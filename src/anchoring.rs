use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use anyhow::Result;
use chrono::Utc;
use crate::receipt::stable_hash;
use crate::persistence::ChainManifest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchorReceipt {
    pub anchor_id: String,
    pub chain_id: String,
    pub root_commitment: String,
    pub manifest_hash: String,
    pub anchored_at: String,
    pub anchor_backend: String,
    pub backend_reference: String,
}

pub trait RootAnchor {
    fn anchor_root(&self, manifest: &ChainManifest) -> Result<AnchorReceipt>;
    fn verify_anchor(&self, anchor: &AnchorReceipt, manifest: &ChainManifest) -> Result<bool>;
}

pub struct LocalAnchorAdapter {
    pub anchor_dir: PathBuf,
}

impl LocalAnchorAdapter {
    pub fn new(dir: impl AsRef<Path>) -> Result<Self> {
        let path = dir.as_ref().to_path_buf();
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        Ok(Self { anchor_dir: path })
    }
}

impl RootAnchor for LocalAnchorAdapter {
    fn anchor_root(&self, manifest: &ChainManifest) -> Result<AnchorReceipt> {
        let manifest_hash = stable_hash(manifest);
        let anchored_at = Utc::now().to_rfc3339();
        let anchor_id = stable_hash(&(manifest.root_commitment.clone() + &anchored_at));

        let receipt = AnchorReceipt {
            anchor_id,
            chain_id: manifest.chain_id.clone(),
            root_commitment: manifest.root_commitment.clone(),
            manifest_hash: manifest_hash.clone(),
            anchored_at,
            anchor_backend: "LocalFile".to_string(),
            backend_reference: "anchor_receipts.ndjson".to_string(),
        };

        let log_path = self.anchor_dir.join("anchor_receipts.ndjson");
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;
        
        writeln!(file, "{}", serde_json::to_string(&receipt)?)?;

        Ok(receipt)
    }

    fn verify_anchor(&self, anchor: &AnchorReceipt, manifest: &ChainManifest) -> Result<bool> {
        let current_manifest_hash = stable_hash(manifest);
        Ok(anchor.root_commitment == manifest.root_commitment && 
           anchor.manifest_hash == current_manifest_hash)
    }
}

/// MockGitAnchorAdapter implements the RootAnchor trait by simulating the anchoring of roots into a Git repository.
/// It performs internal state hashing to verify consistency, providing a cryptographically stable
/// audit trail for V1.
pub struct MockGitAnchorAdapter {
    pub repo_path: PathBuf,
}

impl MockGitAnchorAdapter {
    pub fn new(repo_path: impl AsRef<Path>) -> Self {
        Self { repo_path: repo_path.as_ref().to_path_buf() }
    }
}

impl RootAnchor for MockGitAnchorAdapter {
    fn anchor_root(&self, manifest: &ChainManifest) -> Result<AnchorReceipt> {
        let manifest_hash = stable_hash(manifest);
        let anchored_at = Utc::now().to_rfc3339();
        
        // In a real implementation, we would git add/commit the manifest or a commitment file.
        // For this M11 prototype, we'll simulate the git backend reference with a mock commit hash.
        let backend_reference = stable_hash(&format!("git_commit_{}", manifest.root_commitment));

        let receipt = AnchorReceipt {
            anchor_id: stable_hash(&format!("git_anchor_{}", anchored_at)),
            chain_id: manifest.chain_id.clone(),
            root_commitment: manifest.root_commitment.clone(),
            manifest_hash,
            anchored_at,
            anchor_backend: "Git".to_string(),
            backend_reference,
        };

        Ok(receipt)
    }

    fn verify_anchor(&self, anchor: &AnchorReceipt, manifest: &ChainManifest) -> Result<bool> {
        let current_manifest_hash = stable_hash(manifest);
        Ok(anchor.root_commitment == manifest.root_commitment && 
           anchor.manifest_hash == current_manifest_hash)
    }
}
