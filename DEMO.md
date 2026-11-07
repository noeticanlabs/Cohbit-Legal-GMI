# CohBit Legal GMI Demonstration

This demo guides you through the five key behaviors of the kernel.

## Setup
Ensure you have Rust installed.
```powershell
cargo build
```

## 1. Simulation Claim Commitment
An exact quote can pass the legacy computational gauntlet. This is not legal certification.
- **Action**: Run `cargo test test_llm_proposal_to_commitment_flow`.
- **Outcome**: `Accept`. A `LegalReceipt` is generated.

## 2. Hallucination Detection (Fake Quote)
A claim where the quote cannot be found in the source text is rejected.
- **Action**: Run `cargo test test_memory_cannot_commit_without_verifier`.
- **Outcome**: `Reject`. The manifold remains unchanged.

## 3. Status Check (Zombie Authority)
A claim citing an overruled case is rejected.
- **Action**: Run `cargo test test_overruled_by_record_blocks_commit`.
- **Outcome**: `Reject`.

## 4. Persistent Integrity (Log Tamper)
If a historical record in `receipts.ndjson` is modified, the system detects it.
- **Action**: Run `cargo test test_persistence_tamper_detection`.
- **Outcome**: `Audit Fail`. Recomputation of the root hash fails.

## 5. Portable Evidence Verification (Source Tamper)
If the source text inside an exported bundle is modified, the bundle becomes invalid.
- **Action**: Run `cargo test test_portable_bundle_export_and_verify`.
- **Outcome**: `Bundle Verification Fail`. Source substrate hash mismatch.

---

### Running the Full Audit Loop
You can generate a sample trajectory and verify it using the audit binary:
```powershell
# Replays the traces and checks formal stability/mutation laws
cargo run --bin coh-audit -- --trace-log path/to/sample/traces.ndjson
```
