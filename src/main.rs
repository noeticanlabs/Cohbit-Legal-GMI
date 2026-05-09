use clap::{Parser, Subcommand};
use cohbit_legal_gmi::persistence::PersistenceManager;
use cohbit_legal_gmi::manifold::LegalManifold;
use cohbit_legal_gmi::ReceiptHash;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "cohbit-legal-gmi")]
#[command(about = "Verifier-certified legal reasoning crate")]
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
    /// Verify a legal witness without committing
    Verify {
        #[arg(long)]
        witness: String,
    },
    /// Commit a verified witness to the persistent log
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
            let witness: cohbit_legal_gmi::witness::LegalWitness = serde_json::from_str(&witness_data)?;
            let config = cohbit_legal_gmi::verifier::VerificationConfig {
                support_threshold: "0.5".into(),
                allow_unknown_good_law: true,
                require_conflict_disclosure: false,
                valuation_config: cohbit_legal_gmi::valuation::ValuationConfig::default(),
            };
            
            let result = cohbit_legal_gmi::verifier::AdmissibilityGauntlet::evaluate(&witness, &config, None);
            println!("--- Verification Result ---");
            println!("Decision: {:?}", result.decision);
            println!("Valuation Delta: {}", result.valuation_delta);
        }
        Commands::Commit { witness, db } => {
            println!("Committing witness from: {} to {}", witness, db);
            let pm = PersistenceManager::new(&db)?;
            let witness_data = std::fs::read_to_string(&witness)?;
            let witness: cohbit_legal_gmi::witness::LegalWitness = serde_json::from_str(&witness_data)?;
            
            let manifold = LegalManifold::new(cohbit_legal_gmi::manifold::ProceduralPosture {
                forum: cohbit_legal_gmi::reference::Jurisdiction {
                    court: "Supreme Court".into(), region: "US".into(), level: cohbit_legal_gmi::reference::JurisdictionLevel::Supreme
                },
                matter_type: "General".into(),
                stage: "Initial".into(),
            });

            let config = cohbit_legal_gmi::verifier::VerificationConfig {
                support_threshold: "0.5".into(),
                allow_unknown_good_law: true,
                require_conflict_disclosure: false,
                valuation_config: cohbit_legal_gmi::valuation::ValuationConfig::default(),
            };

            let (_, cohbit) = cohbit_legal_gmi::cohbit::apply(manifold, witness, &config, 0, None)?;
            let manifest = pm.append_receipt(&cohbit.receipt)?;
            println!("Commit successful. Step Index: {}", manifest.latest_step_index);
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
