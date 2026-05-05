# CohBit Legal GMI: Root-Anchored Legal Dynamics Ledger

**CohBit Legal GMI** is a verifier-gated legal reasoning kernel that separates exploratory legal generation from committed legal state. Every committed claim is source-bound, quote-span grounded, status-checked, energy-accounted, receipt-certified, machine-auditable, externally anchored, and exportable as a portable evidence bundle.

> **"Exploration may oscillate; commitment must certify."**

## Core Architecture

The kernel guarantees that no claim commits unless it survives the **Admissibility Gauntlet (C1-C7)** and satisfies the **Formal Transition Laws**:

1.  **Existence (C1)**: Citation verified by source adapter.
2.  **Binding (C2)**: Quote verified by exact byte-span in source.
3.  **Alignment (C3)**: Claim support score meets threshold.
4.  **Jurisdiction (C4)**: Authority is valid in the target forum.
5.  **Status (C5)**: Authority is "Good Law" (not overruled/reversed).
6.  **Inference (C6)**: Logical derivation is structurally sound.
7.  **Disclosure (C7)**: Contrary authorities accounted for.

## Technical Invariants

- **Stability Law**: $\Delta V_i \ge \mathrm{spend}_i - \mathrm{defect}_i$ (No free uncertainty reduction).
- **Audit Integrity**: $root_n = H(root_{n-1}, \rho_n)$ (Tamper-evident receipt chain).
- **Evidence Binding**: $H(\text{source}) \in \rho_i$ (Proof is bound to the substrate).

## Milestone Status

| Milestone | Description | Status |
| :--- | :--- | :--- |
| M1-M3 | Deterministic Dynamics & Receipts | ✅ |
| M4-M6 | Real Source & Status Integration | ✅ |
| M7 | Learning/NPE Workbench | ✅ |
| M8-M9 | Formal Lean Bridge & Trace Audit | ✅ |
| M10-M11 | Persistent Chain & Root Anchoring | ✅ |
| M12 | Portable Evidence Bundle | ✅ |

## Quick Start

### 1. Build and Test
```powershell
cargo build
cargo test
```

### 2. Audit a Trajectory
```powershell
# Verify traces against formal transition laws
cargo run --bin audit_trace path/to/traces.ndjson
```

### 3. Verify an Evidence Bundle
```rust
use cohbit_legal_gmi::bundle::BundleManager;

let bm = BundleManager::new("audit_bundle_v1");
bm.verify_bundle().expect("Audit Failure: Bundle Tampered");
```

## Documentation

- [ARCHITECTURE.md](./ARCHITECTURE.md): Deep dive into the manifold and verifier.
- [THREAT_MODEL.md](./THREAT_MODEL.md): Neutralizing the "citation goblin."
- [DEMO.md](./DEMO.md): Step-by-step demonstration of kernel behaviors.

## License

Copyright (c) 2026 CohBit. All rights reserved. Proprietary and Confidential.

---

*"CohBit Legal GMI: Because legal knowledge should be a ledger, not a guess."*
