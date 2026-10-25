<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/specs/freeform/python-agent-runtime.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `e2608e650caa64708221c5177ad57f6a20668401aa69767a8264e157f4f5f113`.

<book:entry path="docs/specs/freeform/python-agent-runtime.md">
# Python owns the built-in agent

Decision of record, 2026-10-18. Supersedes the 2026-10-08 host-controlled worker
and the later opt-in UI integration. Requirement: remove the built-in Rust
agent, make Python independently runnable, preserve review, confinement,
recording and verified changeset completion. LangGraph is retained by the user's
explicit clarification; the graph is defined entirely in Python.

`apps/agent/src/hickory_agent_worker/core.py` owns prompts, conversation/history,
context inclusion, model requests, retry limits, actions, iteration limits and
completion decisions. It accepts model/tool clients, an executor, recorder and
cancellation callback. It has no process or UI imports. `actions.py` defines the
model JSON response format. `runtime.py` interprets generated snippets in Monty;
it never passes generated text to CPython eval/exec or a shell.

`cli.py` supplies a standalone development entry point, replay and HTTP model
clients, read-only fixture tools and trusted client factories. Temporary
workspaces are the default. These clients do not claim Hickory confinement or
changeset verification. A fake client's acceptance is only development data.

`hosted.py` adapts the core to Hickory's bounded stdio capability protocol:

| Capability | Python supplies | Rust provides |
|---|---|---|
| inference | Complete context manifest with ordered message parts | Reference/coverage validation, delivery receipts, credentialed provider transport, streaming and usage |
| tool | Name and arguments | Allowlist, exact preview, approval, freshness check, application and receipts |
| record | Declared action/observation/error/prepared context | Worker-reported session entry |
| complete | Summary | Independent accepted-artifact freshness check |

Rust sends initial task/history/editor data and tool schemas. Its stored model
request envelopes are historical observations, not a second conversation. It
does not parse model actions, inject prompts, replay messages, retry inference,
or issue completion repair prompts. Protocol size, call count, tool count,
provider timeout and process lifetime limits are resource restrictions, not
agent orchestration. Cancellation drops an in-flight inference/review future
and kills the Python process group. Partial provider usage received before a
cancelled inference is not currently emitted as a completed usage receipt.

There is no Rust runtime selector or fallback. Both CLI entry points, document
agent cells and the default Agent pane launch Python. ACP remains independent.
Historical sessions retain their recorded backend; start a new Python thread
instead of resuming the removed backend. Shared provider clients, tool engine,
XML tool/session readers, lineage and offline economics reporting remain in
`hickory-agent`; its name does not imply an agent loop. The old economics `run`
command is removed; historical report/count tools survive.

The hosted interpreter still uses Seatbelt or Bubblewrap and fails closed.
The installed runtime and dependencies are trusted readable roots; workspace
bytes and credentials are not mounted. Monty retains bounded memory, time,
recursion, callback count and output. Windows hosted support is unimplemented.
No security proof against OS/native interpreter vulnerabilities is claimed.

Every exposed workspace mutation uses Hickory proposal/review/application.
Read/edit weaving is in memory. Generic writes, shell, verify and debug execution
are unavailable, including indirect execution through source edits. Audit tools
write declared records only into the host session. Review-time source changes
invalidate approval. Outside processes can still race final filesystem writes;
there is no global atomic transaction or complete authenticated writer ledger.
Acceptance never erases agent ancestry or proves human composition.

Hickory freezes A before work. The UI uses its existing workspace/editor capture;
standalone hosted CLI runs freeze the selected disk workspace. Python calls
changeset_status and submit_changeset and repairs diagnostics itself. Rust
verifies source-output equality over captured A and actual B without executing
commands. Completion requires a current accepted artifact and rechecks it after
the worker exits. The UI rechecks completed capture before reporting success and
uses the existing document/patch rendering. A model completion or a worker's
claimed check result never creates evidence. Iteration exhaustion is a failed,
incomplete turn with its actual work retained, not a successful handoff.

These choices and their witnesses are in
[the migration audit](../../guarantees/audits/2026-10-18-python-agent-ownership.md).
Run instructions and platform limits are in
[the developer guide](../../developers/python-agent.md).
</book:entry>

</book:doc>
