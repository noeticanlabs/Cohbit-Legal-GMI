# CohBit Legal GMI Architecture

This document describes the structural design of the CohBit Legal GMI kernel, emphasizing the separation of exploration and commitment.

## 1. The Legal Manifold
The `LegalManifold` is the primary state container. It represents a specific legal trajectory through a high-dimensional knowledge space.

- **Claims**: A sequence of verified legal claims.
- **Valuation**: A scalar representing the "energy" or "certainty" of the manifold.
- **Step Index**: A monotonically increasing counter for transitions.
- **Hash State**: A recursive hash of all prior claims and state, ensuring history cannot be rewritten.

## 2. Admissibility Gauntlet (C1-C7)
The `AdmissibilityGauntlet` is the gatekeeper. It evaluates a `LegalWitness` against a set of constraints:

- **C1-C2 (Grounding)**: Does the citation exist? Does the quote actually appear in the source?
- **C3-C6 (Reasoning)**: Is the claim supported? Is the authority "Good Law"? Is the inference valid?
- **C7 (Procedural)**: Have conflicts been disclosed?

## 3. Commitment Membrane
The `CommitmentMembrane` manages the transition from a "Candidate" (proposed by an NPE) to a "Committed" claim. 

```text
NPE Proposal -> Membrane -> Verifier -> Stability Check -> Commitment -> Receipt
```

## 4. Formal Transition Layer (Lean 4)
We define the formal semantics of a legal transition in Lean 4.
- **Stability Law**: Prevents arbitrary increases in certainty without work.
- **Mutation Law**: Ensures that a `Reject` decision results in zero state change.
- **Audit Law**: Every emitted trace must satisfy the formal invariants proven in Lean.

## 5. Persistence & Anchoring
The kernel uses an append-only `.ndjson` log for receipts and traces.
- **Root Commitment**: $root_n = H(root_{n-1}, \rho_n)$.
- **External Anchors**: The `RootAnchor` trait allows stapling the terminal root to Git history or other audit trails.

## 6. Portable Evidence Bundle
The `BundleManager` can export the entire audit state into a self-contained directory, including the actual source text (evidence substrate). This allows third parties to verify the legal trajectory without trusting the original generator.
