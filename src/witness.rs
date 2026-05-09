use serde::{Deserialize, Serialize};
use crate::{
    claim::LegalClaim,
    reference::LegalReference,
};
use num_rational::BigRational;
use num_traits::FromPrimitive;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalWitness {
    pub proposed_claim: LegalClaim,
    pub cited_reference: Option<LegalReference>,
    pub support_score: Option<String>, // RFAP: Stored as string for deterministic BigRational
    pub contrary_authorities: Vec<LegalReference>,
    pub user_asserted_good_law: Option<bool>,
    pub verifier_notes: Vec<String>,
}

impl LegalWitness {
    pub fn support_score_rat(&self) -> BigRational {
        self.support_score.as_ref()
            .map(|s| crate::valuation::parse_decimal(s))
            .unwrap_or_else(|| BigRational::from_i64(0).unwrap())
    }
}
