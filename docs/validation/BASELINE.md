# Local validation baseline

Assessment date: 2026-11-06  
Baseline commit: `3aa977476df4a21576494b4b86f5dfaccacfa2f9`

## Original baseline

- `cargo check --all-targets`: passed after dependencies were available.
- `cargo test --all-targets`: 11 passed.
- `cargo fmt --all -- --check`: failed.
- Clippy component: initially unavailable.
- `lake build`: failed at the whole-state mutation theorems in `LeanKernel/Algebra.lean`.

## Remediated worktree

Pinned Rust: 1.99.1  
Pinned Lean: 4.29.1  
Dependency graph: committed `Cargo.lock`

Required commands:

```bash
cargo fmt --all -- --check
cargo check --all-targets --locked --offline
cargo clippy --all-targets --locked --offline -- -D warnings
cargo test --all-targets --locked --offline
cd lean && lake build
```

The latest recorded local run passed all gates. Rust currently has 25 unit/regression tests. This is local engineering evidence, not external legal or production qualification.
