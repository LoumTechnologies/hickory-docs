use std::sync::OnceLock;
use std::time::{Duration, Instant};

use bench::{
    alpine_wasm_path, alpine_wizer_wasm_path, load_module, shared_engine, wait_for_prompt,
};
use container::ContainerConfig;
use criterion::{criterion_group, criterion_main, Criterion, SamplingMode};
use wasmtime::Module;

// ---------------------------------------------------------------------------
// Module caching — compile once, reuse across all iterations
// ---------------------------------------------------------------------------

fn cached_alpine_module() -> Option<&'static Module> {
    static MODULE: OnceLock<Option<Module>> = OnceLock::new();
    MODULE
        .get_or_init(|| alpine_wasm_path().and_then(|p| load_module(&p)))
        .as_ref()
}

fn cached_alpine_wizer_module() -> Option<&'static Module> {
    static MODULE: OnceLock<Option<Module>> = OnceLock::new();
    MODULE
        .get_or_init(|| alpine_wizer_wasm_path().and_then(|p| load_module(&p)))
        .as_ref()
}

// ---------------------------------------------------------------------------
// Boot benchmarks — time from start to first shell prompt
// ---------------------------------------------------------------------------

fn container_boot(c: &mut Criterion) {
    let engine = shared_engine();

    let mut group = c.benchmark_group("container-boot");
    group.sampling_mode(SamplingMode::Flat);
    group.warm_up_time(Duration::from_nanos(1));

    if let Some(module) = cached_alpine_module() {
        group.sample_size(10);
        group.bench_function("without-wizer", |b| {
            let engine = engine.clone();
            let module = module.clone();
            b.iter_custom(|iters| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    total += rt.block_on(async {
                        let start = Instant::now();
                        let config = ContainerConfig::new(engine.clone(), module.clone());
                        let mut c = container::Container::start(config)
                            .await
                            .expect("failed to start container");
                        wait_for_prompt(&mut c, Duration::from_secs(180))
                            .await
                            .expect("prompt not reached");
                        let elapsed = start.elapsed();
                        c.kill();
                        elapsed
                    });
                }
                total
            });
        });
    } else {
        eprintln!(
            "bench [container-boot/without-wizer]: SKIPPED — \
             alpine WASM fixture not found"
        );
    }

    if let Some(module) = cached_alpine_wizer_module() {
        group.sample_size(10);
        group.bench_function("with-wizer", |b| {
            let engine = engine.clone();
            let module = module.clone();
            b.iter_custom(|iters| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    total += rt.block_on(async {
                        let start = Instant::now();
                        let config = ContainerConfig::new(engine.clone(), module.clone());
                        let mut c = container::Container::start(config)
                            .await
                            .expect("failed to start container");
                        wait_for_prompt(&mut c, Duration::from_secs(180))
                            .await
                            .expect("prompt not reached");
                        let elapsed = start.elapsed();
                        c.kill();
                        elapsed
                    });
                }
                total
            });
        });
    } else {
        eprintln!(
            "bench [container-boot/with-wizer]: SKIPPED — \
             wizer'd alpine fixture not found (generate with: \
             cargo bench -p bench --bench conversion -- 'wizer-only')"
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Run-command benchmarks — time from start through prompt + echo hello
// ---------------------------------------------------------------------------

fn container_run_command(c: &mut Criterion) {
    let engine = shared_engine();

    let mut group = c.benchmark_group("container-run-command");
    group.sampling_mode(SamplingMode::Flat);
    group.warm_up_time(Duration::from_nanos(1));

    if let Some(module) = cached_alpine_module() {
        group.sample_size(10);
        group.bench_function("without-wizer", |b| {
            let engine = engine.clone();
            let module = module.clone();
            b.iter_custom(|iters| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    total += rt.block_on(async {
                        let start = Instant::now();
                        let config = ContainerConfig::new(engine.clone(), module.clone());
                        let mut c = container::Container::start(config)
                            .await
                            .expect("failed to start container");
                        wait_for_prompt(&mut c, Duration::from_secs(180))
                            .await
                            .expect("prompt not reached");
                        bench::run_command(&mut c, "echo hello")
                            .await
                            .expect("command failed");
                        let elapsed = start.elapsed();
                        c.kill();
                        elapsed
                    });
                }
                total
            });
        });
    } else {
        eprintln!(
            "bench [container-run-command/without-wizer]: SKIPPED — \
             alpine WASM fixture not found"
        );
    }

    if let Some(module) = cached_alpine_wizer_module() {
        group.sample_size(10);
        group.bench_function("with-wizer", |b| {
            let engine = engine.clone();
            let module = module.clone();
            b.iter_custom(|iters| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    total += rt.block_on(async {
                        let start = Instant::now();
                        let config = ContainerConfig::new(engine.clone(), module.clone());
                        let mut c = container::Container::start(config)
                            .await
                            .expect("failed to start container");
                        wait_for_prompt(&mut c, Duration::from_secs(180))
                            .await
                            .expect("prompt not reached");
                        bench::run_command(&mut c, "echo hello")
                            .await
                            .expect("command failed");
                        let elapsed = start.elapsed();
                        c.kill();
                        elapsed
                    });
                }
                total
            });
        });
    } else {
        eprintln!(
            "bench [container-run-command/with-wizer]: SKIPPED — \
             wizer'd alpine fixture not found (generate with: \
             cargo bench -p bench --bench conversion -- 'wizer-only')"
        );
    }

    group.finish();
}

criterion_group!(benches, container_boot, container_run_command);
criterion_main!(benches);
