use serde::{Deserialize, Serialize};
use crate::witness::LegalWitness;
use crate::verifier::VerifierResult;
use crate::adapter::TreatmentType;
use num_rational::BigRational;
use num_bigint::BigInt;
use num_traits::{FromPrimitive, ToPrimitive};

pub fn parse_decimal(s: &str) -> BigRational {
    if let Some((int_part, frac_part)) = s.split_once('.') {
        let int_val: BigInt = int_part.parse().unwrap_or(BigInt::from(0));
        let frac_val: BigInt = frac_part.parse().unwrap_or(BigInt::from(0));
        let denominator = BigInt::from(10).pow(frac_part.len() as u32);
        
        BigRational::from(int_val) + BigRational::new(frac_val, denominator)
    } else {
        s.parse::<BigInt>().map(BigRational::from).unwrap_or_else(|_| BigRational::from_i64(0).unwrap())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuationConfig {
    pub mode: ValuationMode,
    pub authority_weight: String, // Stored as String for BigRational serialization
    pub relevance_weight: String,
    pub conflict_penalty_base: String,
}

impl Default for ValuationConfig {
    fn default() -> Self {
        Self {
            mode: ValuationMode::Standard,
            authority_weight: "1".into(),
            relevance_weight: "1".into(),
            conflict_penalty_base: "50".into(),
        }
    }
}

impl ValuationConfig {
    pub fn authority_weight_rat(&self) -> BigRational {
        self.authority_weight.parse().unwrap_or_else(|_| BigRational::from_i64(1).unwrap())
    }
    pub fn relevance_weight_rat(&self) -> BigRational {
        self.relevance_weight.parse().unwrap_or_else(|_| BigRational::from_i64(1).unwrap())
    }
    pub fn conflict_penalty_base_rat(&self) -> BigRational {
        self.conflict_penalty_base.parse().unwrap_or_else(|_| BigRational::from_i64(50).unwrap())
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
) -> BigRational {
    // RFAP-12: Numerical Sobriety. No floats in consensus.
    let base_support = witness.support_score_rat();
    
    let authority_multiplier = match witness.proposed_claim.authority_label {
        crate::claim::AuthorityLabel::Binding => BigRational::new(3.into(), 2.into()),    // 1.5
        crate::claim::AuthorityLabel::Persuasive => BigRational::from_i64(1).unwrap(),    // 1.0
        crate::claim::AuthorityLabel::Dictum => BigRational::new(1.into(), 2.into()),     // 0.5
        crate::claim::AuthorityLabel::Advisory => BigRational::new(3.into(), 10.into()), // 0.3
    };

    // Coherence gain represents the reduction in uncertainty
    let base_gain = BigRational::from_i64(100).unwrap();
    
    // Penalties reduce the coherence gain
    let mut penalty = BigRational::from_i64(witness.contrary_authorities.len() as i64).unwrap() 
                      * config.conflict_penalty_base_rat();

    // M6: Procedural Treatment Penalties (reduce gain)
    if let Some(t) = &verifier_result.adapter_trace {
        if let Some(st) = &t.status_trace {
            for treatment in &st.contrary_treatment {
                match treatment.treatment_type {
                    TreatmentType::DistinguishedBy => penalty += BigRational::from_i64(25).unwrap(),
                    TreatmentType::CriticizedBy => penalty += BigRational::from_i64(40).unwrap(),
                    _ => {}
                }
            }
        }
    }

    // High support and high authority increase the gain
    let support_bonus = base_gain * base_support * authority_multiplier * config.authority_weight_rat();
    
    support_bonus - penalty
}
