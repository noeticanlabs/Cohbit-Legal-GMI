# LEGAL-PROOF-001 equal-input comparison

The comparison fixture executes both paths with the same initial manifold,
witness, spend, and defect. `compare_equal_input_workflows` refuses to report a
comparison unless those inputs hash identically.

| Path | Actual fixture outcome | What it proves | What it does not prove |
|---|---|---|---|
| Legacy computational `apply` without an adapter | `Escalate`; no claim admitted | A canonical state-bound receipt records the attempt; rejected/escalated state is content-idempotent | No legal entailment, temporal calculation, or legal certification |
| Complete `apply_certified` workflow | `Accept`; claim admitted as `Qualified` | Exact claim/state/forum/time binding; signed qualification context; Lean Horn entailment; Rust civil-date/fixed-offset/effective-period replay; Lean normalized temporal arithmetic | No open-world completeness, no real-law conclusion from fictional fixtures, and no removal of the recorded qualification |

The Rust regression test also records both receipt hashes, the shared input
hash, decisions, positive guarantees, and explicit non-guarantees in the
serializable `WorkflowComparison` value.
