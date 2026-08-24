export type DetectionMode = 'exact' | 'renamed' | 'nearMiss'

export type Language =
  | 'rust' | 'go' | 'c' | 'cpp' | 'bash' | 'sql' | 'javascript' | 'typescript'
  | 'python' | 'java' | 'csharp' | 'markup' | 'text'

export interface RepositoryOptions {
  maxFileBytes?: number
  minFragmentLines?: number
  maxFragmentLines?: number
  /** `0` uses the available parallelism. */
  parallelism?: number
  /** Compares fragments across different file extensions. */
  crossExtensions?: boolean
  extensions?: string[]
}

export interface CloneOptions {
  mode?: DetectionMode
  minTokens?: number
  kGram?: number
  winnowingWindow?: number
  /** Final verification threshold in permille, `0 … 1000`. */
  minSimilarity?: number
  /** Candidate threshold in permille; must not exceed `minSimilarity`. */
  candidateSimilarity?: number
  minSharedFingerprints?: number
  maxBucketSize?: number
  maxFragments?: number
  maxTokensPerFragment?: number
  maxCandidates?: number
  compareOverlappingFragments?: boolean
  repository?: RepositoryOptions
}

export interface SourceSpan {
  startByte: number
  endByte: number
  /** One-based. */
  startLine: number
  /** One-based, inclusive. */
  endLine: number
}

export interface SourceFragment {
  id: string
  path: string
  text: string
  /** Defaults to the language implied by `path`. */
  language?: Language
  /** Defaults to the whole of `text`. */
  span?: SourceSpan
}

export interface CloneLocation {
  fragmentId: string
  path: string
  span: SourceSpan
}

export interface CloneEvidence {
  strictEqual: boolean
  renamedEqual: boolean
  sharedFingerprints: number
  fingerprintJaccardPermille: number
  fingerprintContainmentPermille: number
  editDistance: number
  comparedTokens: number
}

export interface ClonePair {
  id: string
  kind: 'type1' | 'type2' | 'type3'
  similarityPermille: number
  left: CloneLocation
  right: CloneLocation
  evidence: CloneEvidence
}

export interface CloneFamily { id: string; members: CloneLocation[]; pairIds: string[] }

export interface CloneStatistics {
  sourceFiles: number
  sourceTokens: number
  inputFragments: number
  analyzedFragments: number
  skippedSmallFragments: number
  tokens: number
  fingerprints: number
  candidatePairs: number
  exactBlockCandidates: number
  verifiedPairs: number
  suppressedBuckets: number
  suppressedExactBuckets: number
}

export declare class CloneReport {
  readonly schema: string
  readonly version: number
  readonly pairs: ClonePair[]
  readonly families: CloneFamily[]
  readonly statistics: CloneStatistics
  readonly pairCount: number
  readonly familyCount: number
  toJSON(): {
    schema: string
    version: number
    pairs: ClonePair[]
    families: CloneFamily[]
    statistics: CloneStatistics
  }
  /** The stable `clone-report/v1` document, exactly as Rust encodes it. */
  toJsonString(): string
  /** A SARIF 2.1.0 run for code-scanning consumers. */
  toSarif(): string
  /** The `BigCloneEval` pair export. */
  toBigCloneEval(): string
}

export declare function detectRepository(root: string, options?: CloneOptions): Promise<CloneReport>
export declare function detectRepositorySync(root: string, options?: CloneOptions): CloneReport
export declare function detectFragments(
  fragments: SourceFragment[],
  options?: CloneOptions,
): CloneReport
/** The JSON Schema document describing `toJsonString()`. */
export declare function jsonSchema(): Record<string, unknown>
export declare function jsonSchemaId(): string
