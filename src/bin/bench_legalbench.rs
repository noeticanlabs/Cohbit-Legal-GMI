use cohbit_legal_gmi::adapter::MockLegalStatusAdapter;
use cohbit_legal_gmi::claim::{AuthorityLabel, InferenceKind, LegalClaim};
use cohbit_legal_gmi::manifold::LegalManifold;
use cohbit_legal_gmi::reference::{
    Jurisdiction, JurisdictionLevel, LegalReference, ProceduralStatus,
};
use cohbit_legal_gmi::verifier::{VerificationConfig, VerificationDecision};
use cohbit_legal_gmi::witness::LegalWitness;
use cohbit_legal_gmi::*;
use num_rational::BigRational;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct LegalBenchEntry {
    text: String,
    answer: String,
    task: String,
}

fn main() -> anyhow::Result<()> {
    let dataset_path = "fixtures/legalbench_sample.jsonl";
    if !Path::new(dataset_path).exists() {
        println!(
            "Dataset not found at {}. Please run the download/creation step first.",
            dataset_path
        );
        return Ok(());
    }

    let file = File::open(dataset_path)?;
    let reader = BufReader::new(file);

    let mut total = 0;
    let mut accepted = 0;
    let mut rejected = 0;

    println!("Starting synthetic Legacy Verifier fixture run (not legal accuracy)...");
    println!("--------------------------------------");

    for line in reader.lines() {
        let line = line?;
        let entry: LegalBenchEntry = serde_json::from_str(&line)?;
        total += 1;

        // 1. Setup Manifold
        let posture = ProceduralPosture {
            forum: Jurisdiction {
                court: "Supreme Court".into(),
                region: "US".into(),
                level: JurisdictionLevel::Supreme,
            },
            matter_type: "LegalBench Benchmark".into(),
            stage: "Verification".into(),
        };
        let manifold = LegalManifold::new(posture);

        // 2. Setup Mock Reference
        let mut refs = HashMap::new();
        let citation = if entry.task == "citation_prediction_open" {
            entry.answer.clone()
        } else {
            "Abercrombie Rules".to_string()
        };

        let reference = LegalReference {
            id: ReferenceId("ref_target".into()),
            citation: citation.clone(),
            jurisdiction: Jurisdiction {
                court: "US Federal".into(),
                region: "US".into(),
                level: JurisdictionLevel::Appellate,
            },
            source_text: Some(format!("Reference text for {}: {}", citation, entry.text)),
            procedural_status: ProceduralStatus::GoodLaw,
            ..Default::default()
        };
        refs.insert(citation.clone(), reference.clone());

        let adapter = MockLegalStatusAdapter {
            reference_store: refs,
            check_time: "2026-05-08".into(),
            ..Default::default()
        };

        // 3. Setup Witness
        let inference = if entry.task == "citation_prediction_open" {
            InferenceKind::DirectQuotation
        } else {
            // For abercrombie, it's a rule application
            InferenceKind::RuleApplication {
                rule_claims: vec![],
                fact_claims: vec![],
            }
        };

        let witness = LegalWitness {
            proposed_claim: LegalClaim {
                id: ClaimId(format!("claim_{}", total)),
                text: entry.text.clone(),
                cited_reference: Some(ReferenceId("ref_target".into())),
                quoted_text: if entry.task == "citation_prediction_open" {
                    Some(entry.text.clone())
                } else {
                    None
                },
                authority_label: AuthorityLabel::Binding,
                inference,
            },
            cited_reference: Some(reference),
            support_score: Some("1".to_string()),
            contrary_authorities: vec![],
            user_asserted_good_law: None,
            verifier_notes: vec![format!("Task: {}", entry.task)],
            ..Default::default()
        };

        // 4. Verification Config
        let config = VerificationConfig {
            support_threshold: "0.5".into(),
            allow_unknown_good_law: true,
            require_conflict_disclosure: true,
            target_jurisdiction: None,
            verification_time: "2026-11-06T00:00:00Z".into(),
            max_status_age_days: 365,
            valuation_config: valuation::ValuationConfig::default(),
        };

        // 5. Apply
        match cohbit::apply(
            manifold,
            witness,
            &config,
            BigRational::from_i64(0).unwrap(),
            BigRational::from_i64(0).unwrap(),
            Some(&adapter),
        ) {
            Ok((_, cohbit)) => {
                if cohbit.receipt.decision == VerificationDecision::Accept {
                    accepted += 1;
                    println!(
                        "[ACCEPT] Task: {:<25} | Answer: {}",
                        entry.task, entry.answer
                    );
                } else {
                    rejected += 1;
                    println!(
                        "[REJECT] Task: {:<25} | Answer: {} | Reason: {:?}",
                        entry.task, entry.answer, cohbit.verifier_result.c1_citation_exists
                    );
                }
            }
            Err(e) => {
                rejected += 1;
                println!("[ERROR ] Task: {:<25} | Error: {}", entry.task, e);
            }
        }
    }

    println!("--------------------------------------");
    println!("Benchmark Complete.");
    println!("Total Entries: {}", total);
    println!("Accepted:      {}", accepted);
    println!("Rejected:      {}", rejected);
    println!(
        "Simulation accept fraction: {:.2}% (no legal-accuracy implication)",
        (accepted as f64 / total as f64) * 100.0
    );

    Ok(())
}
