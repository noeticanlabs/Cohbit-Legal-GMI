use crate::{claim::LegalClaim, reference::LegalReference};
use num_rational::BigRational;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EvidenceVerificationMethod {
    ExactQuotation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaimEvidenceLink {
    pub source_id: crate::ReferenceId,
    pub passage_start: usize,
    pub passage_end: usize,
    pub claim_fragment: String,
    pub verification_method: EvidenceVerificationMethod,
    pub verifier_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdverseAuthoritySearch {
    pub scope: String,
    pub searched_at: String,
    pub method: String,
    pub result_citations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InternalReviewStatus {
    Draft,
    PeerReviewed,
    LegacyCertifiedLabel,
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
    #[serde(default)]
    pub evidence_links: Vec<ClaimEvidenceLink>,
    #[serde(default)]
    pub adverse_authority_search: Option<AdverseAuthoritySearch>,
}

impl LegalWitness {
    pub fn support_score_rat(&self) -> Result<BigRational, anyhow::Error> {
        let score = self
            .support_score
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Support score is missing"))?;
        crate::valuation::try_parse_rational(score)
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
            evidence_links: vec![],
            adverse_authority_search: None,
        }
    }
}
