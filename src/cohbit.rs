use crate::manifold::LegalManifold;
use crate::witness::LegalWitness;
use crate::verifier::{AdmissibilityGauntlet, VerificationConfig, VerificationDecision};
use crate::receipt::{LegalReceipt, TransitionTrace};
use crate::error::GmiError;
use crate::adapter::LegalSourceAdapter;
use crate::ReceiptHash;

/// The Stability Law: deltaV >= spend - defect
pub fn is_stable(delta_v: i64, spend: i64, defect: i64) -> bool {
    delta_v >= (spend - defect)
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
    let valuation_pre = manifold.valuation;

    if result.decision == VerificationDecision::Accept && is_stable(result.valuation_delta, spend, result.defect_reserve) {
        // Atomic state transition
        manifold.claims.push(witness.proposed_claim.clone());
        
        // Coherence gain reduces the valuation (uncertainty)
        if result.valuation_delta >= 0 {
            manifold.valuation = manifold.valuation.saturating_sub(result.valuation_delta as u128);
        } else {
            // Negative coherence gain (net penalty) increases valuation (uncertainty)
            manifold.valuation = manifold.valuation.saturating_add(result.valuation_delta.abs() as u128);
        }

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
            delta_v: result.valuation_delta,
            spend,
            defect: result.defect_reserve,
            valuation_pre: valuation_pre as i64,
            valuation_post: manifold.valuation as i64,
            adapter_trace: result.adapter_trace.clone(),
        };

        let receipt = LegalReceipt::new(
            receipt_hash,
            pre_state_hash,
            post_state_hash,
            last_receipt_hash,
            result.decision.clone(),
            manifold.step_index,
            manifold.valuation as i64,
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
            delta_v: result.valuation_delta,
            spend,
            defect: result.defect_reserve,
            valuation_pre: valuation_pre as i64,
            valuation_post: valuation_pre as i64, // No valuation change on reject
            adapter_trace: result.adapter_trace.clone(),
        };

        let receipt = LegalReceipt::new(
            receipt_hash,
            pre_state_hash.clone(),
            pre_state_hash,
            last_receipt_hash,
            VerificationDecision::Reject,
            manifold.step_index,
            manifold.valuation as i64,
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
