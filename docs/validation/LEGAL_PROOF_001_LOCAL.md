# LEGAL-PROOF-001 local validation record

Run date: 2026-11-06  
Branch: `remediation/legal-proof-001`  
Base commit: `78947cd0391cb408fa141903e54ada6b969023e5` (validation includes the uncommitted remediation worktree)

## Toolchain

- Rust `1.99.1 (58cf8f9c3 2026-10-13)`
- Cargo `1.99.1 (5f94df478 2026-08-27)`
- Lean `4.29.1`, commit `f72c35b3f637c8c6571d353742168ab66cc22c00`
- Lake `5.0.0-src+f72c35b`
- Local checker SHA-256: `f5967f41472c95f87b68f51c167fba434c8b095e2e4c02d3c866d02b0c9b873a`

The runtime computes and requires the checker's BLAKE3 hash from operator configuration. The SHA-256 value above is an additional reproducibility identifier, not the configured runtime value.

## Commands and results

| Command | Result |
|---|---|
| `cd lean && lake build` | pass, 15 jobs |
| `rg -n '\bsorry\b\|\baxiom\b' lean/LeanKernel/LegalProof.lean` | no matches |
| `cargo test --all-targets --locked` | pass, 29 Rust tests; all binary/example targets built |
| `cargo clippy --all-targets --locked -- -D warnings` | pass |
| `cargo fmt --all -- --check` | pass |

The LEGAL-PROOF-001 runtime test invokes the compiled Lean executable. It compares the same rule-application candidate with the checker removed, persists the accepted proof, recomputes lifecycle status, exports and independently rechecks a bundle, constructs civic output, and rejects a post-export mutation.

## Gate status

| Gate | Local evidence | Status |
|---|---|---|
| G1 Scope | Structured policy/context scope plus `LEGAL_PROOF_001.md` | Engineering complete; independent approval pending |
| G2 Trusted premises | Exact passages, source/retrieval/status records, structured facts, applicability, signed role attestations, operator trust roots | Engineering complete; real authority/domain qualification pending |
| G3 Formal calculus | Lean checker and `checker_sound` entailment theorem; explicit resolved-defeater premises | Pass |
| G4 Proof certificate | Strict versioned schema, exact dependencies, independent replay/mutation failures | Pass locally |
| G5 Rust boundary | `apply_certified` invokes pinned checker; legacy verified APIs fail closed | Pass locally |
| G6 Lifecycle | Immutable receipts plus atomic current lifecycle index; revocation/change test | Pass locally |
| G7 Domain validation | Independent review and held-out legal-domain metrics | Pending external evidence |
| G8 Civic pilot | Fictional deadline output derives a separate qualified result from a fresh proof receipt; submission remains human-authorized | Pass locally, domain qualification pending |

Local fixture results measure engineering behavior only. They are not legal-accuracy, professional-review, production-readiness, or judicial-outcome evidence.
