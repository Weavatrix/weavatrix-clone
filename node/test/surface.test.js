'use strict'

// Exercises every exported member, every option key, every language profile,
// and every encoder, so a broken surface fails here rather than in a consumer.

const assert = require('node:assert/strict')
const fs = require('node:fs')
const os = require('node:os')
const path = require('node:path')
const test = require('node:test')
const clone = require('..')

const {
  CloneReport,
  detectFragments,
  detectRepository,
  detectRepositorySync,
  jsonSchema,
  jsonSchemaId,
} = clone

const LANGUAGES = {
  rust: 'rs',
  go: 'go',
  c: 'c',
  cpp: 'cpp',
  bash: 'sh',
  sql: 'sql',
  javascript: 'js',
  typescript: 'ts',
  python: 'py',
  java: 'java',
  csharp: 'cs',
  markup: 'html',
  text: 'txt',
}

function body(name, variable) {
  return [
    `function ${name}(input) {`,
    `  ${variable}_one = collect(input, 1, 2, 3)`,
    `  ${variable}_two = collect(${variable}_one, 4, 5, 6)`,
    `  ${variable}_three = combine(${variable}_one, ${variable}_two, 7)`,
    `  ${variable}_four = combine(${variable}_three, ${variable}_two, 8)`,
    `  ${variable}_five = reduce(${variable}_four, ${variable}_three, 9)`,
    `  return finish(${variable}_five, ${variable}_four, 10)`,
    '}',
    '',
  ].join('\n')
}

function pair(extension) {
  return [
    { id: 'left', path: `src/left.${extension}`, text: body('alpha', 'value') },
    { id: 'right', path: `src/right.${extension}`, text: body('beta', 'result') },
  ]
}

async function withRepository(callback) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'weavatrix-clone-surface-'))
  try {
    fs.mkdirSync(path.join(root, 'src'))
    for (const fragment of pair('ts')) {
      fs.writeFileSync(path.join(root, fragment.path), fragment.text)
    }
    fs.writeFileSync(path.join(root, 'src', 'other.ts'), 'export const solo = 1\n')
    return await callback(root)
  } finally {
    fs.rmSync(root, { recursive: true, force: true })
  }
}

test('exports exactly the documented surface', () => {
  assert.deepEqual(
    Object.keys(clone).sort(),
    ['CloneReport', 'detectFragments', 'detectRepository', 'detectRepositorySync', 'jsonSchema', 'jsonSchemaId'],
  )
  assert.deepEqual(
    Object.getOwnPropertyNames(CloneReport.prototype).sort(),
    ['constructor', 'familyCount', 'pairCount', 'toBigCloneEval', 'toJSON', 'toJsonString', 'toSarif'],
  )
  assert.throws(() => new CloneReport({}), /detectRepository, detectRepositorySync, or detectFragments/)
})

test('the published schema describes the emitted report', () => {
  const schema = jsonSchema()
  assert.equal(schema.$id, jsonSchemaId())
  assert.equal(schema.type, 'object')
  assert.deepEqual(schema.required, ['schema', 'version', 'pairs', 'families', 'statistics'])
  const report = detectFragments(pair('ts'))
  assert.deepEqual(Object.keys(report.toJSON()).sort(), [...schema.required].sort())
})

test('every language profile tokenizes and detects', () => {
  for (const [language, extension] of Object.entries(LANGUAGES)) {
    const byPath = detectFragments(pair(extension), { minTokens: 24 })
    assert.equal(byPath.pairCount, 1, `${language} must detect through the path extension`)

    const explicit = detectFragments(
      pair(extension).map((fragment) => ({ ...fragment, language })),
      { minTokens: 24 },
    )
    assert.deepEqual(explicit.toJSON(), byPath.toJSON(), `${language} must match the explicit form`)
  }
  assert.throws(() => detectFragments(pair('ts').map((f) => ({ ...f, language: 'cobol' }))), /unsupported language/)
})

test('carries every documented detection option', () => {
  const report = detectFragments(pair('ts'), {
    mode: 'nearMiss',
    minTokens: 24,
    kGram: 8,
    winnowingWindow: 4,
    minSimilarity: 800,
    candidateSimilarity: 450,
    minSharedFingerprints: 2,
    maxBucketSize: 128,
    maxFragments: 1000,
    maxTokensPerFragment: 10_000,
    maxCandidates: 100_000,
    compareOverlappingFragments: false,
  })
  assert.equal(report.pairCount, 1)
  const [found] = report.pairs
  assert.deepEqual(Object.keys(found).sort(), ['evidence', 'id', 'kind', 'left', 'right', 'similarityPermille'])
  assert.deepEqual(Object.keys(found.left).sort(), ['fragmentId', 'path', 'span'])
  assert.deepEqual(Object.keys(found.left.span).sort(), ['endByte', 'endLine', 'startByte', 'startLine'])
  assert.deepEqual(Object.keys(found.evidence).sort(), [
    'comparedTokens', 'editDistance', 'fingerprintContainmentPermille',
    'fingerprintJaccardPermille', 'sharedFingerprints', 'strictEqual', 'renamedEqual',
  ].sort())
  assert.deepEqual(Object.keys(report.statistics).sort(), [
    'analyzedFragments', 'candidatePairs', 'exactBlockCandidates', 'fingerprints',
    'inputFragments', 'skippedSmallFragments', 'sourceFiles', 'sourceTokens', 'tokens',
    'suppressedBuckets', 'suppressedExactBuckets', 'verifiedPairs',
  ].sort())
  assert.deepEqual(Object.keys(report.families[0]).sort(), ['id', 'members', 'pairIds'])
})

test('each detection mode narrows what is reported', () => {
  const identical = [
    { id: 'left', path: 'src/left.ts', text: body('alpha', 'value') },
    { id: 'right', path: 'src/right.ts', text: body('alpha', 'value') },
  ]
  assert.equal(detectFragments(identical, { mode: 'exact' }).pairs[0].kind, 'type1')
  assert.equal(detectFragments(pair('ts'), { mode: 'exact' }).pairCount, 0)
  assert.equal(detectFragments(pair('ts'), { mode: 'renamed' }).pairs[0].kind, 'type2')
  assert.equal(detectFragments(pair('ts'), { mode: 'nearMiss' }).pairs[0].kind, 'type2')
})

test('explicit spans and byte ranges are preserved', () => {
  const left = body('alpha', 'value')
  const report = detectFragments([
    { id: 'left', path: 'src/left.ts', text: left, span: { startByte: 40, endByte: 40 + left.length, startLine: 7, endLine: 15 } },
    { id: 'right', path: 'src/right.ts', text: body('beta', 'result') },
  ], { minTokens: 24 })
  assert.equal(report.pairCount, 1)
  assert.deepEqual(report.pairs[0].left.span, {
    startByte: 40,
    endByte: 40 + left.length,
    startLine: 7,
    endLine: 15,
  })
  assert.equal(report.pairs[0].right.span.startLine, 1)
  assert.throws(
    () => detectFragments([{ id: 'x', path: 'a.ts', text: 'y', span: { startByte: 5, endByte: 1, startLine: 1, endLine: 1 } }]),
    /start_byte must not exceed end_byte/,
  )
  assert.throws(
    () => detectFragments([{ id: 'x', path: 'a.ts', text: 'y', span: { startByte: 0, endByte: 1, startLine: 0, endLine: 1 } }]),
    /line range must be one-based/,
  )
})

test('repository detection carries every repository option', async () => {
  await withRepository(async (root) => {
    const report = await detectRepository(root, {
      minTokens: 24,
      repository: {
        maxFileBytes: 1_000_000,
        minFragmentLines: 3,
        maxFragmentLines: 400,
        parallelism: 2,
        crossExtensions: false,
        extensions: ['ts'],
      },
    })
    assert.ok(report instanceof CloneReport)
    assert.equal(report.statistics.sourceFiles, 3)
    assert.ok(report.pairCount >= 1)
    assert.ok(report.statistics.sourceTokens > 0)

    const narrowed = await detectRepository(root, { minTokens: 24, repository: { extensions: ['rs'] } })
    assert.equal(narrowed.statistics.sourceFiles, 0)
    assert.equal(narrowed.pairCount, 0)

    assert.deepEqual(detectRepositorySync(root, { minTokens: 24 }).toJSON(), (await detectRepository(root, { minTokens: 24 })).toJSON())
    await assert.rejects(detectRepository(path.join(root, 'missing'), {}), { code: 'GenericFailure' })
  })
})

test('all three encoders emit their documented formats', () => {
  const report = detectFragments(pair('ts'))
  const json = JSON.parse(report.toJsonString())
  assert.deepEqual(json, report.toJSON())
  assert.equal(json.schema, jsonSchemaId())

  const sarif = JSON.parse(report.toSarif())
  assert.equal(sarif.$schema, 'https://json.schemastore.org/sarif-2.1.0.json')
  assert.deepEqual(
    sarif.runs[0].tool.driver.rules.map((rule) => rule.id),
    ['WEAVATRIX.CLONE.TYPE1', 'WEAVATRIX.CLONE.TYPE2', 'WEAVATRIX.CLONE.TYPE3'],
  )
  assert.equal(sarif.runs[0].results.length, report.pairCount)

  const dataset = pair('java').map((fragment, index) => ({
    ...fragment,
    path: `bench/selected/File${index}.java`,
  }))
  const rows = detectFragments(dataset).toBigCloneEval().trim().split('\n')
  assert.equal(rows.length, 1)
  assert.equal(rows[0].split(',').length, 8)
  assert.equal(rows[0].split(',')[0], 'selected')
})

test('capacity limits and malformed input fail closed', () => {
  assert.throws(() => detectFragments(pair('ts'), { maxFragments: 1 }), /input fragments|capacity/i)
  assert.throws(() => detectFragments(pair('ts'), { kGram: 0 }), /k_gram/)
  assert.throws(() => detectFragments(pair('ts'), { winnowingWindow: 0 }), /winnowing_window/)
  assert.throws(() => detectFragments(pair('ts'), { minTokens: 1 }), /winnowing window/)
  assert.throws(() => detectFragments([{ id: 'a', path: '', text: 'x' }]), /path must not be empty/)
  assert.throws(() => detectFragments([{ id: 'a', path: 'a.ts' }]), /missing field `text`/)
  assert.throws(() => detectFragments([{ id: 'a', path: 'a.ts', text: 'x', extra: 1 }]), /unknown field/)
})
