# Proposed public release: AGPL-3.0-only

**Release audit snapshot (2026-11-07).** The project operator explicitly authorized merging AGPL-3.0-only into `master`. The AGPL license grant applies when the branch is actually merged. Several independent copyright/third-party due-diligence obligations remain **unverified**; do not describe the audit as a complete legal clearance.

## Copyright and rights — blocking
- [ ] Verify legal owner(s) of original code and identify the legal-name spelling. Noetican Labs is presently described as a sole proprietorship, not a separate company.
- [ ] Reconcile `Copyright (c) 2026 CohBit` with the actual individual rights owner; preserve notices unless changes are authorized.
- [ ] Review historical commits and employed/contracted contributors for assignments and permissions.
- [ ] Confirm the person approving the release has authority to grant AGPL to every covered first-party file.

## Third-party notices — blocking
- [x] Produce an exact Cargo.lock package inventory and record pinned Lean toolchain; see `docs/licensing/DEPENDENCY_INVENTORY.md`.
- [ ] Verify each resolved dependency's actual license text, all notices, linked artifacts and distribution obligations.
- [x] Remove the uninspectable legacy archive from the current release tree.
- [ ] Inspect historical `Cohbit-Legal-GMI-1.0.0-hardened.zip` for conflicting rights, source, secrets, and notices; it remains in public Git history.
- [ ] Review evidence bundle fixtures and copied materials for redistribution rights.
- [x] Confirm LICENSE begins with the official GNU AGPL version 3 text and uses the matching SPDX license reference; full upstream byte comparison still advisable.

## Commercial expectations — important
- [x] Choose **genuine OSI-compatible open source**, not BSL/source-available.
- [x] Stage **AGPL-3.0-only** as the strong-copyleft candidate, not Apache/MIT.
- [x] Project operator selected **AGPLv3 plus optional paid commercial licensing** and was informed that AGPL permits compliant commercial use without payment. Final rights-holder signoff to distribute remains a separate, uncompleted requirement.
- [ ] Establish optional separately negotiated commercial licensing terms if dual licensing is desired.
- [ ] Adopt inbound contributor permission/CLA policy before accepting contributions intended for future dual licensing.
- [ ] Verify patent grants and any trademark implications with counsel.

## Technical release
- [x] Prior master Rust/Lean CI was green and basic GitHub text searches for private key patterns returned no direct matches (not a full secret scan).
- [ ] Complete full secret/history scan, post-merge CI, packaging, and third-party compliance verification.
- [ ] Verify AGPL notices and corresponding-source obligations for any network services actually offered.
- [x] Project operator expressly authorized the merge to `master`.
- [ ] Obtain external counsel signoff and complete chain-of-title/third-party compliance review; this audit cannot independently establish either.
- [ ] Confirm public GitHub displays AGPL license correctly and metadata is synchronized.

**Do not claim that AGPL compels a commercial royalty or creates a partnership.**
