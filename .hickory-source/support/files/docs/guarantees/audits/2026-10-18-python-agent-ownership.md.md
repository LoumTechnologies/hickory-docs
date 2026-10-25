<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/audits/2026-10-18-python-agent-ownership.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `647366786aa94c7cc28e92f1029d0e1804a5b845e4c3c940437743b5eb4e7233`.

<book:entry path="docs/guarantees/audits/2026-10-18-python-agent-ownership.md">
# Built-in agent ownership migration

Originating requirement: the user requested complete removal of Rust agent
orchestration, independent Python execution, preservation of review, confinement,
recording, cancellation, UI and verified changesets. The user explicitly allowed
LangGraph to remain provided Python defines the loop.

Decisions: retain LangGraph and Monty; inject clients into a Python core; retain
Rust as capability authority. Standalone development is explicitly unconfined,
with read-only fixture tools by default. Hosted runtime guarantees do not apply
to trusted development factories. This distinguishes developer flexibility from
workspace authority without adding a new sandbox framework.

Removed entry points and behavior:

- `react_loop.rs`, its prompts, model-response orchestration, shell/Python script
  executor, Rust runtime CLI option and native UI picker/review route.
- Rust execution from `agent_command`, `serve/agent`, and `agent_cell_runner`;
  these now use the confined Python supervisor.
- Economics `run_experiment`, its `run` command and `tokens-run` recipe. Offline
  reports, count_tokens clients and historical run formats remain readable.
- Runtime-specific script/turn-budget/cache/stop/model-failure and direct script
  integration tests. Python core and real confined CLI/HTTP/WS/browser flows
  replace those entry points; shared document tools retain their own tests.

Retained shared capabilities: provider transport and keys, usage pricing,
document edit and verification tools used by MCP/ACP/manual workflows, lineage,
session/XML readers, event shapes, audit tools, shared editor and comparison
rendering, ACP adapters and protected workspace filesystem. No crate is solely
an orchestration crate after the extraction; shared dependencies remain used.
The `hickory-agent` crate now contains shared services, not a hidden fallback.

Consequential assumptions: an installed runtime is trusted; OS confinement and
Monty remain the two enforcement boundaries. A tool preview does not prove
semantic safety, authorship or absence of external races. A changeset artifact
proves output equality only. The host records actual operations and approval
inputs; Python observations and model reasoning remain declarations.

Behavior changes: Python owns retry and iteration policy. Exhaustion fails the
turn and retains records/work rather than making an extra Rust handoff call.
The built-in agent cannot run verify/debug/shell or publish derived outputs
without review. Native runtime verification remains explicit and unrun by agent
completion. These limits are not hidden behind a successful completion claim.
Python tools currently edit saved documents, while unsaved/untitled buffers are
context and part of the final capture. Complete per-byte authenticated authorship
and atomic multi-file writes are not implemented. Hosted Windows remains refused;
macOS desktop bundles the runtime, other distribution paths require setup.

Witnesses: Python unit tests exercise injectable clients, independently running
CLI, retry/history/action policy, cancellation before model/action, tool failures,
completion rejection/repair, Monty filesystem/subprocess/allocation/time escapes
and framing. Native Python tests challenge actual confinement, accepted/declined
and stale approval, unsupported tools and terminal cancellation. HTTP/WS tests
exercise review, restart/branch history, spend/events and cancellation. Chromium
uses the actual engine and worker with a scripted provider.

Follow-up inspection retained the former document-language crib and edit-routing
instructions in Python `instructions.py`, without the removed shell/verify policy.
The standalone tool catalogue now describes its actual read-only capabilities,
and prompt completion guidance is conditional on available changeset tools.

The create_doc preview/application audit found whitespace normalization differed
at the two boundaries. Previews now trim paths exactly as application does;
creation receipts read back actual bytes and require equality with the approved
preview, including newline normalization. Confined tests challenge padded paths,
actual write hashes, and executable document content that must remain unrun.

Executed validation is recorded below. Earlier guarantee verification notes
referencing deleted Rust modules are historical evidence, not verification of
this migration.

Human judgment: user approved the architecture direction and permitted
LangGraph; no human correctness or semantic review of final implementation is
inferred. This audit records identified assumptions, not proof of exhaustive
assumption discovery.

Validation completed locally on macOS (2026-10-18):

- `uv run --project apps/agent --locked --no-editable python -m unittest discover -s apps/agent/tests`: 10 passed. The documented `hickory-agent-dev --responses` invocation also completed in an isolated temporary workspace.
- `HICKORY_AGENT_TEST_PYTHON="$PWD/apps/agent/.venv/bin/python" CARGO_INCREMENTAL=0 cargo test -p hickory-cli --test python_agent --test serve_python_agent --test serve_agent --test agent_cell_vertex`: 20 passed.
- `CARGO_INCREMENTAL=0 scripts/test-python-agent-ui.sh`: Chromium acceptance passed.
- `CARGO_INCREMENTAL=0 cargo test -p hickory-cli --test serve_acp -- --test-threads=1`: 14 passed, 2 live-provider tests intentionally ignored.
- `CARGO_INCREMENTAL=0 cargo test -p hickory-agent -p hickory-executor-sandbox --lib --tests`: 127 passed.
- `CARGO_INCREMENTAL=0 cargo clippy -p hickory-agent -p hickory-cli --all-targets -- -D warnings`: passed.
- Web TypeScript and full unit suite: 247 files, 2230 tests passed. Formatting, shell syntax, and diff whitespace checks passed.

Initial broad-check attempts hit a full build disk and a web timeout; after
cleaning the isolated Cargo target, the complete reruns above passed. Hosted
Linux confinement was not exercised on this macOS machine; live paid-provider
coverage was not run. Source-output verification does not execute document
runtime checks. Standalone development clients are trusted, unconfined code
and do not provide hosted approval or completion guarantees.

The repository-wide file-length check remains red on intervening unrelated
changes: api/types.ts, Ribbons.tsx, WorkspaceView.tsx, hick-exec/dag.rs,
hick-lang/lib.rs, hick-literate/lib.rs, and serve/api.rs and serve/mod.rs.
This migration does not change those files or relax their baselines. Thorough
post-commit checks run separately; use `just checks-status` to inspect status.
</book:entry>

</book:doc>
