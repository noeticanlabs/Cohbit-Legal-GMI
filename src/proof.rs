//! LEGAL-PROOF-001. The signature, context qualification and inference checks
//! are separate obligations. TrustPolicy is operator configuration, never input
//! obtained from a proposal or its exported bundle.
use crate::receipt::stable_hash;
use anyhow::{bail, ensure, Context, Result};
use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub const PROOF_POLICY: &str = "LEGAL-PROOF-001/horn-temporal-v2";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProofStatus {
    ProvedWithinScope,
    Qualified,
    Disputed,
    Unknown,
    Rejected,
    ReevaluationRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FactStatus {
    Verified,
    Stipulated,
    Alleged,
    Hypothetical,
    Disputed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HornRule {
    pub body: Vec<u32>,
    #[serde(rename = "resolvedDefeaters")]
    pub resolved_defeaters: Vec<u32>,
    pub head: u32,
    pub priority: u32,
    pub defeats: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProofStep {
    pub atom: u32,
    pub rule: Option<usize>,
}

/// This is the exact JSON input consumed by the compiled Lean checker.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Derivation {
    pub premises: Vec<u32>,
    pub rules: Vec<HornRule>,
    pub steps: Vec<ProofStep>,
    pub conclusion: u32,
    pub temporal: crate::temporal::TemporalCertificate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Passage {
    pub source: String,
    pub start: usize,
    pub end: usize,
    pub exact_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceEvidence {
    pub text: String,
    pub publisher: String,
    pub official_uri: String,
    pub retrieved_at: String,
    pub acquisition_record: String,
    pub revision: String,
    pub status_record: String,
    pub status_checked_at: String,
    pub amendment_history: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FactPremise {
    pub atom: u32,
    pub status: FactStatus,
    pub evidence: Passage,
    pub qualification_record: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthorityEffect {
    Binding,
    Persuasive,
    Inapplicable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleQualification {
    pub passage: Passage,
    pub interpretation: String,
    pub interpretation_version: String,
    pub applicability: String,
    pub applicable_posture_hash: String,
    pub effective_from: String,
    pub effective_until: String,
    pub source_revision: String,
    pub authority_effect: AuthorityEffect,
    /// Explicitly resolved exceptions/priority, not an inferred absence.
    pub exception_resolution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageRecord {
    pub corpus_ids: Vec<String>,
    pub searched_at: String,
    pub method: String,
    pub result_sources: Vec<String>,
    pub limitations: String,
}

/// Signed in full by source, formalization and coverage authorities. Exact
/// domain/context approval is independently configured in TrustPolicy as well.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProofContext {
    pub policy: String,
    pub domain: String,
    pub jurisdiction: String,
    pub procedural_stage: String,
    pub claim_type: String,
    pub claim_id: String,
    pub claim_text: String,
    pub claim_hash: String,
    pub state_hash: String,
    pub posture_hash: String,
    pub verification_time: String,
    pub valid_until: String,
    pub propositions: BTreeMap<u32, String>,
    pub sources: BTreeMap<String, SourceEvidence>,
    pub facts: Vec<FactPremise>,
    pub rules: Vec<HornRule>,
    pub qualifications: Vec<RuleQualification>,
    pub conclusion: u32,
    pub coverage: CoverageRecord,
    pub assumptions: Vec<String>,
    pub unresolved_conflicts: Vec<String>,
    /// IDs resolve against an independently maintained current dependency map.
    pub dependencies: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewAttestation {
    pub key_id: String,
    pub role: String,
    pub reviewed_at: String,
    pub signature_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProofArtifact {
    pub schema_version: String,
    pub context: ProofContext,
    pub reviews: Vec<ReviewAttestation>,
    pub derivation: Derivation,
    pub checker_version: String,
    pub checker_hash: String,
}

/// An authorized key must be provisioned outside the proposal engine. A role
/// is an administrative authorization, not proof of professional competence.
pub struct ReviewAuthority {
    pub key: VerifyingKey,
    pub role: String,
    pub credential_record: String,
    pub authorized_from: String,
    pub authorized_until: String,
}

pub struct DomainReviewAttestation {
    pub record_hash: String,
    pub key_id: String,
    pub reviewed_at: String,
    pub signature: Signature,
}

pub struct TrustPolicy {
    pub domain: String,
    pub jurisdiction: String,
    pub procedural_stage: String,
    pub claim_type: String,
    pub approved_contexts: BTreeSet<String>,
    pub authorities: BTreeMap<String, ReviewAuthority>,
    pub revoked_keys: BTreeSet<String>,
    pub dependencies: BTreeMap<String, String>,
    pub checker_path: PathBuf,
    pub checker_version: String,
    pub checker_hash: String,
    /// External independent-domain review, provisioned by the operator. The
    /// exact record hash must also be a dependency of the reviewed context.
    pub domain_review: Option<DomainReviewAttestation>,
}

/// Deliberately neither Deserialize nor publicly constructible. Callers cannot
/// promote a serialized status to a freshly checked proof.
pub struct CheckedProof {
    artifact: ProofArtifact,
    status: ProofStatus,
}

impl CheckedProof {
    pub fn artifact(&self) -> &ProofArtifact {
        &self.artifact
    }
    pub fn status(&self) -> &ProofStatus {
        &self.status
    }
}

pub fn review_message(
    context: &ProofContext,
    role: &str,
    key_id: &str,
    reviewed_at: &str,
) -> Vec<u8> {
    format!(
        "cohbit/legal-proof/review/v1\n{}\n{}\n{}\n{}",
        role,
        key_id,
        reviewed_at,
        stable_hash(context)
    )
    .into_bytes()
}

fn check_passage(p: &Passage, sources: &BTreeMap<String, SourceEvidence>) -> Result<()> {
    let source = sources.get(&p.source).context("Missing passage source")?;
    ensure!(
        p.start < p.end && source.text.get(p.start..p.end) == Some(p.exact_text.as_str()),
        "Passage does not match exact UTF-8 source bytes"
    );
    Ok(())
}

impl ProofArtifact {
    /// Current usability is recomputed; historical artifacts are not rewritten.
    pub fn usability(&self, policy: &TrustPolicy, now: &str) -> ProofStatus {
        let c = &self.context;
        if self.schema_version != "2.0.0" {
            return ProofStatus::Rejected;
        }
        let dates = chrono::DateTime::parse_from_rfc3339(now)
            .and_then(|n| chrono::DateTime::parse_from_rfc3339(&c.valid_until).map(|e| n <= e));
        if dates != Ok(true)
            || !policy.approved_contexts.contains(&stable_hash(c))
            || self.checker_version != policy.checker_version
            || self.checker_hash != policy.checker_hash
            || c.dependencies
                .iter()
                .any(|(id, hash)| policy.dependencies.get(id) != Some(hash))
            || self.reviews.iter().any(|r| {
                policy.revoked_keys.contains(&r.key_id)
                    || !policy.authorities.contains_key(&r.key_id)
            })
        {
            return ProofStatus::ReevaluationRequired;
        }
        if !c.unresolved_conflicts.is_empty()
            || c.facts.iter().any(|f| f.status == FactStatus::Disputed)
        {
            return ProofStatus::Disputed;
        }
        if c.facts.iter().any(|f| f.status == FactStatus::Unknown) {
            return ProofStatus::Unknown;
        }
        // This function never returns Proved: current integrity still needs check().
        ProofStatus::Qualified
    }

    pub fn check(&self, policy: &TrustPolicy, now: &str) -> Result<CheckedProof> {
        let c = &self.context;
        ensure!(
            self.usability(policy, now) == ProofStatus::Qualified,
            "Context is stale, disputed or unknown"
        );
        ensure!(
            c.policy == PROOF_POLICY
                && c.domain == policy.domain
                && c.jurisdiction == policy.jurisdiction
                && c.procedural_stage == policy.procedural_stage
                && c.claim_type == policy.claim_type,
            "Out of scope policy/domain"
        );
        let at = chrono::DateTime::parse_from_rfc3339(&c.verification_time)?;
        let now = chrono::DateTime::parse_from_rfc3339(now)?;
        ensure!(at <= now, "Future context");
        ensure!(
            !c.sources.is_empty()
                && !c.dependencies.is_empty()
                && !c.coverage.corpus_ids.is_empty()
                && !c.coverage.method.trim().is_empty()
                && !c.coverage.limitations.trim().is_empty(),
            "Missing qualification evidence"
        );
        let searched_at = chrono::DateTime::parse_from_rfc3339(&c.coverage.searched_at)?;
        ensure!(
            searched_at <= at
                && c.coverage
                    .result_sources
                    .iter()
                    .all(|source| c.sources.contains_key(source)),
            "Invalid coverage time or undisclosed search result"
        );
        ensure!(
            c.propositions.get(&c.conclusion) == Some(&c.claim_text),
            "Conclusion is not the precise claim"
        );
        ensure!(
            c.qualifications.len() == c.rules.len(),
            "Missing rule qualification"
        );
        for (id, source) in &c.sources {
            let retrieved_at = chrono::DateTime::parse_from_rfc3339(&source.retrieved_at)?;
            let status_checked_at =
                chrono::DateTime::parse_from_rfc3339(&source.status_checked_at)?;
            ensure!(
                !source.publisher.trim().is_empty()
                    && (source.official_uri.starts_with("https://")
                        || (c.domain.starts_with("fictional-")
                            && source.official_uri.starts_with("fixture://")))
                    && retrieved_at <= at
                    && status_checked_at <= at
                    && !source.acquisition_record.trim().is_empty()
                    && !source.status_record.trim().is_empty()
                    && !source.revision.trim().is_empty(),
                "Incomplete source provenance"
            );
            ensure!(
                c.dependencies.get(&format!("source:{id}")) == Some(&stable_hash(source)),
                "Source missing from dependency graph"
            );
        }
        let mut premises = BTreeSet::new();
        for f in &c.facts {
            ensure!(
                premises.insert(f.atom) && c.propositions.contains_key(&f.atom),
                "Duplicate/undefined premise"
            );
            check_passage(&f.evidence, &c.sources)?;
            ensure!(
                !f.qualification_record.trim().is_empty(),
                "Fact has no qualification record"
            );
        }
        for (index, (r, q)) in c.rules.iter().zip(&c.qualifications).enumerate() {
            ensure!(
                c.propositions.contains_key(&r.head)
                    && r.body
                        .iter()
                        .chain(&r.resolved_defeaters)
                        .all(|a| c.propositions.contains_key(a)),
                "Undefined rule atom"
            );
            check_passage(&q.passage, &c.sources)?;
            let source = c
                .sources
                .get(&q.passage.source)
                .context("Rule source missing")?;
            let effective_from = chrono::DateTime::parse_from_rfc3339(&q.effective_from)?;
            let effective_until = chrono::DateTime::parse_from_rfc3339(&q.effective_until)?;
            ensure!(
                r.defeats.iter().all(|defeated| *defeated != index
                    && c.rules
                        .get(*defeated)
                        .is_some_and(|other| r.priority > other.priority)),
                "Invalid or non-prioritized defeated rule"
            );
            ensure!(
                !q.interpretation.trim().is_empty()
                    && !q.interpretation_version.trim().is_empty()
                    && !q.applicability.trim().is_empty()
                    && !q.exception_resolution.trim().is_empty()
                    && q.authority_effect != AuthorityEffect::Inapplicable
                    && q.applicable_posture_hash == c.posture_hash
                    && q.source_revision == source.revision
                    && effective_from <= at
                    && at <= effective_until,
                "Missing interpretation/applicability/exception resolution"
            );
        }
        let mut roles = BTreeSet::new();
        let mut keys = BTreeSet::new();
        for r in &self.reviews {
            let authority = policy
                .authorities
                .get(&r.key_id)
                .context("Unauthorized reviewer")?;
            let reviewed_at = chrono::DateTime::parse_from_rfc3339(&r.reviewed_at)?;
            let authorized_from = chrono::DateTime::parse_from_rfc3339(&authority.authorized_from)?;
            let authorized_until =
                chrono::DateTime::parse_from_rfc3339(&authority.authorized_until)?;
            ensure!(
                authority.role == r.role
                    && !authority.credential_record.trim().is_empty()
                    && authorized_from <= reviewed_at
                    && reviewed_at <= now
                    && now <= authorized_until
                    && keys.insert(authority.key.to_bytes()),
                "Reviewer role mismatch or reused key"
            );
            let bytes: [u8; 64] = hex::decode(&r.signature_hex)?
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid signature length"))?;
            authority.key.verify_strict(
                &review_message(c, &r.role, &r.key_id, &r.reviewed_at),
                &Signature::from_bytes(&bytes),
            )?;
            roles.insert(r.role.as_str());
        }
        ensure!(
            ["source", "formalization", "coverage"]
                .iter()
                .all(|r| roles.contains(r)),
            "Missing independently authorized review roles"
        );
        ensure!(
            stable_hash(&self.derivation.rules) == stable_hash(&c.rules)
                && self.derivation.premises == c.facts.iter().map(|f| f.atom).collect::<Vec<_>>()
                && self.derivation.conclusion == c.conclusion,
            "Derivation substitutes qualified context"
        );
        self.derivation.temporal.validate(&c.sources)?;
        ensure!(
            self.derivation.temporal.verification_instant == at.timestamp(),
            "Temporal extension selection uses a different verification instant"
        );
        ensure!(
            self.derivation.temporal.canonical_claim_text() == c.claim_text,
            "Temporal calculation does not state the certified claim"
        );
        ensure!(
            self.derivation.steps.len() <= 10_000,
            "Proof exceeds resource limit"
        );
        let input = serde_json::to_vec(&self.derivation)?;
        ensure!(
            input.len() <= 1_000_000,
            "Proof input exceeds resource limit"
        );
        let executable = std::fs::read(&policy.checker_path)?;
        ensure!(
            blake3::hash(&executable).to_hex().as_str() == policy.checker_hash,
            "Checker executable hash mismatch"
        );
        // Deployment must prevent replacement of the operator-owned executable
        // between hashing and exec. This filesystem condition is part of the TCB.
        let mut child = Command::new(&policy.checker_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let mut stdin = child.stdin.take().context("Checker stdin unavailable")?;
        stdin.write_all(&input)?;
        stdin.write_all(b"\n")?;
        drop(stdin);
        let start = Instant::now();
        loop {
            if child.try_wait()?.is_some() {
                break;
            }
            if start.elapsed() > Duration::from_secs(10) {
                child.kill()?;
                child.wait()?;
                bail!("Checker timed out");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        let output = child.wait_with_output()?;
        ensure!(
            output.status.success() && output.stdout == b"LEGAL-PROOF-001:Accept\n",
            "Lean rejected derivation"
        );
        let assumed = !c.assumptions.is_empty()
            || c.facts.iter().any(|f| f.status != FactStatus::Verified)
            || c.qualifications
                .iter()
                .any(|q| q.authority_effect != AuthorityEffect::Binding);
        let independently_qualified = policy.domain_review.as_ref().is_some_and(|review| {
            let Some(authority) = policy.authorities.get(&review.key_id) else {
                return false;
            };
            if authority.role != "domain"
                || authority.credential_record.trim().is_empty()
                || !chrono::DateTime::parse_from_rfc3339(&authority.authorized_from)
                    .is_ok_and(|start| start <= now)
                || !chrono::DateTime::parse_from_rfc3339(&authority.authorized_until)
                    .is_ok_and(|until| until >= now)
                || policy.revoked_keys.contains(&review.key_id)
                || keys.contains(&authority.key.to_bytes())
                || c.dependencies.get("independent-domain-review") != Some(&review.record_hash)
            {
                return false;
            }
            let message = format!(
                "cohbit/legal-proof/domain-review/v1\n{}\n{}\n{}",
                c.domain, review.record_hash, review.reviewed_at
            );
            chrono::DateTime::parse_from_rfc3339(&review.reviewed_at)
                .ok()
                .zip(chrono::DateTime::parse_from_rfc3339(&authority.authorized_from).ok())
                .is_some_and(|(reviewed, start)| start <= reviewed && reviewed <= now)
                && authority
                    .key
                    .verify_strict(message.as_bytes(), &review.signature)
                    .is_ok()
        });
        Ok(CheckedProof {
            artifact: self.clone(),
            status: if assumed || !independently_qualified {
                ProofStatus::Qualified
            } else {
                ProofStatus::ProvedWithinScope
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        anchoring::RootAnchor,
        apply_certified,
        bundle::BundleManager,
        civic::{
            CitizenResponse, CivicClaim, CivicClaimType, CivicVerificationState, HumanReviewState,
        },
        claim::{AuthorityLabel, InferenceKind, LegalClaim},
        cohbit::CertifiedAdmission,
        manifold::{LegalManifold, ProceduralPosture},
        reference::{Jurisdiction, JurisdictionLevel},
        valuation::ValuationConfig,
        verifier::VerificationConfig,
        witness::LegalWitness,
        ClaimId,
    };
    use ed25519_dalek::{Signer, SigningKey};
    use num_rational::BigRational;
    use std::path::Path;

    struct Fixture {
        artifact: ProofArtifact,
        policy: TrustPolicy,
        manifold: LegalManifold,
        witness: LegalWitness,
        config: VerificationConfig,
    }

    fn fixture() -> Fixture {
        let checker_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("lean/.lake/build/bin/legal_proof_check");
        let checker_hash = blake3::hash(
            &std::fs::read(&checker_path)
                .expect("run `lake build legal_proof_check` in lean/ before Rust tests"),
        )
        .to_hex()
        .to_string();
        let forum = Jurisdiction {
            court: "Fictional Public Comment Agency".into(),
            region: "Fictional State".into(),
            level: JurisdictionLevel::Administrative,
        };
        let manifold = LegalManifold::new(ProceduralPosture {
            forum: forum.clone(),
            matter_type: "public-comment-deadline".into(),
            stage: "open-comment".into(),
        });
        let claim = LegalClaim {
            id: ClaimId("deadline-2030-01-31".into()),
            text: "The fictional public-comment deadline is 2030-01-31T17:00:00-06:00.".into(),
            cited_reference: None,
            quoted_text: None,
            authority_label: AuthorityLabel::Binding,
            inference: InferenceKind::RuleApplication {
                rule_claims: vec![],
                fact_claims: vec![],
            },
        };
        let witness = LegalWitness {
            proposed_claim: claim.clone(),
            ..Default::default()
        };
        let source = SourceEvidence {
            text: include_str!("../fixtures/legal_proof_001/fictional_notice.txt").into(),
            publisher: "Fictional Public Comment Agency (test authority)".into(),
            official_uri: "fixture://fictional-agency/notices/2030-v1".into(),
            retrieved_at: "2029-12-20T12:00:00Z".into(),
            acquisition_record: "fixture://notice-2030/retrieved-2029-12-20".into(),
            revision: "notice-v1-with-complete-amendment-history".into(),
            status_record: "fixture status: effective, checked 2030-01-15".into(),
            status_checked_at: "2030-01-15T11:00:00-06:00".into(),
            amendment_history: vec!["No amendments through 2030-01-15 (fictional fixture)".into()],
        };
        let amendment_source = SourceEvidence {
            text: include_str!("../fixtures/legal_proof_001/fictional_amendment_register.txt")
                .into(),
            publisher: "Fictional Public Comment Agency (test authority)".into(),
            official_uri: "fixture://fictional-agency/amendments/2030-v1".into(),
            retrieved_at: "2030-01-15T11:00:00-06:00".into(),
            acquisition_record: "fixture://amendment-register/retrieved-2030-01-15".into(),
            revision: "amendment-register-v1".into(),
            status_record: "fixture register status: current through verification time".into(),
            status_checked_at: "2030-01-15T11:00:00-06:00".into(),
            amendment_history: vec!["No amendments or extensions listed".into()],
        };
        let passage = Passage {
            source: "notice".into(),
            start: 0,
            end: 17,
            exact_text: "FICTIONAL NOTICE:".into(),
        };
        let amendment_fragment = "FICTIONAL AMENDMENT REGISTER:";
        let amendment_passage = Passage {
            source: "amendments".into(),
            start: 0,
            end: amendment_fragment.len(),
            exact_text: amendment_fragment.into(),
        };
        let rule = HornRule {
            body: vec![1],
            resolved_defeaters: vec![3],
            head: 2,
            priority: 10,
            defeats: vec![1],
        };
        let exception_rule = HornRule {
            body: vec![4],
            resolved_defeaters: vec![],
            head: 5,
            priority: 5,
            defeats: vec![],
        };
        let mut sources = BTreeMap::new();
        sources.insert("notice".into(), source.clone());
        sources.insert("amendments".into(), amendment_source.clone());
        let mut propositions = BTreeMap::new();
        propositions.insert(
            1,
            "Authenticated notice date, timezone and no extension as of verification time".into(),
        );
        propositions.insert(2, claim.text.clone());
        propositions.insert(
            3,
            "No authenticated extension or higher-priority exception applies".into(),
        );
        propositions.insert(4, "An authenticated extension applies".into());
        propositions.insert(
            5,
            "The original deadline is displaced by the extension".into(),
        );
        let mut dependencies = BTreeMap::new();
        dependencies.insert("source:notice".into(), stable_hash(&source));
        dependencies.insert("source:amendments".into(), stable_hash(&amendment_source));
        dependencies.insert("calendar-policy".into(), "fictional-calendar-v1".into());
        dependencies.insert("review-policy".into(), "three-role-v1".into());
        let context = ProofContext {
            policy: PROOF_POLICY.into(), domain: "fictional-public-comment-deadline/v1".into(),
            jurisdiction: "Fictional State / Fictional Public Comment Agency".into(),
            procedural_stage: "open-comment".into(), claim_type: "public-comment-deadline".into(),
            claim_id: claim.id.0.clone(), claim_text: claim.text.clone(), claim_hash: stable_hash(&claim),
            state_hash: manifold.get_state_hash().0, posture_hash: stable_hash(&manifold.procedural_posture),
            verification_time: "2030-01-15T12:00:00-06:00".into(),
            valid_until: "2030-01-16T12:00:00-06:00".into(), propositions, sources,
            facts: vec![
                FactPremise { atom: 1, status: FactStatus::Verified, evidence: passage.clone(),
                    qualification_record: "fictional fact review: publication/timezone fields matched exact notice bytes".into() },
                FactPremise { atom: 3, status: FactStatus::Verified, evidence: amendment_passage,
                    qualification_record: "fictional amendment register checked through verification time".into() },
            ],
            rules: vec![rule.clone(), exception_rule.clone()],
            qualifications: vec![
                RuleQualification { passage: passage.clone(),
                    interpretation: "Fictional reviewed rule counts 30 calendar days from publication and uses the stated zone.".into(),
                    interpretation_version: "deadline-formalization-v1".into(),
                    applicability: "Fictional agency, open-comment stage, effective notice v1, verification date 2030-01-15.".into(),
                    applicable_posture_hash: stable_hash(&manifold.procedural_posture),
                    effective_from: "2030-01-01T00:00:00-06:00".into(),
                    effective_until: "2030-02-01T00:00:00-06:00".into(),
                    source_revision: "notice-v1-with-complete-amendment-history".into(),
                    authority_effect: AuthorityEffect::Binding,
                    exception_resolution: "Authenticated amendment history contains no extension; higher-priority base rule defeats the extension rule only with explicit absence premise.".into() },
                RuleQualification { passage,
                    interpretation: "An authenticated extension displaces the original deadline when its premise is established.".into(),
                    interpretation_version: "extension-formalization-v1".into(),
                    applicability: "Same fictional notice, agency, stage, and effective interval.".into(),
                    applicable_posture_hash: stable_hash(&manifold.procedural_posture),
                    effective_from: "2030-01-01T00:00:00-06:00".into(),
                    effective_until: "2030-02-01T00:00:00-06:00".into(),
                    source_revision: "notice-v1-with-complete-amendment-history".into(),
                    authority_effect: AuthorityEffect::Binding,
                    exception_resolution: "Inactive because no authenticated extension premise is admitted.".into() },
            ],
            conclusion: 2,
            coverage: CoverageRecord {
                corpus_ids: vec!["fictional-notice-v1".into(), "fictional-amendment-register-v1".into()],
                searched_at: "2030-01-15T11:00:00-06:00".into(),
                method: "exhaustive scan of the two declared fictional fixture records".into(),
                result_sources: vec!["notice".into(), "amendments".into()],
                limitations: "Only two fictional records through 2030-01-15; no open-world or real-law completeness claim.".into(),
            },
            assumptions: vec![], unresolved_conflicts: vec![], dependencies: dependencies.clone(),
        };
        let keys = [
            SigningKey::from_bytes(&[1; 32]),
            SigningKey::from_bytes(&[2; 32]),
            SigningKey::from_bytes(&[3; 32]),
        ];
        let roles = ["source", "formalization", "coverage"];
        let reviews = keys
            .iter()
            .zip(roles)
            .enumerate()
            .map(|(i, (key, role))| {
                let key_id = format!("review-key-{}", i + 1);
                let reviewed_at = "2030-01-15T11:00:00-06:00";
                ReviewAttestation {
                    signature_hex: hex::encode(
                        key.sign(&review_message(&context, role, &key_id, reviewed_at))
                            .to_bytes(),
                    ),
                    key_id,
                    role: role.into(),
                    reviewed_at: reviewed_at.into(),
                }
            })
            .collect();
        let artifact = ProofArtifact {
            schema_version: "2.0.0".into(),
            context: context.clone(),
            reviews,
            derivation: Derivation {
                premises: vec![1, 3],
                rules: vec![rule, exception_rule],
                steps: vec![
                    ProofStep {
                        atom: 1,
                        rule: None,
                    },
                    ProofStep {
                        atom: 3,
                        rule: None,
                    },
                    ProofStep {
                        atom: 2,
                        rule: Some(0),
                    },
                ],
                conclusion: 2,
                temporal: crate::temporal::TemporalCertificate {
                    publication: crate::temporal::CivilDate {
                        year: 2030,
                        month: 1,
                        day: 1,
                    },
                    publication_day: 21_915,
                    calendar_days: 30,
                    start_day_included: false,
                    target_day: 21_945,
                    close_time: crate::temporal::WallTime {
                        hour: 17,
                        minute: 0,
                        second: 0,
                    },
                    close_second: 61_200,
                    offset_seconds: -21_600,
                    local_second: 1_896_109_200,
                    base_instant: 1_896_130_800,
                    verification_instant: 1_894_730_400,
                    base_evidence_source: "notice".into(),
                    extensions: vec![],
                    selected_extension: None,
                    conclusion_local: crate::temporal::LocalDateTime {
                        date: crate::temporal::CivilDate {
                            year: 2030,
                            month: 1,
                            day: 31,
                        },
                        time: crate::temporal::WallTime {
                            hour: 17,
                            minute: 0,
                            second: 0,
                        },
                        offset_seconds: -21_600,
                    },
                    conclusion_instant: 1_896_130_800,
                },
            },
            checker_version: "lean-4.29.1/legal-proof-horn-temporal-v2".into(),
            checker_hash: checker_hash.clone(),
        };
        let authorities = keys
            .iter()
            .zip(roles)
            .enumerate()
            .map(|(i, (key, role))| {
                (
                    format!("review-key-{}", i + 1),
                    ReviewAuthority {
                        key: key.verifying_key(),
                        role: role.into(),
                        credential_record: format!("fictional-test-credential-{role}"),
                        authorized_from: "2029-01-01T00:00:00Z".into(),
                        authorized_until: "2031-01-01T00:00:00Z".into(),
                    },
                )
            })
            .collect();
        let policy = TrustPolicy {
            domain: context.domain.clone(),
            jurisdiction: context.jurisdiction.clone(),
            procedural_stage: context.procedural_stage.clone(),
            claim_type: context.claim_type.clone(),
            approved_contexts: [stable_hash(&context)].into_iter().collect(),
            authorities,
            revoked_keys: BTreeSet::new(),
            dependencies,
            checker_path,
            checker_version: "lean-4.29.1/legal-proof-horn-temporal-v2".into(),
            checker_hash,
            domain_review: None,
        };
        let config = VerificationConfig {
            support_threshold: "1/2".into(),
            allow_unknown_good_law: false,
            require_conflict_disclosure: true,
            target_jurisdiction: Some(forum),
            verification_time: context.verification_time.clone(),
            max_status_age_days: 1,
            valuation_config: ValuationConfig::default(),
        };
        Fixture {
            artifact,
            policy,
            manifold,
            witness,
            config,
        }
    }

    #[test]
    fn real_lean_checker_controls_certified_runtime_and_receipt() {
        let mut f = fixture();
        let initial = f.manifold.clone();
        let (ablated_state, ablated) = crate::apply(
            initial.clone(),
            f.witness.clone(),
            &f.config,
            BigRational::from_integer(0.into()),
            BigRational::from_integer(0.into()),
            None,
        )
        .unwrap();
        assert_ne!(
            ablated.receipt.decision,
            crate::verifier::VerificationDecision::Accept
        );
        assert!(ablated_state.claims.is_empty());
        let (state, output) = apply_certified(
            f.manifold,
            f.witness,
            &f.config,
            BigRational::from_integer(0.into()),
            BigRational::from_integer(0.into()),
            CertifiedAdmission {
                artifact: &f.artifact,
                policy: &f.policy,
            },
        )
        .unwrap();
        assert_eq!(state.claims.len(), 1);
        assert_eq!(
            state
                .claim_proof_status_at_admission
                .get(&ClaimId("deadline-2030-01-31".into())),
            Some(&ProofStatus::Qualified)
        );
        let trace = output.receipt.trace.as_ref().unwrap();
        assert_eq!(trace.proof_status, Some(ProofStatus::Qualified));
        assert!(trace.legal_proof.is_some());
        let comparison = crate::cohbit::compare_equal_input_workflows(&ablated, &output).unwrap();
        assert_eq!(
            comparison.legacy.decision,
            crate::verifier::VerificationDecision::Escalate
        );
        assert_eq!(
            comparison.certified.decision,
            crate::verifier::VerificationDecision::Accept
        );
        assert!(comparison
            .certified
            .proves
            .iter()
            .any(|p| p.contains("temporal")));
        assert!(serde_json::to_string_pretty(&comparison)
            .unwrap()
            .contains("equal_input_hash"));
        let response = CitizenResponse::build_from_receipts(
            "Fictional deadline pilot".into(),
            vec![CivicClaim {
                claim_id: "deadline-2030-01-31".into(),
                claim_type: CivicClaimType::Procedural,
                text: f.artifact.context.claim_text.clone(),
                agency_id: Some("fictional-agency".into()),
                agency_role: Some("notice issuer".into()),
                jurisdiction: f.artifact.context.jurisdiction.clone(),
                source_ids: vec![],
                effective_date: Some("2030-01-01".into()),
                retrieved_at: Some("2029-12-20T12:00:00Z".into()),
                verification_state: CivicVerificationState::Proposed,
                limitations: vec![
                    "Fictional simulation; qualified pending independent review".into()
                ],
                human_review_state: HumanReviewState::NotReviewed,
            }],
            std::slice::from_ref(&output.receipt),
            &f.policy,
            &f.config.verification_time,
            vec!["Inspect the proof receipt and scope".into()],
        );
        assert!(response.supported_claims.is_empty());
        assert_eq!(response.qualified_claims.len(), 1);
        assert_eq!(
            response.qualified_claims[0].verification_state,
            CivicVerificationState::Qualified
        );
        assert!(!response.external_submission_authorized);

        let mismatched_response = CitizenResponse::build_from_receipts(
            "Mismatched civic claim".into(),
            vec![CivicClaim {
                claim_id: f.artifact.context.claim_id.clone(),
                claim_type: CivicClaimType::Procedural,
                text: "Different claim text under the same ID".into(),
                agency_id: None,
                agency_role: None,
                jurisdiction: f.artifact.context.jurisdiction.clone(),
                source_ids: vec![],
                effective_date: None,
                retrieved_at: None,
                verification_state: CivicVerificationState::Proposed,
                limitations: vec![],
                human_review_state: HumanReviewState::NotReviewed,
            }],
            std::slice::from_ref(&output.receipt),
            &f.policy,
            &f.config.verification_time,
            vec![],
        );
        assert!(mismatched_response.qualified_claims.is_empty());
        assert_eq!(mismatched_response.unresolved_claims.len(), 1);

        let mut rejected = output.receipt.clone();
        rejected.decision = crate::verifier::VerificationDecision::Reject;
        let rejected_trace = rejected.trace.as_mut().unwrap();
        rejected_trace.decision = crate::verifier::VerificationDecision::Reject;
        rejected_trace.verifier_result.decision = crate::verifier::VerificationDecision::Reject;
        let hash = crate::receipt::canonical_receipt_hash(rejected_trace);
        rejected_trace.receipt_hash = hash.clone();
        rejected.receipt_hash = hash;
        assert!(rejected.verify_canonical().is_ok());
        let rejected_response = CitizenResponse::build_from_receipts(
            "Rejected attempt".into(),
            vec![CivicClaim {
                claim_id: f.artifact.context.claim_id.clone(),
                claim_type: CivicClaimType::Procedural,
                text: f.artifact.context.claim_text.clone(),
                agency_id: None,
                agency_role: None,
                jurisdiction: f.artifact.context.jurisdiction.clone(),
                source_ids: vec![],
                effective_date: None,
                retrieved_at: None,
                verification_state: CivicVerificationState::Proposed,
                limitations: vec![],
                human_review_state: HumanReviewState::NotReviewed,
            }],
            &[rejected],
            &f.policy,
            &f.config.verification_time,
            vec![],
        );
        assert!(rejected_response.qualified_claims.is_empty());
        assert_eq!(rejected_response.unresolved_claims.len(), 1);

        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "cohbit-proof-lifecycle-{}-{unique}",
            std::process::id()
        ));
        let pm_dir = root.join("persistence");
        let bundle_dir = root.join("bundle");
        let pm = crate::persistence::PersistenceManager::new(&pm_dir).unwrap();
        pm.initialize_from_manifold("legal-proof-test", &initial)
            .unwrap();
        pm.append_transition(&output.receipt, &state).unwrap();
        let lifecycle = pm
            .assess_proof_lifecycle(&f.policy, &f.config.verification_time)
            .unwrap();
        assert_eq!(lifecycle.len(), 1);
        assert_eq!(lifecycle[0].current_status, ProofStatus::Qualified);
        pm.refresh_proof_lifecycle(&f.policy, &f.config.verification_time)
            .unwrap();
        let manifest = pm.load_manifest().unwrap();
        crate::anchoring::LocalAnchorAdapter::new(&pm_dir)
            .unwrap()
            .anchor_root(&manifest)
            .unwrap();
        let bundle = BundleManager::new(&bundle_dir);
        bundle.export_bundle(&pm, &[]).unwrap();
        assert_eq!(
            bundle
                .verify_certified_bundle(&f.policy, &f.config.verification_time)
                .unwrap(),
            vec![ProofStatus::Qualified]
        );
        let traces = bundle_dir.join("traces.ndjson");
        let mut changed = std::fs::read_to_string(&traces).unwrap();
        changed.push(' ');
        std::fs::write(traces, changed).unwrap();
        assert!(bundle
            .verify_certified_bundle(&f.policy, &f.config.verification_time)
            .is_err());
        f.policy
            .dependencies
            .insert("calendar-policy".into(), "fictional-calendar-v2".into());
        let refreshed = pm
            .refresh_proof_lifecycle(&f.policy, &f.config.verification_time)
            .unwrap();
        assert_eq!(
            refreshed[0].current_status,
            ProofStatus::ReevaluationRequired
        );
        assert!(pm_dir.join("proof-lifecycle.json").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn proof_mutations_and_rust_bypass_fail_closed() {
        let f = fixture();
        let now = &f.config.verification_time;
        assert!(f.artifact.check(&f.policy, now).is_ok());
        for mutation in 0..11 {
            let mut bad = f.artifact.clone();
            match mutation {
                0 => bad.derivation.premises.clear(),
                1 => bad.derivation.steps[2].rule = None,
                2 => bad.derivation.rules[0].head = 99,
                3 => bad
                    .context
                    .sources
                    .get_mut("notice")
                    .unwrap()
                    .text
                    .push_str(" altered"),
                4 => bad.derivation.steps.swap(1, 2),
                5 => {
                    bad.context.sources.remove("amendments");
                }
                6 => {
                    bad.context.claim_text = bad.context.claim_text.replace("-06:00", "-05:00");
                }
                7 => {
                    bad.context.sources.get_mut("notice").unwrap().publisher =
                        "Invented Agency".into();
                }
                8 => {
                    bad.context.claim_text = bad.context.claim_text.replace("01-31", "01-30");
                }
                9 => {
                    bad.context.qualifications[0].interpretation = bad.context.qualifications[0]
                        .interpretation
                        .replace("30", "29");
                }
                _ => {
                    bad.derivation.steps.remove(1);
                }
            }
            assert!(bad.check(&f.policy, now).is_err());
        }
        assert!(crate::apply_verified(
            f.manifold,
            f.witness,
            &f.config,
            BigRational::from_integer(0.into()),
            BigRational::from_integer(0.into()),
            &crate::adapter::MockLegalStatusAdapter::default()
        )
        .is_err());
    }

    #[test]
    fn self_reviewer_wrong_forum_and_expired_status_fail_closed() {
        let mut f = fixture();
        let single_key = SigningKey::from_bytes(&[9; 32]);
        f.policy.authorities.clear();
        f.artifact.reviews = ["source", "formalization", "coverage"]
            .into_iter()
            .enumerate()
            .map(|(i, role)| {
                let key_id = format!("self-{i}");
                let reviewed_at = "2030-01-15T11:00:00-06:00";
                f.policy.authorities.insert(
                    key_id.clone(),
                    ReviewAuthority {
                        key: single_key.verifying_key(),
                        role: role.into(),
                        credential_record: "self-asserted-test-credential".into(),
                        authorized_from: "2029-01-01T00:00:00Z".into(),
                        authorized_until: "2031-01-01T00:00:00Z".into(),
                    },
                );
                ReviewAttestation {
                    signature_hex: hex::encode(
                        single_key
                            .sign(&review_message(
                                &f.artifact.context,
                                role,
                                &key_id,
                                reviewed_at,
                            ))
                            .to_bytes(),
                    ),
                    key_id,
                    role: role.into(),
                    reviewed_at: reviewed_at.into(),
                }
            })
            .collect();
        assert!(f
            .artifact
            .check(&f.policy, &f.config.verification_time)
            .is_err());

        let f = fixture();
        assert_eq!(
            f.artifact.usability(&f.policy, "2030-01-17T12:00:00-06:00"),
            ProofStatus::ReevaluationRequired
        );
        let mut wrong = f.config.clone();
        wrong.target_jurisdiction.as_mut().unwrap().region = "Wrong Forum".into();
        assert!(apply_certified(
            f.manifold,
            f.witness,
            &wrong,
            BigRational::from_integer(0.into()),
            BigRational::from_integer(0.into()),
            CertifiedAdmission {
                artifact: &f.artifact,
                policy: &f.policy
            }
        )
        .is_err());
    }

    #[test]
    fn revoked_review_stale_dependency_and_unqualified_domain_never_overclaim() {
        let mut f = fixture();
        assert_eq!(
            f.artifact
                .check(&f.policy, &f.config.verification_time)
                .unwrap()
                .status(),
            &ProofStatus::Qualified
        );
        let unbound_key = SigningKey::from_bytes(&[8; 32]);
        let unbound_message = format!(
            "cohbit/legal-proof/domain-review/v1\n{}\n{}\n{}",
            f.artifact.context.domain, "unbound-review-record", "2030-01-15T11:00:00-06:00"
        );
        f.policy.authorities.insert(
            "domain-reviewer".into(),
            ReviewAuthority {
                key: unbound_key.verifying_key(),
                role: "domain".into(),
                credential_record: "fictional independent-domain credential".into(),
                authorized_from: "2029-01-01T00:00:00Z".into(),
                authorized_until: "2031-01-01T00:00:00Z".into(),
            },
        );
        f.policy.domain_review = Some(DomainReviewAttestation {
            record_hash: "unbound-review-record".into(),
            key_id: "domain-reviewer".into(),
            reviewed_at: "2030-01-15T11:00:00-06:00".into(),
            signature: unbound_key.sign(unbound_message.as_bytes()),
        });
        assert_eq!(
            f.artifact
                .check(&f.policy, &f.config.verification_time)
                .unwrap()
                .status(),
            &ProofStatus::Qualified
        );
        f.policy.revoked_keys.insert("review-key-2".into());
        assert_eq!(
            f.artifact.usability(&f.policy, &f.config.verification_time),
            ProofStatus::ReevaluationRequired
        );
        f.policy.revoked_keys.clear();
        let mut replaced = fixture();
        replaced
            .policy
            .authorities
            .get_mut("review-key-2")
            .unwrap()
            .key = SigningKey::from_bytes(&[42; 32]).verifying_key();
        assert!(replaced
            .artifact
            .check(&replaced.policy, &replaced.config.verification_time)
            .is_err());
        f.policy
            .dependencies
            .insert("calendar-policy".into(), "fictional-calendar-v2".into());
        assert_eq!(
            f.artifact.usability(&f.policy, &f.config.verification_time),
            ProofStatus::ReevaluationRequired
        );
    }
}
