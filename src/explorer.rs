use crate::proposal::{CandidateClaim, LegalProposal};
use crate::verifier::VerificationDecision;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorationSpace {
    pub active_proposals: Vec<LegalProposal>,
    pub candidate_queue: VecDeque<CandidateClaim>,
    #[serde(alias = "accepted_claims_ids")]
    pub computational_accept_claim_ids: Vec<String>,
    pub rejected_claims_ids: Vec<String>,
    #[serde(default)]
    pub escalated_claims_ids: Vec<String>,
}

impl ExplorationSpace {
    pub fn new() -> Self {
        Self {
            active_proposals: vec![],
            candidate_queue: VecDeque::new(),
            computational_accept_claim_ids: vec![],
            rejected_claims_ids: vec![],
            escalated_claims_ids: vec![],
        }
    }

    pub fn inject_proposal(&mut self, proposal: LegalProposal) {
        for candidate in &proposal.candidate_claims {
            self.candidate_queue.push_back(candidate.clone());
        }
        self.active_proposals.push(proposal);
    }

    pub fn pop_candidate(&mut self) -> Option<CandidateClaim> {
        self.candidate_queue.pop_front()
    }

    pub fn record_decision(&mut self, text: &str, accepted: bool) {
        if accepted {
            self.computational_accept_claim_ids.push(text.to_string());
        } else {
            self.rejected_claims_ids.push(text.to_string());
        }
    }

    pub fn record_outcome(&mut self, text: &str, decision: &VerificationDecision) {
        match decision {
            VerificationDecision::Accept => {
                self.computational_accept_claim_ids.push(text.to_string())
            }
            VerificationDecision::Reject => self.rejected_claims_ids.push(text.to_string()),
            VerificationDecision::Escalate => self.escalated_claims_ids.push(text.to_string()),
        }
    }
}

impl Default for ExplorationSpace {
    fn default() -> Self {
        Self::new()
    }
}
