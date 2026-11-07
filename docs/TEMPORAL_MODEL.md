# LEGAL-PROOF-001 temporal model

The `horn-temporal-v2` certificate uses these exact meanings:

- A `CivilDate` is a date in the proleptic Gregorian calendar.
- A `WallTime` is an hour, minute, and second with no implicit zone.
- A `LocalDateTime` combines a civil date and wall time with an explicit,
  fixed offset from UTC in seconds. Named timezone rules and daylight-saving
  transitions are never guessed.
- An instant is a signed count of seconds from `1970-01-01T00:00:00Z`.
- A calendar-day period advances the civil date. If the start day is excluded,
  `N` days means `publication_day + N`; if included, it means
  `publication_day + N - 1`, and zero is invalid.
- Offset conversion is `UTC instant = local second - UTC offset seconds`.
- An effective period is half-open: `[start_instant, end_exclusive_instant)`.
- An extension is active only when the certificate's verification instant is
  inside its effective period. The active highest-priority notice controls.
  Equal-priority active notices with different replacement instants conflict
  and fail closed. Each notice must name evidence in the signed proof context.

The Rust verifier validates civil dates and wall times, replays every listed
step, checks evidence bindings, resolves extension priority, binds the
verification instant to the signed proof context, and renders the exact claim
text from the result. The Lean checker independently verifies normalized
calendar-day arithmetic, local-to-UTC offset arithmetic, extension selection,
and the conclusion instant. `temporal_checker_sound` proves that an accepted
temporal certificate's conclusion equals its temporal semantics;
`checker_sound` separately proves Horn entailment.

The civil-date-to-day-index conversion and the operator-owned checker binary
remain explicit trusted-computing-base boundaries. This model does not claim a
named timezone database or holiday/business-day calendar.

## Generative demonstration

`TemporalCertificate::derive_from_notice_evidence` parses the bounded notice
grammar from the exact `SourceEvidence.text` bytes. Its arguments contain the
source ID, verification instant, and authenticated extension inputs, but no
deadline, target date, intermediate calculation, conclusion instant, or claim
text. It generates all of those outputs. The complete certified fixture builds
its `LegalClaim` from that generated output and only then signs the proof
context and invokes the Lean checker.

The regression test `derives_substantive_deadline_from_notice_without_conclusion_input`
records the derivation:

```text
publication input       2030-01-01
calendar-day input      30 (start day excluded)
closing-time input      17:00:00 UTC-06:00
derived target day      2030-01-31
derived local second    1896109200
derived UTC instant     1896130800
generated claim         2030-01-31T17:00:00-06:00
```

The same test changes only the authenticated publication input to `2030-01-02`
and obtains `2030-02-01T17:00:00-06:00`, demonstrating that the conclusion is
computed rather than selected from a supplied expected answer.
