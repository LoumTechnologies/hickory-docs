# bench

Shared benchmark harness library for WASM container performance measurements.

Provides fixture path resolution for pre-converted Alpine WASM images (including
automatic `.zip` extraction), a `ConversionPrereqs` struct that checks for
required external tools (`mkisofs`, `wasi-vfs`, `wizer`), and a singleton
`shared_engine()` for a reused Wasmtime engine across benchmark iterations.

Also exposes async container helpers (`wait_for_prompt`, `run_command`) for
driving interactive shell sessions during benchmarks.

Backs two Criterion benchmark suites:
- `conversion` — image-to-WASM conversion throughput
- `container_boot` — cold-start latency
