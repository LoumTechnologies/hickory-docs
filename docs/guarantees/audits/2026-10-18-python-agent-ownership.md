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

Executed checks and final dependency revision: recorded below when validation
finishes. Earlier guarantee verification notes referencing deleted Rust modules
are historical evidence, not verification of this migration.

Human judgment: user approved the architecture direction and permitted
LangGraph; no human correctness or semantic review of final implementation is
inferred. This audit records identified assumptions, not proof of exhaustive
assumption discovery.
