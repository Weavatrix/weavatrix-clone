'use strict'

const assert = require('node:assert/strict')
const fs = require('node:fs')
const os = require('node:os')
const path = require('node:path')
const test = require('node:test')
const {
  CloneReport,
  detectFragments,
  detectRepository,
  detectRepositorySync,
  jsonSchema,
  jsonSchemaId,
} = require('..')

function body(name, variable) {
  return [
    `export function ${name}(input) {`,
    `  const ${variable} = input.map((entry) => entry.value * 2)`,
    `  const filtered = ${variable}.filter((value) => value > 10)`,
    '  const total = filtered.reduce((sum, value) => sum + value, 0)',
    '  if (total > 1000) {',
    '    throw new Error("overflow")',
    '  }',
    '  return { total, count: filtered.length }',
    '}',
    '',
  ].join('\n')
}

function fragments() {
  return [
    { id: 'a', path: 'src/a.ts', text: body('alpha', 'doubled') },
    { id: 'b', path: 'src/b.ts', text: body('beta', 'scaled') },
    { id: 'c', path: 'src/c.ts', text: 'export const unrelated = 1\n' },
  ]
}

async function withRepository(callback) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'weavatrix-clone-test-'))
  try {
    fs.mkdirSync(path.join(root, 'src'))
    for (const fragment of fragments()) {
      fs.writeFileSync(path.join(root, fragment.path), fragment.text)
    }
    return await callback(root)
  } finally {
    fs.rmSync(root, { recursive: true, force: true })
  }
}

test('detects renamed clones over caller-supplied fragments', () => {
  const report = detectFragments(fragments())
  assert.equal(report.pairCount, 1)
  assert.equal(report.pairs.length, 1)
  const pair = report.pairs[0]
  assert.equal(pair.kind, 'type2')
  assert.deepEqual([pair.left.fragmentId, pair.right.fragmentId], ['a', 'b'])
  assert.equal(pair.evidence.renamedEqual, true)
  assert.equal(pair.evidence.strictEqual, false)
  assert.equal(pair.similarityPermille, 1000)
  assert.equal(report.familyCount, 1)
  assert.deepEqual(report.families[0].pairIds, [pair.id])
  assert.equal(report.statistics.inputFragments, 3)
  assert.equal(report.statistics.verifiedPairs, 1)
})

test('exposes the stable schema and the SARIF encoder', () => {
  const report = detectFragments(fragments())
  assert.equal(report.schema, jsonSchemaId())
  assert.equal(report.version, 1)
  assert.deepEqual(JSON.parse(report.toJsonString()), report.toJSON())
  assert.equal(jsonSchema().$id, jsonSchemaId())

  const sarif = JSON.parse(report.toSarif())
  assert.equal(sarif.version, '2.1.0')
  assert.equal(sarif.runs[0].tool.driver.name, 'weavatrix-clone')
  assert.equal(sarif.runs[0].results.length, 1)
  assert.equal(sarif.runs[0].results[0].ruleId, 'WEAVATRIX.CLONE.TYPE2')
})

test('exports BigCloneEval rows only for dataset-shaped paths', () => {
  const dataset = fragments().map((fragment, index) => ({
    ...fragment,
    path: `bench/default/File${index}.java`,
  }))
  const rows = detectFragments(dataset).toBigCloneEval().trim().split('\n')
  assert.equal(rows.length, 1)
  assert.deepEqual(rows[0].split(','), ['default', 'File0.java', '1', '9', 'default', 'File1.java', '1', '9'])
  assert.throws(() => detectFragments(fragments()).toBigCloneEval(), /no dataset category/)
})

test('detects clone families across a repository', async () => {
  await withRepository(async (root) => {
    const report = await detectRepository(root)
    assert.ok(report instanceof CloneReport)
    assert.equal(report.statistics.sourceFiles, 3)
    assert.ok(report.pairCount >= 1)
    assert.ok(report.pairs.every((pair) => pair.left.path.startsWith('src/')))
    const sync = detectRepositorySync(root)
    assert.deepEqual(sync.toJSON(), report.toJSON())
  })
})

test('honors detection mode and similarity thresholds', () => {
  assert.equal(detectFragments(fragments(), { mode: 'exact' }).pairCount, 0)
  assert.equal(detectFragments(fragments(), { minTokens: 4096 }).pairCount, 0)
  assert.equal(detectFragments(fragments(), { minSimilarity: 1000 }).pairCount, 1)
})

test('rejects unusable options and fragments', () => {
  assert.throws(() => detectFragments(fragments(), { mode: 'fuzzy' }), /unsupported detection mode/)
  assert.throws(() => detectFragments(fragments(), { minSimilarity: 1001 }), /permille value/)
  assert.throws(() => detectFragments(fragments(), { candidateSimilarity: 900, minSimilarity: 800 }), /candidate_similarity/)
  assert.throws(() => detectFragments(fragments(), { nonsense: 1 }), /nonsense/)
  assert.throws(() => detectFragments([{ id: '', path: 'a.ts', text: 'x' }]), /id must not be empty/)
  assert.throws(() => detectFragments('not-an-array'), /must be an array/)
})
