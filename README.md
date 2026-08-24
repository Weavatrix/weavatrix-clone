# Weavatrix Clone

[![CI](https://github.com/Weavatrix/weavatrix-clone/actions/workflows/ci.yml/badge.svg)](https://github.com/Weavatrix/weavatrix-clone/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/weavatrix-clone.svg)](https://crates.io/crates/weavatrix-clone)
[![docs.rs](https://docs.rs/weavatrix-clone/badge.svg)](https://docs.rs/weavatrix-clone)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Part of the [Weavatrix ecosystem](https://weavatrix.com/ecosystem): deterministic clone evidence for AI software agents.

Find duplicate code with evidence you can inspect, reproduce, and gate in CI.

`weavatrix-clone` is the deterministic Type-1/2/3 clone engine used by
Weavatrix. It turns token-level matches into stable pairs and families with
source spans, similarity proof, safety limits, JSON/SARIF output, and an
explicit review boundary for semantic Type-4 candidates.

`weavatrix-clone` separates clone detection from search and semantic retrieval:

- exact token blocks detect Type-1 clones at arbitrary locations;
- normalized function fragments detect identifier-renamed Type-2 clones;
- bounded near-miss verification detects Type-3 clones;
- a provider trait leaves Type-4 semantic candidate generation to a separate
  vector/search package.

The library is MIT licensed. Its core has no dependencies. Repository scanning
is an optional default feature backed by `weavatrix-scan`.

## Why a separate crate?

Clone detection is not literal search. Search answers where a query occurs;
clone detection finds related regions, verifies their relationship, groups
occurrences, and returns evidence suitable for automated decisions.

The crate therefore does not depend on ripgrep, a regex engine, a vector
database, Git, or an external parser process.

## Detection modes

| Mode | Contract | Typical use |
|---|---|---|
| `Exact` | Type-1 exact token blocks | fast CI duplication gate |
| `Renamed` | Type-1 plus identifier-renamed Type-2 functions | structural refactoring |
| `NearMiss` | Type-1/2 plus bounded Type-3 edits | architecture and agent analysis |

`NearMiss` is the library default. The explicit modes prevent an unfair speed
comparison between an exact-only scan and a wider near-miss analysis.

## Supported source profiles

The built-in tokenizer recognizes Rust, Go, C, C++, Bash, SQL, JavaScript,
TypeScript, Python, Java, C#, and common markup. Repository defaults cover
their common extensions, including JSX/TSX, module extensions, headers, and
Python stubs.

These are lexical profiles, not claims of full grammar parsing. Unknown syntax
still advances safely as UTF-8 tokens, and invalid UTF-8 selected source is
reported rather than silently accepted.

## Library use

```rust
use weavatrix_clone::{
    CloneConfig, CloneDetector, DetectionMode, Language, SourceFragment,
    SourceSpan,
};

let left = SourceFragment::new(
    "left",
    "src/a.rs",
    Language::Rust,
    SourceSpan::whole("fn value() -> u32 { 42 }"),
    "fn value() -> u32 { 42 }",
)?;
let right = SourceFragment::new(
    "right",
    "src/b.rs",
    Language::Rust,
    SourceSpan::whole("fn result() -> u32 { 42 }"),
    "fn result() -> u32 { 42 }",
)?;
let detector = CloneDetector::new(CloneConfig {
    mode: DetectionMode::Renamed,
    min_tokens: 12,
    ..CloneConfig::default()
})?;
let report = detector.detect(&[left, right])?;
# Ok::<(), weavatrix_clone::CloneError>(())
```

Repository scanning is available with the default `scan` feature:

```rust
use weavatrix_clone::{CloneDetector, RepositoryCloneDetector};

let report = RepositoryCloneDetector::new(CloneDetector::default())
    .detect(".")?;
println!("{} clone families", report.families.len());
# Ok::<(), weavatrix_clone::CloneError>(())
```

Exact blocks are isolated by extension and processed concurrently by default.
This matches the usual language-format contract and avoids accidental
cross-language hits. Set `RepositoryOptions::cross_extensions` or pass
`--cross-extensions` when exact clones between formats are intentional.

Disable all optional dependencies for the fragment-only core:

```toml
[dependencies]
weavatrix-clone = { version = "0.1.3", default-features = false }
```

## CLI

```text
weavatrix-clone --summary --mode exact --min-tokens 24 \
  --min-lines 3 --format rs,go,ts,tsx .
```

The final summary includes source files, fragments, candidate comparisons,
verified pairs, families, Type-1/2/3 counts, and elapsed time. Invalid options,
incomplete scans, and capacity limits produce a non-zero exit code.

Machine-readable output is written to stdout and never contains timing data:

```text
weavatrix-clone --output-format json .
weavatrix-clone --output-format sarif .
weavatrix-clone --output-format bigcloneeval /path/to/bcb/subset
```

`json` follows the checked-in versioned
[`clone-report-v1` schema](schemas/clone-report-v1.schema.json). `sarif`
produces SARIF 2.1.0 with one primary and one related location, stable partial
fingerprints, evidence properties, and Type-1/2/3 rules. `bigcloneeval`
produces the official eight-column clone-pair CSV with no summary text.

## Evidence and determinism

Every verified pair carries:

- stable source paths and byte/line spans;
- a stable pair identifier;
- clone kind and integer-backed similarity;
- strict and renamed equality flags;
- fingerprint overlap;
- bounded edit distance and compared token count.

Inputs are canonicalized before comparison and output is sorted. Hash
collisions only propose candidates: exact repository windows are compared
against their original token text, while structural fragments use global
interned token IDs. Reordering input fragments does not reorder or renumber
results.

Capacity limits cover files, tokens, candidate pairs, and repetitive hash
buckets. Same-file overlapping fragments are suppressed by default.

## Competitor position

| Capability | Weavatrix Clone | jscpd | PMD CPD | SourcererCC | NiCad |
|---|---:|---:|---:|---:|---:|
| embeddable Rust core | yes | no | no | no | no |
| exact arbitrary blocks | yes | yes | yes | yes | yes |
| identifier-renamed clones | yes | not by default | language-dependent option | yes | yes |
| bounded near-miss clones | yes | exact subblocks | limited normalization | yes | yes |
| stable evidence IDs | yes | no | no | no | no |
| no runtime/toolchain process | yes | yes for native builds | no, JVM | no, Python/JVM pipeline | no, OpenTxl |
| format breadth | focused | 223 formats | broad | configurable tokenizer | focused plugins |

This table describes contracts, not a universal accuracy ranking. jscpd has
much broader format support and mature reporters; PMD has grammar-aware
language integrations; SourcererCC targets Internet-scale corpora; NiCad has
deep configurable pretty-printing and normalization.

Public references:

- [jscpd](https://github.com/kucherenko/jscpd)
- [PMD CPD](https://pmd.github.io/pmd/pmd_userdocs_cpd)
- [SourcererCC](https://github.com/Mondego/SourcererCC)
- [Open-NiCad](https://github.com/CordyJ/Open-NiCad)
- [BigCloneBench](https://github.com/clonebench/BigCloneBench)

## Architecture

The crate is a modular library, not a monolithic scanner:

| Layer | Responsibility |
|---|---|
| `model` | stable fragments, locations, evidence, pairs, families, configuration, and errors |
| `token` | lossless lexical profiles, canonicalization, hashing, and fingerprints |
| `detection` | candidate indexing, Type-1/2/3 verification, clustering, and accuracy gates |
| `repository adapter` | optional `weavatrix-scan` traversal and source-block extraction |
| `output` | versioned JSON, SARIF 2.1.0, and BigCloneEval encoders |
| `facade / CLI` | the public Rust API and bounded command-line adapter |

Dependencies point inward: output and repository adapters consume the core;
the core never depends on CLI or report presentation. The checked-in
`.weavatrix/architecture.json` enforces zero runtime cycles, files no larger
than 300 physical lines, functions no larger than 100 physical lines, and no
grandfathered exceptions.

## Correctness gates

The checked-in oracle covers:

- exact copies;
- systematic identifier renaming;
- inserted near-miss statements;
- numeric lookalikes that must remain negative;
- partial exact blocks;
- line thresholds, overlap policy, determinism, Unicode progress, invalid
  configuration, and duplicate fragment identifiers.

On the small Rust oracle, `NearMiss` detects all three intended Type-1/2/3
relations with no negative-pair hit. jscpd 5.0.12 detects the exact relation
and exact subblocks in the near-miss pair, but not the identifier-renamed
relation under its default exact contract. PMD CPD 7.26.0 has the same Rust
oracle result even with identifier normalization.

On the portable Java oracle, Weavatrix and PMD with identifier normalization
link 3/3 intended relations with no unrelated link. NiCad 7.0.1 `default`
links 3/3 but also emits two unrelated pairs; its stricter `type2` mode links
2/3 without an unrelated pair. These are relation-level smoke results, not
general accuracy scores.

That oracle is a regression suite, not a published recall score. A full
external benchmark remains required before making broad precision or recall
claims.

`AccuracyGate` evaluates explicitly labeled relations with the same
coverage-based location rule used by BigCloneEval. It reports overall and
per-Type-1/2/3 counts, precision, recall, and F1; CI fails rather than silently
accepting a missed threshold. Unlabeled detector output is deliberately
ignored because an incomplete oracle cannot establish false positives.

BigCloneBench data is not bundled: its benchmark is CC BY-NC 4.0 and the
IJaDataset files retain their original licenses. After obtaining the official
BigCloneEval release, generate an import file with:

```text
./benchmarks/run_bigcloneeval.sh BCB_REDUCED clones.csv
./benchmarks/run_bigcloneeval.ps1 -DatasetRoot BCB_REDUCED -OutputPath clones.csv
```

The runners process each functionality subset independently, use the official
recommended minimum of 50 tokens and 6 lines, and emit data accepted directly
by BigCloneEval's `importClones`.

## Benchmarks

Run the dependency-free core benchmark:

```text
cargo bench --bench core
cargo bench --bench typescript
```

Benchmark rules used for competitor comparisons:

- release binaries, not `npx` startup;
- identical language scope, minimum 24 tokens, and minimum 3 lines;
- two warmups followed by an odd number of samples;
- median wall-clock process time;
- successful exit code required for every measured run;
- `Exact` is compared with exact-only tools; `NearMiss` is reported
  separately.

Current local wall-clock medians in milliseconds:

| Corpus | Weavatrix | jscpd 5.0.12 | Native result | PMD 7.26.0 |
|---|---:|---:|---:|---:|
| Rust | 20.2 | 41.1 | 2.04x faster | 5,839.0 |
| Go | 47.2 | 80.3 | 1.70x faster | 1,517.2 |
| Python | 175.4 | 344.2 | 1.96x faster | 3,124.0 |
| JavaScript | 233.1 | 390.6 | 1.68x faster | 5,655.6* |
| TypeScript/TSX | 220.8 | 329.2 | 1.49x faster | 18,849.1 |
| Java | 104.5 | 136.3 | 1.30x faster | 1,776.6 |

`*` PMD skips two JavaScript shebang files and reports recoverable lexical
errors. These machine-specific results are not universal. The detailed file
counts, protocol, caveats, correctness tables, and reusable PowerShell runner
are in [`benchmarks/README.md`](benchmarks/README.md).

## Current limits

- No AST binding or type resolution.
- Type-2/3 fragmentation is function-oriented; exact blocks are arbitrary.
- No bundled semantic/vector Type-4 model.
- No incremental on-disk clone index yet.
- No HTML reporter yet.
- No published BigCloneBench recall result yet.

These are explicit boundaries rather than silent fallbacks.

## Development

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo test --no-default-features
cargo bench --bench core
cargo bench --bench typescript
```

## License

MIT. See [`LICENSE`](LICENSE).
