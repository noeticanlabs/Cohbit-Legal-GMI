#[cfg(test)]
mod tests {
    use crate::*;
    use crate::valuation::{ValuationConfig, ValuationMode};
    use crate::adapter::{MockLegalStatusAdapter, TreatmentRecord, TreatmentType};
    use crate::proposal::{LegalProposal, CandidateClaim};
    use crate::npe::{LegalNpe, SyntheticLegalNpe};
    use crate::persistence::{PersistenceManager};
    use std::collections::HashMap;
    use std::fs;
    use chrono::Utc;

    fn setup_manifold() -> LegalManifold {
        let posture = ProceduralPosture {
            forum: Jurisdiction {
                court: "Supreme Court".to_string(),
                region: "US".to_string(),
                level: JurisdictionLevel::Supreme,
            },
            matter_type: "Civil".to_string(),
            stage: "Initial".to_string(),
        };
        LegalManifold::new(posture)
    }

    fn setup_config() -> VerificationConfig {
        VerificationConfig {
            support_threshold: 0.5,
            allow_unknown_good_law: true,
            require_conflict_disclosure: false,
            valuation_config: ValuationConfig::default(),
        }
    }

    #[test]
    fn test_memory_cannot_commit_without_verifier() {
        let mut manifold = setup_manifold();
        let config = setup_config();
        let mut explorer = ExplorationSpace::new();

        let proposal = LegalProposal {
            proposal_id: "prop_1".into(),
            title: "Memory Abuse Attempt".into(),
            description: "Trying to bypass verifier".into(),
            candidate_claims: vec![
                CandidateClaim {
                    text: "Fake Quote".into(),
                    citation: Some("Perfect Case".into()),
                    quoted_text: Some("Non-existent text".into()),
                    authority_label: AuthorityLabel::Binding,
                    inference: InferenceKind::DirectQuotation,
                    model_confidence: 1.0,
                }
            ],
        };

        explorer.inject_proposal(proposal);

        let mut refs = HashMap::new();
        refs.insert("Perfect Case".to_string(), LegalReference {
            id: ReferenceId("r1".into()),
            citation: "Perfect Case".into(),
            jurisdiction: Jurisdiction { court: "Supreme".into(), region: "US".into(), level: JurisdictionLevel::Supreme },
            source_text: Some("Actual text".into()),
            procedural_status: ProceduralStatus::GoodLaw,
        });
        let adapter = MockLegalStatusAdapter {
            status_overrides: HashMap::new(),
            treatment_overrides: HashMap::new(),
            check_time: "2026-05-04T00:00:00Z".into(),
            reference_store: refs,
        };

        let results = CommitmentMembrane::try_commit(&mut manifold, &mut explorer, &config, 0, Some(&adapter));

        assert!(results[0].is_ok());
        let cohbit = results[0].as_ref().unwrap();
        assert_eq!(cohbit.receipt.decision, VerificationDecision::Reject);
        assert_eq!(manifold.claims.len(), 0); 
    }

    #[test]
    fn test_hallucination_trap_penalizes_future_proposal() {
        let mut npe = LegalNpe::new(SyntheticLegalNpe);
        npe.memory.record_failure("Case A", Some("Hallucinated Quote"), "hallucination detected");
        let guidance = npe.memory.get_guidance("Case B", Some("Hallucinated Quote"));
        assert!(guidance < 0.5);
    }

    #[test]
    fn test_npe_memory_learning_loop() {
        let mut manifold = setup_manifold();
        let config = setup_config();
        let mut explorer = ExplorationSpace::new();
        let mut npe = LegalNpe::new(SyntheticLegalNpe);

        let proposal = LegalProposal {
            proposal_id: "prop_1".into(),
            title: "Test".into(),
            description: "Test".into(),
            candidate_claims: vec![CandidateClaim {
                text: "X".into(),
                citation: Some("Case 1".into()),
                quoted_text: None,
                authority_label: AuthorityLabel::Persuasive, 
                inference: InferenceKind::UnsupportedHypothesis,
                model_confidence: 1.0,
            }],
        };
        explorer.inject_proposal(proposal);
        
        let mut refs = HashMap::new();
        refs.insert("Case 1".to_string(), LegalReference {
            id: ReferenceId("r1".into()),
            citation: "Case 1".into(),
            jurisdiction: Jurisdiction { court: "Supreme".into(), region: "US".into(), level: JurisdictionLevel::Supreme },
            source_text: Some("Content".into()),
            procedural_status: ProceduralStatus::GoodLaw,
        });
        let adapter = MockLegalStatusAdapter {
            status_overrides: HashMap::new(),
            treatment_overrides: HashMap::new(),
            check_time: "2026-05-04T00:00:00Z".into(),
            reference_store: refs,
        };

        let results = CommitmentMembrane::try_commit(&mut manifold, &mut explorer, &config, 0, Some(&adapter));

        for res in results {
            if let Ok(cohbit) = res {
                if cohbit.receipt.decision == VerificationDecision::Accept {
                    npe.learn(&cohbit.receipt);
                }
            }
        }

        assert!(npe.memory.total_steps > 0);
    }

    #[test]
    fn test_llm_proposal_to_commitment_flow() {
        let mut manifold = setup_manifold();
        let config = setup_config();
        let mut explorer = ExplorationSpace::new();

        let proposal = LegalProposal {
            proposal_id: "prop_1".into(),
            title: "Test Argument".into(),
            description: "An argument that should pass".into(),
            candidate_claims: vec![
                CandidateClaim {
                    text: "Valid Quote".into(),
                    citation: Some("Case 1".into()),
                    quoted_text: Some("Target Text".into()),
                    authority_label: AuthorityLabel::Persuasive, 
                    inference: InferenceKind::DirectQuotation,
                    model_confidence: 1.0,
                }
            ],
        };

        explorer.inject_proposal(proposal);

        let mut refs = HashMap::new();
        refs.insert("Case 1".to_string(), LegalReference {
            id: ReferenceId("r1".into()),
            citation: "Case 1".into(),
            jurisdiction: Jurisdiction { court: "Supreme".into(), region: "US".into(), level: JurisdictionLevel::Supreme },
            source_text: Some("Target Text exists here.".into()),
            procedural_status: ProceduralStatus::GoodLaw,
        });
        let adapter = MockLegalStatusAdapter {
            status_overrides: HashMap::new(),
            treatment_overrides: HashMap::new(),
            check_time: "2026-05-04T00:00:00Z".into(),
            reference_store: refs,
        };

        let results = CommitmentMembrane::try_commit(&mut manifold, &mut explorer, &config, 0, Some(&adapter));

        assert!(results[0].is_ok());
        assert_eq!(results[0].as_ref().unwrap().receipt.decision, VerificationDecision::Accept);
    }

    #[test]
    fn test_overruled_by_record_blocks_commit() {
        let manifold = setup_manifold();
        let config = setup_config();

        let reference = LegalReference {
            id: ReferenceId("r1".into()),
            citation: "Old Case".into(),
            jurisdiction: Jurisdiction { court: "Appellate".into(), region: "US".into(), level: JurisdictionLevel::Appellate },
            source_text: Some("Content".into()),
            procedural_status: ProceduralStatus::GoodLaw, 
        };

        let mut refs = HashMap::new();
        refs.insert("Old Case".to_string(), reference.clone());

        let mut treatments = HashMap::new();
        treatments.insert("Old Case".to_string(), vec![TreatmentRecord {
            treating_citation: "New Case".into(),
            treatment_type: TreatmentType::OverruledBy,
            treatment_summary: "Overruled on other grounds".into(),
            date: Some("2020-01-01".into()),
        }]);

        let adapter = MockLegalStatusAdapter {
            status_overrides: HashMap::new(),
            treatment_overrides: treatments,
            check_time: "2026-05-04T00:00:00Z".into(),
            reference_store: refs,
        };

        let witness = LegalWitness {
            proposed_claim: LegalClaim {
                id: ClaimId("c1".into()),
                text: "X".into(),
                cited_reference: Some(ReferenceId("r1".into())),
                quoted_text: None,
                authority_label: AuthorityLabel::Binding,
                inference: InferenceKind::UnsupportedHypothesis,
            },
            cited_reference: Some(reference),
            support_score: Some(1.0),
            contrary_authorities: vec![],
            user_asserted_good_law: None,
            verifier_notes: vec![],
        };

        let (_, cohbit) = apply(manifold, witness, &config, 0, Some(&adapter)).unwrap();
        assert_eq!(cohbit.receipt.decision, VerificationDecision::Reject);
    }

    #[test]
    fn test_persistence_manager_full_loop() {
        let temp_dir = std::env::temp_dir().join(format!("coh_test_{}", Utc::now().timestamp_nanos_opt().unwrap_or(0)));
        let pm = PersistenceManager::new(&temp_dir).unwrap();
        
        let manifold = setup_manifold();
        let initial_hash = manifold.get_state_hash();
        pm.initialize_chain("test_chain", initial_hash).unwrap();

        let config = setup_config();

        let reference = LegalReference {
            id: ReferenceId("r1".into()),
            citation: "Case 1".into(),
            jurisdiction: Jurisdiction { court: "Supreme".into(), region: "US".into(), level: JurisdictionLevel::Supreme },
            source_text: Some("Content".into()),
            procedural_status: ProceduralStatus::GoodLaw,
        };

        let witness = LegalWitness {
            proposed_claim: LegalClaim {
                id: ClaimId("c1".into()),
                text: "X".into(),
                cited_reference: Some(ReferenceId("r1".into())),
                quoted_text: None,
                authority_label: AuthorityLabel::Persuasive,
                inference: InferenceKind::UnsupportedHypothesis,
            },
            cited_reference: Some(reference),
            support_score: Some(0.5),
            contrary_authorities: vec![],
            user_asserted_good_law: None,
            verifier_notes: vec![],
        };

        let (_, cohbit) = apply(manifold, witness, &config, 0, None).unwrap();
        assert_eq!(cohbit.receipt.decision, VerificationDecision::Accept);
        
        let manifest = pm.append_receipt(&cohbit.receipt).unwrap();
        assert_eq!(manifest.latest_step_index, 1);
        let root = pm.verify_integrity().unwrap();
        assert_eq!(root, manifest.root_commitment);

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_persistence_tamper_detection() {
        let temp_dir = std::env::temp_dir().join(format!("coh_tamper_test_{}", Utc::now().timestamp_nanos_opt().unwrap_or(0)));
        let pm = PersistenceManager::new(&temp_dir).unwrap();
        
        let manifold = setup_manifold();
        let initial_hash = manifold.get_state_hash();
        pm.initialize_chain("test_chain", initial_hash).unwrap();

        let config = setup_config();

        let reference = LegalReference {
            id: ReferenceId("r1".into()),
            citation: "Case 1".into(),
            jurisdiction: Jurisdiction { court: "Supreme".into(), region: "US".into(), level: JurisdictionLevel::Supreme },
            source_text: Some("Content".into()),
            procedural_status: ProceduralStatus::GoodLaw,
        };

        let witness = LegalWitness {
            proposed_claim: LegalClaim { id: ClaimId("c1".into()), text: "X".into(), cited_reference: Some(ReferenceId("r1".into())), quoted_text: None, authority_label: AuthorityLabel::Persuasive, inference: InferenceKind::UnsupportedHypothesis },
            cited_reference: Some(reference), support_score: Some(0.5), contrary_authorities: vec![], user_asserted_good_law: None, verifier_notes: vec![],
        };

        let (_, cohbit) = apply(manifold, witness, &config, 0, None).unwrap();
        pm.append_receipt(&cohbit.receipt).unwrap();

        // Tamper with the log (receipt_hash)
        let log_path = temp_dir.join("receipts.ndjson");
        let log_content = fs::read_to_string(&log_path).unwrap();
        let tampered_content = log_content.replace("\"receipt_hash\":\"", "\"receipt_hash\":\"TAMPERED");
        fs::write(&log_path, tampered_content).unwrap();

        let result = pm.verify_integrity();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Tamper detected"));

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_local_anchor_integrity_loop() {
        let temp_dir = std::env::temp_dir().join(format!("coh_anchor_test_{}", Utc::now().timestamp_nanos_opt().unwrap_or(0)));
        let pm = PersistenceManager::new(&temp_dir).unwrap();
        let initial_manifold = setup_manifold();
        let manifest = pm.initialize_chain("test_chain", initial_manifold.get_state_hash()).unwrap();

        let anchor_adapter = LocalAnchorAdapter::new(&temp_dir).unwrap();
        
        // Anchor the genesis root
        let anchor = anchor_adapter.anchor_root(&manifest).unwrap();
        assert_eq!(anchor.root_commitment, "genesis");
        assert_eq!(anchor.manifest_hash, stable_hash(&manifest));

        // Verify anchor passes
        let is_valid = anchor_adapter.verify_anchor(&anchor, &manifest).unwrap();
        assert!(is_valid);

        // Tamper with manifest root
        let mut tampered_manifest = manifest.clone();
        tampered_manifest.root_commitment = "TAMPERED".into();
        
        // Verify anchor should FAIL
        let is_valid_tampered = anchor_adapter.verify_anchor(&anchor, &tampered_manifest).unwrap();
        assert!(!is_valid_tampered);

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_portable_bundle_export_and_verify() {
        let temp_base = std::env::temp_dir().join(format!("coh_bundle_test_{}", Utc::now().timestamp_nanos_opt().unwrap_or(0)));
        let pm_dir = temp_base.join("pm");
        let bundle_dir = temp_base.join("bundle");
        
        let pm = PersistenceManager::new(&pm_dir).unwrap();
        let initial_manifold = setup_manifold();
        pm.initialize_chain("bundle_chain", initial_manifold.get_state_hash()).unwrap();

        let config = setup_config();
        
        // Setup a mock reference and source
        let reference = LegalReference {
            id: ReferenceId("r1".into()),
            citation: "Case 1".into(),
            jurisdiction: Jurisdiction { court: "Supreme".into(), region: "US".into(), level: JurisdictionLevel::Supreme },
            source_text: Some("Evidence Substrate Content".into()),
            procedural_status: ProceduralStatus::GoodLaw,
        };
        let mut refs = HashMap::new();
        refs.insert("Case 1".to_string(), reference.clone());
        let adapter = MockLegalStatusAdapter {
            status_overrides: HashMap::new(),
            treatment_overrides: HashMap::new(),
            check_time: "2026-05-04".into(),
            reference_store: refs,
        };

        let witness = LegalWitness {
            proposed_claim: LegalClaim { id: ClaimId("c1".into()), text: "X".into(), cited_reference: Some(ReferenceId("r1".into())), quoted_text: Some("Evidence".into()), authority_label: AuthorityLabel::Persuasive, inference: InferenceKind::DirectQuotation },
            cited_reference: Some(reference), support_score: Some(0.5), contrary_authorities: vec![], user_asserted_good_law: None, verifier_notes: vec![],
        };

        // Create a transition with a trace
        let (_, cohbit) = apply(initial_manifold, witness, &config, 0, Some(&adapter)).unwrap();
        pm.append_receipt(&cohbit.receipt).unwrap();

        // Export bundle
        let bm = BundleManager::new(&bundle_dir);
        let adapters: Vec<&dyn LegalSourceAdapter> = vec![&adapter];
        bm.export_bundle(&pm, &adapters).unwrap();

        // Verify bundle
        bm.verify_bundle().unwrap();

        // Tamper with source file
        let source_manifest_data = fs::read_to_string(bundle_dir.join("source_manifest.json")).unwrap();
        let bundle_manifest: BundleManifest = serde_json::from_str(&source_manifest_data).unwrap();
        let source_path = bundle_dir.join("sources").join(&bundle_manifest.source_entries[0].file_name);
        fs::write(source_path, "TAMPERED CONTENT").unwrap();

        // Verify should FAIL
        let result = bm.verify_bundle();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Source integrity failure"));

        fs::remove_dir_all(temp_base).unwrap();
    }
}
