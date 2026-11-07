# CohBit Legal GMI — Open-source and commercial licensing policy

**Proposed AGPLv3 adoption package • 2026-11-07 • Not adopted on master until merged**

## Proposed open-source license

- **AGPL-3.0-only** applies to first-party source code and documentation for which the actual copyright holder has the authority to grant this license, after adoption by the rights holder.
- It permits commercial use, modification, redistribution, and providing services. There is **no mandatory commercial licensing fee**, revenue threshold, profit-share requirement, or field-of-use restriction under the AGPL.
- Distributors must comply with the AGPL's license notices and corresponding-source conditions. Section 13 adds specific source-offer obligations for **modified versions** supporting remote network interaction. Do not claim that any company's mere profit, unmodified internal use, or all network access automatically triggers payment or source-disclosure obligations.
- The AGPL grant, once made, **cannot be withdrawn from existing compliant recipients merely by changing the project license later**. Future versions may be offered under different terms by an authorized rights holder subject to the rights of other contributors.

## Optional separate paid offerings

**Separate commercial copyright license:** If the actual copyright holder holds all relevant copyrights or has the necessary sublicensing authority, customers may purchase a distinct license permitting specified uses outside AGPL copyleft requirements. Pricing must be established by separate agreement.

**Paid hosting and support:** Subscribers may purchase installation, support, customization, managed hosting, high-availability deployments, integrations, security services, updates, or contractual commitments. The AGPL itself does not require them to buy those services.

**Contribution policy:** Before accepting contributions, establish a signed contributor agreement or another clearly defined inbound licensing policy if dual commercial licensing is intended. An AGPL contribution does not automatically grant the project owner the ability to relicense that contributor's code under proprietary terms.

## Scope and legacy materials

- Existing source notices identify “CohBit” and archived release assets. The proprietor reports operating Noetican Labs as a sole proprietor, but **exact individual ownership and rights chain remain unverified**.
- The old `Cohbit-Legal-GMI-1.0.0-hardened.zip` binary archive remains excluded pending examination of its contents, third-party material, and license notices. Do not treat the repository's general license as relicensing that archive.
- External packages, the Lean toolchain, and other dependencies keep their own licenses and notices.
- A source-file SPDX header is a licensing marker; it is not evidence of copyright ownership or an assignment.

## Product qualification

Open source does not mean production-ready, legally authoritative, or guaranteed to calculate correct legal conclusions. Technical assurance and independent legal-domain validation remain distinct obligations. The existing LEGAL-PROOF-001 documentation continues to control claims about demonstrated soundness.

## Preconditions for applying on master

1. Identify and document the exact copyright owner(s); reconcile historic “CohBit” notices.
2. Confirm that no former employer, contributor, or third party holds rights that prevent release under AGPL.
3. Inventory dependencies and bundled/archive materials and preserve license obligations.
4. Have qualified counsel review AGPL adoption and future dual-licensing rights.
5. Approve the actual irreversible open-source grant and merge the proposal.
6. Update any website, release, package, and Zenodo metadata only after adoption; audit for contradictory proprietary claims.

Official information:
- https://www.gnu.org/licenses/agpl-3.0.en.html
- https://www.gnu.org/licenses/gpl-faq.html
- https://opensource.org/osd
