use serde::{Deserialize, Serialize};
use super::{ReceiptHash, StateHash};
use crate::verifier::VerificationDecision;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionTrace {
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
    pub adapter_trace: Option<crate::adapter::AdapterTrace>,
    pub witness: crate::witness::LegalWitness,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalReceipt {
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

impl LegalReceipt {
    pub fn new(
        receipt_hash: ReceiptHash,
        pre_state_hash: StateHash,
        post_state_hash: StateHash,
        last_receipt_hash: Option<ReceiptHash>,
        decision: VerificationDecision,
        step_index: u64,
        post_valuation: String,
        trace: Option<TransitionTrace>,
    ) -> Self {
        Self {
            receipt_hash,
            pre_state_hash,
            post_state_hash,
            last_receipt_hash,
            decision,
            step_index,
            post_valuation,
            trace,
        }
    }
}

pub fn stable_hash<T: serde::Serialize>(t: &T) -> String {
    // RFAP-04: Determinism First. 
    // Ensure canonical JSON by converting to Value and back to String.
    let value = serde_json::to_value(t).expect("Stable hash: Serialization to Value failed");
    let json = serde_json::to_string(&value).expect("Stable hash: Deterministic serialization failed");
    let hash = blake3::hash(json.as_bytes());
    hash.to_hex().to_string()
}
