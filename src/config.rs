use crate::error::{CloneError, Result};
use crate::model::{DetectionMode, Similarity};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloneConfig {
    pub mode: DetectionMode,
    pub min_tokens: usize,
    pub k_gram: usize,
    pub winnowing_window: usize,
    pub min_similarity: Similarity,
    pub candidate_similarity: Similarity,
    pub min_shared_fingerprints: usize,
    pub max_bucket_size: usize,
    pub max_fragments: usize,
    pub max_tokens_per_fragment: usize,
    pub max_candidates: usize,
    pub compare_overlapping_fragments: bool,
}

impl Default for CloneConfig {
    fn default() -> Self {
        Self {
            mode: DetectionMode::NearMiss,
            min_tokens: 24,
            k_gram: 8,
            winnowing_window: 4,
            min_similarity: Similarity::from_permille(800),
            candidate_similarity: Similarity::from_permille(450),
            min_shared_fingerprints: 2,
            max_bucket_size: 128,
            max_fragments: 1_000_000,
            max_tokens_per_fragment: 100_000,
            max_candidates: 5_000_000,
            compare_overlapping_fragments: false,
        }
    }
}

impl CloneConfig {
    /// Validates safety bounds and detection thresholds.
    ///
    /// # Errors
    ///
    /// Rejects zero bounds, impossible winnowing sizes, and a candidate
    /// threshold above the final verification threshold.
    pub fn validate(self) -> Result<Self> {
        if self.k_gram == 0 {
            return Err(invalid("k_gram", "must be greater than zero"));
        }
        if self.winnowing_window == 0 {
            return Err(invalid("winnowing_window", "must be greater than zero"));
        }
        let guaranteed = self
            .k_gram
            .checked_add(self.winnowing_window)
            .and_then(|value| value.checked_sub(1))
            .ok_or(CloneError::CapacityExceeded {
                resource: "winnowing guarantee",
                limit: usize::MAX,
            })?;
        if self.min_tokens < guaranteed {
            return Err(invalid(
                "min_tokens",
                "must cover at least one complete winnowing window",
            ));
        }
        if self.candidate_similarity > self.min_similarity {
            return Err(invalid(
                "candidate_similarity",
                "must not exceed min_similarity",
            ));
        }
        for (field, value) in [
            ("min_shared_fingerprints", self.min_shared_fingerprints),
            ("max_bucket_size", self.max_bucket_size),
            ("max_fragments", self.max_fragments),
            ("max_tokens_per_fragment", self.max_tokens_per_fragment),
            ("max_candidates", self.max_candidates),
        ] {
            if value == 0 {
                return Err(invalid(field, "must be greater than zero"));
            }
        }
        Ok(self)
    }
}

const fn invalid(field: &'static str, reason: &'static str) -> CloneError {
    CloneError::InvalidConfig { field, reason }
}
