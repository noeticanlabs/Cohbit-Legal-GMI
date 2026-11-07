# CohBit Legal GMI Threat Model: Neutralizing the Citation Goblin

The "Citation Goblin" represents various failure modes in legal AI systems: hallucinations, out-of-date law, and intentional manipulation.

## 1. Hallucinated Quotes (The Fake Span)
- **Threat**: LLM invents a quote and attributes it to a real case.
- **Defense**: **M5 (Quote-Span Receipts)**. The verifier uses a `LegalSourceAdapter` to locate the quote by exact byte-span in the original PDF or text. If the span doesn't match, the claim is rejected.

## 2. Zombie Precedent (The Overruled Case)
- **Threat**: LLM cites a case that was overruled or reversed.
- **Defense**: Legacy status metadata is fail-closed but is not authoritative. Certified admission requires an exact source/status dependency and authenticated source-review role supplied under an external trust policy. Real legal-status coverage remains an external qualification obligation.

## 3. History Rewrite (The Log Edit)
- **Threat**: An attacker modifies an old receipt in the log to change the trajectory.
- **Defense**: **M10 (Persistent Receipt Chain)**. The incremental root commitment ($root_n = H(root_{n-1}, \rho_n)$) ensures that changing any past record invalidates the current root.

## 4. Evidence Swap (The Source Tamper)
- **Threat**: An attacker modifies the source text *after* verification to make a fake claim look real.
- **Defense**: **M12 (Evidence Bundle)**. The bundle hash commits to the actual evidence substrate. If the source text is modified, the bundle verification fails.

## 5. Free Certainty (The Stability Violation)
- **Threat**: A model makes high-certainty claims without providing enough evidentiary "spend."
- **Defense**: **Stability Law**. The kernel enforces $\Delta V \ge spend - defect$. A transition is rejected if the energy balance is negative.
## 6. Commercial Provenance Deletion (The Anonymity Attack)
- **Threat**: An attacker removes or modifies Counsel/Firm identifiers to hide the origin of a hallucinated claim.
- **Defense**: **Hardened Metadata Persistence**. Commercial metadata (CounselId, CitationConfidence) is strictly hashed into the `TransitionTrace`. Any modification to the provenance record invalidates the receipt hash and breaks the chain.

## 7. Self-Authorized Proof (The Reviewer Label)

- **Threat**: A proposal invents a reviewer ID, signs its own corpus, or supplies its own trust root.
- **Defense**: Witness review labels cannot pass interpretation or inference checks. Certified admission obtains authorized role keys, credential records, approved context hashes, revocations, dependencies, and checker hash from operator configuration outside the artifact. Reused reviewer keys fail closed.

## 8. Stale Proof (The Changed Dependency)

- **Threat**: A source, amendment, status record, calendar rule, checker, or credential changes after certification.
- **Defense**: Receipts retain historical artifacts while `refresh_proof_lifecycle` recomputes current usability. Any changed dependency, expiry, or revocation yields `ReevaluationRequired`; certified bundles replay proofs and compare that lifecycle index.

## 9. Trusted Computing Base Limits

The Lean theorem proves entailment relative to admitted propositions. Legal accuracy still depends on source authority, fact qualification, faithful formalization, applicability, coverage, independent reviewers, operator trust configuration, and the checker deployment filesystem. Local tests do not establish those external facts.
