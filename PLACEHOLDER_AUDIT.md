# CohBit Legal GMI — Mock and Placeholder Audit

| Component | Classification | Restriction |
|---|---|---|
| `MockLegalStatusAdapter` | `TestOnly` | Cannot cross certified proof admission |
| `LocalTextSourceAdapter` | `LocalUnverified` | Source presence is not authoritative legal status |
| `LocalPdfSourceAdapter` | `LocalUnverified` | Defaults jurisdiction and status to `Unknown` |
| `MockGitAnchorAdapter` | Simulation | Does not create an external Git commitment |
| benchmark fixtures | Development evidence | Must not be described as certification |
| authenticated proof reviews | Exact-context attestations plus operator-controlled credentials and revocation | Authentication is not independent legal validation |

Remaining work includes authoritative licensed sources, independent held-out evaluation, deployment privacy/access/retention testing, signing-key lifecycle procedures, and human-review governance.

Verdict: research prototype with strengthened admission, persistence, formal, and bundle-integrity boundaries. External legal or advocacy deployment remains blocked.
