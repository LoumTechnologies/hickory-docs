# Native project tools select complete endpoints

Given an evaluated project closure, when native services select New, then every
file comes from its completed tree, including changes in later patches and other
files. Old selects the captured input and refuses materialization as a runnable
project when empty. Local states require an explicit selection. Explicit native
materializations use distinct private temporary directories. Deleted bytes have
no endpoint mapping; ambiguous origins refuse routing. Coordinates stay bytes
until conversion to editor/LSP UTF-16 positions.

The live LSP entry and native document debugger use the same project evaluator
for declared closures. Working-tree manifests are not copied into declared LSP
projects to supplement missing construction outputs.

---

Last LLM verification:

- Date: 2026-10-12
- Reviewer: Codex
- Result: verified for declared closures on the tested macOS arm64 toolchain;
  cross-document protocol mapping and additional scope selectors remain excluded.
- Originating requirement: `docs/specs/freeform/literate-changesets.md`, atomic
  endpoints and separate native identities.
- Evidence: `hick-lsp/src/project.rs`, `document.rs`, `backend.rs`,
  `position_map.rs`; `hick-dap/src/project.rs`, `program.rs`, `session.rs`;
  `hickory-cli/src/project_documents.rs::debug_state` and `debug_sessions.rs`.
- Witnesses and challenges: `hick-lsp/tests/project.rs` checks final library bytes
  despite the incomplete middle patch, distinct materializations, empty Old,
  deleted-line refusal, Unicode coordinates and existing-symlink refusal. The
  explicit `hickory-cli/tests/literate_project_csharp.rs` acceptance test builds
  EFCore, applies the final migration, persists a note, and stops netcoredbg at
  patch-inserted code with the correct document line and watched value. It caught
  diff-prefix column routing and macOS `/var` versus `/private/var` source alias
  failures; line breakpoints and canonical native identities now fix both.
- Assumptions and exclusions: the UI defaults to New; Old/local selection currently
  exists at the native API rather than a new toolbar selector. Cross-document
  origins remain exact in the portable map; the legacy single-document LSP/DAP
  mapping refuses foreign spans instead of binding them to the entry document.
  Explicit nested atomic scopes and cross-worktree tool selection remain pending.
- Executed checks: recorded in `implementation-evidence.md`.
- Dependencies: paths above; native entry/mapping changes invalidate this review.
- Human judgment: not recorded; discovering an adapter is not successful debugging.
