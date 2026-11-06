use crate::adapter::{AdapterTrust, LegalSourceAdapter};
use crate::error::GmiError;
use crate::manifold::LegalManifold;
use crate::receipt::{canonical_receipt_hash, LegalReceipt, TransitionTrace};
use crate::verifier::{AdmissibilityGauntlet, VerificationConfig, VerificationDecision};
use crate::witness::LegalWitness;

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
    if manifold
        .claims
        .iter()
        .any(|claim| claim.id == witness.proposed_claim.id)
    {
        return Err(GmiError::TransitionRejected(format!(
            "Claim id {} is already committed",
            witness.proposed_claim.id.0
        )));
    }
    let zero = BigRational::from_i64(0).unwrap();
    if spend < zero || defect < zero {
        return Err(GmiError::InvalidNumeric(
            "Spend and defect reserve must be non-negative".into(),
        ));
    }
    let mut effective_config = config.clone();
    if effective_config.target_jurisdiction.is_none() {
        effective_config.target_jurisdiction = Some(manifold.procedural_posture.forum.clone());
    }
    let mut result = AdmissibilityGauntlet::evaluate_with_claims(
        &witness,
        &effective_config,
        adapter,
        &manifold.claims,
    )?;

    let pre_state_hash = manifold.get_state_hash();
    let last_receipt_hash = manifold.last_receipt_hash.clone();
    let valuation_pre = manifold.valuation.clone();

    let delta_v = result
        .valuation_delta_rat()
        .map_err(|e| GmiError::InvalidNumeric(e.to_string()))?;

    // Wire defect into result if provided externally
    if defect > BigRational::from_i64(0).unwrap() {
        result.defect_reserve = defect.to_string();
    }
    let actual_defect = result
        .defect_reserve_rat()
        .map_err(|e| GmiError::InvalidNumeric(e.to_string()))?;

    let current_valuation = manifold
        .valuation_rat()
        .map_err(|e| GmiError::InvalidNumeric(e.to_string()))?;
    if result.decision == VerificationDecision::Accept
        && is_stable(&delta_v, &spend, &actual_defect)
        && delta_v <= current_valuation
    {
        // Atomic state transition
        manifold.claims.push(witness.proposed_claim.clone());

        // Coherence gain reduces the valuation (uncertainty)
        let val_next = current_valuation - delta_v;
        manifold.valuation = val_next.to_string();

        manifold.pre_state_hash = pre_state_hash.clone();

        // Increment metadata BEFORE computing post_state_hash if it were part of it,
        // but since we moved to content-only hash, we just need to be consistent.
        manifold.step_index += 1;

        // Compute hashes after state update
        let post_state_hash = manifold.get_state_hash();
        manifold.post_state_hash = post_state_hash.clone();

        let mut trace = TransitionTrace {
            schema_version: "2.0.0".into(),
            policy_version: "admission-v2".into(),
            step_index: manifold.step_index,
            decision: result.decision.clone(),
            pre_state_hash: pre_state_hash.clone(),
            post_state_hash: post_state_hash.clone(),
            prev_receipt_hash: last_receipt_hash.clone(),
            receipt_hash: crate::ReceiptHash(String::new()),
            delta_v: result.valuation_delta.clone(),
            spend: spend.to_string(),
            defect: result.defect_reserve.clone(),
            valuation_pre: valuation_pre.clone(),
            valuation_post: manifold.valuation.clone(),
            adapter_trace: result.adapter_trace.clone(),
            witness: witness.clone(),
            verifier_result: result.clone(),
        };
        let receipt_hash = canonical_receipt_hash(&trace);
        trace.receipt_hash = receipt_hash.clone();
        manifold.last_receipt_hash = Some(receipt_hash.clone());

        let receipt = LegalReceipt {
            schema_version: "2.0.0".into(),
            policy_version: "admission-v2".into(),
            receipt_hash,
            pre_state_hash,
            post_state_hash,
            last_receipt_hash,
            decision: result.decision.clone(),
            step_index: manifold.step_index,
            post_valuation: manifold.valuation.clone(),
            trace: Some(trace),
        };

        Ok((
            manifold,
            LegalCohBit {
                receipt,
                verifier_result: result,
            },
        ))
    } else {
        if result.decision == VerificationDecision::Accept {
            result.decision = VerificationDecision::Reject;
        }
        // Transition rejected: manifold state remains unchanged (valuation/claims frozen),
        // but we still increment step_index and emit a unique rejection trace/receipt.
        manifold.step_index += 1;

        let post_state_hash = pre_state_hash.clone(); // Rejection is state-idempotent
        manifold.pre_state_hash = pre_state_hash.clone();
        manifold.post_state_hash = post_state_hash.clone();

        let mut trace = TransitionTrace {
            schema_version: "2.0.0".into(),
            policy_version: "admission-v2".into(),
            step_index: manifold.step_index,
            decision: result.decision.clone(),
            pre_state_hash: pre_state_hash.clone(),
            post_state_hash: post_state_hash.clone(),
            prev_receipt_hash: last_receipt_hash.clone(),
            receipt_hash: crate::ReceiptHash(String::new()),
            delta_v: result.valuation_delta.clone(),
            spend: spend.to_string(),
            defect: result.defect_reserve.clone(),
            valuation_pre: valuation_pre.clone(),
            valuation_post: valuation_pre.clone(),
            adapter_trace: result.adapter_trace.clone(),
            witness: witness.clone(),
            verifier_result: result.clone(),
        };
        let receipt_hash = canonical_receipt_hash(&trace);
        trace.receipt_hash = receipt_hash.clone();
        manifold.last_receipt_hash = Some(receipt_hash.clone());

        let receipt = LegalReceipt {
            schema_version: "2.0.0".into(),
            policy_version: "admission-v2".into(),
            receipt_hash,
            pre_state_hash: pre_state_hash.clone(),
            post_state_hash,
            last_receipt_hash,
            decision: result.decision.clone(),
            step_index: manifold.step_index,
            post_valuation: manifold.valuation.clone(),
            trace: Some(trace),
        };
        Ok((
            manifold,
            LegalCohBit {
                receipt,
                verifier_result: result,
            },
        ))
    }
}

/// Production admission entry point. Its type requires an independent source
/// adapter; callers that cannot supply one cannot accidentally request a
/// verified commitment.
pub fn apply_verified(
    manifold: LegalManifold,
    witness: LegalWitness,
    config: &VerificationConfig,
    spend: BigRational,
    defect: BigRational,
    adapter: &dyn LegalSourceAdapter,
) -> Result<(LegalManifold, LegalCohBit), GmiError> {
    if adapter.trust_level() != AdapterTrust::Verified {
        return Err(GmiError::VerificationFailed(format!(
            "Adapter {} is {:?}; verified admission requires a Verified adapter",
            adapter.name(),
            adapter.trust_level()
        )));
    }
    if config.allow_unknown_good_law || !config.require_conflict_disclosure {
        return Err(GmiError::VerificationFailed(
            "Verified admission requires unknown authority to fail closed and adverse-authority disclosure to be enabled".into(),
        ));
    }
    apply(manifold, witness, config, spend, defect, Some(adapter))
}

#[derive(Debug)]
pub struct LegalCohBit {
    pub receipt: LegalReceipt,
    pub verifier_result: crate::verifier::VerifierResult,
}
