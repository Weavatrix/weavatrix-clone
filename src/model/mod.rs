use crate::error::{CloneError, Result};
use std::{cmp::Ordering, path::Path};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Language {
    Rust,
    Go,
    C,
    Cpp,
    Bash,
    Sql,
    JavaScript,
    TypeScript,
    Python,
    Java,
    CSharp,
    Markup,
    Text,
}

impl Language {
    #[must_use]
    pub fn from_path(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        match extension.as_str() {
            "rs" => Self::Rust,
            "go" => Self::Go,
            "c" | "h" => Self::C,
            "cc" | "cpp" | "cxx" | "hh" | "hpp" | "hxx" => Self::Cpp,
            "sh" | "bash" | "zsh" => Self::Bash,
            "sql" | "psql" => Self::Sql,
            "js" | "jsx" | "mjs" | "cjs" => Self::JavaScript,
            "ts" | "tsx" | "mts" | "cts" => Self::TypeScript,
            "py" | "pyi" => Self::Python,
            "java" => Self::Java,
            "cs" => Self::CSharp,
            "html" | "htm" | "xml" | "vue" | "svelte" | "md" | "mdx" => Self::Markup,
            _ => Self::Text,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceSpan {
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_line: u32,
    pub end_line: u32,
}

impl SourceSpan {
    #[must_use]
    pub fn whole(text: &str) -> Self {
        Self {
            start_byte: 0,
            end_byte: text.len(),
            start_line: 1,
            end_line: u32::try_from(text.lines().count().max(1)).unwrap_or(u32::MAX),
        }
    }

    #[must_use]
    pub const fn overlaps(self, other: Self) -> bool {
        self.start_byte < other.end_byte && other.start_byte < self.end_byte
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFragment {
    pub id: String,
    pub path: String,
    pub language: Language,
    pub span: SourceSpan,
    pub text: String,
}

impl SourceFragment {
    /// Creates one independently comparable source fragment.
    ///
    /// # Errors
    ///
    /// Rejects empty identifiers/paths and invalid source spans.
    pub fn new(
        id: impl Into<String>,
        path: impl Into<String>,
        language: Language,
        span: SourceSpan,
        text: impl Into<String>,
    ) -> Result<Self> {
        let fragment = Self {
            id: id.into(),
            path: path.into().replace('\\', "/"),
            language,
            span,
            text: text.into(),
        };
        fragment.validate()?;
        Ok(fragment)
    }

    pub(crate) fn validate(&self) -> Result<()> {
        let reason = if self.id.trim().is_empty() {
            Some("id must not be empty")
        } else if self.path.trim().is_empty() {
            Some("path must not be empty")
        } else if self.span.start_byte > self.span.end_byte {
            Some("start_byte must not exceed end_byte")
        } else if self.span.start_line == 0 || self.span.start_line > self.span.end_line {
            Some("line range must be one-based and ordered")
        } else {
            None
        };
        if let Some(reason) = reason {
            return Err(CloneError::InvalidFragment {
                id: self.id.clone(),
                reason,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Similarity(u16);

impl Similarity {
    pub const PERFECT: Self = Self(1_000);

    #[must_use]
    pub const fn from_permille(value: u16) -> Self {
        Self(if value > 1_000 { 1_000 } else { value })
    }

    #[must_use]
    pub fn from_ratio(numerator: usize, denominator: usize) -> Self {
        if denominator == 0 {
            return Self::from_permille(0);
        }
        let scaled = numerator.saturating_mul(1_000) / denominator;
        Self::from_permille(u16::try_from(scaled).unwrap_or(1_000))
    }

    #[must_use]
    pub const fn permille(self) -> u16 {
        self.0
    }

    #[must_use]
    pub fn percent(self) -> f32 {
        f32::from(self.0) / 10.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CloneKind {
    Type1,
    Type2,
    Type3,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DetectionMode {
    Exact,
    Renamed,
    #[default]
    NearMiss,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CloneLocation {
    pub fragment_id: String,
    pub path: String,
    pub span: SourceSpan,
}

impl CloneLocation {
    pub(crate) fn from_fragment(fragment: &SourceFragment) -> Self {
        Self {
            fragment_id: fragment.id.clone(),
            path: fragment.path.clone(),
            span: fragment.span,
        }
    }
}

impl Ord for CloneLocation {
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.path, self.span, &self.fragment_id).cmp(&(
            &other.path,
            other.span,
            &other.fragment_id,
        ))
    }
}

impl PartialOrd for CloneLocation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloneEvidence {
    pub strict_equal: bool,
    pub renamed_equal: bool,
    pub shared_fingerprints: usize,
    pub fingerprint_jaccard: Similarity,
    pub fingerprint_containment: Similarity,
    pub edit_distance: usize,
    pub compared_tokens: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClonePair {
    pub id: String,
    pub left: CloneLocation,
    pub right: CloneLocation,
    pub kind: CloneKind,
    pub similarity: Similarity,
    pub evidence: CloneEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloneFamily {
    pub id: String,
    pub members: Vec<CloneLocation>,
    pub pair_ids: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CloneStatistics {
    pub source_files: usize,
    pub source_tokens: usize,
    pub input_fragments: usize,
    pub analyzed_fragments: usize,
    pub skipped_small_fragments: usize,
    pub tokens: usize,
    pub fingerprints: usize,
    pub candidate_pairs: usize,
    pub exact_block_candidates: usize,
    pub verified_pairs: usize,
    pub suppressed_buckets: usize,
    pub suppressed_exact_buckets: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CloneReport {
    pub pairs: Vec<ClonePair>,
    pub families: Vec<CloneFamily>,
    pub statistics: CloneStatistics,
}
