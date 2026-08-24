'use strict'

const native = require('../index.js')

function encodeOptions(options) {
  return options == null || Object.keys(options).length === 0 ? undefined : JSON.stringify(options)
}

/** A completed detection. Encoders read the same Rust report every time. */
class CloneReport {
  constructor(handle) {
    if (!(handle instanceof native.NativeCloneReport)) {
      throw new TypeError('use detectRepository, detectRepositorySync, or detectFragments')
    }
    this._native = handle
    const decoded = JSON.parse(handle.json())
    this.schema = decoded.schema
    this.version = decoded.version
    this.pairs = decoded.pairs
    this.families = decoded.families
    this.statistics = decoded.statistics
  }

  get pairCount() {
    return this._native.pairCount
  }

  get familyCount() {
    return this._native.familyCount
  }

  toJSON() {
    return {
      schema: this.schema,
      version: this.version,
      pairs: this.pairs,
      families: this.families,
      statistics: this.statistics,
    }
  }

  /** The stable `clone-report/v1` document, exactly as Rust encodes it. */
  toJsonString() {
    return this._native.json()
  }

  /** A SARIF 2.1.0 run for code-scanning consumers. */
  toSarif() {
    return this._native.sarif()
  }

  /** The `BigCloneEval` pair export. */
  toBigCloneEval() {
    return this._native.bigCloneEval()
  }
}

async function detectRepository(root, options) {
  return new CloneReport(await native.detectRepository(root, encodeOptions(options)))
}

function detectRepositorySync(root, options) {
  return new CloneReport(native.detectRepositorySync(root, encodeOptions(options)))
}

function detectFragments(fragments, options) {
  if (!Array.isArray(fragments)) {
    throw new TypeError('fragments must be an array of { id, path, text }')
  }
  return new CloneReport(native.detectFragments(JSON.stringify(fragments), encodeOptions(options)))
}

function jsonSchema() {
  return JSON.parse(native.jsonSchema())
}

function jsonSchemaId() {
  return native.jsonSchemaId()
}

module.exports = {
  CloneReport,
  detectRepository,
  detectRepositorySync,
  detectFragments,
  jsonSchema,
  jsonSchemaId,
}
