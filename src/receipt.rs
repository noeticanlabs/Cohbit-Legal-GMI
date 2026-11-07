use super::{ReceiptHash, StateHash};
use crate::verifier::VerificationDecision;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionTrace {
    pub schema_version: String,
    pub policy_version: String,
    pub step_index: u64,
    pub decision: VerificationDecision,
    pub pre_state_hash: StateHash,
    pub post_state_hash: StateHash,
    pub prev_receipt_hash: Option<ReceiptHash>,
    pub receipt_hash: ReceiptHash,
    pub delta_v: String,
    pub spend: String,
    pub defect: String,
    pub valuation_pre: String,
    pub valuation_post: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_config_hash: Option<String>,
    pub adapter_trace: Option<crate::adapter::AdapterTrace>,
    pub witness: crate::witness::LegalWitness,
    pub verifier_result: crate::verifier::VerifierResult,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legal_proof: Option<crate::proof::ProofArtifact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof_status: Option<crate::proof::ProofStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalReceipt {
    pub schema_version: String,
    pub policy_version: String,
    pub receipt_hash: ReceiptHash,
    pub pre_state_hash: StateHash,
    pub post_state_hash: StateHash,
    pub last_receipt_hash: Option<ReceiptHash>,
    pub decision: VerificationDecision,
    pub step_index: u64,
    pub post_valuation: String,

    // M8: Transition Trace for Formal Verification
    pub trace: Option<TransitionTrace>,
}

#[derive(Serialize)]
struct ReceiptCommitmentView<'a> {
    domain: &'static str,
    schema_version: &'a str,
    policy_version: &'a str,
    step_index: u64,
    decision: &'a VerificationDecision,
    pre_state_hash: &'a StateHash,
    post_state_hash: &'a StateHash,
    previous_receipt_hash: &'a Option<ReceiptHash>,
    valuation_pre: &'a str,
    valuation_post: &'a str,
    spend: &'a str,
    defect: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    verification_config_hash: &'a Option<String>,
    witness: &'a crate::witness::LegalWitness,
    verifier_result: &'a crate::verifier::VerifierResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    legal_proof: &'a Option<crate::proof::ProofArtifact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    proof_status: &'a Option<crate::proof::ProofStatus>,
}

pub fn canonical_receipt_hash(trace: &TransitionTrace) -> ReceiptHash {
    ReceiptHash(stable_hash(&ReceiptCommitmentView {
        domain: "cohbit-legal-gmi/receipt/v2",
        schema_version: &trace.schema_version,
        policy_version: &trace.policy_version,
        step_index: trace.step_index,
        decision: &trace.decision,
        pre_state_hash: &trace.pre_state_hash,
        post_state_hash: &trace.post_state_hash,
        previous_receipt_hash: &trace.prev_receipt_hash,
        valuation_pre: &trace.valuation_pre,
        valuation_post: &trace.valuation_post,
        spend: &trace.spend,
        defect: &trace.defect,
        verification_config_hash: &trace.verification_config_hash,
        witness: &trace.witness,
        verifier_result: &trace.verifier_result,
        legal_proof: &trace.legal_proof,
        proof_status: &trace.proof_status,
    }))
}

impl LegalReceipt {
    pub fn verify_canonical(&self) -> Result<(), String> {
        let trace = self
            .trace
            .as_ref()
            .ok_or_else(|| "Receipt has no transition trace".to_string())?;
        if self.schema_version != trace.schema_version
            || self.policy_version != trace.policy_version
            || self.step_index != trace.step_index
            || self.decision != trace.decision
            || self.pre_state_hash != trace.pre_state_hash
            || self.post_state_hash != trace.post_state_hash
            || self.last_receipt_hash != trace.prev_receipt_hash
            || self.post_valuation != trace.valuation_post
            || trace.decision != trace.verifier_result.decision
        {
            return Err("Receipt fields do not correspond to the transition trace".into());
        }
        let expected = canonical_receipt_hash(trace);
        if self.receipt_hash != expected || trace.receipt_hash != expected {
            return Err("Receipt hash is not the canonical hash of its trace".into());
        }
        Ok(())
    }
}

pub fn stable_hash<T: serde::Serialize>(t: &T) -> String {
    // RFAP-04: Determinism First.
    // Ensure canonical JSON by converting to Value and back to String.
    let value = serde_json::to_value(t).expect("Stable hash: Serialization to Value failed");
    let json =
        serde_json::to_string(&value).expect("Stable hash: Deterministic serialization failed");
    let hash = blake3::hash(json.as_bytes());
    hash.to_hex().to_string()
}
