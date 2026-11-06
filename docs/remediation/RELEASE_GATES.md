# Release gates

| Gate | Current status |
|---|---|
| Rust/Lean build and local regression suite | Passed locally |
| Source authenticity | Production API blocked until a `Verified` provider exists |
| Jurisdiction/status/support checks | Implemented; external provider qualification missing |
| Rejection/escalation invariance | Tested and modeled in Lean |
| Canonical receipt/replay integrity | Implemented and tested |
| Crash recovery/restart | Implemented with injected-boundary tests |
| Bundle origin authentication | Implemented with external-key Ed25519 verification |
| Privacy/access/security | Deployment implementation and independent assessment missing |
| Independent held-out benchmarks | Missing |
| Human-controlled civic interface/pilot | Missing |
| Accurate public claims | Remediated; maintain under review |

No external legal or advocacy pilot may be represented as qualified while any applicable gate remains missing.
