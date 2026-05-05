# CohBit Legal GMI: Schema Definitions

This document defines the JSON/NDJSON schemas used for persistent storage and evidence bundles in the CohBit Legal GMI kernel.

## 1. LegalReceipt (`receipts.ndjson`)

The primary record of a committed transition.

```json
{
  "receipt_hash": "string (hex)",
  "pre_state_hash": "string (hex)",
  "post_state_hash": "string (hex)",
  "last_receipt_hash": "string (hex) | null",
  "decision": "Accept | Reject",
  "step_index": "integer",
  "post_valuation": "integer",
  "trace": "TransitionTrace | null"
}
```

## 2. TransitionTrace (`traces.ndjson`)

Detailed audit trace of the Admissibility Gauntlet and Stability Law verification.

```json
{
  "step_index": "integer",
  "decision": "Accept | Reject",
  "pre_state_hash": "string (hex)",
  "post_state_hash": "string (hex)",
  "prev_receipt_hash": "string (hex) | null",
  "receipt_hash": "string (hex)",
  "delta_v": "integer (coherence gain)",
  "spend": "integer",
  "defect": "integer",
  "valuation_pre": "integer",
  "valuation_post": "integer",
  "adapter_trace": "AdapterTrace | null"
}
```

## 3. ChainManifest (`manifest.json`)

Current state summary of the local receipt chain.

```json
{
  "chain_id": "string",
  "root_commitment": "string (hex) cumulative root",
  "latest_step_index": "integer",
  "latest_receipt_hash": "string (hex) | null",
  "latest_state_hash": "string (hex)",
  "initialized_at": "ISO8601 string"
}
```

## 4. AnchorReceipt (`anchor_receipts.ndjson`)

Notarization records binding the local chain to an external root (e.g., Git, Local).

```json
{
  "anchor_id": "string",
  "root_commitment": "string (hex)",
  "manifest_hash": "string (hex)",
  "anchored_at": "ISO8601 string",
  "adapter_name": "string"
}
```

## 5. BundleManifest (`source_manifest.json`)

Manifest binding source evidence to the exported bundle.

```json
{
  "bundle_id": "string",
  "chain_id": "string",
  "root_commitment": "string (hex)",
  "exported_at": "ISO8601 string",
  "source_entries": [
    {
      "citation": "string",
      "source_id": "string",
      "text_hash": "string (hex)",
      "file_name": "string"
    }
  ]
}
```

## 6. Stability Law (Formal)

For any accepted transition:
$$\text{gain} \ge \text{spend} - \text{defect}$$
where $\text{gain} = \text{valuation\_pre} - \text{valuation\_post}$.
