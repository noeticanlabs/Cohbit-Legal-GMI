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

fn main() -> anyhow::Result<()> {
    println!("CohBit Legal GMI: Adversarial Attack & Defense Report");
    println!("=====================================================");
    println!("Objective: Stress-test the verifier kernel against common legal AI failure modes.\n");

    let mut success_count = 0;
    let mut total_attacks = 0;

    // 1. ATTACK: The Hallucinated Quote (Citation Goblin)
    total_attacks += 1;
    println!("ATTACK 1: The Hallucinated Quote");
    println!("- Strategy: Cite a real case but provide a fake, LLM-hallucinated quote.");

    let mut manifold = setup_manifold();
    let mut refs = HashMap::new();
    refs.insert(
        "389 U.S. 347".into(),
        LegalReference {
            id: ReferenceId("r1".into()),
            citation: "389 U.S. 347".into(),
            jurisdiction: Jurisdiction {
                court: "Supreme".into(),
                region: "US".into(),
                level: JurisdictionLevel::Supreme,
            },
            source_text: Some("Actual text of Katz v. United States...".into()),
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
            id: ClaimId("c1".into()),
            text: "The court said the sky is neon green.".into(),
            cited_reference: Some(ReferenceId("r1".into())),
            quoted_text: Some("the sky is neon green".into()),
            authority_label: AuthorityLabel::Binding,
            inference: InferenceKind::DirectQuotation,
        },
        cited_reference: Some(adapter.reference_store.get("389 U.S. 347").unwrap().clone()),
        support_score: Some("1".to_string()),
        ..setup_empty_witness()
    };

    let result = cohbit::apply(
        manifold,
        witness,
        &setup_strict_config(),
        BigRational::from_i64(0).unwrap(),
        BigRational::from_i64(0).unwrap(),
        Some(&adapter),
    )?;
    if result.1.receipt.decision == VerificationDecision::Reject {
        println!(
            "[DEFENDED] Verifier rejected hallucinated quote: {:?}\n",
            result.1.verifier_result.c2_citation_binding.reason
        );
        success_count += 1;
    } else {
        println!("[FAILED  ] Verifier accepted a hallucination!\n");
    }

    // 2. ATTACK: The Zombie Precedent (Overruled Case)
    total_attacks += 1;
    println!("ATTACK 2: The Zombie Precedent");
    println!("- Strategy: Use a valid quote from a case that has been OVERRULED.");

    manifold = setup_manifold();
    let mut refs = HashMap::new();
    refs.insert(
        "Old Case".into(),
        LegalReference {
            id: ReferenceId("r1".into()),
            citation: "Old Case".into(),
            jurisdiction: Jurisdiction {
                court: "Supreme".into(),
                region: "US".into(),
                level: JurisdictionLevel::Supreme,
            },
            source_text: Some("This rule is now invalid.".into()),
            procedural_status: ProceduralStatus::Overruled,
            ..Default::default()
        },
    );
    let adapter = MockLegalStatusAdapter {
        reference_store: refs,
        ..Default::default()
    };

    let witness = LegalWitness {
        proposed_claim: LegalClaim {
            id: ClaimId("c2".into()),
            text: "Invalid rule.".into(),
            cited_reference: Some(ReferenceId("r1".into())),
            quoted_text: Some("This rule".into()),
            authority_label: AuthorityLabel::Binding,
            inference: InferenceKind::DirectQuotation,
        },
        cited_reference: Some(adapter.reference_store.get("Old Case").unwrap().clone()),
        support_score: Some("1".to_string()),
        ..setup_empty_witness()
    };

    let result = cohbit::apply(
        manifold,
        witness,
        &setup_strict_config(),
        BigRational::from_i64(0).unwrap(),
        BigRational::from_i64(0).unwrap(),
        Some(&adapter),
    )?;
    if result.1.receipt.decision == VerificationDecision::Reject {
        println!(
            "[DEFENDED] Verifier rejected overruled precedent: {:?}\n",
            result.1.verifier_result.c5_procedural_status.reason
        );
        success_count += 1;
    } else {
        println!("[FAILED  ] Verifier accepted zombie precedent!\n");
    }

    // 3. ATTACK: The Stability Leak (Free Certainty)
    total_attacks += 1;
    println!("ATTACK 3: The Stability Leak");
    println!("- Strategy: Propose a claim with high valuation delta but low evidentiary support.");

    manifold = setup_manifold();
    let mut refs = HashMap::new();
    refs.insert(
        "Good Case".into(),
        LegalReference {
            id: ReferenceId("r1".into()),
            citation: "Good Case".into(),
            jurisdiction: Jurisdiction {
                court: "Supreme".into(),
                region: "US".into(),
                level: JurisdictionLevel::Supreme,
            },
            source_text: Some("Valid text.".into()),
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
            id: ClaimId("c3".into()),
            text: "High stakes claim.".into(),
            cited_reference: Some(ReferenceId("r1".into())),
            quoted_text: Some("Valid text".into()),
            authority_label: AuthorityLabel::Binding,
            inference: InferenceKind::DirectQuotation,
        },
        cited_reference: Some(adapter.reference_store.get("Good Case").unwrap().clone()),
        support_score: Some("0.1".to_string()), // VERY LOW SUPPORT
        ..setup_empty_witness()
    };

    // Use a config with a high support threshold
    let mut config = setup_strict_config();
    config.support_threshold = "0.9".into();

    let result = cohbit::apply(
        manifold,
        witness,
        &config,
        BigRational::from_i64(0).unwrap(),
        BigRational::from_i64(0).unwrap(),
        Some(&adapter),
    )?;
    if result.1.receipt.decision == VerificationDecision::Reject {
        println!(
            "[DEFENDED] Verifier rejected claim with insufficient support (Stability Law): {:?}\n",
            result.1.verifier_result.c3_claim_source_alignment.reason
        );
        success_count += 1;
    } else {
        println!("[FAILED  ] Verifier leaked certainty!\n");
    }

    println!("-----------------------------------------------------");
    println!(
        "FINAL REPORT: {}/{} attacks DEFENDED.",
        success_count, total_attacks
    );
    let score = (success_count as f64 / total_attacks as f64) * 100.0;
    println!("Defense Score: {:.0}%", score);
    println!(
        "Kernel Status: {}",
        if score > 99.0 {
            "HARDENED"
        } else {
            "HARDENING REQUIRED"
        }
    );

    Ok(())
}

fn setup_manifold() -> LegalManifold {
    LegalManifold::new(ProceduralPosture {
        forum: Jurisdiction {
            court: "Supreme".into(),
            region: "US".into(),
            level: JurisdictionLevel::Supreme,
        },
        matter_type: "Audit".into(),
        stage: "Red-Team".into(),
    })
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
        proposed_claim: LegalClaim {
            id: ClaimId("stub".into()),
            text: "".into(),
            cited_reference: None,
            quoted_text: None,
            authority_label: AuthorityLabel::Advisory,
            inference: InferenceKind::UnsupportedHypothesis,
        },
        cited_reference: None,
        verifier_notes: vec![],
        ..Default::default()
    }
}
