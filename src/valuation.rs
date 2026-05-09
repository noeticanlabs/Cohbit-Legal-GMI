use serde::{Deserialize, Serialize};
use crate::witness::LegalWitness;
use crate::verifier::VerifierResult;
use crate::adapter::TreatmentType;
use num_rational::BigRational;
use num_bigint::BigInt;
use num_traits::FromPrimitive;

pub fn parse_rational(s: &str) -> BigRational {
    try_parse_rational(s).unwrap_or_else(|_| BigRational::from_i64(0).unwrap())
}

pub fn try_parse_rational(s: &str) -> Result<BigRational, anyhow::Error> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(BigRational::from_i64(0).unwrap());
    }

    // Handle fraction format: "285/2"
    if let Some((num, den)) = s.split_once('/') {
        let n: BigInt = num.trim().parse()?;
        let d: BigInt = den.trim().parse()?;
        if d == BigInt::from(0) {
            return Err(anyhow::anyhow!("Zero denominator in rational: {}", s));
        }
        return Ok(BigRational::new(n, d));
    }

    // Handle decimal format: "142.5"
    if let Some((int_part, frac_part)) = s.split_once('.') {
        let negative = int_part.starts_with('-');
        let int_clean = int_part.trim_start_matches('-');
        let int_val: BigInt = if int_clean.is_empty() { BigInt::from(0) } else { int_clean.parse()? };
        let frac_val: BigInt = frac_part.parse()?;
        let denominator = BigInt::from(10).pow(frac_part.len() as u32);
        let value = BigRational::from(int_val) + BigRational::new(frac_val, denominator);
        return Ok(if negative { -value } else { value });
    }

    // Handle integer format: "100"
    Ok(BigRational::from(s.parse::<BigInt>()?))
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
        parse_rational(&self.authority_weight)
    }
    pub fn relevance_weight_rat(&self) -> BigRational {
        parse_rational(&self.relevance_weight)
    }
    pub fn conflict_penalty_base_rat(&self) -> BigRational {
        parse_rational(&self.conflict_penalty_base)
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
