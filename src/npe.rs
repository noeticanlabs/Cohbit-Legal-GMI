use serde::{Deserialize, Serialize};
use crate::proposal::{LegalProposal, CandidateClaim};
use crate::claim::{AuthorityLabel, InferenceKind};
use crate::error::GmiError;
use crate::memory::LegalNpeMemory;

/// The LLM Hook Trait
/// 
/// Allows the NPE to swap between a synthetic generator, a local model, 
/// or a remote API.
pub trait LegalLlmHook {
    fn generate_proposal(&self, prompt: &str, memory_guidance: &str) -> Result<LegalProposal, GmiError>;
}

/// Neural Proposal Engine (NPE) for Legal GMI
/// 
/// The NPE generates candidate legal trajectories by interacting with an LLM
/// through a secure hook. It uses adaptive memory to guide its brainstorming.
pub struct LegalNpe<H: LegalLlmHook> {
    pub hook: H,
    pub memory: LegalNpeMemory,
}

impl<H: LegalLlmHook> LegalNpe<H> {
    pub fn new(hook: H) -> Self {
        Self { 
            hook,
            memory: LegalNpeMemory::new(),
        }
    }

    pub fn brainstorm(&self, context: &str) -> Result<LegalProposal, GmiError> {
        // Build memory guidance string for the LLM
        let guidance = format!(
            "Total steps: {}, Success rate: {:.2}. Hallucination traps to avoid: {:?}",
            self.memory.total_steps,
            if self.memory.total_steps == 0 { 0.0 } else { self.memory.total_success as f64 / self.memory.total_steps as f64 },
            self.memory.hallucination_traps.keys().take(10).collect::<Vec<_>>()
        );

        self.hook.generate_proposal(context, &guidance)
    }

    pub fn learn(&mut self, receipt: &crate::receipt::LegalReceipt) {
        self.memory.ingest(receipt);
    }
}

/// A Synthetic Mock NPE for Testing and Bootstrapping
pub struct SyntheticLegalNpe;

impl LegalLlmHook for SyntheticLegalNpe {
    fn generate_proposal(&self, _prompt: &str, _guidance: &str) -> Result<LegalProposal, GmiError> {
        // Simulates an LLM generating a complex legal proposal
        Ok(LegalProposal {
            proposal_id: "synthetic_prop_1".into(),
            title: "Fair Use Analysis".into(),
            description: "Evaluating the candidate claims under the four-factor test.".into(),
            candidate_claims: vec![
                CandidateClaim {
                    text: "The purpose and character of the use is transformative.".into(),
                    citation: Some("17 U.S.C. § 107(1)".into()),
                    quoted_text: Some("purpose and character of the use".into()),
                    authority_label: AuthorityLabel::Binding,
                    inference: InferenceKind::DirectQuotation,
                    model_confidence: "0.95".into(),
                },
                CandidateClaim {
                    text: "Market harm is negligible due to the small portion used.".into(),
                    citation: Some("17 U.S.C. § 107(4)".into()),
                    quoted_text: Some("effect of the use upon the potential market".into()),
                    authority_label: AuthorityLabel::Binding,
                    inference: InferenceKind::DirectQuotation,
                    model_confidence: "0.88".into(),
                }
            ],
        })
    }
}
