# CohBit Legal GMI

CohBit Legal GMI is a Rust/Lean research prototype for separating generated legal claims from source-backed committed state. It is not a legal-advice system and is not production-qualified.

## Current qualification

The repository currently provides:

- fail-closed generated-reference defaults;
- exact-quotation checks plus authenticated, exact-context proof-review attestations;
- forum-aware authority checks and time-bounded status traces;
- explicit `Accept`, `Reject`, and `Escalate` outcomes;
- rational arithmetic with bounded, error-returning parsing;
- canonical content-state and receipt commitments;
- restartable transactional persistence with recovery tests;
- Ed25519-signed portable bundles verified against an externally supplied key;
- a Lean model proving that rejection and escalation preserve committed content.

Certified admission requires a complete LEGAL-PROOF-001 artifact, operator-controlled trust policy, and the independently executed pinned Lean checker. A signed corpus is classified as integrity-checked but legally unverified. The experiment remains `Qualified` pending independent legal-domain validation; see [LEGAL_PROOF_001.md](LEGAL_PROOF_001.md).

## Local validation

```bash
cargo fmt --all -- --check
cargo check --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked

cd lean
lake build
```

See `docs/validation/LEGAL_PROOF_001_LOCAL.md` for the scoped proof-validation record.

## Bundle trust boundary

`verify_bundle()` checks internal consistency. Authentic origin requires `verify_bundle_signature()` with a trusted Ed25519 public key obtained outside the bundle. Certified proof replay additionally requires `verify_certified_bundle()` with an external trust policy and current lifecycle dependencies.

## Limitations

- No authoritative live legal source/status provider is included.
- Human review provenance is not a formal proof of legal truth.
- Development benchmarks are not external certifications.
- Deployment-specific privacy, authorization, retention, and data-sharing controls remain required.
- No filing, public comment, official contact, or other advocacy action is performed automatically.

## LEGAL-DOC-002: document ingestion research (component-only)

The experimental [text ingestion tool](tools/ingest_documents.py) reads strict UTF-8 `.txt`/`.md` sources, preserves original byte spans and SHA-256, and emits **uninterpreted lexical candidates** for further independent review. This is *not* independent legal understanding, legal authority verification, a formal proof, or an integrated Rust/Lean runtime. Unsupported PDF and other formats are refused rather than silently guessed.

```bash
python -m unittest discover -s tests -v
python tools/ingest_documents.py --docs-root fixtures/legal_proof_001 --document fictional_notice.txt
```

See [LEGAL-DOC-002 evidence boundaries](docs/validation/LEGAL_DOC_002_TEXT_INGESTION.md) and the [general-document research plan](docs/research/LEGAL_DOC_002_GENERAL_DOCUMENT_UNDERSTANDING.md). The Python research tests are currently **local validation**, not yet an independently attested full-system evaluation.

## Maintainer and licensor identity

**Micheal O. Ellington**, an individual operating as **Noetican Labs** (sole proprietorship), is the project operator's stated original-rights holder and designated commercial licensing contact. No separately incorporated Noetican Labs entity or registered trade name has been independently verified.

See [COPYRIGHT.md](COPYRIGHT.md) for the ownership notice and limitations, and [COMMERCIAL_LICENSE.md](COMMERCIAL_LICENSE.md) for optional separately negotiated licensing.

## License and commercial options

**GNU Affero General Public License v3.0 only (AGPL-3.0-only)** — see [LICENSE](LICENSE) for the terms under which the rights holder offers covered original source code. Third-party material retains its own licenses. See the [release audit](docs/licensing/RELEASE_AUDIT_2026-11-07.md) for unresolved chain-of-title and dependency-compliance evidence.

Under the AGPL, users may use, study, modify, redistribute, and **commercially use** the licensed code while complying with the license. In particular, qualifying distributions and modified network-accessible versions carry corresponding-source obligations. **AGPL does not require profitable users to pay Noetican Labs.** See [OPEN_SOURCE_COMMERCIAL_MODEL.md](OPEN_SOURCE_COMMERCIAL_MODEL.md).

**Micheal O. Ellington, operating as Noetican Labs**, may negotiate optional **paid alternative commercial licenses**, to the extent he owns or has rights to relicense the covered code. Paid hosting, integration, support, and other services may also be offered. No alternative rights are granted without a separately executed agreement.

The project operator has supplied the full name **Micheal O. Ellington**, operating as **Noetican Labs**. The historical “CohBit” notice is preserved as prior project attribution; legal ownership of all historical contributions still requires verification. The legacy `Cohbit-Legal-GMI-1.0.0-hardened.zip` archive was **removed from the current tree** because its contents could not be cleared for redistribution; it remains accessible in Git history and should not be treated as part of the current licensed release. Third-party dependencies remain under their respective licenses. This repository remains a research prototype, not a production-qualified legal-advice system.
