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
    Qualified,
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
    #[serde(default)]
    pub qualified_claims: Vec<CivicClaim>,
    pub unresolved_claims: Vec<CivicClaim>,
    pub next_steps: Vec<String>,
    pub external_submission_authorized: bool,
}

impl CitizenResponse {
    /// Legacy/untrusted construction: public labels cannot create support.
    pub fn build(summary: String, claims: Vec<CivicClaim>, next_steps: Vec<String>) -> Self {
        Self {
            summary,
            supported_claims: vec![],
            qualified_claims: vec![],
            unresolved_claims: claims,
            next_steps,
            external_submission_authorized: false,
        }
    }

    /// Promote only claims bound to canonical proof receipts whose status was
    /// freshly recomputed under operator trust policy.
    pub fn build_from_receipts(
        summary: String,
        claims: Vec<CivicClaim>,
        receipts: &[crate::receipt::LegalReceipt],
        policy: &crate::proof::TrustPolicy,
        now: &str,
        next_steps: Vec<String>,
    ) -> Self {
        let mut proof_statuses = std::collections::BTreeMap::new();
        for receipt in receipts {
            if receipt.decision != crate::verifier::VerificationDecision::Accept
                || receipt.policy_version != crate::proof::PROOF_POLICY
                || receipt.verify_canonical().is_err()
            {
                continue;
            }
            let Some(trace) = &receipt.trace else {
                continue;
            };
            if trace.decision != crate::verifier::VerificationDecision::Accept
                || trace.policy_version != crate::proof::PROOF_POLICY
            {
                continue;
            }
            let Some(proof) = &trace.legal_proof else {
                continue;
            };
            let witness_claim = &trace.witness.proposed_claim;
            if proof.context.claim_id != witness_claim.id.0
                || proof.context.claim_text != witness_claim.text
                || proof.context.claim_hash != crate::receipt::stable_hash(witness_claim)
            {
                continue;
            }
            if let Ok(checked) = proof.check(policy, now) {
                proof_statuses.insert(
                    proof.context.claim_id.clone(),
                    (
                        proof.context.claim_text.clone(),
                        proof.context.jurisdiction.clone(),
                        checked.status().clone(),
                    ),
                );
            }
        }
        let mut supported_claims = Vec::new();
        let mut qualified_claims = Vec::new();
        let mut unresolved_claims = Vec::new();
        for mut claim in claims {
            match proof_statuses.get(&claim.claim_id) {
                Some((text, jurisdiction, crate::proof::ProofStatus::ProvedWithinScope))
                    if text == &claim.text && jurisdiction == &claim.jurisdiction =>
                {
                    claim.verification_state = CivicVerificationState::Supported;
                    supported_claims.push(claim);
                }
                Some((text, jurisdiction, crate::proof::ProofStatus::Qualified))
                    if text == &claim.text && jurisdiction == &claim.jurisdiction =>
                {
                    claim.verification_state = CivicVerificationState::Qualified;
                    claim.limitations.push(
                        "Qualified proof: assumptions, coverage, interpretation, or domain review limits apply"
                            .into(),
                    );
                    qualified_claims.push(claim);
                }
                _ => {
                    if matches!(
                        claim.verification_state,
                        CivicVerificationState::Supported | CivicVerificationState::Qualified
                    ) {
                        claim.verification_state = CivicVerificationState::Uncertain;
                    }
                    unresolved_claims.push(claim);
                }
            }
        }
        Self {
            summary,
            supported_claims,
            qualified_claims,
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
