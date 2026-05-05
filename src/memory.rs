use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::{ReceiptHash, ClaimId, ReferenceId, StateHash};
use crate::receipt::LegalReceipt;
use crate::verifier::VerificationDecision;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryTier {
    Micro, // Recent receipts and specific claims
    Meso,  // Pattern summaries (e.g., successful fair use arguments)
    Macro, // Jurisdictional trends and system-wide stats
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegalMemoryItem {
    pub receipt_hash: ReceiptHash,
    pub decision: VerificationDecision,
    pub step_index: u64,
    pub context_summary: String,
}

/// Adaptive Memory System for Legal NPE
/// 
/// Stores successful and failed trajectories to guide the NPE toward 
/// admissible legal manifolds and away from "hallucination traps."
pub struct LegalNpeMemory {
    /// MICRO: Recent success/failure receipts
    pub receipts: HashMap<ReceiptHash, LegalReceipt>,
    
    /// MESO: Citation success rate (citation -> count_success, count_total)
    pub citation_history: HashMap<String, (u64, u64)>,
    
    /// MESO: Hallucination patterns (quoted_text -> rejection_count)
    pub hallucination_traps: HashMap<String, u64>,
    
    /// MACRO: System-wide metrics
    pub total_steps: u64,
    pub total_success: u64,
}

impl LegalNpeMemory {
    pub fn new() -> Self {
        Self {
            receipts: HashMap::new(),
            citation_history: HashMap::new(),
            hallucination_traps: HashMap::new(),
            total_steps: 0,
            total_success: 0,
        }
    }

    /// Record a successful citation use manually (for testing or bootstrapping)
    pub fn record_success(&mut self, citation: &str) {
        let entry = self.citation_history.entry(citation.to_string()).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += 1;
    }

    /// Ingest a receipt into memory to update the NPE's internal "world model"
    pub fn ingest(&mut self, receipt: &LegalReceipt) {
        self.receipts.insert(receipt.receipt_hash.clone(), receipt.clone());
        self.total_steps += 1;

        if receipt.decision == VerificationDecision::Accept {
            self.total_success += 1;
            // In a full implementation, we'd extract the citation from the witness
            // and call record_success. For now, this is a placeholder for the logic.
        }
    }

    /// Record a known failure (e.g., hallucinated quote) to avoid it in future brainstorming
    pub fn record_failure(&mut self, citation: &str, quote: Option<&str>, reason: &str) {
        let entry = self.citation_history.entry(citation.to_string()).or_insert((0, 0));
        entry.1 += 1;

        if let Some(q) = quote {
            if reason.contains("hallucination") || reason.contains("not found") || reason.contains("rejected") {
                *self.hallucination_traps.entry(q.to_string()).or_insert(0) += 1;
            }
        }
    }

    /// Guidance: Should the NPE propose this citation/quote combination?
    pub fn get_guidance(&self, citation: &str, quote: Option<&str>) -> f64 {
        let (success, total) = self.citation_history.get(citation).cloned().unwrap_or((0, 0));
        
        let mut score = if total == 0 {
            0.5 // Unknown
        } else {
            (success as f64) / (total as f64)
        };

        if let Some(q) = quote {
            if let Some(trap_count) = self.hallucination_traps.get(q) {
                // Heuristic penalty for known traps
                score *= 0.5f64.powi(*trap_count as i32);
            }
        }

        score
    }
}
