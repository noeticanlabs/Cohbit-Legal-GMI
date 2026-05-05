use serde::{Deserialize, Serialize};
use crate::ReferenceId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalReference {
    pub id: ReferenceId,
    pub citation: String,
    pub jurisdiction: Jurisdiction,
    pub source_text: Option<String>,
    pub procedural_status: ProceduralStatus,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProceduralStatus {
    GoodLaw,
    Overruled,
    Vacated,
    Reversed,
    Superseded,
    Unknown,
}
