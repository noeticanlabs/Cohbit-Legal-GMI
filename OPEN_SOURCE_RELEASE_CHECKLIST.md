# Proposed public release: AGPL-3.0-only

**No final release decision recorded by this checklist.** The existing public `master` has no open-source license file and declares a proprietary rights position. Do not treat repository visibility as a software license.

## Copyright and rights — blocking
- [ ] Verify legal owner(s) of original code and identify the legal-name spelling. Noetican Labs is presently described as a sole proprietorship, not a separate company.
- [ ] Reconcile `Copyright (c) 2026 CohBit` with the actual individual rights owner; preserve notices unless changes are authorized.
- [ ] Review historical commits and employed/contracted contributors for assignments and permissions.
- [ ] Confirm the person approving the release has authority to grant AGPL to every covered first-party file.

## Third-party notices — blocking
- [ ] Produce an inventory of exact pinned Rust dependencies in Cargo.lock, Lean toolchain/dependencies, license texts, and obligations.
- [ ] Inspect `Cohbit-Legal-GMI-1.0.0-hardened.zip` for conflicting source, licenses, keys, privileged information, and attribution; exclude/remove it from release if necessary.
- [ ] Review evidence bundle fixtures and copied materials for redistribution rights.
- [ ] Confirm LICENSE is verbatim AGPLv3 text (from SPDX/FSF).

## Commercial expectations — important
- [x] Choose **genuine OSI-compatible open source**, not BSL/source-available.
- [x] Stage **AGPL-3.0-only** as the strong-copyleft candidate, not Apache/MIT.
- [ ] Rights holder acknowledges that AGPL allows legally compliant **commercial use without payment** to Noetican Labs.
- [ ] Establish optional separately negotiated commercial licensing terms if dual licensing is desired.
- [ ] Adopt inbound contributor permission/CLA policy before accepting contributions intended for future dual licensing.
- [ ] Verify patent grants and any trademark implications with counsel.

## Technical release
- [ ] Complete source and secret scan, CI verification, packaging check, and independent proof boundaries.
- [ ] Verify AGPL notices and corresponding-source obligations for any network services actually offered.
- [ ] Merge to master only after rights-holder approval and counsel review.
- [ ] Confirm public GitHub displays AGPL license correctly and metadata is synchronized.

**Do not claim that AGPL compels a commercial royalty or creates a partnership.**
