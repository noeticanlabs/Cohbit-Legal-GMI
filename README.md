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

## Licensing and commercial use

**Proprietary — all rights reserved.** This repository does not grant free commercial or production rights, an open-source license, or any automatic evaluation license.

The copyright notice currently identifies “CohBit”; the rights holder's legal identity and the authority of Noetican Labs to license this work are pending verification. Neither this notice nor the proposed contracts transfers intellectual-property ownership.

See [LICENSE.md](LICENSE.md) for the rights-reservation notice and [legal/README.md](legal/README.md) for the proposed research-evaluation and paid commercial agreements. **The agreement templates are drafts only and do not grant rights until completed and signed by authorized parties.** No commercial pricing or royalty applies without an executed order.

The Software remains a research prototype, not production-qualified for legal advice. Separate technical and independent legal-domain gates govern permitted deployment.

Copyright (c) 2026 CohBit. All rights reserved. Proprietary and Confidential.
