---
name: implement-just-interface
description: >
  Add or standardize a repo-root justfile with commands appropriate to the
  current repository, including commit-triggered restarts for development
  processes. Use when asked to add just commands, give a repo a
  standard task interface, rename nonstandard recipes, or maintain the shared
  command vocabulary.
  This skill is the ongoing reference for standard recipe names and behavior;
  adapt implementations to the repo's existing stack and supported platforms.
---

# Implement Just Interface

Treat `just` as the public interface to recurring repository operations. People and agents should be able to move between repos using familiar command names without knowing their package manager, workspace layout, or scripts. This document owns the shared vocabulary. The target repo's justfile owns its implementation and documents its actual coverage, prerequisites, and outputs.

## Apply to the current repo

1. Before making any changes to a target repo, run
   `git status --porcelain=v1 --untracked-files=all`. If it reports any staged,
   unstaged, or untracked changes, refuse the migration and report the blocking
   paths. Do not stash, commit, reset, clean, create a replacement checkout, or
   prepare a migration patch to work around this prerequisite. Resume only
   when the user has made the target repo clean. This is an entry prerequisite;
   the migration's own subsequent edits are expected.
2. Read project instructions, the existing justfile, manifests, lockfiles,
   scripts, CI, and developer docs. Identify the supported products/platforms,
   primary backend language, and the workflows people actually repeat.
3. Map those workflows to the contracts below. Implement only relevant commands
   backed by real behavior: a library needs no `dev`, and an app without a
   phone target needs no `update-phone-app`. Do not install a new framework,
   container stack, release system, or test harness merely to fill the table.
4. Before editing, record a migration inventory mapping every public recipe
   from its existing name to the canonical name, its behavior, its active
   callers, and the justification for any distinct scoped command. Use this
   inventory to audit behavior, not just spelling. Separate naming changes
   from behavior changes: renames preserve functionality and coverage; additions,
   placeholder removals, and corrections such as moving installation out of
   `doctor` must be explicitly identified and justified in the change summary.
   Do not silently expand a naming migration into a workflow redesign.
   Identify nonstandard names and duplicate entrypoints, rename to the canonical
   name, and remove aliases and alternative spellings. Update all callers in the
   same change: recipe dependencies, scripts, CI, hooks, docs, and agent
   instructions. Search for old names afterward. Report external callers that
   cannot be updated here; do not retain a compatibility alias. Preserve the
   underlying operation and test coverage during a rename. Historical execution
   receipts and archived work logs retain their original command spellings;
   distinguish these from live callers rather than rewriting past evidence.
5. Put the justfile at the repo root. Keep recipes thin, reuse native tools and
   existing scripts, and put substantial new orchestration in the primary
   backend language (or primary language for frontend-only repos). If that
   language is ambiguous, ask and record the decision in project instructions.
   Where files are generated from maintained source (including literate
   Markdown), edit their owners and regenerate. Do not bypass capture/reconcile
   guards or absorb unrelated pending user changes to complete a rename; report
   blocked generation and validation explicitly.
6. Add short recipe comments so `just --list` explains intent. Document special
   prerequisites, outputs, ports/URLs, persistent state, and platform limits
   where relevant. Update the repo's existing developer documentation with the
   clone-to-running path and the routine verification commands.
7. Parse/list the resulting justfile, inspect dry runs, and execute applicable
   local checks. Confirm that renamed commands resolve, their old names no longer
   resolve, and no stale callers remain. Verify invocation from a nested
   directory too. Compare `ci` with the actual required workflow jobs and hook
   checks, including reusable workflows and invoked scripts: account for every
   required gate and document any host or environment exclusions. Verify the
   aggregate dependencies, not just syntax or successful dry runs. Exercise
   changed devenv/dev behavior when practical; report exactly what ran and
   what needs a device, GUI, credentials, or another supported OS. Adding a
   deploy/release/install recipe does not authorize executing it.
   Validate automatic preparation from a fresh checkout or isolated copy with
   project dependencies and generated outputs absent, then repeat to check
   reuse. Do not delete the user's local dependencies or state for this check.

## Standard command contracts

These names are canonical for both new and existing interfaces. The same kind of operation uses the same name across repos, regardless of stack or platform. Only implement applicable operations; do not create empty recipes or recipes that merely print hypothetical steps. A different name requires a materially different behavior or scope, with a strong justification documented beside the recipe. Historical spelling, convenience, and caller compatibility do not justify a second name. Never add aliases or alternative public entrypoints for the same operation.

| Command | Contract and when it belongs |
|---|---|
| `default` | Show available recipes (`just --list`). Bare `just` must not build, install, launch, deploy, or delete anything. Ensure this is the actual default, not merely a recipe named `default` below a different first recipe. |
| `devenv` | Prepare development dependencies and tooling using the repo's existing package manager and lockfiles. Safe to repeat. Document machine prerequisites separately. Rename `setup` to `devenv` and update callers; keep no alias. |
| `doctor` | Diagnose required tools, versions, native SDKs, and local configuration; give actionable fixes and fail when requirements are missing. Do not silently install or change configuration. |
| `dev` | Start the normal development experience using current source, including required local services. Build and open the app or start the appropriate development server as needed. State whether it opens a native window or serves a URL; restart on new commits as described below; clean up owned processes on exit unless project instructions require background mode. Consolidate `launch` and `dev-web` here when they serve this same goal. |
| `check` | Fast, noninteractive feedback on the normal development scope: compile/type checks, static verification, and/or cheap tests as appropriate. Describe exclusions; do not imply this is all of CI. |
| `test` | Run the normal automated test suite. Document excluded GUI, network, device, expensive, or ignored tests and expose meaningful ones separately. |
| `lint` | Run configured linters; fail on violations. Keep automatic fixes explicit rather than silently changing source. |
| `fmt` | Apply source formatting. |
| `fmt-check` | Check formatting without changing source; fail on drift. |
| `build` | Produce the intended distributable artifact: a CLI executable, desktop app/installer, or website assets. Include packaging needed for that output; consolidate `bundle` into this command. Document mode, SDK/signing requirements, and output paths; no installation or publishing. |
| `ci` | Run all applicable gating checks through the same implementations CI uses. Include formatting/codegen checks when required. Explain host-only exclusions; a skipped required gate must not be reported as a full pass. |
| `clean` | Remove reproducible build outputs within the documented scope. Keep user data, credentials, and other worktrees intact. Data resets need distinct, explicit names. |

### Add only when supported

| Command/family | Contract |
|---|---|
| `dev-bridge` | Standalone backend bridge only when it is independently useful and materially distinct from `dev`. Document its relationship to the normal development experience. |
| `dev-stop`, `dev-clean`, `dev-seed` | Development lifecycle commands; behavior and implementation belong to `$implement-dev-environment` and its `dev-environment` instruction. |
| `codegen`, `check-codegen` | Generation and drift checking; behavior and implementation belong to `$implement-dev-environment` and its codegen reference. Reuse these names for generation workflows outside API clients too. |
| `e2e`, `e2e-trace` | Run end-to-end tests, optionally forwarding runner filters; open saved failure traces. State browser/service requirements and who starts/stops services. |
| `smoke` | Small real-system acceptance path. State external runtimes or downloads required; do not substitute a mock while calling it real-system verification. |
| `check-all` | Broader compile/static scope than `check`, such as the native shell. Distinguish it from `ci`, which includes all gating job types. |
| `<component>-build` | Build a distinct component only when its scope materially differs from `build`. Use the component naming rule below; `build-app` and `build-desktop-app` are nonstandard names, not alternatives to `build`. |
| `install` | Build as needed and install the product locally. State destination, signing requirements, and replacement behavior. Rename `local-install` to `install`; preparing development dependencies belongs to `devenv`. |
| `update-phone-app` | Build, replace in place, and open the app on a connected phone. Preserve app identity/data, use the real mobile workflow, and provide device selection when needed. Document platform and signing prerequisites. |
| `hooks-install` | Install this repo's tracked Git hooks. Explain the local Git configuration change. |
| `deploy-staging`, `promote-production`, `teardown-staging`, `launch-state` | Use only for repos with these operational capabilities; retain their environment, approval, and data-safety contracts. `launch-state` reports release readiness; it is distinct from running the app with `dev`. |
| `unstable-release` | Explicit release-channel operation when supported. Rename `release-unstable` to `unstable-release` and update its callers; remove the old entrypoint. State signing, upload, and credential requirements. |

## Restart development processes on commits

Make `dev` and other restartable, long-running local development commands
restart automatically when the current checkout's commit changes. Include
independently useful scoped development servers and bridges when safe to
restart. Keep this behavior inside the canonical command; do not require a
second public watch command. Preserve existing source-file hot reload where
available: a commit adds a full restart, while saving files alone must not
trigger this new supervisor.

- Start immediately, then observe the resolved commit ID of this worktree's
  `HEAD` (for example, `git rev-parse --verify HEAD`). Detect successful commits,
  amends, and commit changes from checkout, reset, merge, or rebase. Ignore
  changes confined to other worktrees. Support an unborn branch becoming its
  first commit. Observe Git state without installing hooks or assuming `.git`
  is a directory; worktrees, detached HEAD, and packed refs must work. A small
  polling supervisor is sufficient; do not watch only `.git/HEAD`, whose
  contents often stay unchanged when a branch advances.
- On a changed commit, stop the owned process tree, wait for shutdown, rerun
  required preparation/build steps, and start the app again with the original
  arguments and environment. Coalesce rapid changes to the latest observed
  commit and check again after startup so commits during a build are not lost.
  Never overlap app instances or leave children holding ports. For native apps
  whose launch command exits immediately, explicitly own and stop the launched
  app; supervising only the launcher cannot restart the app reliably.
- Keep the supervisor in the foreground. Ctrl-C or termination stops both it
  and its owned children without another restart. Surface startup/build/runtime
  failures; do not spin on the same failed revision. Waiting for the next commit
  to retry is acceptable if the failure remains visible and documented.
- Run against the current working tree, including any uncommitted edits. This
  is a restart trigger, not an isolated committed snapshot: do not stash, reset,
  clean, or change branches to implement it. Document that existing hot reload
  can still show edits before a commit. Outside a Git checkout, either run once
  with an explicit notice or fail with actionable guidance.
- Keep one-shot commands (`check`, `test`, `build`, `ci`, formatting, generation)
  one-shot. Do not automatically repeat installs, phone updates, releases,
  deployments, destructive commands, or other external mutations. Document a
  concrete reason when an otherwise eligible development command cannot safely
  restart. Reuse existing lifecycle ownership from `$implement-dev-environment`
  when present, and use the implementation-language/platform rules below.

Verify the supervisor in a disposable Git repo with a harmless long-running
child: initial launch, save without commit (no new full restart), new commit,
amend, rapid commits including one during startup, failure followed by a new
commit, and Ctrl-C cleanup. Check worktree Git resolution and that the old
process tree exits before its replacement starts. Exercise the actual app
restart when practical; report platform or GUI limitations. Document the
trigger, shutdown behavior, failure recovery, and any exclusions in the target
repo's developer docs.

## Names, implementation, and scope

- Every operational recipe must do everything necessary for its documented
  operation once machine prerequisites are installed. Never require the caller
  to run `devenv`, install project dependencies, build another component,
  generate required files, or start required local services first. Wire those
  steps into recipe dependencies or shared preparation helpers. This applies
  to scoped recipes and direct invocations as well as aggregate commands.
  Keep `default` discovery-only and `doctor` diagnostic-only; `clean` must not
  prepare artifacts it is about to remove. Preparation must still respect
  each command's contract, including codegen drift checks and permissions for
  external operations.
- Distinguish machine prerequisites (such as the package manager, compiler,
  Docker runtime, or native SDK) from project-local preparation (such as
  `node_modules`, a virtual environment, repo-pinned CLI tools, generated
  assets, and service initialization). Diagnose missing machine prerequisites
  with actionable instructions; install or prepare project-local requirements
  automatically using the repo's existing tools and lockfiles. For example,
  `just dev` in a pnpm/Tauri project must install missing workspace dependencies
  before invoking the local Tauri CLI, so a fresh checkout cannot fail with
  `tauri: command not found` merely because `node_modules` is absent.
- Use short lowercase kebab-case names and the canonical unqualified command
  for the primary product. A genuinely distinct component or focused feature
  uses `<scope>-<canonical-command>`: `studio-dev`, `studio-check`, `studio-build`,
  `pilot-dev`, `pilot-check`, `<feature>-test`. Keep multiword standard names
  intact, such as `<scope>-dev-bridge`. A separate browser target may use
  `web-dev` only when it serves a materially distinct target from `dev`;
  different underlying tooling alone does not justify a second entrypoint.
  Rename `test-<feature>` and
  `check-<feature>` accordingly. Document what makes the scoped operation
  different; do not qualify a command merely because the repo uses a particular
  language, framework, or platform. Root commands make their aggregate scope
  clear. If two public commands perform the same operation, consolidate them
  under its standard name.
- Private helpers such as `_ensure-node` are implementation details. Dependency
  preparation should detect missing or stale dependencies, respect lockfiles, and
  avoid reinstalling on every invocation. Share this preparation with `devenv`;
  it is an explicit way to prepare the environment, not a mandatory first step.
  Never turn preparation into an implicit package upgrade or overwrite local
  configuration. Stop on preparation failure before running the dependent tool.
- Use recipe dependencies for reusable prerequisites and existing runner
  filters for focused work. Do not add a public command for every temporary
  debugging invocation. Give a named recipe to a durable, recurring workflow.
- Keep supported platforms honest. A macOS app can use macOS launch tooling;
  a cross-platform repo needs wrappers that work on its supported shells.
  Do not copy POSIX shebangs or shell chains into a Windows contract blindly.
- Preserve failure status. No `|| true` or success message that hides a failed
  gate. Avoid invoking cloud operations, launching a GUI, or deleting state
  from ordinary checks. Keep protocol-server stdout free of recipe echo.
- Resolve paths relative to the repo/justfile, not an assumed caller directory.
  Forward arguments with the chosen shell/runtime's quoting rules; validate
  values used for paths, devices, or environments. Avoid unquoted shell
  interpolation of arbitrary user input.
- If `$implement-dev-environment` is installed, it owns the mechanics of its
  development lifecycle and codegen workflows; read it when implementing them.
  Keep those detailed contracts there rather than copying them into this
  catalog. This skill owns their names as part of the wider repo vocabulary.

## Keep this standard useful over time

When asked to standardize a new recurring workflow, check this vocabulary first. Reuse a matching contract before inventing another spelling. Keep project-specific details in the project. When the user requests a shared convention change or a reusable pattern is established, update this source skill in `agent-toolbox` with the name, behavior, applicability, and migration guidance; do not maintain a competing catalog in a consuming repo's copied skill. Report a proposed shared change if the source library is unavailable.

The initial conventions were drawn from sibling repositories: `jobsearch` (setup/doctor, fast checks, browser loop, local install, releases), `focus-todo` (core/native/frontend split and bundles), `pilockdb` (CI gates, component families, codegen and deployment), `pimark` (phone updates), `apple-mcp` and `signal-mcp` (build/launch), `study-cdl` (explicit launch and voice setup), `local-ci` (real-runtime smoke tests), and `agent-toolbox` (discoverability, validation, installation). These are provenance, not dependencies or templates to copy wholesale; this vocabulary records the standardized direction rather than claiming every existing recipe conforms.
