use crate::ReferenceId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalReference {
    pub id: ReferenceId,
    pub citation: String,
    pub jurisdiction: Jurisdiction,
    pub source_text: Option<String>,
    pub procedural_status: ProceduralStatus,
    pub citation_confidence: Option<String>, // RFAP: Stored as string for deterministic BigRational
    pub counsel_id: Option<String>,
    pub law_firm_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Jurisdiction {
    pub court: String,
    pub region: String,
    pub level: JurisdictionLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JurisdictionLevel {
    Supreme,
    Appellate,
    Trial,
    Administrative,
    Statutory,
    Unknown,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProceduralStatus {
    GoodLaw,
    Overruled,
    Vacated,
    Reversed,
    Superseded,
    #[default]
    Unknown,
}
impl Default for Jurisdiction {
    fn default() -> Self {
        Self {
            court: "Unknown".into(),
            region: "Unknown".into(),
            level: JurisdictionLevel::Unknown,
        }
    }
}

impl Default for LegalReference {
    fn default() -> Self {
        Self {
            id: crate::ReferenceId("default".into()),
            citation: "Unknown".into(),
            jurisdiction: Jurisdiction::default(),
            source_text: None,
            procedural_status: ProceduralStatus::default(),
            citation_confidence: None,
            counsel_id: None,
            law_firm_id: None,
        }
    }
}
