# Vendoring notes (Phase A)

All crates under `crates/` were vendored on 2026-08-05 from the
`hickory-docs-task-63` monorepo checkout at
`/home/loumtech/Documents/src/hickory-docs-task-63`. That checkout is a **git
worktree** (branch `task-63` of the pre-reset `hickory-docs` repository, now at
`hickory-docs.old`); its worktree metadata had been pruned, so the commit was
recovered from `hickory-docs.old/.git/worktrees/hickory-docs-task-63/HEAD`:

- **Monorepo source commit (all vendored crates):**
  `e384c5eb1cf7f5a85b7daa364df587703d38c16e` (branch `task-63`)

The monorepo was the source of truth for every crate, chosen for internal
consistency. The standalone upstream repos existed at these commits at
vendoring time (frozen references only — fixes happen here, not there):

| Repo | HEAD at vendoring time |
|---|---|
| `hick-lang` | `a543ba5b805e15277d4efdbee7e25d85ad505639` |
| `hick-exec` | `e2c64cf7c8bf6129bc61571781e9a512078356a1` |
| `hick-security` | `2756c4fcb3c5e7f2820176221d53cb7f0cd51c5a` |
| `hick-store` | `cdd311ec85bf02a104d6b1539b507999b2092e33` |
| `hick-reactive` | not a git repository (no hash available) |

## Crate → monorepo path map

| Crate | Monorepo path |
|---|---|
| hick-lang | `hick-lang/crates/hick-lang` |
| hick-condition | `hick-exec/crates/hick-condition` |
| hick-flow | `hick-reactive/crates/hick-flow` |
| hick-exec | `hick-exec/crates/hick-exec` |
| hick-grove | `hick-reactive/crates/hick-grove` |
| hick-store | `hick-store/crates/hick-store` |
| hick-merge | `hick-store/crates/hick-merge` |
| hick-token | `hick-security/crates/hick-token` |
| hick-classify | `hick-security/crates/hick-classify` |
| hick-sink | `hick-security/crates/hick-sink` |
| hick-secrets | `hick-security/crates/hick-secrets` |
| hick-case | `crates/hick-case` |
| hick-feature | `crates/hick-feature` |
| hick-xml | `crates/hick-xml` |
| hick-handlers | `crates/hick-handlers` |
| hick-literate | `crates/hick-literate` (NOT a workspace member — Phase B) |

## The hick-lang version decision

The standalone `hick-lang` repo gained XML comment support in commit
`ac52ad2`. Checked: the monorepo copy of `hick-lang/src/lib.rs` **already
contains** that XML comment parsing, and is additionally a superset of the
standalone `lib.rs` (it adds `SessionDocument` / `SessionNode` / session
parsing, +340 lines). The monorepo copy was therefore used unchanged. The only
standalone-repo commit newer than the vendored code is `a543ba5` ("Preserve
child diagnostics across document re-parses"), which touches LSP code that was
not vendored.

## Deviations from the monorepo sources

Per-crate code is byte-identical to the monorepo except:

1. **`crates/hick-exec/Cargo.toml`** — path deps rewritten to siblings:
   `../../../hick-reactive/crates/hick-flow` → `../hick-flow`,
   `../../../hick-lang/crates/hick-lang` → `../hick-lang`.
2. **`crates/hick-flow/Cargo.toml`** — path dep rewritten:
   `../../../hick-lang/crates/hick-lang` → `../hick-lang`.
3. All other intra-workspace path deps were already sibling-relative
   (`../hick-classify`, `../hick-store`, etc.) and copied unchanged.
4. **No tests were deleted or `#[ignore]`d** — none of the vendored crates'
   tests depend on wasm container images (verified by grep across all vendored
   `src/` and `tests/` trees).
5. **`crates/hick-literate`** is copied verbatim but is not a workspace member
   (listed under a `# Phase B` comment in the root `Cargo.toml`). Its manifest
   still references wasm/container workspace deps (`hick-container`,
   `wasmtime`, `wasm-net` crates, `hick-api`, `hick-shell`, `hick-live`);
   these are inert because cargo never resolves a non-member. Phase B replaces
   them during the executor surgery.

## New files (not from the monorepo)

- Root `Cargo.toml`: workspace resolver 3, `[workspace.package]` edition 2024
  / publish false, explicit member list, and only the two
  `[workspace.dependencies]` entries the vendored members actually consume
  (`hick-lang`, `hick-exec` — used via `.workspace = true` in
  `crates/hick-handlers/Cargo.toml`). None of the monorepo's wasm-related
  workspace deps (wasmtime, hick-container, wasm-net, native2wasm/c2w) were
  ported.
- `Cargo.lock`: freshly generated. It contains the ubiquitous `wasi` /
  `wasip2` crates only as target-gated transitive deps of `getrandom`/`mio`
  for `wasm32-*` targets; `cargo tree -i wasi` confirms nothing depends on
  them on native targets and they are never compiled here.

## Feature policy

- `hick-grove`'s `sqlite-example` feature (bundled rusqlite) and
  `hick-classify`'s `host-mask` feature (arrow/parquet/blake3) are kept but
  remain off by default, as upstream.
- `hick-store`'s optional `s3` feature (aws-sdk-s3) is kept, off by default.

---

# Phase B: executor surgery (2026-08-05)

`crates/hick-literate` joined the workspace after replacing the wasm
container runtime with the `Executor` trait boundary
(`crates/hickory-executor`: trait + `LocalExecutor`). Every removal/stub
decision made during the port:

## Deleted from hick-literate

- **`src/executor.rs`** (~1471 lines) — the wasm `ContainerExecutor` fused
  with hick-container/wasmtime/hick-api. Replaced by the `Executor` trait;
  `run_pipeline_live` now takes `Arc<dyn Executor>`. The filesystem `.wasm`
  image model (`resolve_image_path`, `ensure_loadable_wasm_file`,
  `PipelineConfig::images_dir`/`toolchain_dir`, `PipelineRunOpts::images_dir`)
  is gone entirely: `image=` attributes are OCI-style references that
  `LocalExecutor` records but ignores (host tools) and the future
  `CanopyExecutor` maps via deployment config.
- **`src/pool.rs`** — warm-pool logic for wasm containers. `LocalExecutor`
  boot cost is ~0 so pooling is pointless; `PipelineResult` lost
  `pool_hits`/`pool_boot_time_saved`.
- **`src/auth.rs`** — the transparent auth-injection stack built on the
  unvendored `hick-net` TLS-intercept crates. Guest networking (and thus
  network-level capability enforcement and auth injection) does not exist in
  `LocalExecutor`; capability tokens are still minted and container
  capability parsing is unchanged (advisory until CanopyExecutor).
- **`src/repl.rs`** — the interactive REPL was built directly on the wasm
  executor + pool (rebuild-on-caps-change, warm adoption). Removed rather
  than stubbed; the transcript builder (`src/transcript.rs`) it fed is kept.
- **`src/main.rs` and `src/bin/{hick-equiv,hick-compact,hick-promote}.rs`**
  — the old `hick` binary and helper bins are superseded by
  `crates/hickory-cli` (`hickory`). The library modules they exposed
  (`equiv`, `compact`, `promote`, `generate_matrix`, `pipeline`, `watch`)
  remain; `hickory promote` wires the vendored promote cleanly.

## Stubbed / degraded

- **`<hick:script>` blocks** error at runtime with a clear message: they ran
  through hick-shell's in-process wasm command interpreter, which was part of
  the removed runtime. Use `<hick:exec>` instead.
- **hick-live reactive handlers** are not registered (`hick-live` was not
  vendored); `<hick:live>`-family tags are ignored by the tag registry.
- **Session replay** (`run` on a `hick:session` file) executes through
  `LocalExecutor`; the wasm-era preopen of the invoking directory at
  `/workspace` no longer exists — replays run in the executor's temp workdir.
- **Fork semantics** degrade to copying the source container's workdir when
  the fork target starts (filesystem-state approximation of command-history
  replay; in-memory state does not carry over). Documented in
  `hickory-executor`'s crate docs.
- **`watch.rs`** lost its container pool; each iteration builds a fresh
  `LocalExecutor`.
- **`visual_regression.rs`** was inspected and is not wasm-bound
  (pure output-diffing); kept unchanged.

## Modifications to other vendored crates

- **`hick-exec/src/dag.rs`** — exec/script command text now excludes any
  `<hick:expect>` subtree (`command_text()`), and `expect` children are
  excluded from `stdin_children`. Expectations are verification metadata,
  never command text or stdin.
- **`hick-handlers`** — `TranscriptEntry` gained `source_line: Option<usize>`;
  the exec handler renders only the entries produced by *its own* exec tag
  when provenance is present (falling back to the whole container transcript
  for legacy/reference execs), and `render_transcript` renders multi-line
  commands under a single `$ ` prompt with indented continuation lines.
- **Clippy fixes** (behavior-preserving, to satisfy `-D warnings`):
  `hick-case/src/lib.rs` (`sort_by_key`), `hick-secrets/src/cache.rs`
  (collapsed `if`), `hick-merge/src/llm_backend.rs` (unused test import),
  `hick-literate/src/text.rs` (`sort_by_key` ×2), plus mechanical
  `cargo clippy --fix` cleanups in `hick-exec/src/dag.rs` and
  `hick-literate/tests/pipeline_tests.rs`.

## Additions

- **`crates/hickory-executor`** — `Executor` trait, transcript event types
  (`{t, kind: cmd|out|err|exit, data}` with millisecond offsets for playback),
  and `LocalExecutor` (containers = per-run temp workdirs, `sh -c` per exec,
  volumes = tar in/out, **no sandboxing, images ignored** — loudly documented
  in the crate docs).
- **`crates/hickory-cli`** — the `hickory` binary (`run`, `check`, `weave`,
  `promote`, `--param`, `--out`, `--json`) with its logic in a library
  (`hickory_cli`) so the server can call the same code paths. Executor
  selection via `HICKORY_EXECUTOR=local|canopy` (canopy errors "not yet
  wired" until `hickory-executor-canopy` lands).
- **`hick-literate/src/expect.rs`** — `<hick:expect>` parsing + evaluation
  (`exact`, `regex-lines`); recorded on `run`, fatal on `check`.
- **`hick-literate/src/render.rs`** — the block model from
  `docs/specs/freeform/api.md` (prose HTML via pulldown-cmark, exec blocks
  with spans/transcripts/statuses, file blocks).
- **`docs/hick-guide.hick`** — vendored from the monorepo `docs/` tree
  (commit `e384c5e`); `hick-literate`'s `test_guide_hick_file_processes`
  includes it at compile time.
- **`examples/bootstrap-ci.hick`** — the file-embedded exec gained
  `show="output"` so the generated SVG contains only the command's stdout
  (the default `show="all"` would have prefixed the SVG with `$ ...` command
  lines).

## Agent port (`crates/hickory-agent`)

Ported 2026-08-05 from the monorepo's `hick-agent/crates/hick-agent-sdk` and
`hick-agent/crates/hick-agent-script-first` (same source commit `e384c5e`).
The port is a minimal, clean rewrite onto this repo's `Executor` boundary —
not a byte-for-byte vendor:

- **Kept**: `LlmClient` trait + Anthropic Messages API client (streaming
  SSE), the script-first `<hick:next>` ReAct protocol and loop, code-block
  extraction (trimmed to shell + python), and session logging. The old
  `XmlSessionLog` (a purpose-built `log:` namespace bundle) was replaced by
  `HickSessionLog`, which writes `hick:session` documents directly so
  sessions parse with `hick_lang::parse_session` and promote with
  `hickory promote`.
- **Adapted**: the old wasm `ContainerBackend` execution was replaced by an
  adapter over `hickory_executor::Executor` — scripts are written into the
  agent container's workspace via `execute_with_stdin` and run as
  shell/python; stdout/stderr/exit come back from the executor transcript.
- **Skipped for now** (revisit when the product needs them):
  `network_filter` / `sink_guard` / `security_config` / `tool_gate` /
  `pane_csp` (the security stack), all MCP layers (`mcp_client`,
  `mcp_config`, `mcp_format`, `mcp_install`, `mcp_project_config`),
  `conversation_tree`, the TUI (`hick-agent-tui`) and `ui_event` /
  `agent_ui_bridge` / `web_server`, `relay_client` (and `hick-relay`),
  the OpenAI-compat + local-llama backends and `model_router`,
  `agent_memory` / `agent_skills` / `agentignore` / `project_context` /
  `suggestion_gen` / `session_bundle_reader`, and the eval/plan-runner
  crates (`hick-eval-store`, `hick-plan-runner`).
