# Use the Python agent in Hickory or a terminal

For Hickory contributors testing the experimental Python agent.

The experimental `hickory-agent` CLI runs a LangGraph worker in a separate
process. Hickory confines that process with Seatbelt on macOS or Bubblewrap on
Linux. Model-written Python runs in Monty inside that confinement. Rust keeps
the provider credentials, conversation, workspace tools and edit approvals.
Windows currently refuses this runtime. There is no unconfined fallback.

From the repository root, install the locked environment and run its tests:

```sh
just test-python-agent
```

`uv` installs the pinned Python version, LangGraph and Monty into
`apps/agent/.venv`. The package must be installed normally: the sandbox cannot
import an editable checkout. Linux also needs a working `bwrap` with user
namespaces enabled. The tests exercise the real process without buying model
calls.

## Use it in the UI

1. Open **Settings → Agents**. Set **Python executable** to the absolute path
   of `apps/agent/.venv/bin/python`, then click **Save Python runtime**.
   Hickory starts the confined worker to check the installation before saving.
2. Add a provider API key in Settings if you have not already.
3. Open the **Agent** pane, open its settings, and choose
   **Hickory Python (experimental)**. Choose the provider and model there.
   Switching agents starts a new thread.
4. Send a task. Each edit opens a comparison; choose **Accept change** or
   **Reject change**. Auto-accept is unavailable for this runtime. **Stop** also
   cancels a pending review without applying it.

Tool activity and usage appear live. Provider answers are collected rather than
streamed token by token. Conversations keep their turn tree, backend, model and
session file when the UI or engine restarts. Rewind selects the prior branch's
prompts and answers for a fresh worker; it does not undo previous edits.
Unsaved editor text is supplied as context. The current tool bridge edits saved
workspace documents; save an untitled note before asking the agent to edit it.

`just test-python-agent` includes real confined-worker HTTP/WS tests and a
Chromium flow covering setup, selection, review, reload and Stop. For the browser
flow alone after setup, run `just test-python-agent-ui`. The browser harness
uses a disposable workspace and scripted model; it makes no paid calls.

## Use it from a terminal

Use a provider key already configured in Hickory Settings or the provider's
environment variable, then run:

```sh
cargo run -p hickory-cli --bin hickory-agent -- \
  --python "$PWD/apps/agent/.venv/bin/python" \
  --dir /absolute/path/to/notes \
  'Read the documents, record consequential assumptions, and propose a new summary.'
```

Add `--doc notes.md` to select a default document. `--provider`, `--model` and
`--max-turns` select the provider, model and turn budget. Once the binary is
built, it can be invoked directly as `target/debug/hickory-agent`.
The same supervisor is available through `hick agent --runtime python
--python /absolute/path/to/venv/bin/python 'Your task'`.

The agent calls Rust tools through its private stdio pipe. A Python snippet
can call `tool("read_doc", {"doc": "notes.md"})`; it cannot open the workspace
directly, launch a shell or connect to a provider. Snippets can author documents
containing the project's language. Python is the execution restriction, not a
restriction on the language of files the agent proposes.

Each edit displays a diff and asks you to type `approve`. This permission comes
from the supervisor's terminal, separate from the worker's stdin. Piped input
cannot approve edits. Existing and unknown authorship receive the same
protection. Acceptance retains the recorded agent proposal; it does not establish
human authorship. A source change detected during review invalidates approval.
Ctrl-C also cancels a pending review without applying its proposal.

The session is written incrementally to `sessions/python-agent-*.md`. Keep or
export it if you want to debug the work later: ordinary Hickory projects ignore
`sessions/`. It retains model requests and responses, Python snippets, actual
tool receipts, assumptions, proposals, approval decisions and failures. Worker
observations are labeled separately from Rust-observed results. The existing
assumption audit includes these records.

This is an opt-in runtime in both the UI and CLI. The default agent remains
Rust. The worker environment is not bundled into releases.
Provider responses are collected rather than streamed; reasoning is not captured
by this path. The existing `verify` and debugger tools are withheld because they
can write derived files without an exact edit preview. The UI and CLI label completion
as **checks unrun**, with the model summary kept apart from verification. Run the appropriate project checks separately;
a model's summary cannot establish their result.

Monty limits memory, execution time, recursion and host callbacks. Rust also
limits protocol frames, total tool requests, context size and overall runtime.
These bounds do not promise protection from every native interpreter or OS
vulnerability. Trusted runtime directories and installed dependencies are readable
to the worker; keep credentials out of them. macOS permits filesystem metadata
and the root directory listing needed for process startup.

The design decisions and remaining authorship work are recorded in
[the runtime specification](../specs/freeform/python-agent-runtime.md).
