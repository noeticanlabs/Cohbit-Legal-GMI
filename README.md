# CohBit Legal GMI

CohBit Legal GMI is a Rust/Lean research prototype for separating generated legal claims from source-backed committed state. It is not a legal-advice system and is not production-qualified.

## Current qualification

The repository currently provides:

- fail-closed generated-reference defaults;
- exact-quotation and human-reviewed claim-evidence links;
- forum-aware authority checks and time-bounded status traces;
- explicit `Accept`, `Reject`, and `Escalate` outcomes;
- rational arithmetic with bounded, error-returning parsing;
- canonical content-state and receipt commitments;
- restartable transactional persistence with recovery tests;
- Ed25519-signed portable bundles verified against an externally supplied key;
- a Lean model proving that rejection and escalation preserve committed content.

The production admission API requires an adapter classified as `Verified`. Included adapters are local-unverified or test-only, so live legal verification remains blocked pending authoritative source integration and independent qualification.

## Local validation

```bash
cargo fmt --all -- --check
cargo check --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked

cd lean
lake build
```

See `docs/validation/BASELINE.md` for the recorded remediation baseline.

## Bundle trust boundary

`verify_bundle()` checks internal consistency. Authentic origin requires `verify_bundle_signature()` with a trusted Ed25519 public key obtained outside the bundle.

## Limitations

- No authoritative live legal source/status provider is included.
- Human review provenance is not a formal proof of legal truth.
- Development benchmarks are not external certifications.
- Deployment-specific privacy, authorization, retention, and data-sharing controls remain required.
- No filing, public comment, official contact, or other advocacy action is performed automatically.

## License

Copyright (c) 2026 CohBit. All rights reserved. Proprietary and Confidential.
