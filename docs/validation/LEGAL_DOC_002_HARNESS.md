# LEGAL-DOC-002 — Independent held-out evaluator guide

The Python evaluator is intentionally **separate** from Rust/Lean's document candidate generator. It is a *scorer*, not a legal reader, a proof verifier, an authority source, a judge, or a substitute for a full CohBit run.

## Input format

Gold JSONL row (held by an independent evaluator, never fed to the system):

```json
{"case_id":"case-1","document_path":"case-1.txt","document_sha256":"<real 64-hex sha256>","document_family":"contract","gold_label":"entailed","evidence_spans":[{"start_byte":0,"end_byte":18,"quote":"<exact source bytes decoded as UTF-8>"}]}
```

Predictions JSONL row, written by an actual evaluated system:

```json
{"case_id":"case-1","document_sha256":"<same real sha256>","decision":"entailed","evidence_spans":[{"start_byte":0,"end_byte":18,"quote":"<exact source bytes decoded as UTF-8>"}]}
```

Gold labels: `entailed`, `contradicted`, `not_mentioned`, `indeterminate`.
Predictions: `entailed`, `contradicted`, `not_mentioned`, `escalate`.

Evidence positions are **UTF-8 byte offsets** in the exact original text file, not codepoint indices or PDF coordinates. Unreadable formats require a separately verified source extraction with linked original offsets; do not silently substitute OCR output and call it original evidence.

## Run

```bash
python -m unittest discover -s tests -v
python tools/evaluate_documents.py \
  --gold path/to/hidden_gold.jsonl \
  --predictions path/to/full_cohbit_predictions.jsonl \
  --docs-root path/to/authorized_source_documents \
  --mode full_integrated \
  --min-cases 300 \
  --min-families 6 \
  --min-coverage 0.70 \
  --report path/to/evaluation.json
```

These command-line thresholds are **illustrative gates**, not a scientifically established legal safety threshold. Independent reviewers must establish the actual operating criteria and document family counts.

The tool exits 2 for malformed or mismatched inputs, 1 if the specified *technical* gate is not met, and 0 if it is met. **Exit 0 is not legal certification.** It does not prove the predictions came from the full model or that gold labels are legally correct.

The score reports the operator-declared `full_integrated` or `component_control` mode, plus `run_mode_independently_verified: false`. Independently attested execution logs and source snapshots must validate the claim. Component controls cannot pass the full-integrated technical gate.

## Study integrity

- Fix held-out splits **before** executing predictions. Separate near-duplicate versions, templates and authors between development and test; track leaked data.
- Require independent legal experts to review authority status, jurisdiction and disputed cases; preserve disagreements rather than overwrite them.
- Record source provenance, scope, coverage and amendment history outside the scorer.
- Freeze/identify CohBit commit SHA, any language model version, retrieval snapshot, trust-policy hash, formal checker hash and full end-to-end execution logs.
- Preserve all failed, abstained and out-of-domain inputs; do not report only passed cases.
- Repeated tuning on the holdout invalidates its independence; create a new sealed holdout for promotion.
- Test the completed integrated system; modules are separately labeled controls only.
- Do not assume LegalBench datasets are automatically redistributable; check task-specific licenses.
