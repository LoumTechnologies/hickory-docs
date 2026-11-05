<maintain:doc xmlns:maintain="http://www.hickorydocs.com/1.0">
# Maintain AGENTS.md

For: maintainers locating this exact input or preserving the migration evidence.

The entry below gives agents the design principles and editing constraints they
need when changing this repository. Keep these instructions brief and actionable.

<maintain:entry path="AGENTS.md">
Post-commit checks are very thorough. Do not watch the logs; it uses too many tokens. Tell the user how to monitor / check status on such things.

Only run focused behavior and safety tests; leave broad regression tests and all regression-check status monitoring to the user.

Before editing, verify this checkout's Git hooks: `git config --get core.hooksPath`
must resolve to `.githooks`, `hickory.projectEntry` must be `project.md`, and
`hickory.projectVerifier` must name an existing executable. Both `.githooks/pre-commit`
and `.githooks/post-commit` must be executable. Run `just literate-bootstrap` to
build the tools and install missing configuration. Recheck before committing;
never bypass a failing hook. Tracked hook files alone do not install hooks.

After pulling or switching revisions, before source edits, run `just literate-sync`.
It adopts the local journal only when current source exactly reconstructs disk,
including new/deleted outputs and file metadata. It preserves capture history.
If it refuses, inspect `behavior.py edits` and reconcile actual differences;
do not delete the journal, ignore captures, or accept an arbitrary old capture.
The hook checks staged consistency; it does not synchronize journal metadata.

Maintain code and tests in the owning entries under `.hickory-source/`, starting
from `project.md`. `hickory-dogfood/behavior-map.json` maps output paths to source
pages. Regenerate with `python3 hickory-dogfood/tools/behavior.py generate`, then
run `check` and `audit`. See `docs/developers/literate-workflow.md` for bootstrap.
Commit Markdown changes and their generated outputs together. Do not rerun the
one-time capture/reconciliation scripts or edit generated files alone.

Current requirements are the contract pages linked from `project.md`.
`docs/guarantees/` and `docs/specs/freeform/` retain captured documents as history;
the canonical contract pages distinguish retained, reconciled and historical text.

Always use the audience-first-literate-docs skill when modifying code or tests,
and for all maintained source changes. Interleave real
explanations, implementation and tests at reasoning boundaries. Current code uses
copy fragments and explicit file/paste assembly; raw entries preserve literal
assets and embedded document syntax. Shared code lives under named mechanisms.
Use `behavior.py locate <output-path>` to find its construction and every fragment.
A narrative owner is not proof of adequate explanation: inspect the explicit gaps
in `hickory-dogfood/narrative-map.json` and do not invent coverage or guarantees.

## UI responsiveness

Prefer a snappy, eventually consistent UI. Open every file — literate or plain,
generated or non-generated — from its available text immediately. Load parsing,
provenance, ownership, language services, diagnostics and other capabilities
independently in the background, and attach them to the existing editor as they
become available. A slow or failed capability must not hide readable content or
block unrelated interaction. Preserve selection, scroll, undo history and unsaved
edits as capabilities arrive; reject obsolete asynchronous replies. Keep write,
permission and source-revision checks authoritative: eventual consistency of the
display does not authorize unsafe edits or claim a save succeeded.

## UI feel

Keep the UI compact and spatially stable: revealing controls must not disturb
the content the user is reading.

- Keep contextual actions beside the folder or line they belong to, with gutter
  icons arranged in a single vertical column on the right.
- Anchor each line's gutter icons to that line. A stack may use the space below
  it, but must never push down the icon center for a later line.
- When a stack would collide with the next line's icons, reserve a slot for a
  clickable `⋯` control. It opens a small floating popover containing the remaining
  icons in a grid, with rows and columns as needed.
- Opening that popover must not reflow, resize or move adjacent content or other
  gutter icons. Apply this same overflow behavior to gutter icons everywhere in
  the app, including Files, document actions and debugger controls.

@.instructions/simple-composable-design.md
@.instructions/config-and-environments.md
@.instructions/continuous-delivery-downloadable.md
@.instructions/continuous-integration.md
@.instructions/dev-environment.md
@.instructions/documentation-layout.md
@.instructions/framework-agnostic-system-tests.md
@.instructions/github-issues.md
@.instructions/just.md
@.instructions/one-man-team.md
@.instructions/pre-commit-ci-parity.md
@.instructions/pre-launch.md
@.instructions/semble.md
@.instructions/specification-levels.md
@.instructions/third-party-integration-mocking.md
@.instructions/user-facing-errors.md
</maintain:entry>
</maintain:doc>
