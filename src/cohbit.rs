use crate::manifold::LegalManifold;
use crate::witness::LegalWitness;
use crate::verifier::{AdmissibilityGauntlet, VerificationConfig, VerificationDecision};
use crate::receipt::{LegalReceipt, TransitionTrace};
use crate::error::GmiError;
use crate::adapter::LegalSourceAdapter;

use num_rational::BigRational;
use num_traits::FromPrimitive;

/// The Stability Law: deltaV >= spend - defect (Numerical Sobriety: Rational-based)
pub fn is_stable(delta_v: &BigRational, spend: &BigRational, defect: &BigRational) -> bool {
    delta_v >= &(spend - defect)
}

/// Apply a legal transition (witnessed claim) to the manifold
pub fn apply(
    mut manifold: LegalManifold,
    witness: LegalWitness,
    config: &VerificationConfig,
    spend: BigRational,
    defect: BigRational,
    adapter: Option<&dyn LegalSourceAdapter>,
) -> Result<(LegalManifold, LegalCohBit), GmiError> {
    let mut result = AdmissibilityGauntlet::evaluate(&witness, config, adapter);
    
    let pre_state_hash = manifold.get_state_hash();
    let last_receipt_hash = manifold.last_receipt_hash.clone();
    let valuation_pre = manifold.valuation.clone();

    let delta_v = result.valuation_delta_rat();
    
    // Wire defect into result if provided externally
    if defect > BigRational::from_i64(0).unwrap() {
        result.defect_reserve = defect.to_string();
    }
    let actual_defect = result.defect_reserve_rat();

    if result.decision == VerificationDecision::Accept && is_stable(&delta_v, &spend, &actual_defect) {
        // Atomic state transition
        manifold.claims.push(witness.proposed_claim.clone());
        
        // Coherence gain reduces the valuation (uncertainty)
        let val_next = manifold.valuation_rat() - delta_v;
        manifold.valuation = val_next.to_string();

        manifold.pre_state_hash = pre_state_hash.clone();
        
        // Increment metadata BEFORE computing post_state_hash if it were part of it, 
        // but since we moved to content-only hash, we just need to be consistent.
        manifold.step_index += 1;
        
        // Compute hashes after state update
        let post_state_hash = manifold.get_state_hash();
        manifold.post_state_hash = post_state_hash.clone();

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
            spend: spend.to_string(),
            defect: result.defect_reserve.clone(),
            valuation_pre: valuation_pre.clone(),
            valuation_post: manifold.valuation.clone(),
            adapter_trace: result.adapter_trace.clone(),
            witness: witness.clone(),
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
        // Transition rejected: manifold state remains unchanged (valuation/claims frozen),
        // but we still increment step_index and emit a unique rejection trace/receipt.
        manifold.step_index += 1;
        
        let post_state_hash = pre_state_hash.clone(); // Rejection is state-idempotent
        manifold.pre_state_hash = pre_state_hash.clone();
        manifold.post_state_hash = post_state_hash.clone();

        let receipt_hash = manifold.compute_receipt_hash(&witness, &result);
        manifold.last_receipt_hash = Some(receipt_hash.clone());

        let trace = TransitionTrace {
            step_index: manifold.step_index,
            decision: VerificationDecision::Reject,
            pre_state_hash: pre_state_hash.clone(),
            post_state_hash: post_state_hash.clone(),
            prev_receipt_hash: last_receipt_hash.clone(),
            receipt_hash: receipt_hash.clone(),
            delta_v: result.valuation_delta.clone(),
            spend: spend.to_string(),
            defect: result.defect_reserve.clone(),
            valuation_pre: valuation_pre.clone(),
            valuation_post: valuation_pre.clone(),
            adapter_trace: result.adapter_trace.clone(),
            witness: witness.clone(),
        };

        let receipt = LegalReceipt::new(
            receipt_hash,
            pre_state_hash.clone(),
            post_state_hash,
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
