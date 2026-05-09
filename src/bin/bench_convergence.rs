use cohbit_legal_gmi::*;
use cohbit_legal_gmi::manifold::LegalManifold;
use cohbit_legal_gmi::witness::LegalWitness;
use cohbit_legal_gmi::verifier::{VerificationConfig, VerificationDecision};
use cohbit_legal_gmi::claim::{AuthorityLabel, InferenceKind, LegalClaim};
use cohbit_legal_gmi::reference::{LegalReference, Jurisdiction, JurisdictionLevel, ProceduralStatus};
use cohbit_legal_gmi::adapter::MockLegalStatusAdapter;
use cohbit_legal_gmi::memory::LegalNpeMemory;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Serialize, Deserialize)]
struct LegalBenchEntry {
    text: String,
    answer: String,
    task: String,
}

fn main() -> anyhow::Result<()> {
    let dataset_path = "fixtures/legalbench_sample.jsonl";
    let file = File::open(dataset_path)?;
    let reader = BufReader::new(file);

    let mut memory = LegalNpeMemory::new();
    let mut refs = HashMap::new();

    // Pre-populate references from the dataset
    // (Simulating a world model where the NPE has seen these citations)
    let entries: Vec<LegalBenchEntry> = reader.lines()
        .map(|l| serde_json::from_str(&l.unwrap()).unwrap())
        .collect();

    for entry in &entries {
        refs.insert(entry.answer.clone(), LegalReference {
            id: ReferenceId("ref".into()),
            citation: entry.answer.clone(),
            jurisdiction: Jurisdiction { court: "US Federal".into(), region: "US".into(), level: JurisdictionLevel::Appellate },
            source_text: Some(format!("Source for {}: {}", entry.answer, entry.text)),
            procedural_status: ProceduralStatus::GoodLaw,
        });
    }

    let adapter = MockLegalStatusAdapter {
        reference_store: refs,
        check_time: "2026-05-08".into(),
        ..Default::default()
    };

    println!("Legal GMI Convergence & Metrics Benchmark");
    println!("=========================================");

    for (i, entry) in entries.iter().enumerate() {
        let manifold = LegalManifold::new(ProceduralPosture {
            forum: Jurisdiction { court: "Supreme".into(), region: "US".into(), level: JurisdictionLevel::Supreme },
            matter_type: "Metric Test".into(),
            stage: "Audit".into(),
        });

        // Simulating "Multi-view": We consider 3 candidates before choosing
        // 1 correct, 2 "hallucinations" (to test correction speed)
        let candidates = vec![
            (entry.answer.clone(), entry.text.clone(), true), // Correct
            ("Fake Citation".to_string(), "Fake Text".to_string(), false), // Wrong
            (entry.answer.clone(), "Hallucinated Text".to_string(), false), // Wrong quote
        ];

        // Track queue depth (Simulated)
        memory.total_queue_depth_sum += candidates.len() as u64;

        for (cit, txt, is_correct) in candidates {
            let witness = LegalWitness {
                proposed_claim: LegalClaim {
                    id: ClaimId(format!("c_{}_{}", i, is_correct)),
                    text: txt.clone(),
                    cited_reference: Some(ReferenceId("ref".into())),
                    quoted_text: Some(txt.clone()),
                    authority_label: AuthorityLabel::Binding,
                    inference: InferenceKind::DirectQuotation,
                },
                cited_reference: Some(LegalReference {
                    id: ReferenceId("ref".into()),
                    citation: cit.clone(),
                    jurisdiction: Jurisdiction { court: "US Federal".into(), region: "US".into(), level: JurisdictionLevel::Appellate },
                    source_text: Some(txt.clone()),
                    procedural_status: ProceduralStatus::GoodLaw,
                }),
                support_score: Some("1".to_string()),
                ..setup_empty_witness()
            };

            let config = VerificationConfig {
                support_threshold: "0.5".into(),
                allow_unknown_good_law: true,
                require_conflict_disclosure: true,
                valuation_config: valuation::ValuationConfig::default(),
            };

            // Apply transition
            let (_, cohbit) = cohbit::apply(manifold.clone(), witness, &config, i as i64, Some(&adapter))?;
            
            // NPE learns from the result
            memory.ingest(&cohbit.receipt);
        }
    }

    println!("-----------------------------------------");
    println!("Total Steps (Decisions):  {}", memory.total_steps);
    println!("Total Successes:          {}", memory.total_success);
    println!("Total Rejections:         {}", memory.total_rejections);
    println!("");
    println!("Multi-view Rate:          {:.2} candidates/step", memory.multi_view_rate());
    println!("Quick-to-Correct Ratio:   {:.2} steps/correction", memory.quick_to_correct_ratio());
    println!("-----------------------------------------");

    Ok(())
}

fn setup_empty_witness() -> LegalWitness {
    LegalWitness {
        proposed_claim: LegalClaim { id: ClaimId("stub".into()), text: "".into(), cited_reference: None, quoted_text: None, authority_label: AuthorityLabel::Advisory, inference: InferenceKind::UnsupportedHypothesis },
        cited_reference: None, support_score: None, contrary_authorities: vec![], user_asserted_good_law: None, verifier_notes: vec![],
    }
}
