# Node.js and Bun benchmark snapshot

This file is generated. Every number below was produced by the
[weavatrix-benchmarks](https://github.com/Weavatrix/weavatrix-benchmarks)
harness and copied out of its recorded run; none of it is typed by hand.
That repository states the rules every suite obeys, including what each
row had to prove equal before it was allowed to be timed.

**Question.** How many planted duplicate pairs does each detector find, and how long does it take?

**Competitor.** `jscpd`

| Property | Value |
| --- | --- |
| Measured | 2026-08-24 |
| Platform | win32 x64, 10.0.26200 |
| CPU | Intel(R) Core(TM) Ultra 7 255U (14 logical cores) |
| Memory | 47.5 GiB |
| Rounds | 7 measured, after 2 warm-ups, alternating order, median reported |
| Independent runs | 3 per suite, each in a fresh process; the table shows the median and the spread |
| Package | weavatrix-clone 0.1.5 |

## node 24.15.0

Corpus: `[{"files":720,"bytes":467184,"plantedExactPairs":60,"plantedRenamedPairs":60,"minTokens":50}]`

| Contract | Parity | Weavatrix | Competitor | Result |
| --- | --- | ---: | ---: | ---: |
| planted duplicate pairs found over the whole corpus | planted ground truth | 72.673 ms | 2247.768 ms | Weavatrix 30.70x faster (30.64x–30.93x) |

## bun 1.3.14

Corpus: `[{"files":720,"bytes":467184,"plantedExactPairs":60,"plantedRenamedPairs":60,"minTokens":50}]`

| Contract | Parity | Weavatrix | Competitor | Result |
| --- | --- | ---: | ---: | ---: |
| planted duplicate pairs found over the whole corpus | planted ground truth | 70.229 ms | 1236.213 ms | Weavatrix 17.88x faster (17.60x–18.08x) |

## Reading these rows

- The two engines do not emit identical reports, so the shared contract is planted ground truth: for how many duplicate file pairs did each tool report a clone covering both files. jscpd matches token sequences literally, so renamed duplicates are a capability gap, not a tuning difference.
- Weavatrix recall on the planted pairs: exact 1.000, renamed 1.000, pairs reported that were never planted: 0.
- Competitor recall on the planted pairs: exact 1.000, renamed 0.000, pairs reported that were never planted: 0.

## Reproduce

```console
git clone https://github.com/Weavatrix/weavatrix-benchmarks
cd weavatrix-benchmarks && npm ci
node run.mjs --suite=clone
bun run.mjs --suite=clone
node export.mjs
```

CPU, memory bandwidth, filesystem, antivirus, and JavaScript engine
version all move these timings. Treat them as a reproducible snapshot of
the environment above, not as a universal result.
