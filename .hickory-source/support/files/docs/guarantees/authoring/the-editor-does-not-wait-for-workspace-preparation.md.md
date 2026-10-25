<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `263bdcf190ef68284e65134d579d7932fa3ab43c3ed2b72a47c74df4eea9b179`.

<book:entry path="docs/guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation.md">
# The Editor Does Not Wait For Workspace Preparation

Given a desktop window opens a workspace, when its watcher has been installed
and startup configuration validated, then the window can load before the initial
weave finishes. Settings, folder identity, layout and draft recovery remain
available. An unsaved note can be edited and its recovery draft persisted while
generated files are being prepared.

Given that preparation is pending, when a request would read or change a file,
execute a tool, or open a collaboration socket, then it waits for initialization.
Ordinary HTTP requests are refused **before dispatch** with an explicit startup
marker; the app shares one readiness wait and submits those requests after
readiness. An unmarked failure is never automatically replayed. Generated-file
ownership, permissions and reverse-edit baselines are established before those
handlers run. A stopped initializer reports a failure instead of waiting forever.
CLI attachment retains its synchronous initial-weave behavior.

Given a window only needs the startup note, when the frontend boots, then parser
loading and keymap retrieval overlap, and Agent, Settings, Lineage and New Project
code loads when its surface is opened. Existing editor buffers stay mounted while
an optional surface loads.

Given a macOS Hickory process opens another window with its recovered PATH
unchanged, then the child reuses that PATH without rerunning shell profiles. A
fresh launch still reads login and interactive profiles, within the existing
timeout and output limit. PATH discovery does not run logout hooks or save a
terminal session on exit.

---

Last LLM verification:

- Date: 2026-10-17
- Reviewer: Codex
- Result: verified for the startup boundaries described here; this is not a
  measured native cold-launch speed comparison.
- Originating requirement: the user reported launch slower than VS Code and
  requested background workspace preparation, reduced frontend boot work, and
  reduced shell-discovery overhead.
- Interpretation: first usable editor is independent of completed initial
  weaving; this does not claim a particular millisecond budget or faster launch
  than another editor on all machines.
- Evidence: `engine/watch.rs::Coordinator::register` acknowledges installed
  watchers before weaving for desktop attaches; `engine/startup.rs` and
  `engine/daemon.rs::dispatch` gate file handlers and sockets. CLI attaches wait
  for completion. `DocIndex::shared_from` reuses discovery; the desktop proxy
  avoids rescanning folder windows. Single-file windows retain their companion
  index so later folder attachment preserves existing document routing.
- Frontend evidence: `main.tsx::boot`, `App.tsx`, `views/deferredAgent.tsx`,
  `api/workspaceStartup.ts`, `api/client.ts::request`, and `StartupNotice.tsx`.
  The retry marker is emitted only before routing to a handler. Requests whose
  outcome may be ambiguous are not retried by this startup mechanism.
  `lib/focusWhenShown.ts` preserves initial keyboard focus when an unnamed
  editor mounts before the spatial shell reveals it.
- Shell evidence: `launch_path.rs::load`, `shell_command`, and `main.rs`;
  shell replacement with `/usr/bin/printf` skips logout work. Only an identical
  inherited PATH and Hickory's resolution marker bypass profile discovery;
  this is not a persistent cache that could miss changed profile dependencies.
- Assumptions and exclusions: directory scanning, engine discovery/spawn,
  watcher installation, fresh shell profiles, WebView initialization, and the
  parser still have startup costs. Existing on-disk output writers retain their
  original locking. This change does not make an external editor's writes wait
  during initialization, nor permit execution before preparation completes.
  Workspace boot APIs bypass the project write gate only for settings reads,
  directory metadata, window-storage operations and native dialogs/close.
- Witnesses and challenges: `engine_lifecycle::background_attach_serves_editor_storage_before_outputs_are_ready`
  holds a real cell open with a filesystem handshake, persists a draft, checks
  that a file PUT has not been dispatched, then releases preparation and checks
  output bytes and editing. `workspaceStartup.test.tsx` checks shared waiting,
  explicit retry authorization and visible failure. PATH tests cover actual
  zsh startup, logout omission and inherited-path changes. A stopped watch
  sender is tested in `engine::startup::tests`.
- Dependencies: working-tree implementation over base revision
  `400a900471f5a8ec035745e8b184620b5c570344`. Critical boundary SHA-256 hashes:
  `engine/watch.rs` = `7fc05aa6faf6b0030b3af5424a57440d9806b70a5e5904580b9645cd932f50c7`;
  `engine/startup.rs` = `54eaf9a2b820d83fdff605deca6532c914cb2fdc67c2510775fb284217b93b4f`;
  `engine/daemon.rs` = `f2bbeda6f6f1e8cc27b30cd8562b7c1d4e090a8d437124ad1c24f069978a58eb`;
  `launch_path.rs` = `983738b86c39a24ee2fee80919a0081cdcc1c8f9db3b2fdb07c8417ae94560e8`;
  `main.tsx` = `706441e7f9383da8a91264c015bca6140b6fd1a3331f0c9bb64dc84496f546a0`;
  `api/workspaceStartup.ts` = `6dd082c23856661a603993abf60d378a8aa42c0261e3d55ee63e560868d8e63d`.
  These hashes cover the critical boundaries, not every transitive dependency.
  Changes to startup ordering, bypass routes, write locks, retries, lazy import
  boundaries or focus handling require review.
- Executed checks: `just test-startup` passed (1 readiness unit test, 11 real
  engine lifecycle tests, 6 PATH tests, 70 frontend tests). After the focus fix,
  `just test-startup-web` passed all 60 tests and
  `just test-startup-browser http://127.0.0.1:5173` passed both Chromium flows.
  The browser preparation test uses an intercepted pre-dispatch response; the
  separate native lifecycle test holds a real executing cell open. Neither
  claims a timed native launch measurement. `just test-agent-web` passed
  2,191 tests before the final focus adjustment; final focused checks cover it.
  `just check-desktop`, production frontend builds, `just test-binary-discovery`,
  file-length checks and `git diff --check` passed. The guarantee itself passed
  `hick run`, `hick lint` and `hick test` with no generated files.
- Broader-check limits: `just test-engine` is not green: the unchanged
  `serve_local::a_prose_edit_in_the_woven_markdown_lands_in_the_document` expects
  `demo.md` among outputs but gets only `greet.rs`, and
  `a_session_with_no_documents_starts_empty` treats `notes.md` as a non-document.
  Both use the existing `prepare` path rather than shared-engine attachment.
  These failures were observed before any changes to that test file; fixture
  investigation was reverted. The initial broad Clippy run exhausted disk
  space while other builds ran; its retry was canceled after waiting behind
  another task's build lock. Broad workspace Clippy is therefore incomplete;
  desktop Clippy completed successfully. Temporary QA server processes were
  stopped after the browser checks.
- Human judgment: no native cold-launch comparison recorded yet.
</book:entry>

</book:doc>
