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
use std::collections::HashMap;
use std::time::Instant;

fn main() -> anyhow::Result<()> {
    println!("CohBit Legal GMI: Production-Grade Benchmarking Suite");
    println!("=====================================================");

    // 1. Performance Test (Micro-benchmarking Apply)
    println!("\n[1/3] PERFORMANCE: Throughput & Latency");
    let latencies = run_performance_test(1000)?;
    let avg_lat = latencies.iter().sum::<u128>() as f64 / latencies.len() as f64;
    println!("- Average Latency:  {:.2} μs", avg_lat);
    println!("- Throughput:       {:.0} steps/sec", 1_000_000.0 / avg_lat);

    // 2. Rigor Test (Adversarial Defense)
    println!("\n[2/3] RIGOR: Adversarial Defense");
    let defense_score = run_defense_test()?;
    println!("- Defense Score:    {:.0}%", defense_score * 100.0);

    // 3. Integrity Test (State Hash Determinism)
    println!("\n[3/3] INTEGRITY: Hashing & Determinism");
    let is_deterministic = run_determinism_test()?;
    println!(
        "- Determinism:      {}",
        if is_deterministic {
            "VERIFIED"
        } else {
            "FAILED"
        }
    );

    println!("\n=====================================================");
    let final_grade = if defense_score > 0.99 && is_deterministic && avg_lat < 500.0 {
        "A (Production Grade)"
    } else {
        "B (Hardening Required)"
    };
    println!("FINAL GRADE: {}", final_grade);

    Ok(())
}

fn run_performance_test(iterations: usize) -> anyhow::Result<Vec<u128>> {
    let manifold = setup_manifold();
    let witness = setup_witness();
    let config = setup_config();
    let mut refs = HashMap::new();
    refs.insert(
        "Case".into(),
        LegalReference {
            id: ReferenceId("r".into()),
            citation: "Case".into(),
            jurisdiction: setup_jurisdiction(),
            source_text: Some("Text".into()),
            procedural_status: ProceduralStatus::GoodLaw,
            ..Default::default()
        },
    );
    let adapter = MockLegalStatusAdapter {
        reference_store: refs,
        ..Default::default()
    };

    let mut latencies = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = cohbit::apply(
            manifold.clone(),
            witness.clone(),
            &config,
            BigRational::from_i64(0).unwrap(),
            BigRational::from_i64(0).unwrap(),
            Some(&adapter),
        )?;
        latencies.push(start.elapsed().as_micros());
    }
    Ok(latencies)
}

fn run_defense_test() -> anyhow::Result<f64> {
    // Simplified defense check for benchmark summary
    let manifold = setup_manifold();
    let config = setup_strict_config();

    // Attack: Hallucination
    let mut refs = HashMap::new();
    refs.insert(
        "Case".into(),
        LegalReference {
            id: ReferenceId("r".into()),
            citation: "Case".into(),
            jurisdiction: setup_jurisdiction(),
            source_text: Some("Real Text".into()),
            procedural_status: ProceduralStatus::GoodLaw,
            ..Default::default()
        },
    );
    let adapter = MockLegalStatusAdapter {
        reference_store: refs,
        ..Default::default()
    };

    let witness = LegalWitness {
        proposed_claim: LegalClaim {
            id: ClaimId("c".into()),
            text: "Fake".into(),
            cited_reference: Some(ReferenceId("r".into())),
            quoted_text: Some("Fake".into()),
            authority_label: AuthorityLabel::Binding,
            inference: InferenceKind::DirectQuotation,
        },
        cited_reference: Some(adapter.reference_store.get("Case").unwrap().clone()),
        support_score: Some("1".to_string()),
        ..setup_empty_witness()
    };

    let result = cohbit::apply(
        manifold,
        witness,
        &config,
        BigRational::from_i64(0).unwrap(),
        BigRational::from_i64(0).unwrap(),
        Some(&adapter),
    )?;
    if result.1.receipt.decision == VerificationDecision::Reject {
        Ok(1.0)
    } else {
        Ok(0.0)
    }
}

fn run_determinism_test() -> anyhow::Result<bool> {
    let manifold = setup_manifold();
    let h1 = manifold.get_state_hash();
    let h2 = manifold.get_state_hash();
    Ok(h1 == h2)
}

fn setup_manifold() -> LegalManifold {
    LegalManifold::new(ProceduralPosture {
        forum: setup_jurisdiction(),
        matter_type: "Bench".into(),
        stage: "Prod".into(),
    })
}

fn setup_jurisdiction() -> Jurisdiction {
    Jurisdiction {
        court: "Court".into(),
        region: "US".into(),
        level: JurisdictionLevel::Supreme,
    }
}

fn setup_witness() -> LegalWitness {
    LegalWitness {
        proposed_claim: LegalClaim {
            id: ClaimId("c".into()),
            text: "Text".into(),
            cited_reference: Some(ReferenceId("r".into())),
            quoted_text: Some("Text".into()),
            authority_label: AuthorityLabel::Binding,
            inference: InferenceKind::DirectQuotation,
        },
        cited_reference: Some(LegalReference {
            id: ReferenceId("r".into()),
            citation: "Case".into(),
            jurisdiction: setup_jurisdiction(),
            source_text: Some("Text".into()),
            procedural_status: ProceduralStatus::GoodLaw,
            ..Default::default()
        }),
        support_score: Some("1".to_string()),
        ..setup_empty_witness()
    }
}

fn setup_config() -> VerificationConfig {
    VerificationConfig {
        support_threshold: "0.5".into(),
        allow_unknown_good_law: true,
        require_conflict_disclosure: false,
        target_jurisdiction: None,
        verification_time: "2026-11-06T00:00:00Z".into(),
        max_status_age_days: 365,
        valuation_config: valuation::ValuationConfig::default(),
    }
}

fn setup_strict_config() -> VerificationConfig {
    VerificationConfig {
        support_threshold: "0.5".into(),
        allow_unknown_good_law: false,
        require_conflict_disclosure: true,
        target_jurisdiction: None,
        verification_time: "2026-11-06T00:00:00Z".into(),
        max_status_age_days: 365,
        valuation_config: valuation::ValuationConfig::default(),
    }
}

fn setup_empty_witness() -> LegalWitness {
    LegalWitness {
        cited_reference: None,
        support_score: None,
        contrary_authorities: vec![],
        user_asserted_good_law: None,
        verifier_notes: vec![],
        ..Default::default()
    }
}
