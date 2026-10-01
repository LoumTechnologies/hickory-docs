# Project environments: detect, explain, offer the project's own command

*Status: adopted, first implementation built 2026-09-30. Audience: the engineer
implementing workspace environment checks in Hickory's local IDE engine.*

The built slice is `hick-project-env`: a provider registry, uv readiness
checks and repair, npm/pnpm missing-install detection, workspace ownership,
typed findings, saved manager choices, terminal actions, periodic/focus/file
rechecks and host Python interpreter selection. The IDE has an Environments
status button, a Problems section and dismissible notices. The guarantee and
verification boundaries are
[`a-project-environment-offers-its-own-sync.md`](../../guarantees/editor-intelligence/a-project-environment-offers-its-own-sync.md).
Full npm/pnpm freshness checks, alternate-profile selection, non-host probes
and independently distributed plugins remain future work.

Open a cloned Python project with `uv.lock` and no environment. Before an
unresolved import becomes a debugging exercise, Hickory should say:

> Python dependencies are not installed for `analysis/`.
> Run `uv sync --locked` in `analysis/`.
> **Sync dependencies** · **Details** · **Dismiss**

Pressing the button opens a terminal showing the package manager's output.
When it exits, Hickory checks again. Installing dependencies outside Hickory
also clears the message when the check next succeeds.

The recommended architecture is a workspace extension with registered
package-manager providers. It belongs beside LSP, DAP and terminals, outside
the document parser and element registry. A provider understands its package
manager; shared code schedules checks, publishes findings and runs actions.

## What existed before this feature

`crates/hick-lsp/src/discovery.rs` finds language servers in project-local
directories: `node_modules/.bin`, virtualenvs, PDM, conda, Composer and Bundler
layouts. This is executable discovery, with no manifest-to-environment
freshness check or sync offer.

`crates/hick-literate/src/needs.rs` checks explicitly declared executables
through the executor before cells run. The existing guarantee,
[`a-document-says-what-it-needs.md`](../../guarantees/execution/a-document-says-what-it-needs.md),
deliberately excludes package-manager knowledge and installation. Keep that
element's behavior; the new workspace extension delegates installation to
the user's package manager when the person chooses its action.

The terminal infrastructure used by `serve/test_run.rs` and
`serve/scaffold.rs` provides the execution surface. `serve/install.rs` is a
separate, currently DAP-only tool installer; project dependencies should not
be installed into its tool prefix.

## Detect the owner before checking its environment

On project open, scan manifests and workspace declarations, pruning generated
and dependency directories. Resolve an active file to its owning package and
that package to its package-manager workspace. One repository may contain
independent Python and JavaScript projects, or nested packages sharing an
environment. Check and offer repair at the owning workspace root.

Use explicit package-manager declarations and native workspace membership
first, an unambiguous lockfile next, and manager-specific configuration as
supporting evidence. `pyproject.toml` alone does not select uv; `package.json`
alone does not select npm. An executable on PATH says what is available, not
what this project chose. Conflicting evidence produces an ambiguity message
with a choice, rather than an arbitrary install command. Save an explicit
choice in user workspace state.

Respect configured environment locations, manager versions, dependency
groups, extras and workspace selection. Stop ancestor discovery at the open
project boundary or its containing repository; use native workspace membership
to distinguish a shared workspace from an independent nested project.

## Check without turning project open into installation

First read the manifest, lockfile and installation metadata. If a more precise
answer needs a subprocess, use a supported native check with a timeout, no
network, no installation, no lifecycle scripts and no workspace writes.
Capability-test the installed manager version. A check may use disposable
cache storage; it must not create an environment or rewrite a lockfile.
Verify these properties for each supported version before enabling automatic
probes. Never substitute a real install for an unavailable check.

For uv, evaluate `uv lock --check --offline` for manifest/lock consistency and
`uv sync --check --locked --offline` for environment consistency. These are
candidate probe commands, subject to the version and side-effect validation
above. Uv documents separate lock and environment checks in its
[CLI reference](https://docs.astral.sh/uv/reference/cli/). A nonzero exit alone
does not establish missing dependencies: it may mean an unsupported option,
missing interpreter, invalid configuration or incomplete offline cache.

Return typed findings with evidence and these distinct states:

| State | Meaning and action |
| --- | --- |
| Manager missing | The selected manager cannot be found; name it and link its installation instructions unless a supported installer exists. |
| Environment missing | The selected installation target is absent; offer installation. |
| Environment stale | A supported check established a mismatch for the selected dependency profile; offer synchronization. |
| Lock missing or stale | Dependency resolution is required; label the action as creating/updating the lockfile. |
| Ambiguous | Project evidence does not select one manager; offer a choice. |
| Unknown | A check could not establish readiness; show why in Details and offer Recheck. |
| Ready | The provider verified its supported readiness criteria. |

Do not infer readiness from `.venv` or `node_modules` existing. Do not infer
staleness from manifest modification times. A changed fingerprint invalidates
a previous check; it is not itself proof that installation is required. A
provider that can only detect a missing installation must report its limited
coverage rather than claim to verify every installed dependency.

## A small provider contract

Start with one new crate, provisionally `hick-project-env`, with a registry
and separate modules such as `providers/uv.rs`, `providers/npm.rs` and
`providers/pnpm.rs`. Its engine-facing interface has four responsibilities:

1. **Discover:** return owned projects, workspace membership and evidence of
   the manager selection.
2. **Inspect:** return typed readiness findings for a target and dependency
   profile, with files and environment metadata that invalidate the result.
3. **Plan action:** return executable, argument vector, working directory,
   environment overrides and effects such as lockfile updates or removal of
   installed packages.
4. **Describe environment:** return the selected interpreter, executable
   directories and relevant launch configuration for integrations to consume.

The shared host owns file access boundaries, probe subprocesses, cancellation,
timeouts, scheduling, terminal sessions and serialization. Providers request
these through host services; the React UI contains no manager-specific logic.
This permits adding a provider without changing the parser, editor, generic
UI or server routing. Extract shared project-path logic from LSP discovery
only where the semantics actually agree.

Register compiled first-party providers initially. This gives modularity
without creating a dynamic library ABI, plugin marketplace or script runtime.
If independently distributed plugins become necessary, expose the same
operations through versioned JSON over stdio and use the host-controlled
probe/action services. Loading third-party executable code requires its own
trust design; modularity now does not require solving distribution now.

## Offer an action whose effects are visible

Use one workspace status response/event and one generic action route. A
finding carries provider, project, target, profile, evidence, revision and
action IDs. The client submits an action ID and revision; the server
revalidates and builds the command, rather than accepting arbitrary shell
text. Show the command, directory and material effects beside the button.

For a current uv lockfile, prefer `uv sync --locked`: ordinary `uv sync` can
update the lockfile, and exact syncing can remove undeclared packages. Uv
also syncs automatically during `uv run`, so a missing environment is not
necessarily a blocker for that command. Editor intelligence may still need
the environment prepared. These behaviors are documented in
[Locking and syncing](https://docs.astral.sh/uv/concepts/projects/sync/).

For pnpm with a current lockfile, the candidate repair is
`pnpm install --frozen-lockfile`, which refuses a manifest/lock mismatch.
Respect workspace scope and manager configuration. See
[pnpm install](https://pnpm.io/cli/install).

For npm with a current lockfile, `npm ci` is a clean-install action: it
removes existing `node_modules`. Name that effect explicitly and avoid
presenting it as an incremental sync. `npm install` can instead be offered
as a separate action that may update the lockfile. See
[npm ci](https://docs.npmjs.com/cli/commands/npm-ci/).

Run the chosen command through an existing terminal session, with output,
cancellation and its exit status retained. Serialize repairs per installation
target. Package-manager actions may download packages and execute build or
lifecycle code; initiating the visible action is the user's decision. Do not
add a confirmation modal for every sync. Reinspect after completion; an exit
code of zero alone is not evidence that every dependency profile is ready.

## Keep the message quiet and useful

Publish one finding per project, execution target and selected dependency
profile. Show it in an expandable workspace environment section of Problems,
with a contextual notice when opening an affected file or invoking an action.
Keep it separate from LSP source diagnostics; do not attach hundreds of
invented import errors or invent a source range for a workspace problem.

Discover on open; prioritize the active project. Debounce checks after
manifest/lockfile changes, branch switches, relevant environment metadata
changes and repair completion. Recheck on focus or explicit request to catch
external installs without recursively watching every dependency file.
Coalesce events while a repair runs. Dismissal applies to the finding's
evidence revision so repeated checks do not continually reopen a notice.

Known readiness failures can explain an affected test/debug action before
it starts. Unknown checks stay advisory. Editing, reading and weaving remain
available, and opening a folder never starts an installation.

## Readiness belongs to the thing that will run

Tag each finding with its execution target: host tools, a particular executor
container, or another machine. A host virtualenv being ready does not prove a
cell in a scratch directory, sandbox or Docker image can use it. The initial
feature covers host IDE tools and project tests. Cell readiness continues
through `<hick:needs>` and the executor until providers can inspect that exact
target. Never offer a host sync as a proven fix for a container failure.

After synchronization, LSP, DAP and test launchers should consume the selected
environment where supported. Installing into `.venv` while continuing to run
tests with system Python leaves the original user problem unresolved. Preserve
existing explicit launch choices; restart or refresh affected language services
only when needed. Dependency installation records no `.hick` execution evidence
and grants no additional cell permissions. Mobile only displays applicable
information; it offers no execution action.

## Build in this order

1. Implement the registry, uv provider and host environment notice/action.
   Prove missing environment, stale environment, stale lock, missing manager,
   unsupported checks and offline uncertainty produce different messages.
2. Wire the selected Python environment into affected LSP, debug and test
   paths. Prove external synchronization clears the notice too.
3. Add npm and pnpm through the same interface. Test nested independent
   projects, shared workspace roots, conflicting lockfiles and manager pins.
4. Add other ecosystems when their native checks and repair semantics are
   verified; add independently distributed plugins only when there is a user
   for that distribution mechanism.

The essential end-to-end test opens a fresh uv project, observes the notice,
presses Sync, watches the real terminal, sees the notice clear and runs a test
using a dependency from the selected environment. Also verify that probes
leave tracked files and environment state unchanged, a failed or cancelled
repair retains its output, and a host success never marks a cell target ready.
