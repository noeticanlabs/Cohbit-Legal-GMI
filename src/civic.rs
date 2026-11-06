use crate::ReferenceId;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CivicClaimType {
    Legal,
    Factual,
    Procedural,
    UserObservation,
    Opinion,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CivicVerificationState {
    Proposed,
    Supported,
    Uncertain,
    Rejected,
    Escalated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HumanReviewState {
    NotReviewed,
    Reviewed {
        reviewer_id: String,
        reviewed_at: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CivicClaim {
    pub claim_id: String,
    pub claim_type: CivicClaimType,
    pub text: String,
    pub agency_id: Option<String>,
    pub agency_role: Option<String>,
    pub jurisdiction: String,
    pub source_ids: Vec<ReferenceId>,
    pub effective_date: Option<String>,
    pub retrieved_at: Option<String>,
    pub verification_state: CivicVerificationState,
    pub limitations: Vec<String>,
    pub human_review_state: HumanReviewState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitizenResponse {
    pub summary: String,
    pub supported_claims: Vec<CivicClaim>,
    pub unresolved_claims: Vec<CivicClaim>,
    pub next_steps: Vec<String>,
    pub external_submission_authorized: bool,
}

impl CitizenResponse {
    pub fn build(summary: String, claims: Vec<CivicClaim>, next_steps: Vec<String>) -> Self {
        let (supported_claims, unresolved_claims) = claims.into_iter().partition(|claim| {
            claim.verification_state == CivicVerificationState::Supported
                && matches!(claim.human_review_state, HumanReviewState::Reviewed { .. })
        });
        Self {
            summary,
            supported_claims,
            unresolved_claims,
            next_steps,
            external_submission_authorized: false,
        }
    }

    pub fn authorize_external_submission(
        &mut self,
        approving_human: &str,
        confirmation_text: &str,
    ) -> Result<()> {
        if approving_human.trim().is_empty() || confirmation_text != "AUTHORIZE SUBMISSION" {
            return Err(anyhow!(
                "Explicit human submission authorization is required"
            ));
        }
        self.external_submission_authorized = true;
        Ok(())
    }
}
