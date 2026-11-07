use cohbit_legal_gmi::adapter::MockLegalStatusAdapter;
use cohbit_legal_gmi::claim::{AuthorityLabel, InferenceKind, LegalClaim};
use cohbit_legal_gmi::manifold::LegalManifold;
use cohbit_legal_gmi::reference::{
    Jurisdiction, JurisdictionLevel, LegalReference, ProceduralStatus,
};
use cohbit_legal_gmi::verifier::{VerificationConfig, VerificationDecision};
use cohbit_legal_gmi::witness::{InternalReviewStatus, LegalWitness};
use cohbit_legal_gmi::*;
use num_rational::BigRational;
use num_traits::FromPrimitive;
use std::collections::HashMap;

fn main() -> anyhow::Result<()> {
    println!("CohBit Legal GMI: Synthetic Metadata and Guardrail Checks");
    println!("=====================================================");

    let mut total_tests = 0;
    let mut passed_tests = 0;

    // Test 1: Precision (Hallucination Detection)
    total_tests += 1;
    println!("\n[1/3] TEST: Precision (Quote Fidelity)");
    let precision_pass = test_quote_fidelity()?;
    if precision_pass {
        passed_tests += 1;
        println!("  >> PASS: Verifier detected semantic perturbation.");
    } else {
        println!("  >> FAIL: Verifier accepted a hallucinated quote.");
    }

    // Test 2: Support Realism (Commercial Stability)
    total_tests += 1;
    println!("\n[2/3] TEST: Support Realism (Valuation Boundary)");
    let realism_pass = test_support_realism()?;
    if realism_pass {
        passed_tests += 1;
        println!("  >> PASS: Verifier enforced stability boundary for low-support claims.");
    } else {
        println!("  >> FAIL: Verifier leaked valuation for unsupported claims.");
    }

    // Test 3: Detail Fidelity (Metadata Persistence)
    total_tests += 1;
    println!("\n[3/3] TEST: Detail Fidelity (Metadata Chain)");
    let detail_pass = test_metadata_persistence()?;
    if detail_pass {
        passed_tests += 1;
        println!("  >> PASS: Commercial metadata persisted through transition.");
    } else {
        println!("  >> FAIL: Commercial metadata was lost or corrupted.");
    }

    println!("\n=====================================================");
    println!(
        "FIXTURE CHECKS: {}/{} passed ({:.0}%; not a legal-accuracy metric)",
        passed_tests,
        total_tests,
        (passed_tests as f64 / total_tests as f64) * 100.0
    );

    Ok(())
}

fn test_quote_fidelity() -> anyhow::Result<bool> {
    let manifold = setup_manifold();
    let config = setup_strict_config();

    let mut refs = HashMap::new();
    let reference = LegalReference {
        id: ReferenceId("r1".into()),
        citation: "Standard v. Precise".into(),
        jurisdiction: setup_jurisdiction(),
        source_text: Some("The defendant shall pay damages.".into()),
        procedural_status: ProceduralStatus::GoodLaw,
        ..Default::default()
    };
    refs.insert("Standard v. Precise".into(), reference.clone());
    let adapter = MockLegalStatusAdapter {
        reference_store: refs,
        ..Default::default()
    };

    // Perturbed Witness (Hallucination: "may" instead of "shall")
    let witness = LegalWitness {
        proposed_claim: LegalClaim {
            id: ClaimId("c1".into()),
            text: "Defendant may pay.".into(),
            cited_reference: Some(ReferenceId("r1".into())),
            quoted_text: Some("The defendant may pay damages".into()),
            authority_label: AuthorityLabel::Binding,
            inference: InferenceKind::DirectQuotation,
        },
        cited_reference: Some(reference),
        support_score: Some("1".to_string()),
        review_status: Some(InternalReviewStatus::LegacyCertifiedLabel),
        ..Default::default()
    };

    let result = cohbit::apply(
        manifold,
        witness,
        &config,
        BigRational::from_i64(0).unwrap(),
        BigRational::from_i64(0).unwrap(),
        Some(&adapter),
    )?;
    Ok(result.1.receipt.decision == VerificationDecision::Reject)
}

fn test_support_realism() -> anyhow::Result<bool> {
    let manifold = setup_manifold();
    let config = setup_strict_config();

    let _reference = LegalReference::default();

    // Witness with high delta but low support
    let witness = LegalWitness {
        proposed_claim: LegalClaim {
            id: ClaimId("c2".into()),
            text: "High stakes.".into(),
            authority_label: AuthorityLabel::Binding,
            ..Default::default()
        },
        support_score: Some("0.05".to_string()), // VERY LOW SUPPORT
        ..Default::default()
    };

    // The verifier should reject if support < threshold (default 0.5)
    let result = cohbit::apply(
        manifold,
        witness,
        &config,
        BigRational::from_i64(0).unwrap(),
        BigRational::from_i64(0).unwrap(),
        None,
    )?;
    Ok(result.1.receipt.decision == VerificationDecision::Reject)
}

fn test_metadata_persistence() -> anyhow::Result<bool> {
    let manifold = setup_manifold();
    let config = setup_config();

    let reference = LegalReference {
        id: ReferenceId("r1".into()),
        citation: "Commercial Case".into(),
        jurisdiction: setup_jurisdiction(),
        counsel_id: Some("COUNSEL-99".into()),
        law_firm_id: Some("FIRM-01".into()),
        ..Default::default()
    };

    let witness = LegalWitness {
        proposed_claim: LegalClaim {
            id: ClaimId("c3".into()),
            text: "Valid".into(),
            ..Default::default()
        },
        cited_reference: Some(reference.clone()),
        review_status: Some(InternalReviewStatus::AdversarialAudit),
        ..Default::default()
    };

    let (_, cohbit) = cohbit::apply(
        manifold,
        witness,
        &config,
        BigRational::from_i64(0).unwrap(),
        BigRational::from_i64(0).unwrap(),
        None,
    )?;

    // Check if metadata is in the receipt's witness (via trace)
    let receipt_witness = &cohbit.receipt.trace.as_ref().unwrap().witness;
    let ref_meta = receipt_witness.cited_reference.as_ref().unwrap();

    Ok(ref_meta.counsel_id == Some("COUNSEL-99".into())
        && ref_meta.law_firm_id == Some("FIRM-01".into())
        && receipt_witness.review_status == Some(InternalReviewStatus::AdversarialAudit))
}

fn setup_manifold() -> LegalManifold {
    LegalManifold::new(ProceduralPosture {
        forum: setup_jurisdiction(),
        matter_type: "Commercial".into(),
        stage: "Archival".into(),
    })
}

fn setup_jurisdiction() -> Jurisdiction {
    Jurisdiction {
        court: "Commercial Court".into(),
        region: "DE".into(),
        level: JurisdictionLevel::Trial,
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
