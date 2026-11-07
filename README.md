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

## License and commercial options

**GNU Affero General Public License v3.0 only (AGPL-3.0-only)** — see [LICENSE](LICENSE) for the terms under which the rights holder offers covered original source code. Third-party material retains its own licenses. See the [release audit](docs/licensing/RELEASE_AUDIT_2026-11-07.md) for unresolved chain-of-title and dependency-compliance evidence.

Under the AGPL, users may use, study, modify, redistribute, and **commercially use** the licensed code while complying with the license. In particular, qualifying distributions and modified network-accessible versions carry corresponding-source obligations. **AGPL does not require profitable users to pay Noetican Labs.** See [OPEN_SOURCE_COMMERCIAL_MODEL.md](OPEN_SOURCE_COMMERCIAL_MODEL.md).

If the copyright owner controls the necessary rights, they may separately offer negotiated **paid commercial licenses** for customers seeking different terms, plus paid integration, hosting, maintenance, and support.

The historical “CohBit” copyright notices are preserved pending verification of the exact individual rights owner. The legacy `Cohbit-Legal-GMI-1.0.0-hardened.zip` archive was **removed from the current tree** because its contents could not be cleared for redistribution; it remains accessible in Git history and should not be treated as part of the current licensed release. Third-party dependencies remain under their respective licenses. This repository remains a research prototype, not a production-qualified legal-advice system.
