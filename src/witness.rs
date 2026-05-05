use serde::{Deserialize, Serialize};
use crate::{
    claim::LegalClaim,
    reference::LegalReference,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalWitness {
    pub proposed_claim: LegalClaim,
    pub cited_reference: Option<LegalReference>,
    pub support_score: Option<f64>,
    pub contrary_authorities: Vec<LegalReference>,
    pub user_asserted_good_law: Option<bool>,
    pub verifier_notes: Vec<String>,
}
