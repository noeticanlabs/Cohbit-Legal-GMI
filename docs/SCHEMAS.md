# CohBit Legal GMI — Storage Schema Summary

The Rust serialization types are authoritative. All safety-critical numeric values are canonical rational strings.

## LegalReceipt

```text
schema_version, policy_version
receipt_hash
pre_state_hash, post_state_hash
last_receipt_hash
decision: Accept | Reject | Escalate
step_index
post_valuation
trace: TransitionTrace
```

A receipt is valid only when its duplicated fields correspond to the trace and `receipt_hash` equals the domain-separated canonical hash recomputed from the trace.

## TransitionTrace

```text
schema_version, policy_version
step_index, decision
pre_state_hash, post_state_hash
prev_receipt_hash, receipt_hash
delta_v, spend, defect
valuation_pre, valuation_post
adapter_trace
witness
verifier_result
```

The verifier result includes C1–C7 outcomes, independent evidence support, verified evidence links, adapter provenance, and the final decision.

## ChainManifest

```text
version, chain_id
genesis_state_hash
latest_step_index
latest_receipt_hash
latest_state_hash
latest_snapshot_hash
root_commitment
```

`state.json` must match all latest manifest fields. Appends use `pending-transaction.json`, fsync, atomic manifest/state replacement, and an exclusive writer lock.

## BundleManifest and signature

`source_manifest.json` binds the chain/root and source entries. `bundle_hash.txt` hashes every accounted bundle file using its normalized relative path, length, and bytes. `bundle_signature.json` contains an Ed25519 signature over that hash.

The signature establishes origin only when checked with a trusted key acquired outside the bundle.

## Formal scope

Lean proves content-state invariance for rejection/escalation and stability requirements for substantive accepted mutation. It does not prove the truth of cited law, status-provider correctness, semantic-review correctness, or deployment security.
