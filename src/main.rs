use clap::{Parser, Subcommand};
use cohbit_legal_gmi::manifold::LegalManifold;
use cohbit_legal_gmi::persistence::PersistenceManager;
// Removed unused import
use anyhow::Result;
use num_rational::BigRational;
use num_traits::FromPrimitive;

#[derive(Parser)]
#[command(name = "cohbit-legal-gmi")]
#[command(about = "Research CLI for computational simulation receipts")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Propose a new legal claim
    Propose {
        #[arg(long)]
        claim: String,
    },
    /// Run the legacy simulation verifier without committing
    Verify {
        #[arg(long)]
        witness: String,
    },
    /// Commit a simulation result to the persistent log (not legal certification)
    Commit {
        #[arg(long)]
        witness: String,
        #[arg(long, default_value = "data/ledger")]
        db: String,
    },
    /// Inspect the current state of the legal manifold and log
    Inspect {
        #[arg(long, default_value = "data/ledger")]
        db: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Propose { claim } => {
            println!("Proposed exploratory claim: {}", claim);
            // In V1, Propose triggers a brainstorm event in the exploration space.
            println!("[NPE] Brainstorming commitment candidates for manifold stability...");
        }
        Commands::Verify { witness } => {
            println!("Verifying witness from: {}", witness);
            let witness_data = std::fs::read_to_string(&witness)?;
            let witness: cohbit_legal_gmi::witness::LegalWitness =
                serde_json::from_str(&witness_data)?;
            let config = cohbit_legal_gmi::verifier::VerificationConfig {
                support_threshold: "0.5".into(),
                allow_unknown_good_law: false,
                require_conflict_disclosure: false,
                target_jurisdiction: None,
                verification_time: chrono::Utc::now().to_rfc3339(),
                max_status_age_days: 30,
                valuation_config: cohbit_legal_gmi::valuation::ValuationConfig::default(),
            };

            let result = cohbit_legal_gmi::verifier::AdmissibilityGauntlet::evaluate(
                &witness, &config, None,
            )?;
            println!("--- Simulation Result (not a legal-proof status) ---");
            println!("Decision: {:?}", result.decision);
            println!("Valuation Delta: {}", result.valuation_delta);
        }
        Commands::Commit { witness, db } => {
            println!("Committing witness from: {} to {}", witness, db);
            let pm = PersistenceManager::new(&db)?;
            let witness_data = std::fs::read_to_string(&witness)?;
            let witness: cohbit_legal_gmi::witness::LegalWitness =
                serde_json::from_str(&witness_data)?;

            let initial_manifold =
                LegalManifold::new(cohbit_legal_gmi::manifold::ProceduralPosture {
                    forum: cohbit_legal_gmi::reference::Jurisdiction {
                        court: "Supreme Court".into(),
                        region: "US".into(),
                        level: cohbit_legal_gmi::reference::JurisdictionLevel::Supreme,
                    },
                    matter_type: "General".into(),
                    stage: "Initial".into(),
                });
            let manifold = if pm.load_manifest().is_ok() {
                pm.recover()?;
                pm.load_state()?
            } else {
                pm.initialize_from_manifold("cohbit-legal-gmi", &initial_manifold)?;
                initial_manifold
            };

            let config = cohbit_legal_gmi::verifier::VerificationConfig {
                support_threshold: "0.5".into(),
                allow_unknown_good_law: false,
                require_conflict_disclosure: false,
                target_jurisdiction: None,
                verification_time: chrono::Utc::now().to_rfc3339(),
                max_status_age_days: 30,
                valuation_config: cohbit_legal_gmi::valuation::ValuationConfig::default(),
            };

            let (next_manifold, cohbit) = cohbit_legal_gmi::cohbit::apply(
                manifold,
                witness,
                &config,
                BigRational::from_i64(0).unwrap(),
                BigRational::from_i64(0).unwrap(),
                None,
            )?;
            let manifest = pm.append_transition(&cohbit.receipt, &next_manifold)?;
            match cohbit.receipt.decision {
                cohbit_legal_gmi::verifier::VerificationDecision::Accept => {
                    println!(
                        "Simulation transition accepted. Step Index: {}",
                        manifest.latest_step_index
                    )
                }
                cohbit_legal_gmi::verifier::VerificationDecision::Reject => {
                    println!(
                        "Evaluation rejected; no claim content committed. Attempt step: {}",
                        manifest.latest_step_index
                    )
                }
                cohbit_legal_gmi::verifier::VerificationDecision::Escalate => {
                    println!("Evaluation escalated for review; no claim content committed. Attempt step: {}", manifest.latest_step_index)
                }
            }
            println!("Root Commitment: {}", manifest.root_commitment);
        }
        Commands::Inspect { db } => {
            println!("Inspecting legal manifold at: {}", db);
            let pm = PersistenceManager::new(&db)?;
            match pm.load_manifest() {
                Ok(manifest) => {
                    println!("--- Chain Manifest ---");
                    println!("Chain ID: {}", manifest.chain_id);
                    println!("Latest Step: {}", manifest.latest_step_index);
                    println!("Latest State Hash: {}", manifest.latest_state_hash);
                    println!("Root Commitment: {}", manifest.root_commitment);
                }
                Err(_) => println!("No manifest found at {}. Run a commit first.", db),
            }

            match pm.verify_integrity() {
                Ok(root) => println!("Integrity Verified. Terminal Root: {}", root),
                Err(e) => println!("INTEGRITY BREACH: {}", e),
            }
        }
    }
    Ok(())
}
