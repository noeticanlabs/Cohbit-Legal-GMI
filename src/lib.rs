// Copyright (c) 2026 CohBit. All rights reserved.
// SPDX-License-Identifier: AGPL-3.0-only
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EvidenceId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ReceiptHash(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StateHash(pub String);

pub mod adapter;
pub mod anchoring;
pub mod bundle;
pub mod civic;
pub mod claim;
pub mod cohbit;
pub mod error;
pub mod explorer;
pub mod manifold;
pub mod membrane;
pub mod memory;
pub mod npe;
pub mod persistence;
pub mod privacy;
pub mod proof;
pub mod proposal;
pub mod receipt;
pub mod reference;
pub mod security;
pub mod sources;
pub mod temporal;
pub mod valuation;
pub mod verifier;
pub mod witness;

pub use adapter::*;
pub use anchoring::*;
pub use bundle::*;
pub use civic::*;
pub use claim::*;
pub use cohbit::*;
pub use error::*;
pub use explorer::*;
pub use manifold::*;
pub use membrane::*;
pub use memory::*;
pub use npe::*;
pub use persistence::*;
pub use privacy::*;
pub use proposal::*;
pub use receipt::*;
pub use reference::*;
pub use security::*;
pub use sources::*;
pub use temporal::*;
pub use valuation::*;
pub use verifier::*;
pub use witness::*;

#[cfg(test)]
mod tests;
