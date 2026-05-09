use cohbit_legal_gmi::*;
use cohbit_legal_gmi::manifold::LegalManifold;
use cohbit_legal_gmi::witness::LegalWitness;
use cohbit_legal_gmi::verifier::{VerificationConfig, VerificationDecision};
use cohbit_legal_gmi::claim::{AuthorityLabel, InferenceKind, LegalClaim};
use cohbit_legal_gmi::reference::{LegalReference, Jurisdiction, JurisdictionLevel, ProceduralStatus};
use cohbit_legal_gmi::adapter::MockLegalStatusAdapter;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let output_dir = "fixtures/bench_receipts";
    if !Path::new(output_dir).exists() {
        fs::create_dir_all(output_dir)?;
    }

    println!("Generating Benchmark Receipts...");

    let manifold = setup_manifold();
    let config = setup_config();
    
    // 1. A Valid Receipt (Benchmark 1)
    let mut refs = HashMap::new();
    let ref1 = LegalReference {
        id: ReferenceId("r1".into()),
        citation: "100 U.S. 1".into(),
        jurisdiction: setup_jurisdiction(),
        source_text: Some("The core principle of the law...".into()),
        procedural_status: ProceduralStatus::GoodLaw,
    };
    refs.insert("100 U.S. 1".into(), ref1.clone());
    let adapter = MockLegalStatusAdapter { reference_store: refs, ..Default::default() };
    
    let witness = LegalWitness {
        proposed_claim: LegalClaim {
            id: ClaimId("bench_c1".into()),
            text: "The core principle.".into(),
            cited_reference: Some(ReferenceId("r1".into())),
            quoted_text: Some("core principle".into()),
            authority_label: AuthorityLabel::Binding,
            inference: InferenceKind::DirectQuotation,
        },
        cited_reference: Some(ref1),
        support_score: Some("1".to_string()),
        ..setup_empty_witness()
    };

    let (_, cohbit) = cohbit::apply(manifold.clone(), witness, &config, 0, Some(&adapter))?;
    let receipt_json = serde_json::to_string_pretty(&cohbit.receipt)?;
    fs::write(format!("{}/accept_benchmark.json", output_dir), receipt_json)?;
    println!("- Generated accept_benchmark.json");

    // 2. A Reject Receipt (Benchmark 2 - Hallucination)
    let witness_reject = LegalWitness {
        proposed_claim: LegalClaim {
            id: ClaimId("bench_c2".into()),
            text: "Hallucinated claim.".into(),
            cited_reference: Some(ReferenceId("r1".into())),
            quoted_text: Some("missing text".into()),
            authority_label: AuthorityLabel::Binding,
            inference: InferenceKind::DirectQuotation,
        },
        cited_reference: Some(adapter.reference_store.get("100 U.S. 1").unwrap().clone()),
        support_score: Some("1".to_string()),
        ..setup_empty_witness()
    };

    let (_, cohbit_reject) = cohbit::apply(manifold, witness_reject, &config, 1, Some(&adapter))?;
    let reject_json = serde_json::to_string_pretty(&cohbit_reject.receipt)?;
    fs::write(format!("{}/reject_benchmark.json", output_dir), reject_json)?;
    println!("- Generated reject_benchmark.json");

    println!("Benchmark receipts generated successfully at {}", output_dir);

    Ok(())
}

fn setup_manifold() -> LegalManifold {
    LegalManifold::new(ProceduralPosture {
        forum: setup_jurisdiction(), matter_type: "Benchmark".into(), stage: "Archival".into(),
    })
}

fn setup_jurisdiction() -> Jurisdiction {
    Jurisdiction { court: "Supreme Court".into(), region: "US".into(), level: JurisdictionLevel::Supreme }
}

fn setup_config() -> VerificationConfig {
    VerificationConfig { support_threshold: "0.5".into(), allow_unknown_good_law: true, require_conflict_disclosure: false, valuation_config: valuation::ValuationConfig::default() }
}

fn setup_empty_witness() -> LegalWitness {
    LegalWitness {
        proposed_claim: LegalClaim { id: ClaimId("stub".into()), text: "".into(), cited_reference: None, quoted_text: None, authority_label: AuthorityLabel::Advisory, inference: InferenceKind::UnsupportedHypothesis },
        cited_reference: None, support_score: None, contrary_authorities: vec![], user_asserted_good_law: None, verifier_notes: vec![],
    }
}
