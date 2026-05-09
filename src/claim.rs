use serde::{Deserialize, Serialize};
use crate::{ClaimId, ReferenceId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalClaim {
    pub id: ClaimId,
    pub text: String,
    pub cited_reference: Option<ReferenceId>,
    pub quoted_text: Option<String>,
    pub authority_label: AuthorityLabel,
    pub inference: InferenceKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthorityLabel {
    Binding,
    Persuasive,
    Dictum,
    Advisory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InferenceKind {
    DirectQuotation,
    RuleApplication {
        rule_claims: Vec<ClaimId>,
        fact_claims: Vec<ClaimId>,
    },
    BindingPrecedent,
    PersuasiveAuthority,
    UnsupportedHypothesis,
}
impl Default for AuthorityLabel {
    fn default() -> Self {
        AuthorityLabel::Advisory
    }
}

impl Default for InferenceKind {
    fn default() -> Self {
        InferenceKind::UnsupportedHypothesis
    }
}

impl Default for LegalClaim {
    fn default() -> Self {
        Self {
            id: crate::ClaimId("default".into()),
            text: "".into(),
            cited_reference: None,
            quoted_text: None,
            authority_label: AuthorityLabel::default(),
            inference: InferenceKind::default(),
        }
    }
}
