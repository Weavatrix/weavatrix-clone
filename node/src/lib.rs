#![deny(unsafe_op_in_unsafe_fn)]

mod options;

use napi::bindgen_prelude::AsyncTask;
use napi::{Env, Result, Task};
use napi_derive::napi;
use options::{argument, clone_error, decode_detector, decode_fragments, decode_repository};
use std::path::PathBuf;
use weavatrix_clone::output::{
    JSON_SCHEMA, JSON_SCHEMA_DOCUMENT, to_bigcloneeval, to_json, to_sarif,
};
use weavatrix_clone::{CloneDetector, CloneReport, RepositoryCloneDetector, RepositoryOptions};

/// One completed detection, kept in Rust so every encoder sees the same
/// deterministic report.
#[napi]
pub struct NativeCloneReport {
    report: CloneReport,
}

#[napi]
impl NativeCloneReport {
    #[napi(getter)]
    pub fn pair_count(&self) -> Result<u32> {
        u32::try_from(self.report.pairs.len()).map_err(clone_error)
    }

    #[napi(getter)]
    pub fn family_count(&self) -> Result<u32> {
        u32::try_from(self.report.families.len()).map_err(clone_error)
    }

    /// Returns the stable `clone-report/v1` JSON document.
    #[napi]
    pub fn json(&self) -> String {
        to_json(&self.report)
    }

    /// Returns a SARIF 2.1.0 run for code-scanning consumers.
    #[napi]
    pub fn sarif(&self) -> String {
        to_sarif(&self.report)
    }

    /// Returns the `BigCloneEval` pair export.
    #[napi]
    pub fn big_clone_eval(&self) -> Result<String> {
        to_bigcloneeval(&self.report).map_err(clone_error)
    }
}

/// The identifier of the report schema `json()` emits.
#[napi]
pub fn json_schema_id() -> String {
    JSON_SCHEMA.to_owned()
}

/// The JSON Schema document describing `json()`.
#[napi]
pub fn json_schema() -> String {
    JSON_SCHEMA_DOCUMENT.to_owned()
}

pub struct DetectRepositoryTask {
    request: Option<(PathBuf, CloneDetector, RepositoryOptions)>,
}

impl Task for DetectRepositoryTask {
    type Output = NativeCloneReport;
    type JsValue = NativeCloneReport;

    fn compute(&mut self) -> Result<Self::Output> {
        let (root, detector, options) = self
            .request
            .take()
            .ok_or_else(|| argument("detection task was already executed"))?;
        detect(root, detector, options)
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output)
    }
}

/// Scans a repository once and detects clone families off the event loop.
#[napi]
pub fn detect_repository(
    root: String,
    options_json: Option<String>,
) -> Result<AsyncTask<DetectRepositoryTask>> {
    let detector = decode_detector(options_json.as_deref())?;
    let options = decode_repository(options_json.as_deref())?;
    Ok(AsyncTask::new(DetectRepositoryTask {
        request: Some((PathBuf::from(root), detector, options)),
    }))
}

/// Scans a repository once and detects clone families on the calling thread.
#[napi]
pub fn detect_repository_sync(
    root: String,
    options_json: Option<String>,
) -> Result<NativeCloneReport> {
    let detector = decode_detector(options_json.as_deref())?;
    let options = decode_repository(options_json.as_deref())?;
    detect(PathBuf::from(root), detector, options)
}

/// Detects clones over fragments the caller supplies; no filesystem access.
#[napi]
pub fn detect_fragments(
    fragments_json: String,
    options_json: Option<String>,
) -> Result<NativeCloneReport> {
    let detector = decode_detector(options_json.as_deref())?;
    let fragments = decode_fragments(&fragments_json)?;
    Ok(NativeCloneReport {
        report: detector.detect(&fragments).map_err(clone_error)?,
    })
}

fn detect(
    root: PathBuf,
    detector: CloneDetector,
    options: RepositoryOptions,
) -> Result<NativeCloneReport> {
    let report = RepositoryCloneDetector::new(detector)
        .options(options)
        .detect(root)
        .map_err(clone_error)?;
    Ok(NativeCloneReport { report })
}
