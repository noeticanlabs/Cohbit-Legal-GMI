# CohBit Legal GMI — AGPL release audit

**Review date:** 2026-11-07  
**Target:** `master`, from `licensing/agpl3-open-source-proposal`  
**Requested by:** repository operator, explicit merge authorization  
**Scope:** code licensing metadata, release artifacts, documentation, limited security checks and pre-merge CI records

## Findings

| Control | Finding | Assessment |
|---|---|---|
| Open-source license text | Verbatim GNU AGPLv3 license candidate staged at root `LICENSE` (SPDX identifier `AGPL-3.0-only`) | Prepared |
| Rust package metadata | `Cargo.toml` updated to AGPL-3.0-only, `publish = false` | Prepared |
| Notices | Rust entry header updated with SPDX marker; historical copyright identification “CohBit” retained | Prepared; owner identification pending |
| Dual commercial offering | Optional separate paid license documented; AGPL compliant commercial use is still free | Described accurately |
| Research qualification | README retains not-legal-advice and not-production-qualified limitations | Preserved |
| Latest pre-merge CI | Rust and Lean workflows on `master` SHA `20207b4` completed successfully | Passed at baseline only; post-merge CI pending |
| Third-party dependencies | 97 locked third-party Rust package/version entries inventoried; Lean 4.29.1 pinned | Licenses and notices NOT independently verified |
| Secret text search | GitHub code search for `BEGIN PRIVATE KEY` and `ghp_` returned no hits. Fixed development SigningKeys occur in tests. | Narrow negative only, NOT a comprehensive secrets/history scan |
| Legacy ZIP | `Cohbit-Legal-GMI-1.0.0-hardened.zip` was unreadable through available connector; removed from release tree, remains in Git history | Historical archive still unverified |
| Legal ownership | Historical notices identify “CohBit”, earlier commits list author “True A”, and company operates as a sole proprietorship | Exact individual legal owner/chain of title NOT independently established |
| Commercial license contracts | Separate commercial licensing is a business option, not an executed dual-license offer to any customer | Draft/negotiation only |

## Unresolved legal/compliance blockers

1. Confirm original copyright ownership and right to relicense *all* original material; verify historical contributor/contractor and employment agreements.
2. Reconcile “CohBit” label with sole-proprietor legal identity and verify patent/trademark implications.
3. Inspect historical ZIP contents and scan Git history for secrets, third-party content, confidential work, and license notices. Deleting it from the current tree does not erase public history.
4. Complete version-specific Rust/Lean dependency-license checks and mandatory notice/attribution requirements.
5. Qualified software/IP counsel should review rights, AGPL grants, commercialization compatibility and contributor intake before commercial launch.

## Audit conclusion

The mechanical licensing changes and selected dual-license model are **technically ready to merge** based on the reviewed diff. **This is not a clean legal-rights or third-party redistribution audit.** User authorization to merge does not, by itself, furnish third-party permissions or establish copyright title. The AGPL grant, once issued by a rights holder, is enduring for downstream compliant recipients. No legal-accuracy or production-readiness claim is made.

## Post-merge verification steps

1. Verify `master/LICENSE` contains AGPL and that GitHub detects AGPLv3.
2. Verify `master/Cargo.toml`, `src/lib.rs`, README, decision record and Zenodo metadata agree.
3. Confirm old ZIP is absent from current tree, while documenting history retention.
4. Check Rust and Lean CI on the actual merge SHA.
5. Run a full compliance and secrets/history audit before external binary distribution or paid customer onboarding.
