# Third-party rights and dependency-license review

**Status: Inventory for legal review; not a completed software bill of materials or redistribution clearance.**

The repository includes Rust and Lean code and depends on external packages. The proprietary rights notice covers only first-party material owned or lawfully licensed for that purpose. It does not relicense third-party work.

## Direct Rust dependencies listed in Cargo.toml
- serde, serde_json
- sha2, hex, thiserror
- num-rational, num-traits, num-bigint
- clap, anyhow, chrono
- blake3, ed25519-dalek

Cargo.lock also includes transitive dependencies. License obligations must be inventoried **at resolved versions**; this file does not assume every dependency has the same license.

## Lean ecosystem
- Lean 4, Lake, and the Lean runtime/compiler have their own upstream licenses.
- External Lean modules and proof infrastructure (if any) need review before binary distribution.

## Remaining compliance work
- Generate an SBOM from the pinned Cargo.lock and Lean toolchain, ideally with version, origin, license ID, license-file text and hashes.
- Examine multi-licensed, attribution, patent-notice, source-offer, and redistributable-binary obligations.
- Reconcile third-party files already bundled or copied into archives and fixtures.
- Preserve required attribution texts in distributed binaries/source and in commercial packages.
- Check license compatibility for any separately released proof checker before considering a distinct open-source license.
- Obtain written rights assignments/authorizations for first-party contributions and account for any materials created under employment or other contracts.
- Repeat the license scan upon dependency upgrades.

**No automated dependency-license scan was performed as part of this licensing documentation change.**
