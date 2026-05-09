# CohBit Legal GMI Architecture

This document describes the structural design of the CohBit Legal GMI kernel, emphasizing the separation of exploration and commitment.

## 1. The Legal Manifold
The `LegalManifold` is the primary state container. It represents a specific legal trajectory through a high-dimensional knowledge space.

- **Claims**: A sequence of verified legal claims.
- **Valuation**: A scalar (represented as `BigRational`) representing the "energy" or "certainty" of the manifold.
- **Step Index**: A monotonically increasing counter for transitions (both Accept and Reject).
- **Hash State**: A recursive hash of all prior claims and state, ensuring history cannot be rewritten.

## 2. Numerical Sobriety (BigRational)
To eliminate non-deterministic rounding errors across different hardware architectures, the kernel uses exact rational arithmetic (`num-rational::BigRational`) for all valuation calculations.
- **Decimal Parsing**: All input scores (e.g., "0.95") are parsed using a deterministic decimal-to-rational algorithm.
- **Stability Enforcement**: The Stability Law is enforced using exact rational comparisons, preventing "Stability Leaks" where floating-point imprecision could allow unauthorized certainty gains.

## 3. Admissibility Gauntlet (C1-C7)
The `AdmissibilityGauntlet` is the gatekeeper. It evaluates a `LegalWitness` against a set of constraints:

- **C1-C2 (Grounding)**: Does the citation exist? Does the quote actually appear in the source?
- **C3-C6 (Reasoning)**: Is the claim supported? Is the authority "Good Law"? Is the inference valid?
- **C7 (Procedural)**: Have conflicts been disclosed?

## 4. Commitment Membrane
The `CommitmentMembrane` manages the transition from a "Candidate" (proposed by an NPE) to a "Committed" claim. 

```text
NPE Proposal -> Membrane -> Verifier -> Stability Check -> Commitment -> Receipt
```

## 5. Formal Transition Layer (Lean 4)
We define the formal semantics of a legal transition in Lean 4.
- **Stability Law**: Prevents arbitrary increases in certainty without work.
- **Mutation Law**: Ensures that a `Reject` decision results in zero state change (except for metadata like `stepIndex`).
- **Audit Law**: Every emitted trace must satisfy the formal invariants proven in Lean.

## 6. Persistence & Anchoring
The kernel uses an append-only `.ndjson` log for receipts and traces.
- **Root Commitment**: $root_n = H(root_{n-1}, \rho_n)$.
- **External Anchors**: The `RootAnchor` trait allows stapling the terminal root to Git history or other audit trails.

## 7. Portable Evidence Bundle
The `BundleManager` can export the entire audit state into a self-contained directory, including the actual source text (evidence substrate). This allows third parties to verify the legal trajectory without trusting the original generator.
## 8. Commercial-Grade Metadata
To support scholarly archival and law-firm-level auditing, the kernel tracks rich metadata:
- **Counsel & Firm ID**: Identifies the generating entities for accountability.
- **Citation Confidence**: Quantifies the model's self-assessment of source binding.
- **Review Status**: Tracks the lifecycle of a witness (Draft, Certified, Adversarial Audit).
- **Metadata Persistence**: All commercial metadata is hashed into the `TransitionTrace`, ensuring that audit trails remain intact across years of storage.
