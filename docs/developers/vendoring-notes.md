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
