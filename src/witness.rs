use serde::{Deserialize, Serialize};
use crate::{
    claim::LegalClaim,
    reference::LegalReference,
};
use num_rational::BigRational;
use num_traits::FromPrimitive;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InternalReviewStatus {
    Draft,
    PeerReviewed,
    Certified,
    AdversarialAudit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalWitness {
    pub proposed_claim: LegalClaim,
    pub cited_reference: Option<LegalReference>,
    pub support_score: Option<String>, // RFAP: Stored as string for deterministic BigRational
    pub contrary_authorities: Vec<LegalReference>,
    pub user_asserted_good_law: Option<bool>,
    pub verifier_notes: Vec<String>,
    pub review_status: Option<InternalReviewStatus>,
}

impl LegalWitness {
    pub fn support_score_rat(&self) -> BigRational {
        self.support_score.as_ref()
            .map(|s| crate::valuation::parse_rational(s))
            .unwrap_or_else(|| BigRational::from_i64(0).unwrap())
    }
}
impl Default for LegalWitness {
    fn default() -> Self {
        Self {
            proposed_claim: crate::claim::LegalClaim {
                id: crate::ClaimId("default".into()),
                text: "".into(),
                cited_reference: None,
                quoted_text: None,
                authority_label: crate::claim::AuthorityLabel::Advisory,
                inference: crate::claim::InferenceKind::UnsupportedHypothesis,
            },
            cited_reference: None,
            support_score: None,
            contrary_authorities: vec![],
            user_asserted_good_law: None,
            verifier_notes: vec![],
            review_status: None,
        }
    }
}
