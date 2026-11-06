use crate::adapter::LegalSourceAdapter;
use crate::cohbit::{apply, LegalCohBit};
use crate::explorer::ExplorationSpace;
use crate::manifold::LegalManifold;
use crate::verifier::VerificationConfig;
use crate::verifier::VerificationDecision;

#[derive(Debug)]
pub enum CommitOutcome {
    Accepted(LegalCohBit),
    Rejected {
        cohbit: LegalCohBit,
        reasons: Vec<String>,
    },
    Escalated {
        cohbit: LegalCohBit,
        reasons: Vec<String>,
    },
    EvaluationError {
        candidate: String,
        error: String,
    },
}

fn failure_reasons(cohbit: &LegalCohBit) -> Vec<String> {
    let result = &cohbit.verifier_result;
    [
        &result.c1_citation_exists,
        &result.c2_citation_binding,
        &result.c3_claim_source_alignment,
        &result.c4_jurisdictional_validity,
        &result.c5_procedural_status,
        &result.c6_inference_validity,
        &result.c7_conflict_disclosure,
    ]
    .into_iter()
    .filter(|check| !check.passed)
    .map(|check| check.reason.clone())
    .collect()
}

pub struct CommitmentMembrane;

impl CommitmentMembrane {
    pub fn try_commit(
        manifold: &mut LegalManifold,
        explorer: &mut ExplorationSpace,
        config: &VerificationConfig,
        spend: num_rational::BigRational,
        defect: num_rational::BigRational,
        adapter: Option<&dyn LegalSourceAdapter>,
    ) -> Vec<CommitOutcome> {
        let mut results = vec![];

        while let Some(candidate) = explorer.pop_candidate() {
            let witness = candidate.to_witness(&format!("claim_{}", manifold.claims.len()));

            match apply(
                manifold.clone(),
                witness,
                config,
                spend.clone(),
                defect.clone(),
                adapter,
            ) {
                Ok((new_manifold, cohbit)) => {
                    *manifold = new_manifold;
                    explorer.record_outcome(&candidate.text, &cohbit.receipt.decision);
                    let reasons = failure_reasons(&cohbit);
                    match cohbit.receipt.decision {
                        VerificationDecision::Accept => {
                            results.push(CommitOutcome::Accepted(cohbit))
                        }
                        VerificationDecision::Reject => {
                            results.push(CommitOutcome::Rejected { cohbit, reasons })
                        }
                        VerificationDecision::Escalate => {
                            results.push(CommitOutcome::Escalated { cohbit, reasons })
                        }
                    }
                }
                Err(e) => {
                    explorer.record_outcome(&candidate.text, &VerificationDecision::Reject);
                    results.push(CommitOutcome::EvaluationError {
                        candidate: candidate.text.clone(),
                        error: e.to_string(),
                    });
                }
            }
        }

        results
    }
}
