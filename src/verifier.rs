use crate::{
    adapter::{AdapterTrace, LegalSourceAdapter, QuoteSpan, TreatmentType},
    claim::InferenceKind,
    receipt::stable_hash,
    reference::{Jurisdiction, JurisdictionLevel, ProceduralStatus},
    valuation::{compute_delta_v, try_parse_rational, ValuationConfig},
    witness::{ClaimEvidenceLink, EvidenceVerificationMethod, LegalWitness},
};
use num_rational::BigRational;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationConfig {
    pub support_threshold: String, // RFAP: String for BigRational
    pub allow_unknown_good_law: bool,
    pub require_conflict_disclosure: bool,
    #[serde(default)]
    pub target_jurisdiction: Option<Jurisdiction>,
    pub verification_time: String,
    pub max_status_age_days: i64,
    pub valuation_config: ValuationConfig,
}

impl VerificationConfig {
    pub fn support_threshold_rat(&self) -> Result<BigRational, anyhow::Error> {
        try_parse_rational(&self.support_threshold)
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
    pub evidence_support: String,
    pub verified_evidence_links: Vec<ClaimEvidenceLink>,
}

impl VerifierResult {
    pub fn valuation_delta_rat(&self) -> Result<BigRational, anyhow::Error> {
        try_parse_rational(&self.valuation_delta)
    }
    pub fn defect_reserve_rat(&self) -> Result<BigRational, anyhow::Error> {
        try_parse_rational(&self.defect_reserve)
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
    pub fn evaluate_verified(
        witness: &LegalWitness,
        config: &VerificationConfig,
        adapter: &dyn LegalSourceAdapter,
    ) -> Result<VerifierResult, crate::error::GmiError> {
        if adapter.trust_level() != crate::adapter::AdapterTrust::Verified {
            return Err(crate::error::GmiError::VerificationFailed(format!(
                "Adapter {} is not trusted for verified admission",
                adapter.name()
            )));
        }
        Self::evaluate(witness, config, Some(adapter))
    }

    pub fn evaluate(
        witness: &LegalWitness,
        config: &VerificationConfig,
        adapter: Option<&dyn LegalSourceAdapter>,
    ) -> Result<VerifierResult, crate::error::GmiError> {
        Self::evaluate_with_claims(witness, config, adapter, &[])
    }

    pub fn evaluate_with_claims(
        witness: &LegalWitness,
        config: &VerificationConfig,
        adapter: Option<&dyn LegalSourceAdapter>,
        available_claims: &[crate::claim::LegalClaim],
    ) -> Result<VerifierResult, crate::error::GmiError> {
        // 1. Build Trace if adapter is present
        let trace = if let Some(a) = adapter {
            build_adapter_trace(witness, a)
        } else {
            None
        };

        let (c3_result, verified_evidence_links, evidence_support) =
            check_c3(witness, config, &trace);

        let mut result = VerifierResult {
            c1_citation_exists: check_c1(witness, adapter),
            c2_citation_binding: check_c2(witness, adapter, &trace),
            c3_claim_source_alignment: c3_result,
            c4_jurisdictional_validity: check_c4(witness, config, &trace),
            c5_procedural_status: check_c5(witness, config, adapter, &trace),
            c6_inference_validity: check_c6(witness, available_claims),
            c7_conflict_disclosure: check_c7(witness, config),
            adapter_trace: trace,
            decision: VerificationDecision::Accept,
            valuation_delta: "0".to_string(),
            defect_reserve: "0".to_string(),
            evidence_support,
            verified_evidence_links,
        };

        // Missing independent evidence is an unresolved review state, not a
        // successful verification and not necessarily an evidentiary falsity.
        if adapter.is_none()
            || (result.adapter_trace.is_none() && witness.cited_reference.is_some())
            || config.target_jurisdiction.is_none()
            || result
                .adapter_trace
                .as_ref()
                .is_some_and(|trace| trace.status_trace.is_none())
            || (!result.c5_procedural_status.passed
                && (result.c5_procedural_status.reason.contains("stale")
                    || result.c5_procedural_status.reason.contains("timestamp")
                    || result.c5_procedural_status.reason.contains("no auditable")))
            || matches!(
                result.adapter_trace.as_ref().map(|t| &t.procedural_status),
                Some(ProceduralStatus::Unknown)
            )
        {
            result.decision = VerificationDecision::Escalate;
        } else if result
            .c3_claim_source_alignment
            .reason
            .starts_with("Invalid")
            || result
                .c3_claim_source_alignment
                .reason
                .contains("outside [0, 1]")
            || !result.c1_citation_exists.passed
            || !result.c2_citation_binding.passed
            || !result.c4_jurisdictional_validity.passed
            || !result.c5_procedural_status.passed
            || !result.c6_inference_validity.passed
            || !result.c7_conflict_disclosure.passed
        {
            result.decision = VerificationDecision::Reject;
        } else if !result.c3_claim_source_alignment.passed {
            result.decision = VerificationDecision::Escalate;
        }

        // Energy/Valuation calc (Numerical Sobriety: Rational-based)
        let delta_v = compute_delta_v(witness, &result, &config.valuation_config)
            .map_err(|e| crate::error::GmiError::InvalidNumeric(e.to_string()))?;
        result.valuation_delta = delta_v.to_string();
        result.defect_reserve = "0".to_string(); // Simplified for now

        Ok(result)
    }
}

fn build_adapter_trace(
    witness: &LegalWitness,
    adapter: &dyn LegalSourceAdapter,
) -> Option<AdapterTrace> {
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
        resolved_source_id: resolved.id.clone(),
        resolved_jurisdiction: resolved.jurisdiction.clone(),
        extracted_text_hash: stable_hash(&full_text),
        quote_span,
        status_trace,
        procedural_status: status,
        status_source: adapter.name(),
        adapter_trust: adapter.trust_level(),
    })
}

fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn check_c1(witness: &LegalWitness, adapter: Option<&dyn LegalSourceAdapter>) -> CheckResult {
    if let Some(reference) = &witness.cited_reference {
        if let Some(a) = adapter {
            match a.resolve_reference(&reference.citation) {
                Ok(_) => CheckResult {
                    passed: true,
                    reason: "Citation verified by adapter".into(),
                },
                Err(e) => CheckResult {
                    passed: false,
                    reason: format!("Adapter failed to resolve citation: {:?}", e),
                },
            }
        } else {
            CheckResult { passed: false, reason: "Citation is asserted by the witness but no independent source adapter is available".into() }
        }
    } else {
        CheckResult {
            passed: false,
            reason: "No citation provided".into(),
        }
    }
}

fn check_c2(
    witness: &LegalWitness,
    adapter: Option<&dyn LegalSourceAdapter>,
    trace: &Option<AdapterTrace>,
) -> CheckResult {
    let claim = &witness.proposed_claim;
    if let Some(quoted) = &claim.quoted_text {
        if let Some(t) = trace {
            if t.quote_span.is_some() {
                return CheckResult {
                    passed: true,
                    reason: "Quote verified by adapter trace span".into(),
                };
            } else {
                return CheckResult {
                    passed: false,
                    reason: "Quote missing from adapter-resolved source text".into(),
                };
            }
        }

        if let Some(reference) = &witness.cited_reference {
            let source_text = if let Some(a) = adapter {
                a.extract_source_text(reference).ok()
            } else {
                reference.source_text.clone()
            };

            if let Some(full_text) = source_text {
                if full_text.contains(quoted) {
                    CheckResult {
                        passed: true,
                        reason: "Quote found in source text".into(),
                    }
                } else {
                    CheckResult {
                        passed: false,
                        reason: "Quote missing from source text".into(),
                    }
                }
            } else {
                CheckResult {
                    passed: false,
                    reason: "Source text unavailable for quote verification".into(),
                }
            }
        } else {
            CheckResult {
                passed: false,
                reason: "Quoted text provided without reference".into(),
            }
        }
    } else {
        if witness.evidence_links.is_empty() {
            CheckResult {
                passed: false,
                reason: "No quoted text or claim-evidence link is available".into(),
            }
        } else {
            CheckResult { passed: true, reason: "No literal quotation asserted; claim-evidence link will be evaluated separately".into() }
        }
    }
}

fn check_c3(
    witness: &LegalWitness,
    config: &VerificationConfig,
    trace: &Option<AdapterTrace>,
) -> (CheckResult, Vec<ClaimEvidenceLink>, String) {
    if let Some(submitted_score) = &witness.support_score {
        let score = match try_parse_rational(submitted_score) {
            Ok(score) => score,
            Err(e) => {
                return (
                    CheckResult {
                        passed: false,
                        reason: format!("Invalid submitted support score: {}", e),
                    },
                    vec![],
                    "0".into(),
                )
            }
        };
        let zero = BigRational::from_integer(0.into());
        let one = BigRational::from_integer(1.into());
        if score < zero || score > one {
            return (
                CheckResult {
                    passed: false,
                    reason: format!("Submitted support score {} is outside [0, 1]", score),
                },
                vec![],
                "0".into(),
            );
        }
    }

    let threshold = match config.support_threshold_rat() {
        Ok(threshold) => threshold,
        Err(e) => {
            return (
                CheckResult {
                    passed: false,
                    reason: format!("Invalid support threshold: {}", e),
                },
                vec![],
                "0".into(),
            )
        }
    };
    let zero = BigRational::from_integer(0.into());
    let one = BigRational::from_integer(1.into());
    if threshold < zero || threshold > one {
        return (
            CheckResult {
                passed: false,
                reason: format!("Support threshold {} is outside [0, 1]", threshold),
            },
            vec![],
            "0".into(),
        );
    }

    let Some(trace) = trace else {
        return (
            CheckResult {
                passed: false,
                reason: "Independent source trace is unavailable".into(),
            },
            vec![],
            "0".into(),
        );
    };

    if let (Some(quote), Some(span)) = (
        witness.proposed_claim.quoted_text.as_ref(),
        trace.quote_span.as_ref(),
    ) {
        if normalize_text(&witness.proposed_claim.text) == normalize_text(quote) {
            let link = ClaimEvidenceLink {
                source_id: span.source_id.clone(),
                passage_start: span.start_byte,
                passage_end: span.end_byte,
                claim_fragment: quote.clone(),
                verification_method: EvidenceVerificationMethod::ExactQuotation,
                verifier_version: "exact-quote-v1".into(),
            };
            return (
                CheckResult {
                    passed: true,
                    reason: "Claim is an exact, source-bound quotation".into(),
                },
                vec![link],
                "1".into(),
            );
        }
    }

    for link in &witness.evidence_links {
        if link.source_id != trace.resolved_source_id
            || link.passage_start >= link.passage_end
            || link.claim_fragment.trim().is_empty()
        {
            continue;
        }
        if let EvidenceVerificationMethod::HumanReviewed { reviewer_id } = &link.verification_method
        {
            if !reviewer_id.trim().is_empty() && !link.verifier_version.trim().is_empty() {
                return (
                    CheckResult {
                        passed: true,
                        reason: format!("Claim support approved by human reviewer {}", reviewer_id),
                    },
                    vec![link.clone()],
                    "1".into(),
                );
            }
        }
    }

    (
        CheckResult {
            passed: false,
            reason: "Source exists, but claim-to-evidence support requires a valid exact-quotation or human-reviewed evidence link".into(),
        },
        vec![],
        "0".into(),
    )
}

fn check_c4(
    witness: &LegalWitness,
    config: &VerificationConfig,
    trace: &Option<AdapterTrace>,
) -> CheckResult {
    let Some(target) = &config.target_jurisdiction else {
        return CheckResult {
            passed: false,
            reason: "Target forum is not configured".into(),
        };
    };
    let Some(trace) = trace else {
        return CheckResult {
            passed: false,
            reason: "Resolved source jurisdiction is unavailable".into(),
        };
    };
    let source = &trace.resolved_jurisdiction;
    if source.level == JurisdictionLevel::Unknown || source.region == "Unknown" {
        return CheckResult {
            passed: false,
            reason: "Resolved source jurisdiction is unknown".into(),
        };
    }

    match witness.proposed_claim.authority_label {
        crate::claim::AuthorityLabel::Binding => {
            let binding = source.region == target.region
                && (source.court == target.court
                    || source.level == JurisdictionLevel::Supreme
                    || (source.level == JurisdictionLevel::Appellate
                        && target.level == JurisdictionLevel::Trial));
            if binding {
                CheckResult {
                    passed: true,
                    reason: "Authority is within the configured forum hierarchy".into(),
                }
            } else {
                CheckResult {
                    passed: false,
                    reason:
                        "Claim labels an authority as binding outside the verified forum hierarchy"
                            .into(),
                }
            }
        }
        _ => {
            if source.region == target.region {
                CheckResult { passed: true, reason: "Authority is relevant within the configured region but is not treated as binding".into() }
            } else {
                CheckResult {
                    passed: true,
                    reason: "Cross-jurisdiction authority is treated as non-binding".into(),
                }
            }
        }
    }
}

fn check_c5(
    witness: &LegalWitness,
    config: &VerificationConfig,
    adapter: Option<&dyn LegalSourceAdapter>,
    trace: &Option<AdapterTrace>,
) -> CheckResult {
    if let Some(trace) = trace {
        let Some(status_trace) = &trace.status_trace else {
            return CheckResult {
                passed: false,
                reason: "Authority status has no auditable status trace".into(),
            };
        };
        if !crate::adapter::verify_status_trace_hash(status_trace) {
            return CheckResult {
                passed: false,
                reason: "Authority status trace hash is invalid".into(),
            };
        }
        let parse_time = |value: &str| {
            chrono::DateTime::parse_from_rfc3339(value)
                .map(|time| time.with_timezone(&chrono::Utc))
                .or_else(|_| {
                    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").map(|date| {
                        date.and_hms_opt(0, 0, 0)
                            .expect("midnight is valid")
                            .and_utc()
                    })
                })
        };
        let checked_at = match parse_time(&status_trace.checked_at) {
            Ok(time) => time,
            Err(_) => {
                return CheckResult {
                    passed: false,
                    reason: "Authority status timestamp is invalid".into(),
                }
            }
        };
        let verification_time = match parse_time(&config.verification_time) {
            Ok(time) => time,
            Err(_) => {
                return CheckResult {
                    passed: false,
                    reason: "Verification policy timestamp is invalid".into(),
                }
            }
        };
        let age = verification_time
            .signed_duration_since(checked_at)
            .num_days();
        if age < 0 || age > config.max_status_age_days {
            return CheckResult {
                passed: false,
                reason: format!(
                    "Authority status is stale or future-dated (age {} days)",
                    age
                ),
            };
        }
    }

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
            ProceduralStatus::Unknown
        }
    } else {
        return CheckResult {
            passed: false,
            reason: "No reference to check procedural status".into(),
        };
    };

    // Check for blocking treatment in trace if available
    if let Some(t) = trace {
        if let Some(st) = &t.status_trace {
            for treatment in &st.contrary_treatment {
                match treatment.treatment_type {
                    TreatmentType::OverruledBy
                    | TreatmentType::ReversedBy
                    | TreatmentType::VacatedBy
                    | TreatmentType::SupersededByStatute => {
                        return CheckResult {
                            passed: false,
                            reason: format!(
                                "Reference is blocked by {:?} in status trace",
                                treatment.treatment_type
                            ),
                        };
                    }
                    _ => {}
                }
            }
        }
    }

    match status {
        ProceduralStatus::GoodLaw => CheckResult {
            passed: true,
            reason: "Reference is good law".into(),
        },
        ProceduralStatus::Overruled
        | ProceduralStatus::Vacated
        | ProceduralStatus::Reversed
        | ProceduralStatus::Superseded => CheckResult {
            passed: false,
            reason: format!("Reference is invalid authority ({:?})", status),
        },
        ProceduralStatus::Unknown => {
            if config.allow_unknown_good_law {
                CheckResult {
                    passed: true,
                    reason: "Unknown status allowed by config".into(),
                }
            } else {
                CheckResult {
                    passed: false,
                    reason: "Reference status is unknown".into(),
                }
            }
        }
    }
}

fn check_c6(witness: &LegalWitness, available_claims: &[crate::claim::LegalClaim]) -> CheckResult {
    match &witness.proposed_claim.inference {
        InferenceKind::DirectQuotation => {
            if witness.proposed_claim.quoted_text.is_some() {
                CheckResult {
                    passed: true,
                    reason: "Direct quotation requires quoted text (satisfied)".into(),
                }
            } else {
                CheckResult {
                    passed: false,
                    reason: "Direct quotation requires quoted text (missing)".into(),
                }
            }
        }
        InferenceKind::RuleApplication {
            rule_claims,
            fact_claims,
        } => {
            if rule_claims.is_empty() || fact_claims.is_empty() {
                return CheckResult { passed: false, reason: "Rule application requires at least one verified rule premise and one verified fact premise".into() };
            }
            if rule_claims
                .iter()
                .chain(fact_claims)
                .any(|id| id == &witness.proposed_claim.id)
            {
                return CheckResult {
                    passed: false,
                    reason: "Inference cannot reference itself as a premise".into(),
                };
            }
            let missing: Vec<_> = rule_claims
                .iter()
                .chain(fact_claims)
                .filter(|id| !available_claims.iter().any(|claim| &claim.id == *id))
                .map(|id| id.0.clone())
                .collect();
            if missing.is_empty() {
                CheckResult {
                    passed: true,
                    reason: "Every rule-application premise is already committed".into(),
                }
            } else {
                CheckResult {
                    passed: false,
                    reason: format!(
                        "Inference references unavailable premises: {}",
                        missing.join(", ")
                    ),
                }
            }
        }
        InferenceKind::UnsupportedHypothesis => CheckResult {
            passed: false,
            reason: "Unsupported hypothesis cannot cross the commitment membrane".into(),
        },
        _ => CheckResult {
            passed: true,
            reason: "Inference kind accepted by V1 policy".into(),
        },
    }
}

fn check_c7(witness: &LegalWitness, config: &VerificationConfig) -> CheckResult {
    if config.require_conflict_disclosure {
        let Some(search) = &witness.adverse_authority_search else {
            return CheckResult {
                passed: false,
                reason: "No adverse-authority search scope or results were supplied".into(),
            };
        };
        if search.scope.trim().is_empty()
            || search.searched_at.trim().is_empty()
            || search.method.trim().is_empty()
        {
            return CheckResult {
                passed: false,
                reason: "Adverse-authority search provenance is incomplete".into(),
            };
        }
        let undisclosed: Vec<_> = search
            .result_citations
            .iter()
            .filter(|citation| {
                !witness
                    .contrary_authorities
                    .iter()
                    .any(|authority| &authority.citation == *citation)
            })
            .cloned()
            .collect();
        if !undisclosed.is_empty() {
            return CheckResult {
                passed: false,
                reason: format!(
                    "Adverse-authority search results were not disclosed: {}",
                    undisclosed.join(", ")
                ),
            };
        }
        CheckResult {
            passed: true,
            reason: format!(
                "Adverse-authority search recorded with {} disclosed result(s)",
                search.result_citations.len()
            ),
        }
    } else {
        CheckResult {
            passed: true,
            reason: "Conflict disclosure not required by config".into(),
        }
    }
}
