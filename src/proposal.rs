use serde::{Deserialize, Serialize};
use crate::{ClaimId, ReferenceId};
use crate::claim::{AuthorityLabel, InferenceKind};
use crate::reference::{LegalReference, Jurisdiction, JurisdictionLevel, ProceduralStatus};

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
    pub model_confidence: f64,
}

impl CandidateClaim {
    pub fn to_witness(&self, id: &str) -> crate::witness::LegalWitness {
        let cited_reference = self.citation.as_ref().map(|cit| LegalReference {
            id: ReferenceId("unresolved".to_string()),
            citation: cit.clone(),
            jurisdiction: Jurisdiction {
                court: "Unresolved".to_string(),
                region: "Unresolved".to_string(),
                level: JurisdictionLevel::Appellate,
            },
            source_text: None,
            procedural_status: ProceduralStatus::GoodLaw,
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
            support_score: Some(self.model_confidence),
            contrary_authorities: vec![],
            user_asserted_good_law: None,
            verifier_notes: vec![format!("Model confidence: {:.2}", self.model_confidence)],
        }
    }
}
