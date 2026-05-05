// Copyright (c) 2026 CohBit. All rights reserved. Proprietary and Confidential.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClaimId(pub String);

impl From<&str> for ClaimId {
    fn from(s: &str) -> Self {
        ClaimId(s.to_string())
    }
}

impl From<String> for ClaimId {
    fn from(s: String) -> Self {
        ClaimId(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReferenceId(pub String);

impl From<&str> for ReferenceId {
    fn from(s: &str) -> Self {
        ReferenceId(s.to_string())
    }
}

impl From<String> for ReferenceId {
    fn from(s: String) -> Self {
        ReferenceId(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EvidenceId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReceiptHash(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StateHash(pub String);

pub mod claim;
pub mod reference;
pub mod manifold;
pub mod witness;
pub mod verifier;
pub mod receipt;
pub mod cohbit;
pub mod error;
pub mod valuation;
pub mod adapter;
pub mod proposal;
pub mod explorer;
pub mod membrane;
pub mod npe;
pub mod memory;
pub mod persistence;
pub mod anchoring;
pub mod bundle;

pub use claim::*;
pub use reference::*;
pub use manifold::*;
pub use witness::*;
pub use verifier::*;
pub use receipt::*;
pub use cohbit::*;
pub use error::*;
pub use valuation::*;
pub use adapter::*;
pub use proposal::*;
pub use explorer::*;
pub use membrane::*;
pub use npe::*;
pub use memory::*;
pub use persistence::*;
pub use anchoring::*;
pub use bundle::*;

#[cfg(test)]
mod tests;
