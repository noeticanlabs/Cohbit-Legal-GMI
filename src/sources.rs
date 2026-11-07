use crate::adapter::{
    verify_status_trace_hash, AdapterError, AdapterTrust, LegalSourceAdapter, StatusTrace,
};
use crate::receipt::stable_hash;
use crate::reference::{LegalReference, ProceduralStatus};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegritySourceRecord {
    pub reference: LegalReference,
    pub publisher: String,
    pub official_url: String,
    pub retrieved_at: String,
    pub effective_date: Option<String>,
    pub revision: String,
    pub text: String,
    pub status_trace: StatusTrace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedCorpusManifest {
    pub version: String,
    pub key_id: String,
    pub records_hash: String,
    pub signature_hex: String,
}

impl SignedCorpusManifest {
    pub fn sign(
        records: &BTreeMap<String, IntegritySourceRecord>,
        key_id: &str,
        key: &SigningKey,
    ) -> Self {
        let records_hash = stable_hash(records);
        let signature = key.sign(records_hash.as_bytes());
        Self {
            version: "1.0.0".into(),
            key_id: key_id.into(),
            records_hash,
            signature_hex: hex::encode(signature.to_bytes()),
        }
    }
}

/// Signature-verified corpus integrity. This does not establish publisher
/// authority, legal status, applicability, or faithful interpretation.
pub struct IntegrityCheckedCorpusAdapter {
    records: BTreeMap<String, IntegritySourceRecord>,
    manifest: SignedCorpusManifest,
}

impl IntegrityCheckedCorpusAdapter {
    pub fn open(
        records: BTreeMap<String, IntegritySourceRecord>,
        manifest: SignedCorpusManifest,
        trusted_key: &VerifyingKey,
    ) -> Result<Self, AdapterError> {
        if records.is_empty() {
            return Err(AdapterError::Internal("Integrity corpus is empty".into()));
        }
        let computed_hash = stable_hash(&records);
        if computed_hash != manifest.records_hash {
            return Err(AdapterError::Internal(
                "Corpus manifest hash mismatch".into(),
            ));
        }
        let signature_bytes: [u8; 64] = hex::decode(&manifest.signature_hex)
            .map_err(|_| AdapterError::Internal("Invalid corpus signature encoding".into()))?
            .try_into()
            .map_err(|_| AdapterError::Internal("Invalid corpus signature length".into()))?;
        trusted_key
            .verify(
                manifest.records_hash.as_bytes(),
                &Signature::from_bytes(&signature_bytes),
            )
            .map_err(|_| AdapterError::Internal("Corpus signature is not trusted".into()))?;

        for (citation, record) in &records {
            if citation != &record.reference.citation
                || record.publisher.trim().is_empty()
                || !record.official_url.starts_with("https://")
                || record.retrieved_at.trim().is_empty()
                || record.revision.trim().is_empty()
                || record.text.trim().is_empty()
                || record.status_trace.status != record.reference.procedural_status
                || record.status_trace.checked_at.trim().is_empty()
                || record.status_trace.status_source.trim().is_empty()
                || record.status_trace.status_source_version.trim().is_empty()
                || !verify_status_trace_hash(&record.status_trace)
            {
                return Err(AdapterError::Internal(format!(
                    "Incomplete integrity-corpus source record: {}",
                    citation
                )));
            }
        }
        Ok(Self { records, manifest })
    }
}

impl LegalSourceAdapter for IntegrityCheckedCorpusAdapter {
    fn resolve_reference(&self, citation: &str) -> Result<LegalReference, AdapterError> {
        self.records
            .get(citation)
            .map(|record| {
                let mut reference = record.reference.clone();
                reference.source_text = None;
                reference
            })
            .ok_or_else(|| AdapterError::ReferenceNotFound(citation.into()))
    }

    fn extract_source_text(&self, reference: &LegalReference) -> Result<String, AdapterError> {
        self.records
            .get(&reference.citation)
            .map(|record| record.text.clone())
            .ok_or_else(|| AdapterError::SourceTextUnavailable(reference.citation.clone()))
    }

    fn check_status(&self, reference: &LegalReference) -> Result<ProceduralStatus, AdapterError> {
        self.records
            .get(&reference.citation)
            .map(|record| record.status_trace.status.clone())
            .ok_or_else(|| AdapterError::StatusCheckFailed(reference.citation.clone()))
    }

    fn get_status_trace(&self, reference: &LegalReference) -> Result<StatusTrace, AdapterError> {
        self.records
            .get(&reference.citation)
            .map(|record| record.status_trace.clone())
            .ok_or_else(|| AdapterError::StatusCheckFailed(reference.citation.clone()))
    }

    fn name(&self) -> String {
        format!("IntegrityCheckedCorpus:{}", self.manifest.key_id)
    }

    fn version(&self) -> String {
        self.manifest.version.clone()
    }

    fn trust_level(&self) -> AdapterTrust {
        AdapterTrust::LocalUnverified
    }
}

/// Compatibility names retained for serialized/API consumers. Both mean
/// integrity checked only and confer no authority or legal verification.
pub type AuthoritativeSourceRecord = IntegritySourceRecord;
pub type VerifiedCorpusAdapter = IntegrityCheckedCorpusAdapter;
