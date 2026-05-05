use anyhow::{Context, Result, anyhow};
use clap::Parser;
use cohbit_legal_gmi::receipt::TransitionTrace;
use cohbit_legal_gmi::verifier::VerificationDecision;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "Formal Audit Runner for CohBit Legal GMI Traces", long_about = None)]
struct Args {
    /// Path to a single TransitionTrace JSON file
    #[arg(short, long)]
    trace: Option<PathBuf>,

    /// Path to a traces.ndjson file for full trajectory audit
    #[arg(short, long)]
    trace_log: Option<PathBuf>,

    /// Path to the previous TransitionTrace JSON file (optional, for continuity check)
    #[arg(short, long)]
    prev: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("--- CohBit Legal GMI Trace Auditor ---");

    if let Some(trace_path) = args.trace {
        let trace_data = fs::read_to_string(&trace_path)
            .with_context(|| format!("Failed to read trace file: {:?}", trace_path))?;
        let trace: TransitionTrace = serde_json::from_str(&trace_data)?;
        
        let mut prev_trace = None;
        if let Some(prev_path) = args.prev {
            let prev_data = fs::read_to_string(&prev_path)?;
            prev_trace = Some(serde_json::from_str::<TransitionTrace>(&prev_data)?);
        }

        audit_single_trace(&trace, prev_trace.as_ref())?;
        println!("[RESULT] SINGLE TRACE AUDIT PASS");
    } else if let Some(log_path) = args.trace_log {
        let file = fs::File::open(&log_path)
            .with_context(|| format!("Failed to open trace log: {:?}", log_path))?;
        let reader = BufReader::new(file);
        let mut prev_trace: Option<TransitionTrace> = None;

        for (i, line) in reader.lines().enumerate() {
            let line = line?;
            let trace: TransitionTrace = serde_json::from_str(&line)
                .with_context(|| format!("Failed to parse trace at line {}", i + 1))?;
            
            audit_single_trace(&trace, prev_trace.as_ref())
                .with_context(|| format!("Audit failed at step {}", trace.step_index))?;
            
            prev_trace = Some(trace);
        }
        println!("[RESULT] FULL TRAJECTORY AUDIT PASS");
    } else {
        anyhow::bail!("Please provide either --trace or --trace-log");
    }

    Ok(())
}

fn audit_single_trace(trace: &TransitionTrace, prev: Option<&TransitionTrace>) -> Result<()> {
    println!("Auditing Step {}: {:?}", trace.step_index, trace.decision);
    let mut issues = Vec::new();

    // 1. Logic Audit
    match trace.decision {
        VerificationDecision::Accept => {
            // Stability Law Check
            let required_delta = trace.spend - trace.defect;
            if trace.delta_v < required_delta {
                issues.push(format!(
                    "Stability Law Violation: coherence_gain ({}) < spend - defect ({} - {} = {})",
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
        }
        VerificationDecision::Reject => {
            // Mutation Law Check (Reject -> No Mutation)
            if trace.pre_state_hash != trace.post_state_hash {
                issues.push(format!(
                    "Mutation Law Violation (Reject): pre_state_hash ({:?}) != post_state_hash ({:?})",
                    trace.pre_state_hash, trace.post_state_hash
                ));
            }

            // Valuation Law Check (Reject -> No Change)
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

    // 2. Chained Continuity Audit
    if let Some(prev) = prev {
        if trace.step_index != prev.step_index + 1 {
            issues.push(format!(
                "Step Index Discontinuity: current index ({}) != prev index + 1 ({})",
                trace.step_index, prev.step_index + 1
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

    if issues.is_empty() {
        Ok(())
    } else {
        for issue in &issues {
            println!("  - [ERROR] {}", issue);
        }
        Err(anyhow!("Audit failed with {} issues", issues.len()))
    }
}
