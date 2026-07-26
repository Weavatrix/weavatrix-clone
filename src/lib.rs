#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]

mod accuracy;
#[cfg(feature = "scan")]
mod block;
mod canonical;
mod cluster;
mod config;
mod detector;
mod error;
mod fast_hash;
mod fingerprint;
#[cfg(feature = "scan")]
mod fragment;
mod index;
mod model;
pub mod output;
#[cfg(feature = "scan")]
mod repository;
mod token;
mod verify;

pub use accuracy::{AccuracyCounts, AccuracyGate, AccuracyReport, OracleLocation, OraclePair};
pub use config::CloneConfig;
pub use detector::CloneDetector;
pub use error::{CloneError, Result};
pub use model::{
    CloneEvidence, CloneFamily, CloneKind, CloneLocation, ClonePair, CloneReport, CloneStatistics,
    DetectionMode, Language, Similarity, SourceFragment, SourceSpan,
};
#[cfg(feature = "scan")]
pub use repository::{RepositoryCloneDetector, RepositoryOptions};

/// Optional Type-4 candidate source implemented by a future vector package.
///
/// The deterministic Type-1/2/3 detector never calls this trait. Higher layers
/// may use it to propose semantic candidates and then apply their own evidence
/// and confidence policy.
pub trait SemanticCandidateProvider {
    /// Returns candidate fragment identifiers for one fragment.
    ///
    /// # Errors
    ///
    /// Provider failures remain explicit and never change deterministic clone
    /// output.
    fn candidates(&self, fragment: &SourceFragment, limit: usize) -> Result<Vec<String>>;
}
