# weavatrix-clone

Type-1, Type-2, and bounded near-miss Type-3 code clone detection, written in
Rust and exposed to Node.js and Bun through Node-API.

Every reported pair carries the evidence that produced it — whether the tokens
were strictly equal, whether they were equal after renaming, how many
fingerprints they shared, the edit distance, the tokens compared — so a
consumer applies its own confidence policy instead of trusting a score. It
reads; it never writes.

```console
npm install weavatrix-clone
# or
bun add weavatrix-clone
```

```js
const { detectRepository } = require('weavatrix-clone')

const report = await detectRepository(process.cwd(), { minTokens: 50 })
for (const pair of report.pairs) {
  console.log(pair.kind, pair.similarityPermille, pair.left.path, '↔', pair.right.path)
}
report.familyCount               // clusters of mutually cloned sites
report.statistics.sourceFiles    // files actually read
```

---

## Clone types, concretely

| Type | Means | Found by |
| --- | --- | --- |
| **Type-1** | Byte-identical after whitespace and comments | every mode |
| **Type-2** | Identical after renaming identifiers | `renamed` and `nearMiss` |
| **Type-3** | Near-miss: statements inserted, deleted, or reordered | `nearMiss` only |

`mode: 'exact'` skips fragmentation entirely and reports only exact blocks,
which is the cheapest useful run.

---

## API

### `detectRepository(root, options?) → Promise<CloneReport>`

Scans `root` once and detects clone families off the JavaScript event loop.

### `detectRepositorySync(root, options?) → CloneReport`

The blocking form.

### `detectFragments(fragments, options?) → CloneReport`

Detects clones over text the caller already holds. **No filesystem access at
all** — use it for editor buffers, a diff, or content pulled from a database.

```js
detectFragments([
  { id: 'a', path: 'src/a.ts', text: sourceA },
  { id: 'b', path: 'src/b.ts', text: sourceB },
])
```

| Fragment field | Type | Notes |
| --- | --- | --- |
| `id` | `string` | Must be unique and non-empty. |
| `path` | `string` | Non-empty. Backslashes are normalized to `/`. Also selects the language when `language` is omitted. |
| `text` | `string` | The fragment source. |
| `language` | `Language` | Optional override. |
| `span` | `SourceSpan` | Optional. Defaults to the whole of `text`. Set it when the fragment is a slice of a larger file, and the reported spans will use your coordinates. |

### `jsonSchema() → object` and `jsonSchemaId() → string`

The JSON Schema describing `toJsonString()`, and its `$id`. `report.schema`
always equals `jsonSchemaId()`.

---

## Options

| Option | Type | Default | Effect |
| --- | --- | --- | --- |
| `mode` | `'exact' \| 'renamed' \| 'nearMiss'` | `'nearMiss'` | See the table above. |
| `minTokens` | `number` | `24` | Smallest fragment considered. Must cover at least one complete winnowing window (`kGram + winnowingWindow - 1`). |
| `kGram` | `number` | `8` | Token k-gram width for fingerprinting. |
| `winnowingWindow` | `number` | `4` | Winnowing window; together with `kGram` it sets the guaranteed-match length. |
| `minSimilarity` | `number` (permille) | `800` | Final verification threshold, `0 … 1000`. |
| `candidateSimilarity` | `number` (permille) | `450` | Cheap pre-filter threshold. Must not exceed `minSimilarity`. |
| `minSharedFingerprints` | `number` | `2` | Fingerprints two fragments must share to be compared at all. |
| `maxBucketSize` | `number` | `128` | Fingerprint buckets larger than this are suppressed as noise; the count lands in `statistics.suppressedBuckets`. |
| `maxFragments` | `number` | `1000000` | Hard capacity bound. |
| `maxTokensPerFragment` | `number` | `100000` | Hard capacity bound. |
| `maxCandidates` | `number` | `5000000` | Hard capacity bound. |
| `compareOverlappingFragments` | `boolean` | `false` | Whether two fragments that overlap in the same file may be reported as clones of each other. |
| `repository` | `RepositoryOptions` | | Only used by `detectRepository`. |

Similarities are **permille** (0–1000), matching `similarityPermille` in the
report, so a threshold and a result are always the same unit.

### `RepositoryOptions`

| Option | Default | Effect |
| --- | --- | --- |
| `maxFileBytes` | `1500000` | Largest source file read. |
| `minFragmentLines` | `3` | Smallest fragment cut from a file. |
| `maxFragmentLines` | `400` | Largest fragment cut from a file. |
| `parallelism` | `0` | `0` uses the available parallelism. |
| `crossExtensions` | `false` | Whether files with different extensions may be compared. |
| `extensions` | 34 source extensions | Which files are read at all. |

An incomplete scan is an error, not a partial result: a truncated or stopped
scan fails rather than reporting a clone set derived from files it never saw.

---

## `CloneReport`

| Member | Type | Meaning |
| --- | --- | --- |
| `schema`, `version` | `string`, `number` | Schema identity of the report document. |
| `pairs` | `ClonePair[]` | |
| `families` | `CloneFamily[]` | `{ id, members, pairIds }` — connected components over the pairs. |
| `statistics` | `CloneStatistics` | |
| `pairCount`, `familyCount` | `number` | |
| `toJSON()` | `object` | Plain object, so `JSON.stringify(report)` works. |
| `toJsonString()` | `string` | The stable `clone-report/v1` document, exactly as Rust encodes it. |
| `toSarif()` | `string` | SARIF 2.1.0 for code scanning, with rules `WEAVATRIX.CLONE.TYPE1/2/3`. |
| `toBigCloneEval()` | `string` | Eight-column BigCloneEval rows. Requires dataset-shaped paths (`…/default\|selected\|sample/<file>`) and throws a clear error otherwise. |

### `ClonePair`

| Field | Meaning |
| --- | --- |
| `id` | Stable identifier, derived from the pair, not from run order. |
| `kind` | `type1`, `type2`, or `type3`. |
| `similarityPermille` | `0 … 1000`. |
| `left`, `right` | `{ fragmentId, path, span }` with `span` = `{ startByte, endByte, startLine, endLine }`, lines one-based and inclusive. |
| `evidence` | See below. |

### `CloneEvidence`

| Field | Meaning |
| --- | --- |
| `strictEqual` | Token sequences were identical. |
| `renamedEqual` | Identical after identifier canonicalization — this is what makes a pair Type-2. |
| `sharedFingerprints` | Winnowed fingerprints in common. |
| `fingerprintJaccardPermille` | Symmetric overlap. |
| `fingerprintContainmentPermille` | Asymmetric overlap; high containment with low Jaccard means one site is a subset of the other. |
| `editDistance` | Token edit distance actually computed. |
| `comparedTokens` | How many tokens the verification looked at. |

### `CloneStatistics`

`sourceFiles`, `sourceTokens`, `inputFragments`, `analyzedFragments`,
`skippedSmallFragments`, `tokens`, `fingerprints`, `candidatePairs`,
`exactBlockCandidates`, `verifiedPairs`, `suppressedBuckets`,
`suppressedExactBuckets`.

`candidatePairs` versus `verifiedPairs` shows how much the cheap filter
discarded; the two `suppressed*` counters show where a bound stopped the run
from exploding.

---

## Determinism

Pair ids, family ids, and output order are derived from content, not from
traversal order or thread scheduling. The same repository produces a
byte-identical report on every run and every platform, which is what makes the
output usable as a CI gate.

---

## Errors

| `code` | Cause |
| --- | --- |
| `InvalidArg` | Unknown option key or enum value, similarity outside `0 … 1000`, malformed fragment array. |
| `GenericFailure` | Empty fragment id or path, invalid span, `candidateSimilarity` above `minSimilarity`, a capacity bound exceeded, an incomplete scan, non-UTF-8 source, BigCloneEval paths without a dataset category. |

---

## What ships

| | |
| --- | --- |
| Runtimes | Node.js 18+ (Node-API 8), Bun 1.4+ |
| Platforms | Windows x64/arm64, macOS x64/arm64, glibc Linux x64/arm64 |
| Install script | none |
| Network at install | none |
| Runtime dependencies | none |
| Platform packages | none — all six bindings are in this one tarball |
| Writes to disk | none |

---

## Measured

[`benchmark/RESULTS.md`](benchmark/RESULTS.md) is generated from the
[weavatrix-benchmarks](https://github.com/Weavatrix/weavatrix-benchmarks)
harness. The two engines do not emit identical reports, so the shared contract
is **planted ground truth**: a generated 720-file corpus with 60 byte-identical
duplicate pairs, 60 pairs differing only in identifier names, and 480 files
duplicated nowhere.

On the recorded run, against `jscpd` 4.3.0 at the same `minTokens`:

| | exact recall | renamed recall | pairs never planted |
| --- | ---: | ---: | ---: |
| Weavatrix | 1.000 | 1.000 | 0 |
| jscpd 4.3.0 | 1.000 | 0.000 | 0 |

Weavatrix was **30.7x** faster on Node (30.6–30.9 across three independent
runs) and **17.9x** on Bun (17.6–18.1). The renamed column is a capability
difference, not a tuning difference: `jscpd` matches token sequences literally,
so it finds every byte-identical pair and none of the renamed ones.

---

Repository: [Weavatrix/weavatrix-clone](https://github.com/Weavatrix/weavatrix-clone) ·
Rust crate: [crates.io/crates/weavatrix-clone](https://crates.io/crates/weavatrix-clone) ·
License: [MIT](https://github.com/Weavatrix/weavatrix-clone/blob/main/LICENSE)
