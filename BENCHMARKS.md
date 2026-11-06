# CohBit Legal GMI — Benchmark and Qualification Policy

This repository does not currently support a production-grade, certified, or universally accurate legal-verification claim.

## Locally tested gates

The remediation suite covers generated trust defaults, missing sources, quotation mismatch, unrelated claims, status treatment and staleness, jurisdiction, inference premises, adverse-authority search provenance, malformed numbers, canonical receipts, interrupted-write recovery, restart continuity, bundle tampering, and external-key signatures.

Passing these tests means only that the checked fixtures behaved as specified under the recorded toolchain. It does not establish live-law accuracy or a real-world false-accept rate.

## Required external evaluation

| Dimension | Required measures |
|---|---|
| Citation resolution | false accepts, false rejects, unavailable/indeterminate |
| Quotation integrity | exact-span accuracy and extraction failures |
| Claim support | results by claim type and human/automated method |
| Jurisdiction | binding/persuasive/irrelevant classification errors |
| Status | stale, overruled, reversed, vacated, superseded, unknown |
| Inference | missing, circular, contradictory, unsupported premises |
| Persistence | mutation, truncation, reordering, crash, concurrent writers |
| Security | path traversal, symlink, prompt injection, authorization, disclosure |

Reports must preserve provenance, held-out status, sample size, seed, toolchain, hardware, confidence intervals where applicable, and indeterminate cases. A zero-error sample is not a 100% guarantee.

The binaries under `src/bin/bench_*.rs` are development fixtures, not certifications.
