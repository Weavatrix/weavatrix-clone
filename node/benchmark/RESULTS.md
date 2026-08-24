# Node.js and Bun benchmark snapshot

Measured on 2026-08-24 on Windows x64 over a generated corpus of 720
TypeScript files, 467,184 bytes total, with planted ground truth: 60
byte-identical file pairs, 60 pairs that differ only in identifier names, and
480 files duplicated nowhere. Both tools ran with `minTokens = 50`. Values are
medians of five measured rounds after one warm-up round.

The competitor is [`jscpd`](https://www.npmjs.com/package/jscpd) 4.3.0, the
JavaScript copy-paste detector. The two engines do not emit identical reports,
so the shared contract is the planted ground truth: for how many planted
duplicate file pairs did each tool report a clone covering both files, and how
long did it take.

| Tool | Runtime | Exact recall | Renamed recall | Unplanted pairs | Median |
| --- | --- | ---: | ---: | ---: | ---: |
| Weavatrix | Node 24.15.0 | 1.000 | 1.000 | 0 | 79.009 ms |
| jscpd 4.3.0 | Node 24.15.0 | 1.000 | 0.000 | 0 | 1827.005 ms |
| Weavatrix | Bun 1.3.14 | 1.000 | 1.000 | 0 | 65.761 ms |
| jscpd 4.3.0 | Bun 1.3.14 | 1.000 | 0.000 | 0 | 1095.780 ms |

Weavatrix was 23.12x faster on Node and 16.66x faster on Bun. Neither tool
reported a file pair outside the planted ground truth on this corpus.

The renamed column is a capability difference, not a tuning difference. `jscpd`
matches token sequences literally, so it finds every byte-identical pair and
none of the renamed ones. Weavatrix canonicalizes identifiers, so the same run
reports the renamed pairs as Type-2 clones with `renamedEqual` evidence. Read
the speed rows with that in mind: Weavatrix is doing strictly more work per
run and is still faster here.

This corpus is synthetic and its duplicates are whole files. Real repositories
carry partial, drifted, and interleaved duplication, and the near-miss Type-3
path — which this corpus does not exercise — is the expensive one.

Reproduce from `node/`:

```console
npm ci
npm run build
npm run bench
bun run benchmark/jscpd.mjs
```

`WV_CLONE_PAIRS`, `WV_CLONE_FILLERS`, `WV_CLONE_MIN_TOKENS`, and
`WV_CLONE_ROUNDS` change the corpus and round count. CPU, filesystem cache,
antivirus, and corpus shape can materially change these timings. Treat them as
a reproducible snapshot, not a universal result.
