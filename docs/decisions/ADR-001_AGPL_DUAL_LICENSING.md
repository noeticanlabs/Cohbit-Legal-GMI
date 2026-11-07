# ADR-001 — AGPLv3 + optional commercial dual licensing

**Decision recorded:** 2026-11-07  
**Status:** Business model SELECTED; operator authorized merge on 2026-11-07; chain-of-title and third-party compliance audit remains OPEN

## Decision
CohBit Legal GMI will pursue:
1. **GNU Affero General Public License v3.0 only (AGPL-3.0-only)** as the public open-source license for first-party material that the authorizing licensor has the rights to license.
2. **Optional, separately negotiated commercial licenses** for customers seeking alternate proprietary license permissions, supplemented by paid installation, integration, hosting, support, audit, training, and other services.

## Important boundaries
- Compliant AGPL users, including commercial companies, are **not required to pay** Noetican Labs. This is genuine open source, not a commercial-use-restricted license.
- Selling a different commercial copyright license requires documented rights over covered first-party material and inbound contributions.
- A public repository with no license is not an AGPL release. This decision document and staging branch do not make `master` or prior releases newly AGPL-licensed.
- The proprietary commercial-licensing branch is a distinct **historical alternative proposal**, not a second license automatically applied to AGPL recipients.
- The old archived ZIP requires an explicit rights audit and is not granted an AGPL license by this decision.
- Open source does not establish fitness for production legal advice, formal verification beyond documented scope, or legal correctness.

## Blocking steps before release
- [ ] Verify the sole proprietor's exact legal identity and reconcile “CohBit” copyright ownership claims.
- [ ] Review original authorship, employment-related rights, contributions, and copied materials.
- [ ] Inspect dependencies and the legacy ZIP's licensing and redistribution permissions.
- [ ] Obtain appropriate IP/software-license review.
- [ ] Obtain final authorization for the enduring AGPL release and merge the tested license package onto `master`.

## Commercial follow-through
- [ ] Draft and approve a separate commercial license and order schedule for eligible customers.
- [ ] Decide whether inbound contributions require a CLA, assignment, or alternative scoped permission.
- [ ] Define pricing and technical services after independent legal-domain qualification.

**The operator authorized the AGPL release. Customer commercial-license contracting and external legal due diligence remain outstanding; no customer agreement is implied.**
