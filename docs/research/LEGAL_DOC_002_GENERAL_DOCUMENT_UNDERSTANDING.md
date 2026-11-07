# LEGAL-DOC-002 — Independent Understanding and Validation of Unseen Legal Documents

**Status: Research architecture and falsifiable qualification plan. Implementation NOT yet demonstrated.**  
**Date:** 2026-11-07  
**Baseline:** CohBit Legal GMI master `98412f34a61625c705768e6ccc25ff20ff5dbf78`  
**Priority:** Stop any assertion that arbitrary legal documents are currently understood or validated automatically.

## Objective

For an unfamiliar legally relevant document and a concrete question, the **complete integrated CohBit system** shall independently identify material passages, propositions, possible obligations and permissions, interpretations, jurisdiction and temporal context; retrieve and verify required external authority; construct candidate formal rules and proof obligations; run independent proof verification; and return either a **bounded, evidenced conclusion** or a **reasoned refusal/escalation**.

"Arbitrary" is an adversarial *input-distribution goal*, not a universal correctness promise. Undecidable, ambiguous, incomplete, misleading, scanned, conflicting, stale or previously unseen domains are permitted inputs, but unqualified conclusions are not permitted outputs.

## 1. Baseline audit — presently implemented

- `src/adapter.rs`: local text and in-memory PDF-page adapters, with a mock legal status source. No general PDF renderer/OCR or authoritative legal retrieval provider is included.
- `src/sources.rs`: corpus signing verifies integrity, not legal authority or legal currentness.
- `src/temporal.rs`: executable derivation from a *bounded fictional notice grammar* for calendar-day closing dates. Other legal drafting, named timezone rules and holiday calendars are not supported.
- `src/proof.rs` and Lean `LegalProof`: exact context/signature binding and a conditional Horn/temporal proof. A sound formal proof from inaccurate, incomplete or mistranslated premises is NOT a sound claim about real-world law.
- `docs/LEGAL_PROOF_001_INDEPENDENT_REVIEW.md`: still an empty external review template.
- `fixtures/held_out/README.md`: no externally labeled held-out set committed.

**No independent free-form legal semantic interpreter, multi-jurisdiction authority coverage guarantee, completeness proof, or end-to-end held-out evidence was found.**

## 2. Decompose "understand" and "validate"

A. **Document acquisition & fidelity** — Support textual and machine-readable formats first; PDF extraction and OCR later. Keep original bytes, MIME/format, cryptographic hash, extraction engine/version, page/section identifiers, order, tables/footnotes, unreadable regions and document version. Exclude or escalate untrusted/defective extraction rather than guessing.

B. **Document structure & meaning** — Independently extract parties, roles, definitions, cross-references, legal modalities (obligation, permission, prohibition, exception), conditions, scope, dates, negation and material ambiguities. Link *every* extracted assertion to exact source byte or page spans. Store alternative interpretations where plausible, with separate provenance for each.

C. **Source authority and applicability** — Distinguish submitted document from governing external law. Determine document type, competent jurisdiction/forum, temporal version, legally relevant facts, controlling versus persuasive authority, amendments, conflicting versions, effective dates, court treatment and exceptions. Fetch independent trust roots where applicable. Lack of exhaustive jurisdictional search must remain a qualified coverage limitation.

D. **Formalization and proof** — Construct typed candidate facts, rules and conditions FROM evidence without passing in target conclusions. Require explicit modality, quantifier, exception and priority semantics. The existing Rust/Lean checker may prove the resulting *formal* statement, but a separate semantic-faithfulness obligation remains.

E. **Challenge & independent checking** — Check extraction against original bytes; run adversarial counterexamples and conflicting readings; compare with independently produced legal expert annotations; reconstruct evidence/receipts from a different verifier process. An LLM judging its own interpretation is not independent verification.

F. **Outputs & governance** — Distinguish `DocumentTextSupported` (what the supplied text says), `ProvedWithinDeclaredFormalization` (logical inference), `LegallyQualified` (independent legal applicability review), `Contradicted`, `Unresolved`, and `OutsideScope`. These are proposed research labels; do NOT substitute them for existing `ProofStatus` without versioned policy and proof obligations. The system must explain why it escalates. Automatic filings/legal advice remain outside scope.

## 3. Canonical processing chain

```text
Unseen source bytes
 -> independent format extraction + exact-span validation
 -> structural parse and document-type classification
 -> typed propositions and alternative interpretations
 -> jurisdiction + authority retrieval (RAR/DGR only if explicitly integrated)
 -> effective-date, amendment, exception and priority analysis
 -> formalization with explicit assumptions + defeaters
 -> independent Rust / Lean proof and trace checks
 -> calibrated scope-limited decision or refusal
 -> canonical immutable receipt / external review
```

Existing CohField RAR/DGR is NOT assumed integrated; if used, measure the *actual* integrated deployment, not stand-in solvers.

## 4. First minimal implementation milestones

- **LD-002-A (now):** Independent evaluation harness with exact-document SHA-256, exact UTF-8 byte-span checks, held-out gold labels, wrong-conclusion and evidence failures, per-family coverage, and explicit full-integrated vs component-control status. This is an evaluation tool, NOT an interpreter.
- **LD-002-B:** Native document ingestion (plain text first, then structured XML/HTML, PDF and OCR) with immutable original/source mapping and extraction failures.
- **LD-002-C:** Unseen-document candidate extraction of definitions, legal norms and exceptions, evidence anchored at original spans; *no prewritten answer templates*.
- **LD-002-D:** Independent validation of semantic translation, legal authority, exceptions and temporal/version queries; generate proof certificates from source spans.
- **LD-002-E:** Integrate LD-002-B–D into full CohBit runtime with existing proof checker, receipts and immutable transitions. No component-only results labeled end-to-end.
- **LD-002-F:** Externally supervised held-out field studies, calibration and measured release thresholds. No legal-advice production claim before qualification.

## 5. Evaluation protocol — independent and reproducible

**Gold annotations:** Legal-domain-qualified evaluators not producing the model predictions; hidden document-level/author-level splits; provenance, permitted dataset use, conflicts and versioning recorded. Gold includes case ID, unseen document bytes/hash, document family, hypothesis, label (entailed / contradicted / not mentioned / indeterminate), and exact evidence spans. “Not mentioned” refers to the specific document, NEVER to all applicable law.

**Predictions:** Generated by the complete running CohBit workflow, without access to the hidden answer labels, prewritten schemas specific to test cases or external solvers standing in for modules. Record code SHA, model and prompt versions, retrieval corpus snapshot, time, execution logs, proof/context hashes, external dependencies and all failures. Module tests are controls only.

**Adversarial strata:** Contracts, statutes, regulations, judicial decisions, administrative notices, policies; long/cross-referenced text, buried exceptions, ambiguous negation, conflicting authority, revoked or superseded versions, time-zone/legal holiday edge cases, contradictory attachments, adversarial prompt injection, missing pages/OCR, fabricated citations, out-of-jurisdiction sources, no-answer cases and previously unseen document families. Use licensed/public data only.

**Primary safety metrics:** False decisive conclusions (including wrong unsupported absence claims), exact quote integrity, incorrect or omitted material evidence, authority/version false-positive rate, uncertainty recognition and unqualified legal-output rate. Report count and binomial uncertainty. **Zero observed errors in a small sample is NOT proof of safety**; evaluate per-document family and difficult stratum.

**Secondary performance metrics:** Answer coverage, correct decisive conclusions, escalation quality, citation completeness, retrieval recall, independent interpretation agreement, temporal calculation correctness, latency/cost and reviewer effort.

**Provisional proposed gates (not statistically qualified):**
1. Zero false decisive conclusions in a frozen, independently labeled hold-out set.
2. Every decisive supported/contradicted assertion cites real matching evidence; zero fabricated citations.
3. Meaningful coverage above a predefined baseline, so always-escalate cannot pass.
4. No improvements measured solely on component tests presented as complete-model results.
5. Runtime proof checker remains independent; formalization and legal qualification are audited separately.
6. Results and confidence intervals published with provenance and limitations. Reviewer-established operating thresholds are required for any deployment.

The included `tools/evaluate_documents.py` is a local **scoring harness**. It cannot attest that the gold labels are correct, a run is truly end-to-end, that legal authority was checked, or that the submitted prediction was created by CohBit. Those are separate proof/evaluation tasks.

## 6. Source-backed frameworks

- ContractNLI (Koreeda & Manning, EMNLP 2021): https://aclanthology.org/2021.findings-emnlp.164/ — document-level entailment/contradiction/not-mentioned and evidence spans.
- LegalBench (Guha et al., NeurIPS 2023): https://papers.neurips.cc/paper_files/paper/2023/hash/89e44582fd28ddfea1ea4dcb0ebbf4b0-Abstract-Datasets_and_Benchmarks.html — 162 tasks across six legal reasoning categories.
- OASIS Akoma Ntoso v1.0: https://www.oasis-open.org/standards/ — legal document interchange structure.
- OASIS LegalRuleML: https://docs.oasis-open.org/legalruleml/legalruleml-core-spec/v1.0/os/ — normative modalities, defeasibility, priority, temporal rules, source attribution.
- NIST AI 600-1 Generative AI Profile: https://nvlpubs.nist.gov/nistpubs/ai/NIST.AI.600-1.pdf — confabulation as an evaluation risk.

**Dataset licensing caution:** LegalBench includes tasks with differing underlying data licenses. Do not mirror or redistribute benchmark data without reviewing the respective task permissions.

## 7. Definition of success

Demonstrate that the **actual full system** constructs substantive, evidence-linked propositions and formal proofs on genuinely unfamiliar document families, detects unsupported or legally unresolved conditions, and consistently refuses to certify conclusions outside its justified scope. Only independent evidence can support the eventual capability claim. A universal guarantee for all possible legal documents cannot be responsibly claimed.
