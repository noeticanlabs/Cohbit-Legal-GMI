use crate::adapter::TreatmentType;
use crate::verifier::VerifierResult;
use crate::witness::LegalWitness;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};

pub fn try_parse_rational(s: &str) -> Result<BigRational, anyhow::Error> {
    let s = s.trim();
    if s.is_empty() {
        return Err(anyhow::anyhow!("Rational value is empty"));
    }
    if s.len() > 256 {
        return Err(anyhow::anyhow!("Rational value exceeds 256 characters"));
    }
    if s.matches('/').count() > 1 || s.matches('.').count() > 1 {
        return Err(anyhow::anyhow!("Malformed rational: {}", s));
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
        let int_val: BigInt = if int_clean.is_empty() {
            BigInt::from(0)
        } else {
            int_clean.parse()?
        };
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
            relevance_weight: "1/10".into(),
            conflict_penalty_base: "1/2".into(),
        }
    }
}

impl ValuationConfig {
    pub fn authority_weight_rat(&self) -> Result<BigRational, anyhow::Error> {
        try_parse_rational(&self.authority_weight)
    }
    pub fn relevance_weight_rat(&self) -> Result<BigRational, anyhow::Error> {
        try_parse_rational(&self.relevance_weight)
    }
    pub fn conflict_penalty_base_rat(&self) -> Result<BigRational, anyhow::Error> {
        try_parse_rational(&self.conflict_penalty_base)
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
) -> Result<BigRational, anyhow::Error> {
    // RFAP-12: Numerical Sobriety. No floats in consensus.
    let base_support = try_parse_rational(&verifier_result.evidence_support)?;

    // A proposal's authority label is untrusted metadata. It cannot increase
    // valuation until jurisdiction and authority are independently established.
    let authority_multiplier = BigRational::from_i64(1).unwrap();

    // Coherence gain represents the reduction in uncertainty
    let base_gain = BigRational::from_i64(1).unwrap();

    // Penalties reduce the coherence gain
    let conflict_penalty = config.conflict_penalty_base_rat()?;
    let authority_weight = config.authority_weight_rat()?;
    let relevance_weight = config.relevance_weight_rat()?;
    let zero = BigRational::from_integer(0.into());
    let one = BigRational::from_integer(1.into());
    if conflict_penalty < zero || conflict_penalty > one {
        return Err(anyhow::anyhow!("Conflict penalty must be within [0, 1]"));
    }
    if authority_weight < zero || authority_weight > one {
        return Err(anyhow::anyhow!("Authority weight must be within [0, 1]"));
    }
    if relevance_weight < zero || relevance_weight > one {
        return Err(anyhow::anyhow!("Relevance weight must be within [0, 1]"));
    }
    let mut penalty = BigRational::from_i64(witness.contrary_authorities.len() as i64).unwrap()
        * conflict_penalty;

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
    let support_bonus =
        base_gain * base_support * authority_multiplier * authority_weight * relevance_weight;

    Ok(support_bonus - penalty)
}
