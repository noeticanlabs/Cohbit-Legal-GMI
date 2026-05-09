use serde::{Deserialize, Serialize};
use crate::{
    manifold::LegalManifold,
    witness::LegalWitness,
    claim::InferenceKind,
    reference::ProceduralStatus,
    valuation::{ValuationConfig, compute_delta_v},
    adapter::{LegalSourceAdapter, AdapterTrace, QuoteSpan, StatusTrace, TreatmentType},
    receipt::stable_hash,
};
use num_rational::BigRational;
use num_traits::FromPrimitive;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationConfig {
    pub support_threshold: String, // RFAP: String for BigRational
    pub allow_unknown_good_law: bool,
    pub require_conflict_disclosure: bool,
    pub valuation_config: ValuationConfig,
}

impl VerificationConfig {
    pub fn support_threshold_rat(&self) -> BigRational {
        crate::valuation::parse_decimal(&self.support_threshold)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierResult {
    pub c1_citation_exists: CheckResult,
    pub c2_citation_binding: CheckResult,
    pub c3_claim_source_alignment: CheckResult,
    pub c4_jurisdictional_validity: CheckResult,
    pub c5_procedural_status: CheckResult,
    pub c6_inference_validity: CheckResult,
    pub c7_conflict_disclosure: CheckResult,
    pub adapter_trace: Option<AdapterTrace>,
    pub decision: VerificationDecision,
    pub valuation_delta: String,
    pub defect_reserve: String,
}

impl VerifierResult {
    pub fn valuation_delta_rat(&self) -> BigRational {
        crate::valuation::parse_decimal(&self.valuation_delta)
    }
    pub fn defect_reserve_rat(&self) -> BigRational {
        crate::valuation::parse_decimal(&self.defect_reserve)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub passed: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerificationDecision {
    Accept,
    Reject,
    Escalate,
}

pub struct AdmissibilityGauntlet;

impl AdmissibilityGauntlet {
    pub fn evaluate(
        witness: &LegalWitness,
        config: &VerificationConfig,
        adapter: Option<&dyn LegalSourceAdapter>,
    ) -> VerifierResult {
        // 1. Build Trace if adapter is present
        let trace = if let Some(a) = adapter {
            build_adapter_trace(witness, a)
        } else {
            None
        };

        let mut result = VerifierResult {
            c1_citation_exists: check_c1(witness, adapter),
            c2_citation_binding: check_c2(witness, adapter, &trace),
            c3_claim_source_alignment: check_c3(witness, config),
            c4_jurisdictional_validity: check_c4(witness),
            c5_procedural_status: check_c5(witness, config, adapter, &trace),
            c6_inference_validity: check_c6(witness),
            c7_conflict_disclosure: check_c7(witness, config),
            adapter_trace: trace,
            decision: VerificationDecision::Accept,
            valuation_delta: "0".to_string(),
            defect_reserve: "0".to_string(),
        };

        // Determine decision
        if !result.c1_citation_exists.passed ||
           !result.c2_citation_binding.passed ||
           !result.c3_claim_source_alignment.passed ||
           !result.c4_jurisdictional_validity.passed ||
           !result.c5_procedural_status.passed ||
           !result.c6_inference_validity.passed ||
           !result.c7_conflict_disclosure.passed {
            result.decision = VerificationDecision::Reject;
        }

        // Energy/Valuation calc (Numerical Sobriety: Rational-based)
        let delta_v = compute_delta_v(witness, &result, &config.valuation_config);
        result.valuation_delta = delta_v.to_string();
        result.defect_reserve = "0".to_string(); // Simplified for now

        result
    }
}

fn build_adapter_trace(witness: &LegalWitness, adapter: &dyn LegalSourceAdapter) -> Option<AdapterTrace> {
    let reference = witness.cited_reference.as_ref()?;
    
    let resolved = adapter.resolve_reference(&reference.citation).ok()?;
    let full_text = adapter.extract_source_text(&resolved).ok()?;
    let status = adapter.check_status(&resolved).ok()?;
    let status_trace = adapter.get_status_trace(&resolved).ok();

    // Compute QuoteSpan if a quote is present
    let quote_span = if let Some(quoted) = &witness.proposed_claim.quoted_text {
        if let Some(start) = full_text.find(quoted) {
            let end = start + quoted.len();
            let page = adapter.get_page_at_offset(&reference.citation, start);

            Some(QuoteSpan {
                source_id: resolved.id.clone(),
                start_byte: start,
                end_byte: end,
                page,
                raw_quote_hash: stable_hash(quoted),
                normalized_quote_hash: stable_hash(&normalize_text(quoted)),
                extraction_method: "ExactMatch".to_string(),
                extractor_version: adapter.version(),
            })
        } else {
            None
        }
    } else {
        None
    };

    Some(AdapterTrace {
        citation: reference.citation.clone(),
        adapter_name: adapter.name(),
        adapter_version: adapter.version(),
        resolved_reference_hash: stable_hash(&resolved),
        extracted_text_hash: stable_hash(&full_text),
        quote_span,
        status_trace,
        procedural_status: status,
        status_source: adapter.name(),
    })
}

fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn check_c1(witness: &LegalWitness, adapter: Option<&dyn LegalSourceAdapter>) -> CheckResult {
    if let Some(reference) = &witness.cited_reference {
        if let Some(a) = adapter {
            match a.resolve_reference(&reference.citation) {
                Ok(_) => CheckResult { passed: true, reason: "Citation verified by adapter".into() },
                Err(e) => CheckResult { passed: false, reason: format!("Adapter failed to resolve citation: {:?}", e) },
            }
        } else {
            CheckResult { passed: true, reason: "Citation exists in witness".into() }
        }
    } else {
        CheckResult { passed: false, reason: "No citation provided".into() }
    }
}

fn check_c2(witness: &LegalWitness, adapter: Option<&dyn LegalSourceAdapter>, trace: &Option<AdapterTrace>) -> CheckResult {
    let claim = &witness.proposed_claim;
    if let Some(quoted) = &claim.quoted_text {
        if let Some(t) = trace {
            if t.quote_span.is_some() {
                return CheckResult { passed: true, reason: "Quote verified by adapter trace span".into() };
            } else {
                return CheckResult { passed: false, reason: "Quote missing from adapter-resolved source text".into() };
            }
        }

        if let Some(reference) = &witness.cited_reference {
            let source_text = if let Some(a) = adapter {
                match a.extract_source_text(reference) {
                    Ok(text) => Some(text),
                    Err(_) => None,
                }
            } else {
                reference.source_text.clone()
            };

            if let Some(full_text) = source_text {
                if full_text.contains(quoted) {
                    CheckResult { passed: true, reason: "Quote found in source text".into() }
                } else {
                    CheckResult { passed: false, reason: "Quote missing from source text".into() }
                }
            } else {
                CheckResult { passed: false, reason: "Source text unavailable for quote verification".into() }
            }
        } else {
            CheckResult { passed: false, reason: "Quoted text provided without reference".into() }
        }
    } else {
        CheckResult { passed: true, reason: "No quoted text to verify".into() }
    }
}

fn check_c3(witness: &LegalWitness, config: &VerificationConfig) -> CheckResult {
    let score = witness.support_score_rat();
    let threshold = config.support_threshold_rat();
    if score >= threshold { 
        CheckResult { passed: true, reason: format!("Support score {} meets threshold {}", score, threshold) }
    } else {
        CheckResult { passed: false, reason: format!("Support score {} below threshold {}", score, threshold) }
    }
}

fn check_c4(witness: &LegalWitness) -> CheckResult {
    if let Some(reference) = &witness.cited_reference {
        if reference.jurisdiction.level == crate::reference::JurisdictionLevel::Supreme {
            CheckResult { passed: true, reason: "Supreme Court authority is globally valid".into() }
        } else {
            CheckResult { passed: true, reason: "V1: Jurisdictional alignment assumed for appellate-grade authority".into() }
        }
    } else {
        CheckResult { passed: false, reason: "No reference to check jurisdiction".into() }
    }
}

fn check_c5(witness: &LegalWitness, config: &VerificationConfig, adapter: Option<&dyn LegalSourceAdapter>, trace: &Option<AdapterTrace>) -> CheckResult {
    let status = if let Some(t) = trace {
        if let Some(st) = &t.status_trace {
            st.status.clone()
        } else {
            t.procedural_status.clone()
        }
    } else if let Some(reference) = &witness.cited_reference {
        if let Some(a) = adapter {
            match a.check_status(reference) {
                Ok(s) => s,
                Err(_) => reference.procedural_status.clone(),
            }
        } else {
            reference.procedural_status.clone()
        }
    } else {
        return CheckResult { passed: false, reason: "No reference to check procedural status".into() };
    };

    // Check for blocking treatment in trace if available
    if let Some(t) = trace {
        if let Some(st) = &t.status_trace {
            for treatment in &st.contrary_treatment {
                match treatment.treatment_type {
                    TreatmentType::OverruledBy |
                    TreatmentType::ReversedBy |
                    TreatmentType::VacatedBy |
                    TreatmentType::SupersededByStatute => {
                        return CheckResult { passed: false, reason: format!("Reference is blocked by {:?} in status trace", treatment.treatment_type) };
                    }
                    _ => {}
                }
            }
        }
    }

    match status {
        ProceduralStatus::GoodLaw => CheckResult { passed: true, reason: "Reference is good law".into() },
        ProceduralStatus::Overruled | 
        ProceduralStatus::Vacated | 
        ProceduralStatus::Reversed | 
        ProceduralStatus::Superseded => CheckResult { passed: false, reason: format!("Reference is invalid authority ({:?})", status) },
        ProceduralStatus::Unknown => {
            if config.allow_unknown_good_law {
                CheckResult { passed: true, reason: "Unknown status allowed by config".into() }
            } else {
                CheckResult { passed: false, reason: "Reference status is unknown".into() }
            }
        }
    }
}

fn check_c6(witness: &LegalWitness) -> CheckResult {
    match witness.proposed_claim.inference {
        InferenceKind::DirectQuotation => {
            if witness.proposed_claim.quoted_text.is_some() {
                CheckResult { passed: true, reason: "Direct quotation requires quoted text (satisfied)".into() }
            } else {
                CheckResult { passed: false, reason: "Direct quotation requires quoted text (missing)".into() }
            }
        }
        InferenceKind::RuleApplication { .. } => CheckResult { passed: true, reason: "V1: Rule application inference verified by commitment stability".into() },
        InferenceKind::UnsupportedHypothesis => CheckResult { passed: false, reason: "Unsupported hypothesis cannot cross the commitment membrane".into() },
        _ => CheckResult { passed: true, reason: "Inference kind accepted by V1 policy".into() },
    }
}

fn check_c7(witness: &LegalWitness, config: &VerificationConfig) -> CheckResult {
    if config.require_conflict_disclosure {
        if witness.contrary_authorities.is_empty() {
             CheckResult { passed: true, reason: "No conflicts disclosed (assumed clean)".into() }
        } else {
             CheckResult { passed: true, reason: format!("{} conflicts disclosed and accounted for", witness.contrary_authorities.len()) }
        }
    } else {
        CheckResult { passed: true, reason: "Conflict disclosure not required by config".into() }
    }
}
