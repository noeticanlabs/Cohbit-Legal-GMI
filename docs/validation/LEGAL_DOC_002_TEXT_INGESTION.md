# LEGAL-DOC-002-B/C — Byte-faithful text ingestion and lexical candidates

**Status: Tested research component; NOT a legal semantics engine or end-to-end capability.**

Following the independently scored evaluation harness on master, this stage provides strict, locally run evidence preparation for documents not previously seen by the extractor.

## Scope and contracts

- Accept only UTF-8 .txt or .md files up to 2 MB under an authorized local folder. Other formats, including PDF/OCR, are rejected and require separate reviewed extractors.
- Reject byte-order marks, invalid UTF-8, binary control characters, blank files, traversal and symlinked input; preserve original bytes, CRLF and non-ASCII offsets.
- Produce SHA-256 of exact original bytes, per-line source units with original UTF-8 byte spans, and lexical markers for potentially relevant normative language, exceptions, definitions, cross-references and temporal conditions.
- Treat those markers as **uninterpreted candidates**. A word such as 'may' does not justify an assertion about legal permission; the scope of 'unless' is unknown; no external law is retrieved or validated.
- Every candidate includes a trigger span and parent evidence line. The verifier replays exact byte equality.
- This Python tool is not the integrated Rust/Lean runtime, does not construct or sign a proof context, does not verify source legal authority, and never promotes candidate language to legal truth.

## Run locally

```bash
python -m unittest discover -s tests -v
python tools/ingest_documents.py --docs-root fixtures/legal_proof_001 --document fictional_notice.txt --output /tmp/cohbit_ingest.json
```

Inspect the resulting JSON under `assurance`: `legal_interpretation=not_performed`, `external_authority=not_checked`, `formal_proof=not_generated`, and `full_integrated_runtime=false`.

## Independent evaluation correction

LEGAL-DOC-002 evaluator now refuses a claimed `not_mentioned` conclusion until an independently qualified complete-document coverage obligation exists, and all-escalation results fail the scoring gate even with zero configured minimum coverage. These protections do not prove an end-to-end execution: run mode remains operator-declared and not independently attested.

## Next research gate — LEGAL-DOC-002-D

Build a typed candidate formalization interface in Rust which accepts these verified source spans, preserves ambiguity and exception scopes, requires independent source/authority and semantic-faithfulness review, and only then invokes existing Lean proof checking. Add held-out previously unseen contracts, regulations, and notices with external expert annotations and coverage attestations. Do not call lexical extraction legal understanding.

**Local evidence:** 34 Python unit tests pass for the scorer and ingestion modules (verified locally, not through GitHub Actions).

In the fictional notice in `fixtures/legal_proof_001/fictional_notice.txt`, the current lexical extractor emits an uninterpreted `may` marker. This demonstrates exact-source candidate detection, not knowledge of whether any amendment exists or legal derivation of the deadline.
