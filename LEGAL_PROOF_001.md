# LEGAL-PROOF-001: fictional public-comment deadlines

The current certificate policy is `LEGAL-PROOF-001/horn-temporal-v2`. Deadline
certificates must contain replayable temporal calculation steps; an opaque
deadline conclusion atom is insufficient. See `docs/TEMPORAL_MODEL.md`.

Status: engineering implementation complete; independent legal-domain qualification pending.

This experiment supports one conditional conclusion type: a deadline stated by a fictional public-comment notice, after applying an explicitly reviewed calendar/timezone rule and explicit amendment or extension treatment. It does not support real legal advice, environmental-law merits, litigation predictions, filing automation, or a claim about any real court or agency.

## Bounded domain

| Dimension | Included | Excluded |
|---|---|---|
| Jurisdiction | `Fictional State`, `Fictional Public Comment Agency` | Every real jurisdiction and institution |
| Source | Exact UTF-8 bytes of notice v1 plus its declared amendment history | Web search, model recollection, user-described authority |
| Time | The certificate's RFC 3339 verification time through `valid_until` | Later status without reevaluation |
| Stage | Open public-comment period | Enforcement, adjudication, appeal, merits |
| Claim | One exact RFC 3339 deadline proposition | General legal correctness or outcome prediction |
| Rules | Reviewed calendar-day, timezone, deadline, extension and priority rules | Unenumerated exceptions or open-world completeness |

The coverage record in each certificate identifies the finite corpus, search time and method, disclosed result sources, and limitations. It never means that no exception exists outside that corpus.

## Assurance separation

The artifact contains exact sources, source hashes, acquisition/status records, factual classifications, text passages, interpretation and applicability statements, exception resolution, dependencies, formal rules, proof steps, checker hash, and three signed review roles. The operator supplies the authorized keys, approved context hashes, current dependency hashes, revocations, domain, and checker binary independently of the artifact.

A corpus signature establishes integrity only. A review signature establishes that an operator-authorized, unexpired key with an external credential record signed the exact context only. Professional qualification and legal correctness require separate evidence. Until the context binds the exact external independent-review hash configured by the operator, the runtime returns `Qualified`, never `ProvedWithinScope`.

## Formal semantics

`LeanKernel.LegalProof` defines finite positive propositions and Horn rules. Every rule lists its ordinary body and `resolvedDefeaters`: positive, qualified propositions recording the treatment of exceptions or higher-priority rules. The executable accepts only an ordered derivation in which every premise is admitted and every rule prerequisite was already derived. Lean proves:

```text
LegalProof.check certificate = true
  -> Entails certificate.premises certificate.rules certificate.conclusion
```

There are no `sorry` declarations or added axioms in the legal-proof module. The theorem is conditional on the supplied formal context; it does not prove that the context accurately states law or facts.

## Certified boundary and trusted computing base

`apply_certified` is the only certified admission function. It binds the certificate to the exact claim, manifold content state, forum/posture, verification time, dependency versions, and executable hash. The legacy `apply_verified` always fails. The generic `apply` function is explicitly computational/simulation admission and its `Accept` value is not a legal-proof status.

The trusted computing base is:

- Lean 4.29.1 kernel, compiler/runtime, JSON decoder, and the checked `LegalProof` definitions/theorem;
- the compiled `legal_proof_check` executable whose BLAKE3 hash is operator configuration;
- the Rust certificate/context validation and process interface;
- Ed25519 and BLAKE3 implementations;
- operator-controlled checker filesystem, reviewer-key registry, revocation set, approved-context set, clock, and dependency registry;
- the qualified people and procedures that establish source, interpretation, applicability, facts, and coverage.

Deployment must prevent checker replacement between hashing and execution. Tests demonstrate the interface behavior; they do not prove universal Rust/Lean equivalence.

## Lifecycle and output

Canonical receipts commit the complete proof artifact and original status. Persistence retains exact dependency versions. `refresh_proof_lifecycle` recomputes and atomically records current status from operator trust state without rewriting history; expiry, revocation, source/rule replacement, or dependency change produces `ReevaluationRequired`. Certified bundle checking obtains trust roots externally, replays every proof, and compares the lifecycle index. Civic output is reconstructed from canonical, freshly checked proof receipts and keeps `Qualified` claims separate from unconditionally supported claims.

Statuses retain their defined meanings: `ProvedWithinScope`, `Qualified`, `Disputed`, `Unknown`, `Rejected`, and `ReevaluationRequired`. Assumptions, non-verified facts, incomplete domain qualification, unresolved conflicts, and missing context cannot become an unconditional proved result.

## Validation evidence

Local evidence is produced by:

```text
cd lean && lake build
cargo test --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
cargo fmt --all -- --check
rg 'sorry|axiom' lean/LeanKernel/LegalProof.lean
```

The adversarial tests cover source alteration, missing or circular premises, rule substitution, absent defeater resolution, checker hash binding, wrong forum, expired context, revoked review, changed dependencies, repeated reviewer key, and certified-boundary bypass. Existing bundle tests cover missing/substituted files and terminal hash verification.

## Pending independent gate

G7 and the domain-qualified portion of G1/G2 remain pending until reviewers independent of the proposal generator provide identities/credentials, conflict disclosures, signed scope and formalization decisions, a held-out dataset, and measured false-accept, false-reject, escalation, review-burden, sample-size, and coverage results. Use [the review record template](docs/LEGAL_PROOF_001_INDEPENDENT_REVIEW.md). No test result in this repository substitutes for that evidence.

## Requirement traceability

| Obligation | Implementation/evidence | Current state |
|---|---|---|
| LP-001 bounded domain | Structured domain, jurisdiction, stage, claim type, approved-context hash, and this scope table | Enforced; independent scope approval pending |
| LP-002 authentic sources | Exact source bytes, URI, retrieval, acquisition, revision, status, amendment history, dependency hash, source-role signature | Enforced structurally; real authority qualification pending |
| LP-003 faithful formalization | Exact passage bytes, interpretation, applicability, exception treatment, revision, formalization-role signature | Enforced structurally; independent legal judgment pending |
| LP-004 factual premises | `FactStatus`, evidence passage, qualification record, assumptions; only all-verified/no-assumption context can be proved | Enforced |
| LP-005 applicability | Exact posture hash, effective interval, source revision, authority effect, policy scope | Enforced structurally |
| LP-006 exceptions/priorities | Explicit resolved-defeater premises; defeated-rule indices and strict priority checked by Rust and Lean | Enforced within finite declared coverage |
| LP-007 sound inference | Lean `checker_sound` theorem and executable checker; no `sorry`/axiom | Pass locally |
| LP-008 proof certificates | Strict artifact schema, exact context/rules/premises/steps/checker/dependencies/reviews | Pass locally with mutation tests |
| LP-009 Rust/Lean boundary | Mandatory external checker process and executable hash in `apply_certified`; legacy verified APIs fail | Pass locally; universal refinement remains a stated TCB obligation |
| LP-010 fail-closed statuses | Six explicit proof/lifecycle statuses, claim-status map, receipt binding, civic reconstruction | Enforced |
| LP-011 lifecycle | Immutable artifact dependencies plus atomic recomputed lifecycle index and bundle comparison | Pass locally |
| LP-012 independent validation | External review template and promotion guard requiring the exact review-record dependency hash | Pending external reviewers and held-out results |

