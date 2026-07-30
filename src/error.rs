use std::{error::Error, fmt};

pub type Result<T> = std::result::Result<T, CloneError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloneError {
    InvalidConfig {
        field: &'static str,
        reason: &'static str,
    },
    InvalidFragment {
        id: String,
        reason: &'static str,
    },
    DuplicateFragment(String),
    CapacityExceeded {
        resource: &'static str,
        limit: usize,
    },
    InvalidOutput(String),
    AccuracyGate {
        metric: &'static str,
        actual: u16,
        required: u16,
    },
    Repository(String),
}

impl fmt::Display for CloneError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig { field, reason } => {
                write!(formatter, "invalid clone config {field}: {reason}")
            }
            Self::InvalidFragment { id, reason } => {
                write!(formatter, "invalid fragment {id}: {reason}")
            }
            Self::DuplicateFragment(id) => write!(formatter, "duplicate fragment id: {id}"),
            Self::CapacityExceeded { resource, limit } => {
                write!(formatter, "{resource} exceeds configured limit {limit}")
            }
            Self::InvalidOutput(message) => write!(formatter, "invalid output: {message}"),
            Self::AccuracyGate {
                metric,
                actual,
                required,
            } => write!(
                formatter,
                "accuracy gate failed for {metric}: {actual} permille is below {required}"
            ),
            Self::Repository(message) => write!(formatter, "repository scan failed: {message}"),
        }
    }
}

impl Error for CloneError {}

#[cfg(test)]
mod tests {
    use crate::error::CloneError;

    #[test]
    fn formats_every_error_variant() {
        let errors = [
            CloneError::InvalidConfig {
                field: "limit",
                reason: "zero",
            },
            CloneError::InvalidFragment {
                id: "fragment".to_owned(),
                reason: "empty",
            },
            CloneError::DuplicateFragment("fragment".to_owned()),
            CloneError::CapacityExceeded {
                resource: "tokens",
                limit: 1,
            },
            CloneError::InvalidOutput("path".to_owned()),
            CloneError::AccuracyGate {
                metric: "recall",
                actual: 800,
                required: 900,
            },
            CloneError::Repository("incomplete".to_owned()),
        ];
        for error in errors {
            assert!(!error.to_string().is_empty());
            let as_error: &dyn std::error::Error = &error;
            assert!(as_error.source().is_none());
        }
    }
}
