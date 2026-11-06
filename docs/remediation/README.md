# Remediation status

Baseline: `3aa977476df4a21576494b4b86f5dfaccacfa2f9`  
Branch: `remediation/legal-gmi-hardening`

Implemented in the current worktree:

- FIX-001: pinned and green Rust/Lean local gates;
- FIX-002–009: fail-closed proposal, source, evidence, outcome, jurisdiction, status, inference, adverse-search, and numeric boundaries;
- FIX-010–012: canonical receipts/content hashes, transactional restartable persistence, and corrected Lean content-state theorems;
- FIX-013: externally keyed Ed25519 bundle signatures and path-bound complete bundle hashing;
- FIX-017: public claim-language and schema corrections.

Still requiring external/deployment work:

- authoritative provider integrations and their licensing/coverage evidence;
- privacy/authorization/retention implementation for a selected deployment;
- independent held-out benchmarks and security assessment;
- civic schema/UI and a fictional, human-controlled pilot.

A fix is not promoted merely because code exists. Release gates in `RELEASE_GATES.md` remain authoritative.
