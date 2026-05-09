use cohbit_legal_gmi::*;
use std::collections::HashMap;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let base_dir = Path::new("fixtures/demo_data");
    let bundle_dir = Path::new("fixtures/demo_bundle");

    let pm = PersistenceManager::new(base_dir)?;
    
    // 1. Initialize
    let posture = ProceduralPosture {
        forum: Jurisdiction { court: "Supreme Court".into(), region: "US".into(), level: JurisdictionLevel::Supreme },
        matter_type: "Civil".into(),
        stage: "Initial".into(),
    };
    let manifold = LegalManifold::new(posture);
    pm.initialize_chain("demo_chain", manifold.get_state_hash())?;

    // 2. Setup Source
    let reference = LegalReference {
        id: ReferenceId("r1".into()),
        citation: "123 U.S. 456".into(),
        jurisdiction: Jurisdiction { court: "Supreme".into(), region: "US".into(), level: JurisdictionLevel::Supreme },
        source_text: Some("The right of the people to be secure...".into()),
        procedural_status: ProceduralStatus::GoodLaw,
    };
    let mut refs = HashMap::new();
    refs.insert("123 U.S. 456".to_string(), reference.clone());
    let adapter = MockLegalStatusAdapter {
        status_overrides: HashMap::new(),
        treatment_overrides: HashMap::new(),
        check_time: "2026-05-04".into(),
        reference_store: refs,
    };

    // 3. Commit a claim
    let witness = LegalWitness {
        proposed_claim: LegalClaim { 
            id: ClaimId("c1".into()), 
            text: "Privacy Right".into(), 
            cited_reference: Some(ReferenceId("r1".into())), 
            quoted_text: Some("secure".into()), 
            authority_label: AuthorityLabel::Persuasive, 
            inference: InferenceKind::DirectQuotation 
        },
        cited_reference: Some(reference),
        support_score: Some("0.5".to_string()),
        contrary_authorities: vec![],
        user_asserted_good_law: None,
        verifier_notes: vec![],
    };

    let config = VerificationConfig {
        support_threshold: "0.5".into(),
        allow_unknown_good_law: true,
        require_conflict_disclosure: false,
        valuation_config: valuation::ValuationConfig::default(),
    };

    let (_, cohbit) = cohbit::apply(manifold, witness, &config, 0, Some(&adapter)).unwrap();
    pm.append_receipt(&cohbit.receipt)?;

    // 4. Export Bundle
    let bm = BundleManager::new(bundle_dir);
    let adapters: Vec<&dyn LegalSourceAdapter> = vec![&adapter];
    bm.export_bundle(&pm, &adapters)?;

    println!("Demo bundle generated at: {:?}", bundle_dir);
    
    // 5. Verify
    bm.verify_bundle()?;
    println!("Bundle verification: PASS");

    Ok(())
}
