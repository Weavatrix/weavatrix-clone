use crate::error::{CloneError, Result};
use crate::model::{CloneLocation, CloneReport};
use std::fmt::Write;

/// Encodes clone pairs in `BigCloneEval`'s eight-column import format.
///
/// # Errors
///
/// Rejects paths that cannot be reduced to one of `BigCloneBench`'s `default`,
/// `selected`, or `sample` source directories and a comma-free file name.
pub fn to_bigcloneeval(report: &CloneReport) -> Result<String> {
    let mut pairs = report.pairs.iter().collect::<Vec<_>>();
    pairs.sort_unstable_by(|left, right| left.id.cmp(&right.id));
    let mut output = String::new();
    for pair in pairs {
        let left = fields(&pair.left)?;
        let right = fields(&pair.right)?;
        let _ = writeln!(
            output,
            "{},{},{},{},{},{},{},{}",
            left.0,
            left.1,
            pair.left.span.start_line,
            pair.left.span.end_line,
            right.0,
            right.1,
            pair.right.span.start_line,
            pair.right.span.end_line
        );
    }
    Ok(output)
}

fn fields(location: &CloneLocation) -> Result<(&str, &str)> {
    if location.path.contains(',') {
        return Err(CloneError::InvalidOutput(format!(
            "BigCloneEval paths must not contain commas: {}",
            location.path
        )));
    }
    let parts = location.path.split(['/', '\\']).collect::<Vec<_>>();
    let Some((index, category)) = parts
        .iter()
        .enumerate()
        .rev()
        .find(|(_, value)| matches!(**value, "default" | "selected" | "sample"))
    else {
        return Err(CloneError::InvalidOutput(format!(
            "BigCloneEval path has no dataset category: {}",
            location.path
        )));
    };
    let Some(file) = parts.get(index + 1) else {
        return Err(CloneError::InvalidOutput(format!(
            "BigCloneEval path has no file name: {}",
            location.path
        )));
    };
    if index + 2 != parts.len() || file.is_empty() {
        return Err(CloneError::InvalidOutput(format!(
            "BigCloneEval expects a flat category directory: {}",
            location.path
        )));
    }
    Ok((category, file))
}
