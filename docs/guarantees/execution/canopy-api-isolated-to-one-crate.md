# Only `hickory-executor-canopy` Knows Cloud Canopy's API

Given that Cloud Canopy's API is under concurrent development, when any code
in this repository talks to canopy (gRPC, sandbox channel protocol, image
path mapping), then that code lives in `crates/hickory-executor-canopy`
and nowhere else; all other code depends only on the `Executor` trait in
`crates/hickory-executor`.

---

Last LLM verification:
- Date: 2026-08-05
- Reviewer: Claude (Fable 5)
- Result: verified
- Evidence: the entire coupling is the vendored
  `crates/hickory-executor-canopy/proto/canopy.proto` (source commit
  recorded in its header); the crate has no path/git dependency on any
  cloud-canopy crate (`crates/hickory-executor-canopy/Cargo.toml`). The
  adapter (`src/executor.rs`, `src/frame.rs`, `src/config.rs`) is the only
  code speaking gRPC/`AttachSandbox`/the guest pty protocol or reading
  `CANOPY_*` env. `hickory-cli` constructs `CanopyExecutor` behind
  `Arc<dyn Executor>` (`crates/hickory-cli/src/lib.rs`,
  `ExecutorChoice::build`) and knows nothing else about canopy.
- Test coverage: `crates/hickory-executor-canopy/tests/isolation.rs` scans
  every Rust file in the repo for canopy API tokens (proto package, RPC
  names, metadata key, sentinels, `CANOPY_*` env names) outside the adapter
  crate, and restricts dependents/importers of the adapter to an explicit
  allowlist (currently `hickory-cli`). The bare word "canopy" stays legal
  outside (executor selection and docs name the backend without knowing its
  API). Contract fidelity is covered by `tests/contract.rs` (mock
  `CanopyAgent` gRPC server + fake pty guest, full execute round-trip).
- Caveats: the grep rule is token-based; a leak that renames every API
  identifier would evade it (LLM review still applies on changes touching
  canopy).
