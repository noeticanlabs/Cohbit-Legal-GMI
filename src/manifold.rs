use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{
    claim::LegalClaim,
    reference::{Jurisdiction, LegalReference},
    EvidenceId, ReceiptHash, ReferenceId, StateHash,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalManifold {
    pub claims: Vec<LegalClaim>,
    #[serde(
        default,
        alias = "claim_statuses",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub claim_proof_status_at_admission: BTreeMap<crate::ClaimId, crate::proof::ProofStatus>,
    pub references: BTreeMap<ReferenceId, LegalReference>,
    pub jurisdiction_graph: Vec<JurisdictionEdge>,
    pub procedural_posture: ProceduralPosture,
    pub evidence: BTreeMap<EvidenceId, EvidenceItem>,

    // Core Physics (Numerical Sobriety: Rational-based)
    pub valuation: String,
    pub step_index: u64,

    pub pre_state_hash: StateHash,
    pub post_state_hash: StateHash,
    pub last_receipt_hash: Option<ReceiptHash>,

    #[serde(skip)]
    pub knowledge_trajectory: Vec<ReceiptHash>,
}

impl LegalManifold {
    pub fn valuation_rat(&self) -> Result<num_rational::BigRational, anyhow::Error> {
        crate::valuation::try_parse_rational(&self.valuation)
    }
}

/// A canonical view of the manifold's committed state for hashing
#[derive(Serialize)]
struct ManifoldCommitmentView<'a> {
    domain: &'static str,
    schema_version: &'static str,
    claims: &'a Vec<LegalClaim>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    claim_proof_status_at_admission: &'a BTreeMap<crate::ClaimId, crate::proof::ProofStatus>,
    references: &'a BTreeMap<ReferenceId, LegalReference>,
    jurisdiction_graph: &'a Vec<JurisdictionEdge>,
    valuation: &'a String,
    procedural_posture: &'a ProceduralPosture,
    evidence: &'a BTreeMap<EvidenceId, EvidenceItem>,
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
            claim_proof_status_at_admission: BTreeMap::new(),
            references: BTreeMap::new(),
            jurisdiction_graph: Vec::new(),
            procedural_posture,
            evidence: BTreeMap::new(),
            valuation: "1.0".to_string(), // Initial uncertainty normalized to 1.0
            step_index: 0,
            pre_state_hash: StateHash("genesis".to_string()),
            post_state_hash: StateHash("genesis".to_string()),
            last_receipt_hash: None,
            knowledge_trajectory: Vec::new(),
        }
    }

    pub fn get_state_hash(&self) -> StateHash {
        let view = ManifoldCommitmentView {
            domain: "cohbit-legal-gmi/content-state/v2",
            schema_version: "2.0.0",
            claims: &self.claims,
            claim_proof_status_at_admission: &self.claim_proof_status_at_admission,
            references: &self.references,
            jurisdiction_graph: &self.jurisdiction_graph,
            valuation: &self.valuation,
            procedural_posture: &self.procedural_posture,
            evidence: &self.evidence,
        };
        StateHash(crate::receipt::stable_hash(&view))
    }
}
