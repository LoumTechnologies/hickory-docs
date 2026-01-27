use std::time::Duration;

use bench::{alpine_wasm_path, ConversionPrereqs};
use clap::Parser;
use container2wasm::cli::Cli;
use criterion::{criterion_group, criterion_main, Criterion, SamplingMode};

fn conversion_without_wizer(c: &mut Criterion) {
    let prereqs = ConversionPrereqs::check();
    if !prereqs.can_convert() {
        eprintln!(
            "bench [conversion/without-wizer]: SKIPPED — {}",
            prereqs.summary()
        );
        return;
    }

    let mut group = c.benchmark_group("conversion");
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(10);
    group.warm_up_time(Duration::from_nanos(1));

    group.bench_function("without-wizer", |b| {
        b.iter(|| {
            let dir = tempfile::tempdir().expect("failed to create tempdir");
            let output = dir.path().join("out.wasm");

            let cli = Cli::parse_from([
                "c2w",
                "--no-docker",
                "alpine:3.23.2",
                output.to_str().unwrap(),
            ]);
            container2wasm::run(cli).expect("conversion failed");
        });
    });

    group.finish();
}

fn conversion_with_wizer(c: &mut Criterion) {
    let prereqs = ConversionPrereqs::check();
    if !prereqs.can_convert_with_wizer() {
        eprintln!(
            "bench [conversion/with-wizer]: SKIPPED — {}",
            prereqs.summary()
        );
        return;
    }

    let mut group = c.benchmark_group("conversion");
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(10);
    group.warm_up_time(Duration::from_nanos(1));

    group.bench_function("with-wizer", |b| {
        b.iter(|| {
            let dir = tempfile::tempdir().expect("failed to create tempdir");
            let output = dir.path().join("out.wasm");

            let cli = Cli::parse_from([
                "c2w",
                "--no-docker",
                "--wizer",
                "alpine:3.23.2",
                output.to_str().unwrap(),
            ]);
            container2wasm::run(cli).expect("conversion with wizer failed");
        });
    });

    group.finish();
}

fn conversion_wizer_only(c: &mut Criterion) {
    let prereqs = ConversionPrereqs::check();
    if !prereqs.can_wizer_only() {
        eprintln!(
            "bench [conversion/wizer-only]: SKIPPED — {}",
            prereqs.summary()
        );
        return;
    }

    let input = match alpine_wasm_path() {
        Some(p) => p,
        None => {
            eprintln!(
                "bench [conversion/wizer-only]: SKIPPED — \
                 alpine fixture not found (run conversion benchmark first or unzip manually)"
            );
            return;
        }
    };

    // Output goes to the fixture directory so it can be reused by boot benchmarks
    let output = bench::fixture_dir().join("3.23.2-wizer.wasm");

    let mut group = c.benchmark_group("conversion");
    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(5);
    group.warm_up_time(Duration::from_nanos(1));

    group.bench_function("wizer-only", |b| {
        b.iter(|| {
            let cli = Cli::parse_from([
                "c2w",
                "--wizer-only",
                input.to_str().unwrap(),
                output.to_str().unwrap(),
            ]);
            container2wasm::run(cli).expect("wizer-only failed");
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    conversion_without_wizer,
    conversion_with_wizer,
    conversion_wizer_only
);
criterion_main!(benches);
