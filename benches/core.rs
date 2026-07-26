use std::{
    hint::black_box,
    time::{Duration, Instant},
};
use weavatrix_clone::{CloneDetector, Language, SourceFragment, SourceSpan};

fn main() {
    let fragments = std::env::var("WEAVATRIX_CLONE_BENCH_FRAGMENTS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(10_000);
    let sources = fixture(fragments);
    let mut verified = 0;
    for _ in 0..2 {
        let report = CloneDetector::default().detect(&sources).unwrap();
        verified = report.pairs.len();
        black_box(report);
    }
    let mut samples = (0..9)
        .map(|_| {
            let started = Instant::now();
            let report = CloneDetector::default().detect(&sources).unwrap();
            black_box(report);
            started.elapsed()
        })
        .collect::<Vec<_>>();
    samples.sort_unstable();
    report(
        "clone_core",
        fragments,
        verified,
        samples[samples.len() / 2],
    );
}

fn fixture(count: usize) -> Vec<SourceFragment> {
    (0..count)
        .map(|index| {
            let family = index / 2;
            let variable = if index % 2 == 0 { "value" } else { "item" };
            let text = format!(
                "fn calculate_{family}({variable}: i64) -> i64 {{\n\
                 let baseline = {variable} * 31;\n\
                 let adjusted = baseline + {family};\n\
                 if adjusted > 100 {{\n\
                 return adjusted / 2;\n\
                 }}\n\
                 let fallback = adjusted * adjusted;\n\
                 fallback + baseline\n\
                 }}\n"
            );
            SourceFragment::new(
                format!("fragment:{index}"),
                format!("src/file_{index}.rs"),
                Language::Rust,
                SourceSpan {
                    start_byte: 0,
                    end_byte: text.len(),
                    start_line: 1,
                    end_line: 9,
                },
                text,
            )
            .unwrap()
        })
        .collect()
}

fn report(name: &str, fragments: usize, verified: usize, median: Duration) {
    let rate = u128::try_from(fragments).expect("usize fits u128") * 1_000_000_000
        / median.as_nanos().max(1);
    println!(
        "{name} fragments={fragments} verified_pairs={verified} median_ms={:.3} fragments_per_second={rate}",
        median.as_secs_f64() * 1_000.0
    );
}
