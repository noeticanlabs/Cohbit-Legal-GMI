use crate::claim::{AuthorityLabel, InferenceKind};
use crate::reference::{Jurisdiction, JurisdictionLevel, LegalReference, ProceduralStatus};
use crate::{ClaimId, ReferenceId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalProposal {
    pub proposal_id: String,
    pub title: String,
    pub description: String,
    pub candidate_claims: Vec<CandidateClaim>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateClaim {
    pub text: String,
    pub citation: Option<String>,
    pub quoted_text: Option<String>,
    pub authority_label: AuthorityLabel,
    pub inference: InferenceKind,
    pub model_confidence: String, // RFAP: String for BigRational
}

impl CandidateClaim {
    pub fn to_witness(&self, id: &str) -> crate::witness::LegalWitness {
        let cited_reference = self.citation.as_ref().map(|cit| LegalReference {
            id: ReferenceId("unresolved".to_string()),
            citation: cit.clone(),
            jurisdiction: Jurisdiction {
                court: "Unresolved".to_string(),
                region: "Unresolved".to_string(),
                level: JurisdictionLevel::Unknown,
            },
            source_text: None,
            procedural_status: ProceduralStatus::Unknown,
            citation_confidence: None,
            counsel_id: None,
            law_firm_id: None,
        });

        crate::witness::LegalWitness {
            proposed_claim: crate::claim::LegalClaim {
                id: ClaimId(id.to_string()),
                text: self.text.clone(),
                cited_reference: cited_reference.as_ref().map(|r| r.id.clone()),
                quoted_text: self.quoted_text.clone(),
                authority_label: self.authority_label.clone(),
                inference: self.inference.clone(),
            },
            cited_reference,
            // Generator confidence is proposal metadata, not verifier-authorized
            // evidentiary support. Only an independent verifier may populate a
            // support score after resolving and evaluating source evidence.
            support_score: None,
            contrary_authorities: vec![],
            user_asserted_good_law: None,
            verifier_notes: vec![format!("Model confidence: {}", self.model_confidence)],
            review_status: None,
            evidence_links: vec![],
            adverse_authority_search: None,
        }
    }
}
