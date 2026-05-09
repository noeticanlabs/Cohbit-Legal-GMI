use crate::manifold::LegalManifold;
use crate::witness::LegalWitness;
use crate::verifier::{AdmissibilityGauntlet, VerificationConfig, VerificationDecision};
use crate::receipt::{LegalReceipt, TransitionTrace};
use crate::error::GmiError;
use crate::adapter::LegalSourceAdapter;
use crate::ReceiptHash;

use num_rational::BigRational;
use num_traits::FromPrimitive;

/// The Stability Law: deltaV >= spend - defect (Numerical Sobriety: Rational-based)
pub fn is_stable(delta_v: &BigRational, spend: i64, defect: &BigRational) -> bool {
    let spend_rat = BigRational::from_i64(spend).unwrap();
    delta_v >= &(spend_rat - defect)
}

/// Apply a legal transition (witnessed claim) to the manifold
pub fn apply(
    mut manifold: LegalManifold,
    witness: LegalWitness,
    config: &VerificationConfig,
    spend: i64,
    adapter: Option<&dyn LegalSourceAdapter>,
) -> Result<(LegalManifold, LegalCohBit), GmiError> {
    let result = AdmissibilityGauntlet::evaluate(&witness, config, adapter);
    
    let pre_state_hash = manifold.get_state_hash();
    let last_receipt_hash = manifold.last_receipt_hash.clone();
    let valuation_pre = manifold.valuation.clone();

    let delta_v = result.valuation_delta_rat();
    let defect = result.defect_reserve_rat();

    if result.decision == VerificationDecision::Accept && is_stable(&delta_v, spend, &defect) {
        // Atomic state transition
        manifold.claims.push(witness.proposed_claim.clone());
        
        // Coherence gain reduces the valuation (uncertainty)
        // valuation_next = valuation_pre - delta_v
        let val_next = manifold.valuation_rat() - delta_v;
        manifold.valuation = val_next.to_string();

        manifold.pre_state_hash = pre_state_hash.clone();
        
        let post_state_hash = manifold.get_state_hash();
        manifold.post_state_hash = post_state_hash.clone();
        manifold.step_index += 1;

        let receipt_hash = manifold.compute_receipt_hash(&witness, &result);
        manifold.last_receipt_hash = Some(receipt_hash.clone());

        let trace = TransitionTrace {
            step_index: manifold.step_index,
            decision: result.decision.clone(),
            pre_state_hash: pre_state_hash.clone(),
            post_state_hash: post_state_hash.clone(),
            prev_receipt_hash: last_receipt_hash.clone(),
            receipt_hash: receipt_hash.clone(),
            delta_v: result.valuation_delta.clone(),
            spend,
            defect: result.defect_reserve.clone(),
            valuation_pre: valuation_pre.clone(),
            valuation_post: manifold.valuation.clone(),
            adapter_trace: result.adapter_trace.clone(),
        };

        let receipt = LegalReceipt::new(
            receipt_hash,
            pre_state_hash,
            post_state_hash,
            last_receipt_hash,
            result.decision.clone(),
            manifold.step_index,
            manifold.valuation.clone(),
            Some(trace),
        );

        Ok((manifold, LegalCohBit { receipt, verifier_result: result }))
    } else {
        // Transition rejected: manifold state remains mostly unchanged (valuation/claims frozen),
        // but we still increment step_index and emit a unique rejection trace/receipt.
        manifold.step_index += 1;
        let receipt_hash = manifold.compute_receipt_hash(&witness, &result);
        manifold.last_receipt_hash = Some(receipt_hash.clone());

        let trace = TransitionTrace {
            step_index: manifold.step_index,
            decision: VerificationDecision::Reject,
            pre_state_hash: pre_state_hash.clone(),
            post_state_hash: pre_state_hash.clone(), // No state change on reject
            prev_receipt_hash: last_receipt_hash.clone(),
            receipt_hash: receipt_hash.clone(),
            delta_v: result.valuation_delta.clone(),
            spend,
            defect: result.defect_reserve.clone(),
            valuation_pre: valuation_pre.clone(),
            valuation_post: valuation_pre.clone(), // No valuation change on reject
            adapter_trace: result.adapter_trace.clone(),
        };

        let receipt = LegalReceipt::new(
            receipt_hash,
            pre_state_hash.clone(),
            pre_state_hash,
            last_receipt_hash,
            VerificationDecision::Reject,
            manifold.step_index,
            manifold.valuation.clone(),
            Some(trace),
        );
        Ok((manifold, LegalCohBit { receipt, verifier_result: result }))
    }
}

#[derive(Debug)]
pub struct LegalCohBit {
    pub receipt: LegalReceipt,
    pub verifier_result: crate::verifier::VerifierResult,
}
