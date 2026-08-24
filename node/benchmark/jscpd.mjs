// Comparison against jscpd, the JavaScript copy-paste detector, over a corpus
// with planted ground truth.
//
// The two engines do not produce identical reports, so this measures something
// both can be judged on: for how many planted duplicate file pairs did each
// tool report a clone covering both files, and how long did it take. Exact
// pairs are byte-identical; renamed pairs differ only in identifier names.
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { performance } from 'node:perf_hooks'
import { createRequire } from 'node:module'

const require = createRequire(import.meta.url)
const { detectClones } = require('jscpd')
const { detectRepositorySync } = require('../lib/index.js')

const pairs = Number(process.env.WV_CLONE_PAIRS ?? 60)
const fillers = Number(process.env.WV_CLONE_FILLERS ?? 480)
const rounds = Number(process.env.WV_CLONE_ROUNDS ?? 5)
const minTokens = Number(process.env.WV_CLONE_MIN_TOKENS ?? 50)
const root = fs.mkdtempSync(path.join(os.tmpdir(), 'weavatrix-clone-bench-'))
// jscpd's output directory must live outside the scanned corpus. Leaving it
// inside makes jscpd re-crawl its own growing output on every later round,
// which inflates its time and our ratio.
const jscpdOutput = fs.mkdtempSync(path.join(os.tmpdir(), 'weavatrix-clone-bench-jscpd-'))
// jscpd resolves its input through fast-glob, which only accepts POSIX
// separators, so a Windows temporary directory has to be converted.
const globRoot = root.split(path.sep).join('/')

function unit(seed, names) {
  return [
    `export function ${names.fn}(source) {`,
    `  const ${names.a} = source.map((entry) => entry.value * ${seed})`,
    `  const ${names.b} = ${names.a}.filter((value) => value > ${seed % 7})`,
    ...Array.from({ length: 10 }, (_, line) => `  const ${names.c}${line} = combine(${names.b}, ${line}, ${seed})`),
    `  const ${names.d} = ${names.c}0 + ${names.c}1 + ${names.c}2`,
    `  if (${names.d} > ${seed * 13}) { throw new Error("${seed}") }`,
    `  return { ${names.a}, ${names.b}, ${names.d} }`,
    '}',
    '',
  ].join('\n')
}

const plain = (suffix) => ({ fn: `run${suffix}`, a: 'mapped', b: 'kept', c: 'step', d: 'total' })
const renamed = (suffix) => ({ fn: `execute${suffix}`, a: 'projected', b: 'retained', c: 'phase', d: 'aggregate' })

const truth = { exact: [], renamed: [] }
for (let index = 0; index < pairs; index += 1) {
  const seed = index + 2
  const left = `exact${index}_a.ts`
  const right = `exact${index}_b.ts`
  fs.writeFileSync(path.join(root, left), unit(seed, plain(index)))
  fs.writeFileSync(path.join(root, right), unit(seed, plain(index)))
  truth.exact.push([left, right])
}
for (let index = 0; index < pairs; index += 1) {
  const seed = index + 2 + pairs
  const left = `renamed${index}_a.ts`
  const right = `renamed${index}_b.ts`
  fs.writeFileSync(path.join(root, left), unit(seed, plain(index)))
  fs.writeFileSync(path.join(root, right), unit(seed, renamed(index)))
  truth.renamed.push([left, right])
}
for (let index = 0; index < fillers; index += 1) {
  const seed = index + 2 + pairs * 2
  fs.writeFileSync(path.join(root, `unique${index}.ts`), unit(seed, plain(`Unique${index}`)))
}

const files = pairs * 2 + pairs * 2 + fillers
const corpusBytes = fs.readdirSync(root)
  .reduce((total, file) => total + fs.statSync(path.join(root, file)).size, 0)

function pairKey(left, right) {
  return [left, right].sort().join('|')
}

function score(reported) {
  const found = new Set(reported)
  const measure = (list) => list.filter(([left, right]) => found.has(pairKey(left, right))).length
  const planted = new Set([...truth.exact, ...truth.renamed].map(([left, right]) => pairKey(left, right)))
  return {
    exactRecall: Number((measure(truth.exact) / truth.exact.length).toFixed(4)),
    renamedRecall: Number((measure(truth.renamed) / truth.renamed.length).toFixed(4)),
    filePairs: found.size,
    unplantedFilePairs: [...found].filter((key) => !planted.has(key)).length,
  }
}

function weavatrix() {
  const report = detectRepositorySync(root, { minTokens, repository: { extensions: ['ts'] } })
  return report.pairs.map((pair) => pairKey(path.basename(pair.left.path), path.basename(pair.right.path)))
}

async function jscpd() {
  const clones = await detectClones({
    path: [globRoot],
    silent: true,
    gitignore: false,
    reporters: [],
    minTokens,
    minLines: 5,
    output: jscpdOutput,
  })
  return clones.map((clone) =>
    pairKey(path.basename(clone.duplicationA.sourceId), path.basename(clone.duplicationB.sourceId)))
}

function median(samples) {
  const sorted = [...samples].sort((left, right) => left - right)
  return Number(sorted[Math.floor(sorted.length / 2)].toFixed(3))
}

async function measure(operation) {
  const samples = []
  for (let round = 0; round < rounds + 1; round += 1) {
    const start = performance.now()
    await operation()
    if (round > 0) samples.push(performance.now() - start)
  }
  return median(samples)
}

try {
  const weavatrixScore = score(weavatrix())
  const jscpdScore = score(await jscpd())
  const weavatrixMs = await measure(weavatrix)
  const jscpdMs = await measure(jscpd)

  console.log(JSON.stringify({
    runtime: process.versions.bun ? `bun ${process.versions.bun}` : `node ${process.version}`,
    files,
    corpusBytes,
    plantedExactPairs: truth.exact.length,
    plantedRenamedPairs: truth.renamed.length,
    minTokens,
    rounds,
    weavatrix: { ...weavatrixScore, medianMs: weavatrixMs },
    jscpd: { ...jscpdScore, medianMs: jscpdMs },
    speedup: Number((jscpdMs / weavatrixMs).toFixed(2)),
  }, null, 2))
} finally {
  fs.rmSync(root, { recursive: true, force: true })
  fs.rmSync(jscpdOutput, { recursive: true, force: true })
}
