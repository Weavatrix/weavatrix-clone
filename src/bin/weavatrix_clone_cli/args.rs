use weavatrix_clone::{CloneConfig, DetectionMode, RepositoryOptions};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OutputKind {
    Text,
    Json,
    Sarif,
    BigCloneEval,
}

pub(crate) struct Arguments {
    pub root: String,
    pub summary_only: bool,
    pub min_tokens: usize,
    pub min_lines: usize,
    pub max_lines: usize,
    pub mode: DetectionMode,
    pub output: OutputKind,
    pub cross_extensions: bool,
    pub extensions: Option<Vec<String>>,
}

pub(crate) fn parse_arguments(
    arguments: impl IntoIterator<Item = String>,
) -> Result<Arguments, String> {
    let mut values = arguments.into_iter();
    let mut parsed = Arguments {
        root: ".".to_owned(),
        summary_only: false,
        min_tokens: CloneConfig::default().min_tokens,
        min_lines: RepositoryOptions::default().min_fragment_lines,
        max_lines: RepositoryOptions::default().max_fragment_lines,
        mode: DetectionMode::NearMiss,
        output: OutputKind::Text,
        cross_extensions: false,
        extensions: None,
    };
    let mut root_set = false;
    while let Some(argument) = values.next() {
        match argument.as_str() {
            "--summary" => parsed.summary_only = true,
            "--min-tokens" => parsed.min_tokens = next_usize(&mut values, "--min-tokens")?,
            "--min-lines" => parsed.min_lines = next_usize(&mut values, "--min-lines")?,
            "--max-lines" => parsed.max_lines = next_usize(&mut values, "--max-lines")?,
            "--mode" => parsed.mode = parse_mode(&next_value(&mut values, "--mode")?)?,
            "--output-format" => {
                parsed.output = parse_output(&next_value(&mut values, "--output-format")?)?;
            }
            "--cross-extensions" => parsed.cross_extensions = true,
            "--format" => {
                let value = next_value(&mut values, "--format")?;
                parsed.extensions = Some(value.split(',').map(str::to_owned).collect());
            }
            "--help" | "-h" => return Err(usage()),
            value if value.starts_with('-') => {
                return Err(format!("unknown option {value}\n{}", usage()));
            }
            value if !root_set => {
                value.clone_into(&mut parsed.root);
                root_set = true;
            }
            _ => return Err(format!("only one repository may be scanned\n{}", usage())),
        }
    }
    if parsed.summary_only && parsed.output != OutputKind::Text {
        return Err("--summary is only valid with text output".to_owned());
    }
    Ok(parsed)
}

fn parse_mode(value: &str) -> Result<DetectionMode, String> {
    match value {
        "exact" => Ok(DetectionMode::Exact),
        "renamed" => Ok(DetectionMode::Renamed),
        "near" => Ok(DetectionMode::NearMiss),
        _ => Err("--mode must be exact, renamed, or near".to_owned()),
    }
}

fn parse_output(value: &str) -> Result<OutputKind, String> {
    match value {
        "text" => Ok(OutputKind::Text),
        "json" => Ok(OutputKind::Json),
        "sarif" => Ok(OutputKind::Sarif),
        "bigcloneeval" => Ok(OutputKind::BigCloneEval),
        _ => Err("--output-format must be text, json, sarif, or bigcloneeval".to_owned()),
    }
}

fn next_usize(
    values: &mut impl Iterator<Item = String>,
    option: &'static str,
) -> Result<usize, String> {
    next_value(values, option)?
        .parse()
        .map_err(|_| format!("{option} requires a non-negative integer"))
}

fn next_value(
    values: &mut impl Iterator<Item = String>,
    option: &'static str,
) -> Result<String, String> {
    values
        .next()
        .ok_or_else(|| format!("{option} requires a value"))
}

fn usage() -> String {
    "usage: weavatrix-clone [OPTIONS] [repository]\n\
     options: --summary --mode exact|renamed|near --min-tokens N \
     --min-lines N --max-lines N --format rs,go,... --cross-extensions \
     --output-format text|json|sarif|bigcloneeval"
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::{OutputKind, parse_arguments};

    #[test]
    fn parses_thresholds_formats_and_output() {
        let parsed = parse_arguments(
            [
                "--min-tokens",
                "32",
                "--min-lines",
                "5",
                "--format",
                "rs,go",
                "--output-format",
                "sarif",
                "repo",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .expect("valid command line");
        assert_eq!(parsed.min_tokens, 32);
        assert_eq!(parsed.min_lines, 5);
        assert_eq!(parsed.output, OutputKind::Sarif);
        assert_eq!(
            parsed.extensions,
            Some(vec!["rs".to_owned(), "go".to_owned()])
        );
        assert_eq!(parsed.root, "repo");
    }

    #[test]
    fn rejects_invalid_or_conflicting_options() {
        assert!(parse_arguments(["--wat".to_owned()]).is_err());
        assert!(parse_arguments(["--min-lines".to_owned()]).is_err());
        assert!(parse_arguments(["--min-tokens".to_owned(), "x".to_owned()]).is_err());
        assert!(parse_arguments(["--mode".to_owned(), "semantic".to_owned()]).is_err());
        assert!(parse_arguments(["a".to_owned(), "b".to_owned()]).is_err());
        assert!(parse_arguments(["--help".to_owned()]).is_err());
        assert!(
            parse_arguments(
                ["--summary", "--output-format", "json"]
                    .into_iter()
                    .map(str::to_owned)
            )
            .is_err()
        );
    }
}
