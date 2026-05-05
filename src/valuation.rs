use serde::{Deserialize, Serialize};
use crate::witness::LegalWitness;
use crate::verifier::VerifierResult;
use crate::adapter::TreatmentType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuationConfig {
    pub mode: ValuationMode,
    pub authority_weight: f64,
    pub relevance_weight: f64,
    pub conflict_penalty_base: f64,
}

impl Default for ValuationConfig {
    fn default() -> Self {
        Self {
            mode: ValuationMode::Standard,
            authority_weight: 1.0,
            relevance_weight: 1.0,
            conflict_penalty_base: 50.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValuationMode {
    Standard,
    DefectPermittedComplexification, // M5 doctrine
}

pub fn compute_delta_v(
    witness: &LegalWitness,
    verifier_result: &VerifierResult,
    config: &ValuationConfig,
) -> i128 {
    let base_support = witness.support_score.unwrap_or(0.0);
    
    let authority_multiplier = match witness.proposed_claim.authority_label {
        crate::claim::AuthorityLabel::Binding => 1.5,
        crate::claim::AuthorityLabel::Persuasive => 1.0,
        crate::claim::AuthorityLabel::Dictum => 0.5,
        crate::claim::AuthorityLabel::Advisory => 0.3,
    };

    // Coherence gain represents the reduction in uncertainty
    let base_gain = 100.0;
    
    // Penalties reduce the coherence gain
    let mut penalty = (witness.contrary_authorities.len() as f64) * config.conflict_penalty_base;

    // M6: Procedural Treatment Penalties (reduce gain)
    if let Some(t) = &verifier_result.adapter_trace {
        if let Some(st) = &t.status_trace {
            for treatment in &st.contrary_treatment {
                match treatment.treatment_type {
                    TreatmentType::DistinguishedBy => penalty += 25.0,
                    TreatmentType::CriticizedBy => penalty += 40.0,
                    _ => {}
                }
            }
        }
    }

    // High support and high authority increase the gain
    let support_bonus = base_gain * base_support * authority_multiplier * config.authority_weight;
    
    let gain = support_bonus - penalty;

    gain as i128
}
