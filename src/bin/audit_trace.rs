use anyhow::{Context, Result};
use clap::Parser;
use cohbit_legal_gmi::receipt::TransitionTrace;
use cohbit_legal_gmi::verifier::VerificationDecision;
use std::path::PathBuf;
use std::fs;

#[derive(Parser, Debug)]
#[command(author, version, about = "Formal Audit Runner for CohBit Legal GMI Traces", long_about = None)]
struct Args {
    /// Path to the TransitionTrace JSON file
    #[arg(short, long)]
    trace: PathBuf,

    /// Path to the previous TransitionTrace JSON file (optional, for continuity check)
    #[arg(short, long)]
    prev: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("--- CohBit Legal GMI Trace Auditor ---");
    println!("Loading trace from: {:?}", args.trace);

    let trace_data = fs::read_to_string(&args.trace)
        .with_context(|| format!("Failed to read trace file: {:?}", args.trace))?;
    
    let trace: TransitionTrace = serde_json::from_str(&trace_data)
        .with_context(|| "Failed to parse TransitionTrace JSON")?;

    let mut issues = Vec::new();

    // 1. Decision Logic Audit
    match trace.decision {
        VerificationDecision::Accept => {
            // Stability Law Check
            let required_delta = trace.spend - trace.defect;
            if trace.delta_v < required_delta {
                issues.push(format!(
                    "Stability Law Violation: delta_v ({}) < spend - defect ({} - {} = {})",
                    trace.delta_v, trace.spend, trace.defect, required_delta
                ));
            }

            // Valuation Continuity Check
            let expected_post_valuation = trace.valuation_pre - trace.delta_v;
            if trace.valuation_post != expected_post_valuation {
                issues.push(format!(
                    "Valuation Inconsistency (Accept): valuation_post ({}) != valuation_pre - delta_v ({} - {} = {})",
                    trace.valuation_post, trace.valuation_pre, trace.delta_v, expected_post_valuation
                ));
            }

            // Mutation Check (Hash should change on accept usually, but at least must be valid)
            if trace.pre_state_hash == trace.post_state_hash && trace.step_index > 0 {
                 println!("[WARNING] Accepted transition resulted in no state hash change.");
            }
        }
        VerificationDecision::Reject => {
            // Mutation Law Check (Reject -> No Mutation)
            if trace.pre_state_hash != trace.post_state_hash {
                issues.push(format!(
                    "Mutation Law Violation (Reject): pre_state_hash ({:?}) != post_state_hash ({:?})",
                    trace.pre_state_hash, trace.post_state_hash
                ));
            }

            // Valuation Law Check (Reject -> No Spend)
            if trace.valuation_pre != trace.valuation_post {
                issues.push(format!(
                    "Valuation Law Violation (Reject): valuation_pre ({}) != valuation_post ({})",
                    trace.valuation_pre, trace.valuation_post
                ));
            }
        }
        _ => {
            issues.push(format!("Unsupported decision type: {:?}", trace.decision));
        }
    }

    // 2. Chained Continuity Audit (if prev trace is provided)
    if let Some(prev_path) = args.prev {
        println!("Loading previous trace from: {:?}", prev_path);
        let prev_data = fs::read_to_string(&prev_path)
            .with_context(|| format!("Failed to read previous trace file: {:?}", prev_path))?;
        let prev: TransitionTrace = serde_json::from_str(&prev_data)
            .with_context(|| "Failed to parse previous TransitionTrace JSON")?;

        // Chain Continuity Checks
        if trace.step_index != prev.step_index + 1 {
            issues.push(format!(
                "Step Index Discontinuity: current index ({}) != prev index + 1 ({} + 1 = {})",
                trace.step_index, prev.step_index, prev.step_index + 1
            ));
        }

        if trace.pre_state_hash != prev.post_state_hash {
            issues.push(format!(
                "State Hash Discontinuity: current pre_hash ({:?}) != prev post_hash ({:?})",
                trace.pre_state_hash, prev.post_state_hash
            ));
        }

        if let Some(prev_rho) = &trace.prev_receipt_hash {
            if prev_rho.0 != prev.receipt_hash.0 {
                issues.push(format!(
                    "Receipt Chain Discontinuity: current prev_rho ({:?}) != prev rho ({:?})",
                    prev_rho, prev.receipt_hash
                ));
            }
        }

        if trace.valuation_pre != prev.valuation_post {
            issues.push(format!(
                "Valuation Chain Discontinuity: current valuation_pre ({}) != prev valuation_post ({})",
                trace.valuation_pre, prev.valuation_post
            ));
        }
    }

    // Result Summary
    if issues.is_empty() {
        println!("\n[RESULT] AUDIT PASS");
        println!("Step: {}", trace.step_index);
        println!("Decision: {:?}", trace.decision);
        println!("Stability: {}", trace.delta_v >= (trace.spend - trace.defect));
        Ok(())
    } else {
        println!("\n[RESULT] AUDIT FAIL");
        for issue in issues {
            println!("  - {}", issue);
        }
        anyhow::bail!("Trace Audit Failed")
    }
}
