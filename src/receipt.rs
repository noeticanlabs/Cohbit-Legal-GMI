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
    pub delta_v: i64,
    pub spend: i64,
    pub defect: i64,
    pub valuation_pre: i64,
    pub valuation_post: i64,
    pub adapter_trace: Option<crate::adapter::AdapterTrace>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalReceipt {
    pub receipt_hash: ReceiptHash,
    pub pre_state_hash: StateHash,
    pub post_state_hash: StateHash,
    pub last_receipt_hash: Option<ReceiptHash>,
    pub decision: VerificationDecision,
    pub step_index: u64,
    pub post_valuation: i64,
    
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
        post_valuation: i64,
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
    let value = serde_json::to_value(t).unwrap_or(serde_json::Value::Null);
    let json = serde_json::to_string(&value).unwrap_or_default();
    let hash = blake3::hash(json.as_bytes());
    hash.to_hex().to_string()
}
