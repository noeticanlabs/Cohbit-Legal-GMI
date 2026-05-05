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
            println!("Proposed exploratory claim from: {}", claim);
            // Implementation placeholder for NPE logic
        }
        Commands::Verify { witness } => {
            println!("Verifying witness from: {}", witness);
            // Implementation placeholder for Gauntlet logic
        }
        Commands::Commit { witness, db } => {
            println!("Committing witness from: {} to {}", witness, db);
            let _pm = PersistenceManager::new(&db)?;
            // In a real scenario, we would load the manifold, apply the witness, 
            // and then append the receipt. For the CLI demo, we acknowledge the flow.
            println!("Persistence manager initialized. Ready for commitment.");
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
