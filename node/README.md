# weavatrix-clone

The evidence-first Type-1/2/3 code clone engine behind Weavatrix, exposed as a native library for Node.js and Bun. It is the Rust `weavatrix-clone` core through Node-API—not a JavaScript rewrite, not a CLI wrapper, and not an MCP server.

## Install

```console
npm install weavatrix-clone
# or
bun add weavatrix-clone
```

```js
const { detectRepository } = require('weavatrix-clone')

const report = await detectRepository(process.cwd(), { minTokens: 50 })
for (const pair of report.pairs) {
  console.log(pair.kind, pair.similarityPermille, pair.left.path, '<->', pair.right.path)
}
console.log(report.familyCount, 'families over', report.statistics.sourceFiles, 'files')
```

`detectRepository` scans once and runs off the JavaScript event loop; `detectRepositorySync` is the blocking form. `detectFragments` takes text the caller already has and touches no filesystem:

```js
const { detectFragments } = require('weavatrix-clone')

const report = detectFragments([
  { id: 'a', path: 'src/a.ts', text: sourceA },
  { id: 'b', path: 'src/b.ts', text: sourceB },
])
```

## Evidence and encoders

Every pair carries why it was accepted — `strictEqual`, `renamedEqual`, shared fingerprints, Jaccard and containment in permille, edit distance, and compared tokens — so a consumer can apply its own confidence policy instead of trusting a score.

```js
report.toJsonString()   // the stable clone-report/v1 document
report.toSarif()        // SARIF 2.1.0 for code scanning
report.toBigCloneEval() // BigCloneEval pair rows for dataset-shaped paths
```

`jsonSchema()` returns the JSON Schema for the report document and `jsonSchemaId()` its identifier.

## Native product boundary

One self-contained npm package supports Node.js 18+ and Bun 1.4+ and includes Windows, macOS, and glibc Linux bindings for x64 and arm64. It has no install script, performs no network download, creates no public platform-package names, and writes nothing: detection is read-only.

The surface covers exact, renamed, and bounded near-miss detection, deterministic families, per-pair evidence, repository fragmentation limits, capacity bounds, and the JSON, SARIF, and BigCloneEval encoders.

## Measured

[`benchmark/RESULTS.md`](benchmark/RESULTS.md) compares against `jscpd` 4.3.0 on a 720-file corpus with planted ground truth. Both tools found all 60 byte-identical pairs with no unplanted pairs; Weavatrix also found all 60 renamed pairs, which `jscpd` does not detect, and was 23.12x faster on Node and 16.66x faster on Bun.

Repository: [Weavatrix/weavatrix-clone](https://github.com/Weavatrix/weavatrix-clone) · Rust crate: [crates.io/crates/weavatrix-clone](https://crates.io/crates/weavatrix-clone) · License: [MIT](https://github.com/Weavatrix/weavatrix-clone/blob/main/LICENSE)
