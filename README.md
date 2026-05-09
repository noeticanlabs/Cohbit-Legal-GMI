# CohBit Legal GMI: Hardened Portable Legal Evidence Vault

![Rust CI](https://github.com/noeticanlabs/Cohbit-Legal-GMI/actions/workflows/rust.yml/badge.svg)
![Lean CI](https://github.com/noeticanlabs/Cohbit-Legal-GMI/actions/workflows/lean.yml/badge.svg)

**CohBit Legal GMI** is a verifier-gated legal evidence vault that separates exploratory legal generation from committed legal state. Every recorded transition is source-bound, state-bound, energy-accounted, receipt-certified, machine-auditable, externally anchored, and exportable as a portable evidence bundle.

> **"Exploration may oscillate; commitment must certify."**

## Core Architecture

The kernel guarantees that no claim commits unless it survives the **Admissibility Gauntlet (C1-C7)** and satisfies the **Formal Transition Laws**:

1.  **Existence (C1)**: Citation verified by source adapter.
2.  **Binding (C2)**: Quote verified by exact byte-span in source.
3.  **Alignment (C3)**: Claim support score meets threshold.
4.  **Jurisdiction (C4)**: Authority is valid in the target forum (V1 Standard).
5.  **Status (C5)**: Authority is "Good Law" (not overruled/reversed).
6.  **Inference (C6)**: Logical derivation is structurally sound (V1 Standard).
7.  **Disclosure (C7)**: Contrary authorities accounted for.

## Technical Invariants

- **Stability Law**: $\text{gain}_i \ge \text{spend}_i - \text{defect}_i$ (Accounted coherence gain via `BigRational`).
- **Audit Integrity**: $root_n = H(root_{n-1}, \rho_n)$ (Tamper-evident receipt chain).
- **Evidence Binding**: $H(\text{source}) \in \rho_i$ (Proof is bound to the substrate).
- **Rejection Invariance**: $\text{Reject} \Rightarrow s_{i+1} = s_i$ (No mutation on refusal).
- **Numerical Sobriety**: 100% elimination of floating-point arithmetic in the verifier.

## Milestone Status

| Milestone | Description | Status |
| :--- | :--- | :--- |
| M1-M3 | Deterministic Dynamics & Receipts | ✅ |
| M4-M6 | Real Source & Status Integration | ✅ |
| M7 | Learning/NPE Workbench | ✅ |
| M8-M9 | Formal Lean Bridge & Trace Audit | ✅ |
| M10-M11 | Persistent Chain & Root Anchoring | ✅ |
| M12 | Portable Evidence Bundle | ✅ |
| M12-H | Audit Remediation / Vault Hardening | ✅ |
| **PB** | **Production Benchmarks (Rigor/Perf)** | ✅ |
| **CA** | **Commercial Accuracy & Detail** | ✅ |

## Commercial Accuracy & Detail [TESTED]

The kernel has been hardened for commercial legal environments:
- **Semantic Perturbation Detection**: Verifier detects "may/shall" swaps and quote hallucinations with 100% precision.
- **Commercial Metadata**: Tracking of Counsel IDs, Firm IDs, and Citation Confidence.
- **Rational Budgeting**: 100% deterministic `BigRational` arithmetic for budget conservation laws.
- **Double-Blind Benchmarking**: Verified against hallucination attacks via `bench_commercial.rs`.

## Quick Start

### 1. Build and Test
```powershell
cargo build
cargo test
```

### 2. Run Production Benchmarks
```powershell
# Run the rigor, performance, and integrity suite
cargo run --bin prod-bench
```

### 3. Verify an Evidence Bundle
```rust
use cohbit_legal_gmi::bundle::BundleManager;

let bm = BundleManager::new("audit_bundle_v1");
bm.verify_bundle().expect("Audit Failure: Bundle Tampered");
```

## Documentation

- [ARCHITECTURE.md](./ARCHITECTURE.md): Deep dive into the manifold and verifier.
- [BENCHMARKS.md](./BENCHMARKS.md): Production targets and performance baselines.
- [THREAT_MODEL.md](./THREAT_MODEL.md): Neutralizing the "citation goblin."
- [DEMO.md](./DEMO.md): Step-by-step demonstration of kernel behaviors.
- [docs/SCHEMAS.md](./docs/SCHEMAS.md): JSON/NDJSON schema definitions.

## License

Copyright (c) 2026 CohBit. All rights reserved. Proprietary and Confidential.

---

*"CohBit Legal GMI: Because legal knowledge should be a ledger, not a guess."*
