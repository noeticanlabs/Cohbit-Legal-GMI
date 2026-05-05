use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

use crate::{
    claim::LegalClaim,
    ClaimId, EvidenceId, ReceiptHash, ReferenceId, StateHash,
    reference::{Jurisdiction, LegalReference},
    witness::LegalWitness,
    verifier::VerifierResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalManifold {
    pub claims: Vec<LegalClaim>,
    pub references: BTreeMap<ReferenceId, LegalReference>,
    pub jurisdiction_graph: Vec<JurisdictionEdge>,
    pub procedural_posture: ProceduralPosture,
    pub evidence: BTreeMap<EvidenceId, EvidenceItem>,
    
    // Core Physics
    pub valuation: u128,
    pub step_index: u64,
    
    pub pre_state_hash: StateHash,
    pub post_state_hash: StateHash,
    pub last_receipt_hash: Option<ReceiptHash>,

    #[serde(skip)]
    pub knowledge_trajectory: Vec<ReceiptHash>,
}

/// A canonical view of the manifold's committed state for hashing
#[derive(Serialize)]
struct ManifoldCommitmentView<'a> {
    claims: &'a Vec<LegalClaim>,
    references: &'a BTreeMap<ReferenceId, LegalReference>,
    valuation: u128,
    step_index: u64,
    last_receipt_hash: &'a Option<ReceiptHash>,
    procedural_posture: &'a ProceduralPosture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JurisdictionEdge {
    pub from: Jurisdiction,
    pub to: Jurisdiction,
    pub relation: JurisdictionRelation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JurisdictionRelation {
    BindingOn,
    PersuasiveIn,
    NotAuthorityFor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceduralPosture {
    pub forum: Jurisdiction,
    pub matter_type: String,
    pub stage: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub id: EvidenceId,
    pub label: String,
    pub text: Option<String>,
    pub source_path: Option<String>,
}

impl LegalManifold {
    pub fn new(procedural_posture: ProceduralPosture) -> Self {
        Self {
            claims: Vec::new(),
            references: BTreeMap::new(),
            jurisdiction_graph: Vec::new(),
            procedural_posture,
            evidence: BTreeMap::new(),
            valuation: 1000000,
            step_index: 0,
            pre_state_hash: StateHash("genesis".to_string()),
            post_state_hash: StateHash("genesis".to_string()),
            last_receipt_hash: None,
            knowledge_trajectory: Vec::new(),
        }
    }

    pub fn get_state_hash(&self) -> StateHash {
        let view = ManifoldCommitmentView {
            claims: &self.claims,
            references: &self.references,
            valuation: self.valuation,
            step_index: self.step_index,
            last_receipt_hash: &self.last_receipt_hash,
            procedural_posture: &self.procedural_posture,
        };
        let json = serde_json::to_string(&view).unwrap_or_default();
        StateHash(blake3::hash(json.as_bytes()).to_hex().to_string())
    }

    pub fn compute_receipt_hash(&self, witness: &LegalWitness, result: &VerifierResult) -> ReceiptHash {
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.pre_state_hash.0.as_bytes());
        hasher.update(self.post_state_hash.0.as_bytes());
        hasher.update(serde_json::to_string(witness).unwrap_or_default().as_bytes());
        hasher.update(serde_json::to_string(result).unwrap_or_default().as_bytes());
        
        if let Some(last) = &self.last_receipt_hash {
            hasher.update(last.0.as_bytes());
        }

        ReceiptHash(hasher.finalize().to_hex().to_string())
    }
}
