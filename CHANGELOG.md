# Changelog

All notable changes to this project are documented here.

## 0.1.2 - 2026-07-30

- Normalize the crate into one idiomatic module-tree form with no parallel
  `foo.rs` and `foo/` ownership.
- Add a strict, zero-baseline architecture contract with 300-line file,
  100-line function, and zero-runtime-cycle release budgets.
- Track the released `weavatrix-scan` 0.4.6 repository adapter.

## 0.1.1 - 2026-07-29

- Add deterministic Type-1 exact block detection.
- Add identifier-normalized Type-2 function detection.
- Add bounded Type-3 near-miss verification with reusable workspaces.
- Add explicit exact, renamed, and near-miss modes.
- Add stable evidence, pair IDs, clone families, and safety limits.
- Add optional repository integration through `weavatrix-scan`.
- Add a dependency-free fragment core, CLI, tests, and benchmarks.
- Add stable versioned JSON and SARIF 2.1.0 report encoders.
- Add direct BigCloneEval output and cross-platform benchmark runners.
- Add coverage-based accuracy gates with per-clone-type metrics.
- Add Rust and Java competitor oracles plus a reproducible CLI benchmark runner.
- Remove rolling-hash allocations and inline the common occurrence bucket.
- Correct numeric exponent tokenization and accelerate TypeScript lexing.
- License the crate under MIT.
