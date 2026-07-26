use std::{
    fs,
    hint::black_box,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime},
};
use weavatrix_clone::{
    CloneConfig, CloneDetector, DetectionMode, RepositoryCloneDetector, RepositoryOptions,
};

fn main() {
    let files = std::env::var("WEAVATRIX_CLONE_BENCH_TS_FILES")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1_024);
    let corpus = Corpus::new(files);
    let detector = RepositoryCloneDetector::new(
        CloneDetector::new(CloneConfig {
            mode: DetectionMode::Exact,
            min_tokens: 24,
            ..CloneConfig::default()
        })
        .unwrap(),
    )
    .options(RepositoryOptions::default().with_extensions(["ts", "tsx"]));
    let mut verified = 0;
    for _ in 0..2 {
        let report = detector.detect(corpus.path()).unwrap();
        verified = report.pairs.len();
        black_box(report);
    }
    let mut samples = (0..9)
        .map(|_| {
            let started = Instant::now();
            black_box(detector.detect(corpus.path()).unwrap());
            started.elapsed()
        })
        .collect::<Vec<_>>();
    samples.sort_unstable();
    report(files, verified, samples[samples.len() / 2]);
}

struct Corpus(PathBuf);

impl Corpus {
    fn new(files: usize) -> Self {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "weavatrix-clone-ts-bench-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        for index in 0..files {
            let family = index / 2;
            let extension = if family % 2 == 0 { "ts" } else { "tsx" };
            fs::write(
                path.join(format!("fixture-{index}.{extension}")),
                source(family),
            )
            .unwrap();
        }
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Corpus {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn source(family: usize) -> String {
    format!(
        "export function calculate{family}(values: readonly number[]): number {{\n\
         let total = {family};\n\
         for (const value of values) {{\n\
         const scaled = value * 31;\n\
         total += scaled > 100 ? scaled / 2 : scaled;\n\
         }}\n\
         return total + {family};\n\
         }}\n"
    )
}

fn report(files: usize, verified: usize, median: Duration) {
    let rate =
        u128::try_from(files).expect("usize fits u128") * 1_000_000_000 / median.as_nanos().max(1);
    println!(
        "typescript_repository files={files} verified_pairs={verified} median_ms={:.3} files_per_second={rate}",
        median.as_secs_f64() * 1_000.0
    );
}
