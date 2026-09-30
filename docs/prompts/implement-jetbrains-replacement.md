# Implement the JetBrains replacement backlog

You are implementing the GitHub issues created from a review of Hickory as a
replacement for a JetBrains All Products Pack subscription. The target user is
an engineer moving daily work from JetBrains to Hickory, so measure success by
whether they can safely edit, navigate, run, debug, test, inspect data, and
move changes in a real repository.

Work through this backlog in the order below. Do not treat an issue as done
because an API, protocol type, or backend capability exists: finish the
user-facing flow and prove it through the real application. Preserve the
product's local-only design. Do not add hosted accounts, telemetry, relays, or
cloud services. Do not introduce `launch.json`-style configuration files;
derive behavior from the open project and make an honest refusal when it
cannot be derived.

## Before changing code

1. Read `AGENTS.md`, then `docs/specs/freeform/notes-ide.md`,
   `docs/specs/freeform/local-only.md`, and
   `docs/specs/freeform/architecture.md`.
2. Read `docs/specs/freeform/replacing-the-jetbrains-pack.md` and each linked
   issue before beginning its work. Re-check current code and existing issues;
   the issue describes the problem found during the review, not necessarily the
   implementation that should be chosen.
3. Make one cohesive change at a time. Keep existing user changes intact.
4. Add or update a guarantee when the change establishes a durable product
   behavior. Use a real end-to-end test whenever a claim depends on a language
   server, debugger adapter, test runner, database, or HTTP process.

## Delivery order

### 1. Fix correctness defects before adding capability

- [#32: LSP workspace edits can leave a rename incomplete and unsaved](https://github.com/LoumTechnologies/hickory-docs/issues/32)

  Implement a reviewable, all-or-nothing workspace-edit path. A cross-file
  rename must update every affected file, make each changed buffer dirty, and
  persist all changes through Save All. Do not silently apply only the focused
  editor's ranges. Add an integration test using a real language server that
  renames a symbol across files, saves, restarts/reloads, and verifies every
  file on disk.

### 2. Make project debugging useful beyond one-file examples

- [#33: Debug real projects using their build system and launch inputs](https://github.com/LoumTechnologies/hickory-docs/issues/33)
- [#34: Add a debugger variables inspector and set-variable control](https://github.com/LoumTechnologies/hickory-docs/issues/34)

  For C and C++, discover and use the existing project build mechanism instead
  of compiling a selected source file in isolation. Derive executable,
  arguments, environment, and working directory where the project gives enough
  evidence; otherwise show what is missing and how to resolve it. Keep the
  launch plan visible. Then add an accessible debugger variables tree driven by
  DAP scopes and `variablesReference`; permit variable edits only when the
  adapter advertises support. Verify a real multi-file project, an argument or
  environment-dependent launch, nested value expansion, frame selection, and a
  set-variable request.

### 3. Complete safe code navigation and refactoring

- [#35: Surface LSP structure and call hierarchy in the workspace UI](https://github.com/LoumTechnologies/hickory-docs/issues/35)
- [#40: Add safe project refactor operations beyond rename](https://github.com/LoumTechnologies/hickory-docs/issues/40)

  Draw a structure pane and breadcrumbs from `documentSymbol`, then draw
  incoming and outgoing call hierarchies with direct navigation to the target
  file or document. Execute server commands only through an explicit,
  capability-gated path, and route every multi-file refactor through the
  workspace-edit transaction from #32. Before applying a broad refactor, show
  the affected files and give the person a clean way to cancel. Exercise this
  against a real server, including one multi-file operation.

### 4. Build the test feedback loop

- [#36: Provide project test results and code coverage](https://github.com/LoumTechnologies/hickory-docs/issues/36)

  Keep the terminal transcript as the authoritative record of a run, but add a
  project test tree and durable pass/fail/skipped result view. Make failure
  output and source navigation immediate, and let a person rerun failures
  without reconstructing commands. Add coverage only where the ecosystem can
  produce it honestly; map it to source files without implying universal
  support.

### 5. Fill frequent editor and Git workflows

- [#41: Fill daily editor workflow gaps](https://github.com/LoumTechnologies/hickory-docs/issues/41)
- [#42: Support cherry-pick and patch workflows in the Git pane](https://github.com/LoumTechnologies/hickory-docs/issues/42)

  Add snippets/templates, bookmarks, a configurable TODO list, and the editor
  behavior that can correctly read `.editorconfig`; leave formatter-owned
  behavior to the formatter. Add local cherry-pick and patch import/export to
  the Git pane, respecting the publication floor and reusing the existing
  conflict flow. Do not add GitHub pull-request integration.

### 6. Close product-sized tool gaps

- [#37: Add a local HTTP request document and response viewer](https://github.com/LoumTechnologies/hickory-docs/issues/37)
- [#38: Build a local database query tool with secure connection storage](https://github.com/LoumTechnologies/hickory-docs/issues/38)

  Design each as a complete local workflow before committing to an interface.
  An HTTP request document must show requests and inspectable responses while
  keeping credentials out of document bytes and routine logs. A database tool
  needs local credential storage, connection management, schema navigation, a
  query console, and a result grid. Make the active connection and write risk
  clear. Use disposable local test services and assert that secrets never land
  in the project tree.

### 7. Make language readiness legible

- [#39: Close language-support gaps or present a migration matrix](https://github.com/LoumTechnologies/hickory-docs/issues/39)

  Publish and render a machine-readable language matrix that distinguishes
  syntax drawing, language intelligence, project loading, debugging, tests,
  and installation state. Every claimed workflow needs a real end-to-end test;
  discovery alone is not support. Prioritize Kotlin, Scala, Groovy, Ruby, and
  PHP based on actual demand, and make an unavailable capability explicit in
  the app instead of presenting a dead control.

## Acceptance standard for every issue

- Start from the observed user action and finish at the visible result.
- Test the live integration when the feature crosses a process boundary.
- Run focused checks first, then the relevant repository checks before closing
  the issue.
- Update the issue with the behavior delivered, the evidence, and known limits.
- Do not close an issue merely because its backend route, wire type, or unit
  test exists. Close it only when the application delivers the documented
  end-to-end behavior.

When a design decision materially changes the local-only or document-first
model, stop before implementing that decision and present the concrete options
and their consequences.
