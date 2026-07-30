use crate::block::{BlockDetection, BlockSource, detect_blocks};
use crate::canonical::suppress_contained;
use crate::cluster::families_for_pairs;
use crate::detector::CloneDetector;
use crate::error::{CloneError, Result};
use crate::fragment::fragment_file;
use crate::model::{CloneReport, DetectionMode, Language, SourceFragment};
use std::sync::{Arc, Mutex};
use weavatrix_scan::{
    ContentDiscoveryMode, ContentFileStatus, ContentValidationPolicy, ContentVisitControl,
    ContentVisitEvent, ScanOptions, Scanner,
};

const DEFAULT_EXTENSIONS: &[&str] = &[
    "rs", "go", "c", "h", "cc", "cpp", "cxx", "hh", "hpp", "hxx", "sh", "bash", "zsh", "sql",
    "psql", "js", "jsx", "mjs", "cjs", "ts", "tsx", "mts", "cts", "py", "pyi", "java", "cs",
    "html", "htm", "xml", "vue", "svelte", "md", "mdx",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryOptions {
    pub max_file_bytes: u64,
    pub min_fragment_lines: usize,
    pub max_fragment_lines: usize,
    pub parallelism: usize,
    pub cross_extensions: bool,
    pub extensions: Vec<String>,
}

impl Default for RepositoryOptions {
    fn default() -> Self {
        Self {
            max_file_bytes: 1_500_000,
            min_fragment_lines: 3,
            max_fragment_lines: 400,
            parallelism: 0,
            cross_extensions: false,
            extensions: DEFAULT_EXTENSIONS
                .iter()
                .map(|value| (*value).to_owned())
                .collect(),
        }
    }
}

impl RepositoryOptions {
    #[must_use]
    pub fn with_extensions<I, S>(mut self, extensions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.extensions = extensions
            .into_iter()
            .map(|value| value.as_ref().trim_start_matches('.').to_ascii_lowercase())
            .filter(|value| !value.is_empty())
            .collect();
        self
    }
}

#[derive(Debug, Clone)]
pub struct RepositoryCloneDetector {
    detector: CloneDetector,
    options: RepositoryOptions,
}

impl RepositoryCloneDetector {
    #[must_use]
    pub fn new(detector: CloneDetector) -> Self {
        Self {
            detector,
            options: RepositoryOptions::default(),
        }
    }

    #[must_use]
    pub fn options(mut self, options: RepositoryOptions) -> Self {
        self.options = options;
        self
    }

    /// Scans a repository once, fragments selected source files, and detects
    /// deterministic clone families.
    ///
    /// # Errors
    ///
    /// Rejects incomplete scans, invalid UTF-8 source, fragment limits, and
    /// clone-core validation failures without returning partial output.
    pub fn detect(&self, root: impl Into<std::path::PathBuf>) -> Result<CloneReport> {
        self.validate_options()?;
        let files = Arc::new(Mutex::new(Vec::<CollectedFile>::new()));
        let sink = Arc::clone(&files);
        let mut scan_options = ScanOptions::default()
            .with_extensions(&self.options.extensions)
            .with_content_discovery(ContentDiscoveryMode::BufferedParallel)
            .with_content_validation(ContentValidationPolicy::Strict)
            .with_parallelism(self.options.parallelism)
            .selected_files_only();
        scan_options.max_file_bytes = self.options.max_file_bytes;
        let summary = Scanner::new(root)
            .options(scan_options)
            .visit_content(move |_worker| {
                let sink = Arc::clone(&sink);
                let mut current = None::<CollectedFile>;
                move |event| {
                    match event {
                        ContentVisitEvent::FileStart { file, .. } => {
                            current = Some(CollectedFile {
                                path: file.relative.to_owned(),
                                bytes: Vec::with_capacity(usize::try_from(file.bytes).unwrap_or(0)),
                            });
                        }
                        ContentVisitEvent::Chunk { bytes, .. } => {
                            if let Some(file) = &mut current {
                                file.bytes.extend_from_slice(bytes);
                            }
                        }
                        ContentVisitEvent::FileEnd {
                            status: ContentFileStatus::Selected,
                            ..
                        } => {
                            if let Some(file) = current.take() {
                                sink.lock()
                                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                                    .push(file);
                            }
                        }
                        ContentVisitEvent::FileEnd { .. } => current = None,
                    }
                    ContentVisitControl::Continue
                }
            })
            .map_err(|error| CloneError::Repository(error.to_string()))?;
        if !summary.complete || summary.stopped {
            return Err(CloneError::Repository(
                "scan did not produce a complete repository view".to_owned(),
            ));
        }
        self.build_report(&files)
    }

    fn validate_options(&self) -> Result<()> {
        if self.options.max_file_bytes == 0
            || self.options.min_fragment_lines == 0
            || self.options.max_fragment_lines < self.options.min_fragment_lines
            || self.options.extensions.is_empty()
        {
            return Err(CloneError::Repository(
                "invalid repository fragment or byte limits".to_owned(),
            ));
        }
        Ok(())
    }

    fn build_report(&self, files: &Mutex<Vec<CollectedFile>>) -> Result<CloneReport> {
        let mut files = {
            let mut collected = files
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            std::mem::take(&mut *collected)
        };
        files.sort_unstable_by(|left, right| left.path.cmp(&right.path));
        let source_files = files.len();
        let mut fragments = Vec::<SourceFragment>::new();
        let mut block_sources = Vec::<BlockSource>::with_capacity(files.len());
        for file in files {
            let language = Language::from_path(&file.path);
            let text = String::from_utf8(file.bytes).map_err(|_| {
                CloneError::Repository(format!("selected source is not UTF-8: {}", file.path))
            })?;
            if self.detector.config().mode != DetectionMode::Exact {
                fragments.extend(fragment_file(
                    &file.path,
                    language,
                    &text,
                    self.options.min_fragment_lines,
                    self.options.max_fragment_lines,
                )?);
            }
            block_sources.push(BlockSource {
                path: file.path,
                language,
                text,
            });
        }
        let mut report = self.detector.detect(&fragments)?;
        let blocks = self.detect_blocks(block_sources)?;
        report.pairs.extend(blocks.pairs);
        report.pairs = suppress_contained(report.pairs);
        report.statistics.source_files = source_files;
        report.statistics.source_tokens = blocks.tokens;
        report.statistics.exact_block_candidates = blocks.candidates;
        report.statistics.candidate_pairs = report
            .statistics
            .candidate_pairs
            .saturating_add(blocks.candidates);
        report.statistics.suppressed_exact_buckets = blocks.suppressed_buckets;
        report.statistics.verified_pairs = report.pairs.len();
        report.families = families_for_pairs(&report.pairs);
        Ok(report)
    }

    fn detect_blocks(&self, sources: Vec<BlockSource>) -> Result<BlockDetection> {
        if self.options.cross_extensions || sources.len() < 2 {
            return detect_blocks(
                &sources,
                self.detector.config(),
                self.options.min_fragment_lines,
                self.options.parallelism,
            );
        }
        let mut by_extension = std::collections::BTreeMap::<String, Vec<BlockSource>>::new();
        for source in sources {
            let extension = std::path::Path::new(&source.path)
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            by_extension.entry(extension).or_default().push(source);
        }
        let groups = by_extension.into_values().collect::<Vec<_>>();
        let available = std::thread::available_parallelism().map_or(1, usize::from);
        let workers = if self.options.parallelism == 0 {
            available
        } else {
            self.options.parallelism
        }
        .clamp(1, groups.len());
        let lexer_workers = if self.options.parallelism == 0 {
            0
        } else {
            self.options.parallelism.div_ceil(workers)
        };
        let mut assignments = (0..workers)
            .map(|_| Vec::<Vec<BlockSource>>::new())
            .collect::<Vec<_>>();
        for (index, group) in groups.into_iter().enumerate() {
            assignments[index % workers].push(group);
        }
        std::thread::scope(|scope| {
            let handles = assignments
                .into_iter()
                .map(|assignment| {
                    scope.spawn(move || {
                        let mut combined = BlockDetection::default();
                        for group in assignment {
                            combined.merge(detect_blocks(
                                &group,
                                self.detector.config(),
                                self.options.min_fragment_lines,
                                lexer_workers,
                            )?);
                        }
                        Ok::<_, CloneError>(combined)
                    })
                })
                .collect::<Vec<_>>();
            let mut combined = BlockDetection::default();
            for handle in handles {
                combined.merge(
                    handle.join().map_err(|_| {
                        CloneError::Repository("block worker panicked".to_owned())
                    })??,
                );
            }
            Ok(combined)
        })
    }
}

struct CollectedFile {
    path: String,
    bytes: Vec<u8>,
}
