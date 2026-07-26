//! Stable, dependency-free report encoders.

mod bigcloneeval;
mod encode;
mod json;
mod sarif;

pub use bigcloneeval::to_bigcloneeval;
pub use json::{JSON_SCHEMA, JSON_SCHEMA_DOCUMENT, to_json};
pub use sarif::to_sarif;
