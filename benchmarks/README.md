# Competitor benchmarks

This directory separates three questions that must not be collapsed into one
score:

1. does a detector link the intended Type-1/2/3 relations;
2. does it avoid unrelated relations;
3. how long does an end-to-end CLI run take on the same input snapshot?

The checked-in corpora are redistributable regression oracles, not substitutes
for BigCloneBench or a statistically representative accuracy study.

## Correctness oracles

`corpus/rust` checks:

- `exact_a.rs` and `exact_b.rs`: Type-1;
- `renamed_a.rs` and `renamed_b.rs`: Type-2;
- `near_a.rs` and `near_b.rs`: Type-3;
- `negative_a.rs` and `negative_b.rs`: similar shape but different
  behavior-defining numeric literals.

`corpus/java` carries equivalent intended relations for detectors that do not
support Rust. Its two negative files deliberately use unrelated control flow.

The tables use 24 tokens and 3 lines where the tool exposes both controls. A
relation is linked when any reported pair connects its two expected files.
This is weaker than saying the whole functions satisfy a BigCloneEval coverage
threshold, so exact-subblock-only results are called out.

### Rust oracle

| Detector and mode | Linked relations | Negative links | Interpretation |
|---|---:|---:|---|
| Weavatrix `near` | 3/3 | 0 | emits an explicit whole-fragment Type-3 pair |
| jscpd 5.0.12 | 2/3 | 0 | Type-1 plus exact subblocks inside the Type-3 pair |
| PMD CPD 7.26.0 exact | 2/3 | 0 | Type-1 plus an exact Type-3 subblock |
| PMD `--ignore-identifiers` | 2/3 | 0 | Rust identifier normalization did not add Type-2 |
| NiCad 7.0.1 | N/A | N/A | the official distribution has no Rust plugin |

### Java oracle

An unrelated hit is a pair involving either negative file or crossing two
different intended families.

| Detector and mode | Linked relations | Unrelated hits | Interpretation |
|---|---:|---:|---|
| Weavatrix `near` | 3/3 | 0 | explicit Type-1, Type-2, and Type-3 evidence |
| jscpd 5.0.12 | 2/3 | 0 | misses renamed Type-2; links Type-3 through subblocks |
| PMD CPD exact | 2/3 | 0 | exact contract |
| PMD `--ignore-identifiers` | 3/3 | 0 | Type-2 added; Type-3 is still exact subblocks |
| NiCad `type2` | 2/3 | 0 | 0.00 threshold, blind renaming |
| NiCad `default` | 3/3 | 2 | 0.30 near-miss threshold also adds two false links |

This small oracle establishes a regression contract only. It does not justify
a universal precision, recall, or superiority claim.

## Throughput snapshot

Windows 11 x64, Intel Core Ultra 7 255U, Rust 1.97.1, 2026-07-26.
Compared versions:

- Weavatrix Clone 0.1.0 release binary;
- native jscpd 5.0.12 release binary;
- PMD CPD 7.26.0 on Temurin JRE 21.0.11.

Each corpus is a temporary snapshot of tracked source files for one language,
prefiltered to at most 1.5 MB per file. No private repository name is retained
in the results. All tools receive the same snapshot directory, although jscpd
does not count empty or otherwise rejected inputs as analyzed files.

Native rows use two warmups and 21 alternating process launches. PMD uses one
warmup and 11 process launches. Values are wall-clock medians in milliseconds;
they include walking, reading, tokenization, detection, reporting, and process
startup.

| Corpus | Input / jscpd files | Input MiB | Weavatrix exact | jscpd exact | Result | PMD exact |
|---|---:|---:|---:|---:|---:|---:|
| Rust | 78 / 77 | 0.36 | 20.2 | 41.1 | 2.04x faster | 5,839.0 |
| Go | 265 / 260 | 1.04 | 47.2 | 80.3 | 1.70x faster | 1,517.2 |
| Python | 1,008 / 1,000 | 7.45 | 175.4 | 344.2 | 1.96x faster | 3,124.0 |
| JavaScript | 897 / 878 | 7.97 | 233.1 | 390.6 | 1.68x faster | 5,655.6* |
| TypeScript/TSX | 1,589 / 1,443 | 7.00 | 220.8 | 329.2 | 1.49x faster | 18,849.1 |
| Java | 381 / 375 | 2.35 | 104.5 | 136.3 | 1.30x faster | 1,776.6 |

`*` PMD rejected two executable JavaScript files whose first line is a Unix
shebang. The measured PMD run used its documented recoverable-error mode and
skipped those two files. All other PMD rows exit cleanly without that mode.

Finding counts are intentionally absent from the speed table: Weavatrix emits
canonical evidence pairs and families, while competitors use different
overlap, suppression, and clone-class contracts.

### In-process microbenchmarks

The dependency-free fragment core and repository path report:

```text
clone_core fragments=10000 verified_pairs=5000 median_ms=58.799 \
  fragments_per_second=170071
typescript_repository files=1024 verified_pairs=2042 median_ms=31.237 \
  files_per_second=32781
```

These harnesses run with `cargo bench --bench core` and
`cargo bench --bench typescript`. They are not mixed with the process-level
competitor table.

## Why the timings differ

jscpd 5 is a strong native baseline. Its Rust engine uses pre-hashed detection
tokens, a preallocated `FxHashMap`, format-level Rayon parallelism, an
open-clone state machine, and a bounded secondary occurrence pass.

Weavatrix uses the same broad rolling-window strategy but verifies original
token text after a hash match rather than treating a 64-bit token hash as final
evidence. It also canonicalizes stable pair IDs and evidence families. The
current measurements show that this stricter contract still wins on all six
tested snapshots; they do not imply every machine or corpus will do so.

PMD contributes grammar-aware language integrations and useful normalization
switches. Its table column is end-to-end CLI latency, so JVM startup is part of
the measured contract and must not be presented as an in-process algorithm
benchmark.

NiCad extracts and pretty-prints functions before configurable renaming and
near-miss comparison. It was run from its official Linux distribution for the
correctness oracle. Cross-OS container startup was deliberately excluded from
the throughput table because it would measure the harness more than NiCad.

## Reproduce a throughput row

Build Weavatrix first:

```console
cargo build --release
```

Then run a prepared, language-only snapshot:

```powershell
.\benchmarks\run_competitors.ps1 `
  -Corpus C:\bench\typescript `
  -Weavatrix .\target\release\weavatrix-clone.exe `
  -Jscpd C:\tools\jscpd.exe `
  -WeavatrixFormat "ts,tsx" `
  -JscpdFormat "typescript,tsx" `
  -Pmd C:\tools\pmd\bin\pmd.bat `
  -JavaHome C:\tools\jre-21 `
  -PmdLanguage typescript `
  -OutputJson C:\bench\typescript-result.json
```

The runner measures native tools separately from PMD so a long JVM process
does not contaminate adjacent sub-second samples. It fails on PMD lexical
errors by default; use `-PmdAllowErrors` only when skipped files are explicitly
reported with the result.

## BigCloneEval

Obtain BigCloneEval and its IJaDataset distribution from the official project.
The benchmark is CC BY-NC 4.0 and is intentionally not vendored here.

Linux/macOS:

```console
./benchmarks/run_bigcloneeval.sh /path/to/bcb_reduced clones.csv
```

PowerShell:

```powershell
.\benchmarks\run_bigcloneeval.ps1 `
  -DatasetRoot C:\path\to\bcb_reduced `
  -OutputPath clones.csv
```

Both runners use the official recommended 50-token and 6-line minimums and
emit the eight CSV fields accepted by `importClones`.

## Public references

- [jscpd](https://github.com/kucherenko/jscpd)
- [PMD CPD](https://pmd.github.io/pmd/pmd_userdocs_cpd)
- [Open-NiCad](https://github.com/CordyJ/Open-NiCad)
- [BigCloneEval](https://github.com/jeffsvajlenko/BigCloneEval)
- [BigCloneBench](https://github.com/clonebench/BigCloneBench)
