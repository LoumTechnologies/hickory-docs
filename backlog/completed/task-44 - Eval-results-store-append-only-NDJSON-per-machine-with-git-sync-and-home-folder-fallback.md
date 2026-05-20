---
id: TASK-44
title: >-
  Eval results store: append-only NDJSON per machine with git-sync and
  home-folder fallback
status: Done
assignee: []
created_date: '2026-05-20 12:16'
updated_date: '2026-05-20 16:31'
labels:
  - eval
  - persistence
dependencies: []
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Design and implement the eval persistence layer. Each machine writes exclusively to its own file named by a stable machine ID (hostname + UUID written once to `~/.hick/machine-id`). Storage location: `.hick/eval/<machine-id>.ndjson` when inside a git repo, otherwise `~/.hick/eval/<machine-id>.ndjson`. Since each machine owns exactly one file, git merges are always conflict-free. An aggregating reader merges all machine files at query time. Records are append-only — pruning decisions are written as metadata records on top of existing entries, never deletions. On startup, `hick` runs `git pull` (if in a repo and a remote exists). After each eval session, it commits and pushes the machine file. A storage trait abstracts git vs. future server backends so the transport can be swapped without changing the record format.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Machine ID is stable across restarts and written to ~/.hick/machine-id on first run
- [x] #2 Each eval result (query, config, response, scores, token cost, timestamp, machine-id) is written as one NDJSON line
- [x] #3 Pruning decisions are appended as metadata records; no lines are ever deleted
- [x] #4 Aggregating reader merges all <machine-id>.ndjson files and presents a unified view
- [x] #5 git pull on startup and git commit+push after session complete when inside a repo with a remote
- [x] #6 Falls back to ~/.hick/eval/ when not in a git repo, with no error
- [x] #7 A `StorageBackend` trait exists with a git implementation; server impl is stubbed for future use
- [x] #8 Two machines writing concurrently then merging via git produces no conflicts
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented `hick-eval-store` crate at `hick-agent/crates/hick-eval-store/`.

**Files created:**
- `Cargo.toml` — deps: anyhow, chrono+serde, serde, serde_json, uuid+v4
- `src/lib.rs` — public re-exports
- `src/path.rs` — `git_toplevel()` via `git rev-parse --show-toplevel`, `resolve_store_dir()`, `home_dir()`
- `src/machine_id.rs` — `MachineId`: UUID persisted in `~/.hick/machine-id`, filename = `<hostname>_<uuid>.ndjson`; `load_from(path)` for test isolation
- `src/record.rs` — `EvalRecord` open enum (EvalResult / Prune / Unknown) with custom Serialize/Deserialize; `EvalResultRecord`, `PruneRecord`, `TokenCost`; `Unknown` variant preserves unrecognised record types verbatim
- `src/backend.rs` — `StorageBackend` trait: `fn append(&self, record) -> Result<()>` and `fn read_all(&self) -> Result<Vec<EvalRecord>>`
- `src/git.rs` — `GitBackend`: git root cached at construction; `git pull --ff-only` on `new()` (non-fatal); `sync_session()` does `git add + commit + push` (non-fatal); `read_all()` aggregates all `.ndjson` files in store dir, skips malformed lines
- `src/noop.rs` — `NoopBackend`: in-memory Mutex-backed store for tests

**21 tests, all passing.** Full workspace test suite clean.
<!-- SECTION:FINAL_SUMMARY:END -->
