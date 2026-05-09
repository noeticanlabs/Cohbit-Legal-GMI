use crate::manifold::LegalManifold;
use crate::explorer::ExplorationSpace;
use crate::cohbit::{apply, LegalCohBit};
use crate::verifier::VerificationConfig;
use crate::adapter::LegalSourceAdapter;

pub struct CommitmentMembrane;

impl CommitmentMembrane {
    pub fn try_commit(
        manifold: &mut LegalManifold,
        explorer: &mut ExplorationSpace,
        config: &VerificationConfig,
        spend: num_rational::BigRational,
        defect: num_rational::BigRational,
        adapter: Option<&dyn LegalSourceAdapter>,
    ) -> Vec<Result<LegalCohBit, String>> {
        let mut results = vec![];

        while let Some(candidate) = explorer.pop_candidate() {
            let witness = candidate.to_witness(&format!("claim_{}", manifold.claims.len()));
            
            match apply(manifold.clone(), witness, config, spend.clone(), defect.clone(), adapter) {
                Ok((new_manifold, cohbit)) => {
                    *manifold = new_manifold;
                    explorer.record_decision(&candidate.text, true);
                    results.push(Ok(cohbit));
                }
                Err(e) => {
                    explorer.record_decision(&candidate.text, false);
                    results.push(Err(format!("Commitment failed for '{}': {}", candidate.text, e)));
                }
            }
        }

        results
    }
}
