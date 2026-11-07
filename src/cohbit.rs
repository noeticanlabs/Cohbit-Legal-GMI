use crate::adapter::LegalSourceAdapter;
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

/// Computational/simulation admission only. Accept is not legal certification.
pub fn apply(
    manifold: LegalManifold,
    witness: LegalWitness,
    config: &VerificationConfig,
    spend: BigRational,
    defect: BigRational,
    adapter: Option<&dyn LegalSourceAdapter>,
) -> Result<(LegalManifold, LegalCohBit), GmiError> {
    apply_inner(manifold, witness, config, spend, defect, adapter, None)
}

fn apply_inner(
    mut manifold: LegalManifold,
    witness: LegalWitness,
    config: &VerificationConfig,
    spend: BigRational,
    defect: BigRational,
    adapter: Option<&dyn LegalSourceAdapter>,
    proof: Option<crate::proof::CheckedProof>,
) -> Result<(LegalManifold, LegalCohBit), GmiError> {
    let policy_version = if proof.is_some() {
        crate::proof::PROOF_POLICY
    } else {
        "admission-v2"
    };
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
    let mut result = if proof.is_some() {
        let checked = crate::verifier::CheckResult {
            passed: true,
            reason: "Authenticated scoped context and independent Lean derivation; see proof_status and legal_proof".into(),
        };
        crate::verifier::VerifierResult {
            c1_citation_exists: checked.clone(),
            c2_citation_binding: checked.clone(),
            c3_claim_source_alignment: checked.clone(),
            c4_jurisdictional_validity: checked.clone(),
            c5_procedural_status: checked.clone(),
            c6_inference_validity: checked.clone(),
            c7_conflict_disclosure: checked,
            adapter_trace: None,
            decision: VerificationDecision::Accept,
            valuation_delta: "0".into(),
            defect_reserve: "0".into(),
            evidence_support: "1".into(),
            verified_evidence_links: vec![],
        }
    } else {
        AdmissibilityGauntlet::evaluate_with_claims(
            &witness,
            &effective_config,
            adapter,
            &manifold.claims,
        )?
    };

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
        manifold.claim_proof_status_at_admission.insert(
            witness.proposed_claim.id.clone(),
            proof
                .as_ref()
                .map(|p| p.status().clone())
                .unwrap_or(crate::proof::ProofStatus::Unknown),
        );

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
            policy_version: policy_version.into(),
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
            legal_proof: proof.as_ref().map(|p| p.artifact().clone()),
            proof_status: proof.as_ref().map(|p| p.status().clone()),
        };
        let receipt_hash = canonical_receipt_hash(&trace);
        trace.receipt_hash = receipt_hash.clone();
        manifold.last_receipt_hash = Some(receipt_hash.clone());

        let receipt = LegalReceipt {
            schema_version: "2.0.0".into(),
            policy_version: policy_version.into(),
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
            policy_version: policy_version.into(),
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
            legal_proof: proof.as_ref().map(|p| p.artifact().clone()),
            proof_status: proof.as_ref().map(|p| p.status().clone()),
        };
        let receipt_hash = canonical_receipt_hash(&trace);
        trace.receipt_hash = receipt_hash.clone();
        manifold.last_receipt_hash = Some(receipt_hash.clone());

        let receipt = LegalReceipt {
            schema_version: "2.0.0".into(),
            policy_version: policy_version.into(),
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

/// Legacy API fails closed: an adapter trust label is not a legal proof.
pub fn apply_verified(
    _manifold: LegalManifold,
    _witness: LegalWitness,
    _config: &VerificationConfig,
    _spend: BigRational,
    _defect: BigRational,
    _adapter: &dyn LegalSourceAdapter,
) -> Result<(LegalManifold, LegalCohBit), GmiError> {
    Err(GmiError::VerificationFailed(
        "Use apply_certified with an independently checked proof certificate".into(),
    ))
}

pub struct CertifiedAdmission<'a> {
    pub artifact: &'a crate::proof::ProofArtifact,
    pub policy: &'a crate::proof::TrustPolicy,
}

/// The only certified admission path. Proof input and trust roots are separate.
pub fn apply_certified(
    manifold: LegalManifold,
    witness: LegalWitness,
    config: &VerificationConfig,
    spend: BigRational,
    defect: BigRational,
    admission: CertifiedAdmission<'_>,
) -> Result<(LegalManifold, LegalCohBit), GmiError> {
    let c = &admission.artifact.context;
    if c.state_hash != manifold.get_state_hash().0
        || c.posture_hash != crate::receipt::stable_hash(&manifold.procedural_posture)
        || c.claim_id != witness.proposed_claim.id.0
        || c.claim_text != witness.proposed_claim.text
        || c.claim_hash != crate::receipt::stable_hash(&witness.proposed_claim)
        || c.verification_time != config.verification_time
        || config.target_jurisdiction.as_ref().is_none_or(|j| {
            crate::receipt::stable_hash(j)
                != crate::receipt::stable_hash(&manifold.procedural_posture.forum)
        })
    {
        return Err(GmiError::VerificationFailed(
            "Proof is bound to a different claim, state, time or forum".into(),
        ));
    }
    let checked = admission
        .artifact
        .check(admission.policy, &config.verification_time)
        .map_err(|e| GmiError::VerificationFailed(e.to_string()))?;
    apply_inner(
        manifold,
        witness,
        config,
        spend,
        defect,
        None,
        Some(checked),
    )
}

#[derive(Debug)]
pub struct LegalCohBit {
    pub receipt: LegalReceipt,
    pub verifier_result: crate::verifier::VerifierResult,
}
