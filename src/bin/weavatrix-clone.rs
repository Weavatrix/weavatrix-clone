#[path = "weavatrix_clone_cli/args.rs"]
mod args;
#[path = "weavatrix_clone_cli/render.rs"]
mod render;

use args::parse_arguments;
use render::render;
use std::{env, process::ExitCode, time::Instant};
use weavatrix_clone::{CloneConfig, CloneDetector, RepositoryCloneDetector, RepositoryOptions};

fn main() -> ExitCode {
    match run() {
        Ok(output) => {
            print!("{output}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("weavatrix-clone: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    run_with(env::args().skip(1))
}

fn run_with(arguments: impl IntoIterator<Item = String>) -> Result<String, String> {
    let arguments = parse_arguments(arguments)?;
    let config = CloneConfig {
        min_tokens: arguments.min_tokens,
        mode: arguments.mode,
        ..CloneConfig::default()
    };
    let mut repository = RepositoryOptions {
        min_fragment_lines: arguments.min_lines,
        max_fragment_lines: arguments.max_lines,
        cross_extensions: arguments.cross_extensions,
        ..RepositoryOptions::default()
    };
    if let Some(extensions) = arguments.extensions {
        repository = repository.with_extensions(extensions);
    }
    let started = Instant::now();
    let report = RepositoryCloneDetector::new(
        CloneDetector::new(config).map_err(|error| error.to_string())?,
    )
    .options(repository)
    .detect(&arguments.root)
    .map_err(|error| error.to_string())?;
    render(
        &report,
        arguments.output,
        arguments.summary_only,
        started.elapsed(),
    )
}

#[cfg(test)]
mod tests {
    use super::run_with;

    #[test]
    fn executes_exact_summary_on_the_checked_in_oracle() {
        let corpus = format!(
            "{}/benchmarks/corpus/rust",
            env!("CARGO_MANIFEST_DIR").replace('\\', "/")
        );
        let output = run_with(
            [
                "--summary",
                "--mode",
                "exact",
                "--min-tokens",
                "24",
                "--min-lines",
                "3",
                "--max-lines",
                "400",
                "--format",
                "rs",
                &corpus,
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .expect("oracle run");
        assert!(output.contains("type1="));
    }
}
