use crate::{
    error::GmiError,
    receipt::stable_hash,
    reference::{LegalReference, ProceduralStatus},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdapterError {
    ReferenceNotFound(String),
    SourceTextUnavailable(String),
    StatusCheckFailed(String),
    Internal(String),
}

impl From<AdapterError> for GmiError {
    fn from(err: AdapterError) -> Self {
        GmiError::AdapterError(format!("{:?}", err))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteSpan {
    pub source_id: crate::ReferenceId,
    pub start_byte: usize,
    pub end_byte: usize,
    pub page: Option<u32>,
    pub raw_quote_hash: String,
    pub normalized_quote_hash: String,
    pub extraction_method: String,
    pub extractor_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TreatmentType {
    OverruledBy,
    ReversedBy,
    VacatedBy,
    SupersededByStatute,
    DistinguishedBy,
    CriticizedBy,
    FollowedBy,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreatmentRecord {
    pub treating_citation: String,
    pub treatment_type: TreatmentType,
    pub treatment_summary: String,
    pub date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusTrace {
    pub citation: String,
    pub status: ProceduralStatus,
    pub checked_at: String,
    pub status_source: String,
    pub status_source_version: String,
    pub contrary_treatment: Vec<TreatmentRecord>,
    pub trace_hash: String,
}

pub fn verify_status_trace_hash(trace: &StatusTrace) -> bool {
    let mut canonical = trace.clone();
    let supplied = std::mem::take(&mut canonical.trace_hash);
    !supplied.is_empty() && supplied == stable_hash(&canonical)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterTrace {
    pub citation: String,
    pub adapter_name: String,
    pub adapter_version: String,
    pub resolved_reference_hash: String,
    pub resolved_source_id: crate::ReferenceId,
    pub resolved_jurisdiction: crate::reference::Jurisdiction,
    pub extracted_text_hash: String,
    pub quote_span: Option<QuoteSpan>,
    pub status_trace: Option<StatusTrace>,
    pub procedural_status: ProceduralStatus,
    pub status_source: String,
    pub adapter_trust: AdapterTrust,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum AdapterTrust {
    TestOnly,
    #[default]
    LocalUnverified,
    Verified,
}

pub trait LegalSourceAdapter {
    fn resolve_reference(&self, citation: &str) -> Result<LegalReference, AdapterError>;
    fn extract_source_text(&self, reference: &LegalReference) -> Result<String, AdapterError>;
    fn check_status(&self, reference: &LegalReference) -> Result<ProceduralStatus, AdapterError>;
    fn get_status_trace(&self, reference: &LegalReference) -> Result<StatusTrace, AdapterError>;

    fn name(&self) -> String;
    fn version(&self) -> String;
    fn trust_level(&self) -> AdapterTrust {
        AdapterTrust::LocalUnverified
    }

    fn get_page_at_offset(&self, citation: &str, offset: usize) -> Option<u32> {
        let _ = citation;
        let _ = offset;
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfPage {
    pub number: u32,
    pub text: String,
    pub start_offset: usize,
}

pub struct LocalTextSourceAdapter {
    pub references: HashMap<String, LegalReference>,
}

impl LegalSourceAdapter for LocalTextSourceAdapter {
    fn resolve_reference(&self, citation: &str) -> Result<LegalReference, AdapterError> {
        self.references
            .get(citation)
            .cloned()
            .ok_or_else(|| AdapterError::ReferenceNotFound(citation.to_string()))
    }

    fn extract_source_text(&self, reference: &LegalReference) -> Result<String, AdapterError> {
        reference
            .source_text
            .clone()
            .ok_or_else(|| AdapterError::SourceTextUnavailable(reference.citation.clone()))
    }

    fn check_status(&self, reference: &LegalReference) -> Result<ProceduralStatus, AdapterError> {
        Ok(reference.procedural_status.clone())
    }

    fn get_status_trace(&self, reference: &LegalReference) -> Result<StatusTrace, AdapterError> {
        let status = self.check_status(reference)?;
        let mut trace = StatusTrace {
            citation: reference.citation.clone(),
            status,
            checked_at: "2026-05-04".into(),
            status_source: self.name(),
            status_source_version: self.version(),
            contrary_treatment: vec![],
            trace_hash: "".into(),
        };
        trace.trace_hash = stable_hash(&trace);
        Ok(trace)
    }

    fn name(&self) -> String {
        "LocalTextSourceAdapter".to_string()
    }
    fn version(&self) -> String {
        "1.0.0".to_string()
    }
}

pub struct LocalPdfSourceAdapter {
    pub file_map: HashMap<String, Vec<PdfPage>>,
}

impl LegalSourceAdapter for LocalPdfSourceAdapter {
    fn resolve_reference(&self, citation: &str) -> Result<LegalReference, AdapterError> {
        if self.file_map.contains_key(citation) {
            Ok(LegalReference {
                id: crate::ReferenceId(format!("pdf_{}", citation)),
                citation: citation.to_string(),
                jurisdiction: crate::reference::Jurisdiction {
                    court: "Unknown".into(),
                    region: "Unknown".into(),
                    level: crate::reference::JurisdictionLevel::Unknown,
                },
                source_text: None,
                procedural_status: ProceduralStatus::Unknown,
                citation_confidence: None,
                counsel_id: None,
                law_firm_id: None,
            })
        } else {
            Err(AdapterError::ReferenceNotFound(citation.to_string()))
        }
    }

    fn extract_source_text(&self, reference: &LegalReference) -> Result<String, AdapterError> {
        self.file_map
            .get(&reference.citation)
            .map(|pages| {
                pages
                    .iter()
                    .map(|p| p.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .ok_or_else(|| AdapterError::SourceTextUnavailable(reference.citation.clone()))
    }

    fn check_status(&self, _reference: &LegalReference) -> Result<ProceduralStatus, AdapterError> {
        Ok(ProceduralStatus::Unknown)
    }

    fn get_status_trace(&self, reference: &LegalReference) -> Result<StatusTrace, AdapterError> {
        let mut trace = StatusTrace {
            citation: reference.citation.clone(),
            status: ProceduralStatus::Unknown,
            checked_at: "2026-05-04".into(),
            status_source: self.name(),
            status_source_version: self.version(),
            contrary_treatment: vec![],
            trace_hash: "".into(),
        };
        trace.trace_hash = stable_hash(&trace);
        Ok(trace)
    }

    fn name(&self) -> String {
        "LocalPdfSourceAdapter".to_string()
    }
    fn version(&self) -> String {
        "1.2.0".to_string()
    }

    fn get_page_at_offset(&self, citation: &str, offset: usize) -> Option<u32> {
        let pages = self.file_map.get(citation)?;
        pages
            .iter()
            .rfind(|p| p.start_offset <= offset)
            .map(|p| p.number)
    }
}

#[derive(Default)]
pub struct MockLegalStatusAdapter {
    pub status_overrides: HashMap<String, ProceduralStatus>,
    pub treatment_overrides: HashMap<String, Vec<TreatmentRecord>>,
    pub check_time: String,
    pub reference_store: HashMap<String, LegalReference>, // Added for tests
}

impl LegalSourceAdapter for MockLegalStatusAdapter {
    fn resolve_reference(&self, citation: &str) -> Result<LegalReference, AdapterError> {
        self.reference_store
            .get(citation)
            .cloned()
            .ok_or_else(|| AdapterError::ReferenceNotFound(citation.to_string()))
    }

    fn extract_source_text(&self, reference: &LegalReference) -> Result<String, AdapterError> {
        reference
            .source_text
            .clone()
            .ok_or_else(|| AdapterError::SourceTextUnavailable(reference.citation.clone()))
    }

    fn check_status(&self, reference: &LegalReference) -> Result<ProceduralStatus, AdapterError> {
        if let Some(status) = self.status_overrides.get(&reference.citation) {
            Ok(status.clone())
        } else {
            Ok(reference.procedural_status.clone())
        }
    }

    fn get_status_trace(&self, reference: &LegalReference) -> Result<StatusTrace, AdapterError> {
        let status = self.check_status(reference)?;
        let contrary_treatment = self
            .treatment_overrides
            .get(&reference.citation)
            .cloned()
            .unwrap_or_default();

        let mut trace = StatusTrace {
            citation: reference.citation.clone(),
            status,
            checked_at: self.check_time.clone(),
            status_source: self.name(),
            status_source_version: self.version(),
            contrary_treatment,
            trace_hash: "".into(),
        };
        trace.trace_hash = stable_hash(&trace);
        Ok(trace)
    }

    fn name(&self) -> String {
        "MockLegalStatusAdapter".to_string()
    }
    fn version(&self) -> String {
        "2.0.0".to_string()
    }
    fn trust_level(&self) -> AdapterTrust {
        AdapterTrust::TestOnly
    }
}
