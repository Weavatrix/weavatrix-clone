//! Decoding of the JSON policy and fragment documents crossing the boundary.

use napi::{Error, Result, Status};
use serde::Deserialize;
use weavatrix_clone::{
    CloneConfig, CloneDetector, DetectionMode, Language, RepositoryOptions, Similarity,
    SourceFragment, SourceSpan,
};

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct NodeCloneOptions {
    mode: Option<String>,
    min_tokens: Option<usize>,
    k_gram: Option<usize>,
    winnowing_window: Option<usize>,
    /// Final verification threshold in permille, `0..=1000`.
    min_similarity: Option<u16>,
    /// Candidate threshold in permille, `0..=1000`.
    candidate_similarity: Option<u16>,
    min_shared_fingerprints: Option<usize>,
    max_bucket_size: Option<usize>,
    max_fragments: Option<usize>,
    max_tokens_per_fragment: Option<usize>,
    max_candidates: Option<usize>,
    compare_overlapping_fragments: Option<bool>,
    repository: Option<NodeRepositoryOptions>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct NodeRepositoryOptions {
    max_file_bytes: Option<u64>,
    min_fragment_lines: Option<usize>,
    max_fragment_lines: Option<usize>,
    parallelism: Option<usize>,
    cross_extensions: Option<bool>,
    extensions: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NodeFragment {
    id: String,
    path: String,
    text: String,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    span: Option<NodeSpan>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NodeSpan {
    start_byte: usize,
    end_byte: usize,
    start_line: u32,
    end_line: u32,
}

pub(crate) fn decode_detector(options_json: Option<&str>) -> Result<CloneDetector> {
    let input = parse(options_json)?;
    CloneDetector::new(clone_config(&input)?).map_err(clone_error)
}

pub(crate) fn decode_repository(options_json: Option<&str>) -> Result<RepositoryOptions> {
    let input = parse::<NodeCloneOptions>(options_json)?;
    let mut options = RepositoryOptions::default();
    let Some(repository) = input.repository else {
        return Ok(options);
    };
    if let Some(value) = repository.max_file_bytes {
        options.max_file_bytes = value;
    }
    if let Some(value) = repository.min_fragment_lines {
        options.min_fragment_lines = value;
    }
    if let Some(value) = repository.max_fragment_lines {
        options.max_fragment_lines = value;
    }
    if let Some(value) = repository.parallelism {
        options.parallelism = value;
    }
    if let Some(value) = repository.cross_extensions {
        options.cross_extensions = value;
    }
    if let Some(value) = repository.extensions {
        options = options.with_extensions(value);
    }
    Ok(options)
}

pub(crate) fn decode_fragments(fragments_json: &str) -> Result<Vec<SourceFragment>> {
    serde_json::from_str::<Vec<NodeFragment>>(fragments_json)
        .map_err(invalid)?
        .into_iter()
        .map(fragment)
        .collect()
}

fn fragment(input: NodeFragment) -> Result<SourceFragment> {
    let language = match &input.language {
        Some(value) => language(value)?,
        None => Language::from_path(&input.path),
    };
    let span = input.span.map_or_else(
        || SourceSpan::whole(&input.text),
        |span| SourceSpan {
            start_byte: span.start_byte,
            end_byte: span.end_byte,
            start_line: span.start_line,
            end_line: span.end_line,
        },
    );
    SourceFragment::new(input.id, input.path, language, span, input.text).map_err(clone_error)
}

fn clone_config(input: &NodeCloneOptions) -> Result<CloneConfig> {
    let mut config = CloneConfig::default();
    if let Some(value) = &input.mode {
        config.mode = mode(value)?;
    }
    if let Some(value) = input.min_tokens {
        config.min_tokens = value;
    }
    if let Some(value) = input.k_gram {
        config.k_gram = value;
    }
    if let Some(value) = input.winnowing_window {
        config.winnowing_window = value;
    }
    if let Some(value) = input.min_similarity {
        config.min_similarity = permille("minSimilarity", value)?;
    }
    if let Some(value) = input.candidate_similarity {
        config.candidate_similarity = permille("candidateSimilarity", value)?;
    }
    if let Some(value) = input.min_shared_fingerprints {
        config.min_shared_fingerprints = value;
    }
    if let Some(value) = input.max_bucket_size {
        config.max_bucket_size = value;
    }
    if let Some(value) = input.max_fragments {
        config.max_fragments = value;
    }
    if let Some(value) = input.max_tokens_per_fragment {
        config.max_tokens_per_fragment = value;
    }
    if let Some(value) = input.max_candidates {
        config.max_candidates = value;
    }
    if let Some(value) = input.compare_overlapping_fragments {
        config.compare_overlapping_fragments = value;
    }
    Ok(config)
}

fn parse<T: Default + for<'de> Deserialize<'de>>(options_json: Option<&str>) -> Result<T> {
    match options_json {
        None => Ok(T::default()),
        Some(raw) => serde_json::from_str(raw).map_err(invalid),
    }
}

fn permille(field: &str, value: u16) -> Result<Similarity> {
    if value > 1_000 {
        return Err(argument(&format!(
            "{field} is a permille value between 0 and 1000"
        )));
    }
    Ok(Similarity::from_permille(value))
}

fn mode(value: &str) -> Result<DetectionMode> {
    match value {
        "exact" => Ok(DetectionMode::Exact),
        "renamed" => Ok(DetectionMode::Renamed),
        "nearMiss" => Ok(DetectionMode::NearMiss),
        _ => Err(argument(&format!("unsupported detection mode: {value}"))),
    }
}

fn language(value: &str) -> Result<Language> {
    match value
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase()
        .as_str()
    {
        "rust" | "rs" => Ok(Language::Rust),
        "go" => Ok(Language::Go),
        "c" | "h" => Ok(Language::C),
        "cpp" | "c++" | "cc" | "cxx" | "hpp" => Ok(Language::Cpp),
        "bash" | "sh" | "zsh" => Ok(Language::Bash),
        "sql" | "psql" => Ok(Language::Sql),
        "javascript" | "js" | "jsx" | "mjs" | "cjs" => Ok(Language::JavaScript),
        "typescript" | "ts" | "tsx" | "mts" | "cts" => Ok(Language::TypeScript),
        "python" | "py" | "pyi" => Ok(Language::Python),
        "java" => Ok(Language::Java),
        "csharp" | "cs" => Ok(Language::CSharp),
        "markup" | "html" | "xml" | "vue" | "svelte" | "md" | "mdx" => Ok(Language::Markup),
        "text" => Ok(Language::Text),
        _ => Err(argument(&format!("unsupported language: {value}"))),
    }
}

pub(crate) fn argument(message: &str) -> Error {
    Error::new(Status::InvalidArg, message.to_owned())
}

pub(crate) fn clone_error(error: impl core::fmt::Display) -> Error {
    Error::new(Status::GenericFailure, error.to_string())
}

fn invalid(error: serde_json::Error) -> Error {
    Error::new(Status::InvalidArg, error.to_string())
}
