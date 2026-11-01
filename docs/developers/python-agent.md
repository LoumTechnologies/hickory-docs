# Run and change the Python agent

For contributors developing the agent without starting Hickory, then connecting
that same agent to the desktop application or CLI.

Python owns the LangGraph loop, system instructions (`instructions.py`), branch history, context
selection, model messages, action processing, retries, iteration limit and
completion attempts. Hickory owns tools, exact edit proposals, approvals,
application, recording and independent changeset verification. There is one
built-in agent: Python. External ACP agents remain supported.

## Start independently

Install `uv`, then run from the repository root:

```sh
uv sync --project apps/agent --locked --no-editable
printf '[{"python":"tool(\"list_docs\", {})"},{"done":"Inspected the fixture"}]\n' > /tmp/hickory-responses.json
uv run --project apps/agent --locked --no-editable hickory-agent-dev \
  --responses /tmp/hickory-responses.json 'Inspect the development workspace'
uv run --project apps/agent --locked --no-editable python -m unittest discover -s apps/agent/tests -v
```

This creates an isolated temporary workspace and deletes it on exit. Add
`--workspace /absolute/path/to/fixtures` to read an existing fixture. The supplied
development tool client only lists and reads documents; unknown tools fail.
Development completion is explicitly unverified. CPython itself is unconfined
in this mode; Monty still interprets generated snippets with resource limits.

To call an OpenAI-compatible provider directly from Python, set `OPENAI_API_KEY`
and choose a model your endpoint supports (these calls can cost money):

```sh
uv run --project apps/agent --locked --no-editable hickory-agent-dev \
  --api-model "$MODEL" --base-url "$OPENAI_BASE_URL" \
  --workspace /absolute/path/to/fixtures 'Summarize the documents'
```

The default base URL is `https://api.openai.com/v1`. No Rust inference proxy is
involved in standalone mode. Missing keys, missing fixtures, malformed fake
responses, exhausted response lists and iteration exhaustion produce errors.

## Inject clients

`core.Agent(model, tools, execute, record, cancelled)` is the independent API.
`model.complete(messages)` returns raw text or `ModelResponse(text, finish_reason, metrics)`.
Use the response object to preserve token-limit status; truncated actions never execute.
`tools.call(name, arguments)` implements capabilities; `tools.complete(summary)`
returns an acceptance receipt. The executor interprets a snippet and returns an
observation. The recorder and cancellation callback are optional.

For a terminal experiment, put a trusted module on Python's import path with a
factory accepting a workspace path and returning `(model_client, tool_client)`:

```python
from hickory_agent_worker.cli import DevelopmentTools, ReplayModel

def clients(workspace):
    return ReplayModel([{"done": "Fixture answer"}]), DevelopmentTools(workspace)
```

```sh
PYTHONPATH="$PWD" uv run --project apps/agent --locked hickory-agent-dev \
  --factory experiment:clients 'Try the injected clients'
```

Factories are trusted development code, not model snippets. They can have all
the permissions of your terminal. An injected development receipt is never
Hickory verification evidence. The unit tests use the same interface to exercise
failed tools, rejected completion, retries, history and cancellation.

## Connect to Hickory

Desktop macOS builds bundle the installed runtime. In a source build, set
Settings → Agents → Python executable to the absolute path of
`apps/agent/.venv/bin/python`, then save. Saving probes the confined runtime and
Monty without calling a provider. Add a provider key in Settings and send a task
from the Agent pane. Each edit requires Accept change or Reject change; Stop
also cancels pending review. Python has no auto-accept mode.

From a terminal, run the same confined agent with Hickory tools:

```sh
cargo run -p hickory-cli --bin hickory-agent -- \
  --python "$PWD/apps/agent/.venv/bin/python" \
  --dir /absolute/path/to/notes 'Read the notes and propose a summary'
```

The equivalent command is `hick agent --python /absolute/path/to/python TASK`.
`--doc notes.md`, `--provider`, `--model` and `--max-turns` configure the run.
Document `<hick:agent>` cells use `HICKORY_AGENT_PYTHON` (or the bundled runtime),
the same supervisor and terminal review. CI baseline playback makes no model calls.
The old Rust runtime and `--runtime` selector are removed; old sessions remain
readable but their backend cannot be resumed as Rust. Start a Python thread.

## Follow the boundaries

`hosted.py` adapts the core to bounded JSON-lines stdio. Rust sends task data,
branch exchanges, editor snapshots and capability schemas. Python constructs the
conversation. Its inference client sends complete messages to a credential-holding
proxy; Rust translates them to the chosen provider and streams tokens, reasoning
and usage to the existing run channel. Rust never parses model actions, appends
observations to a second conversation, chooses retries or prompts for repair.
Recorded request envelopes are evidence snapshots, not conversation state.

The hosted process uses Seatbelt on macOS or Bubblewrap on Linux; Windows and
unavailable confinement fail closed. There is no unconfined fallback. Install
normally: confined workers cannot import an editable checkout. Runtime directories
are trusted readable inputs; do not place credentials in them. macOS permits
startup filesystem metadata and the root directory listing. This is OS process
confinement, not a VM or a native-interpreter security proof.

Model snippets have only Monty's `tool` callback, no workspace mounts, shell,
network, CPython execution or filesystem-write callback. The host independently
allowlists reads, searches, audit records, reviewed document edits, changeset
and attached-window UI tools. Direct shell, generic filesystem-write, verify and
debug tools remain absent from the confined snippet interface. When attached to
a window, ui_observe/ui_act expose registered host terminal/setup/run/debug/profile
and filesystem controls under the same local host policy as their human controls.
Standalone CLI runs have no window and do not advertise these UI tools.

Document edit tools still preview, independently approve and recheck exact edits.
Host process/filesystem actions can mutate the workspace without that document
preview; they retain actual UI receipts and their existing host checks instead.
Never use them to bypass edit review. Known terminal prompts require actual human
input, and the host rechecks recognized prompts before agent input. Prompt
recognition is best effort; this is not a same-user process sandbox. Arbitrary
outside editors can still race filesystem operations; cross-file changes are not
atomic. UI dispatch completion and source-output equality do not establish a
runtime pass; explicitly inspect actual source-bound job receipts.

Python decides when to call `changeset_status`, repair and `submit_changeset`,
then attempts completion. Hickory freezes A before work, verifies the supplied
literate artifact against actual B, and rechecks freshness at completion. The UI
uses its existing workspace/editor capture and shared artifact rendering. A bare
`done`, a forged observation or a claim of passing tests cannot establish
acceptance. Equality verification does not execute programs or verify explanation
accuracy. Runtime checks are separate from the completion equality gate. Python exhausts its configured iterations
with an error rather than reporting unverified work as successful completion.

Sessions retain actual tool receipts and approval decisions at the host boundary,
separately labeled from worker observations and model declarations. Branches,
provider/model identity and unsent responses use the existing persistence. Rewind
replays an earlier branch into a fresh Python conversation; it does not undo edits.
Unsaved editor text is context and part of capture, but the current Python tools
edit saved documents. Save untitled notes before requesting tool edits.

## Check the integration

```sh
just test-python-agent
just test-python-agent-ui
```

The first installs the locked environment and runs Python units, real confined
CLI/HTTP/WS tests, confinement checks and Chromium acceptance. Both use fake
models, temporary workspaces and no paid provider calls. Linux confinement needs
working `bwrap` and user namespaces. The runtime is bundled in macOS desktop
builds; CLI archives and other desktop platforms still need a configured runtime.

## Continue from an accepted construction

Successful completion now presents the accepted literate artifact as the answer.
Its explanatory order describes the construction, not the chronology of discovery.
The session retains model summaries, wrong turns, tool receipts and exact requests.
After a successful submission, the worker can continue from the accepted document
instead of its exploratory messages. The host rechecks freshness first; stale
acceptance cannot become a handoff. A trusted stage instruction then asks the
model to assess remaining requirements and finish, rather than restart discovery.
It grants no new authority and does not certify that the task is fulfilled. The original task and selected editor context
remain. Ancestry records condensation as an opaque derivation: byte equality does
not establish that every important lesson survived.

`HICKORY_AGENT_DOCUMENT_HANDOFF=false` disables this request replacement for paired
experiments. The final completion gate is unchanged. Previous accepted documents
are retained as assistant exchanges on continuation, with their historical scope.

## Run a bounded local experiment

The independent local client uses an installed loopback Ollama model, without a
cloud fallback or model download. It sets CPU execution, context allocation,
sampling and thinking explicitly and records Ollama's load, prompt and generation
timings. It refuses an obviously oversized load on a busy Linux host. The input
size estimate is a heuristic; inspect actual allocated context and reported usage.

For a read-only fixture smoke test:

```sh
PYTHONPATH="$PWD/apps/agent/src" apps/agent/.venv/bin/python -m hickory_agent_worker.cli \
  --ollama-model qwen3.5:latest --instruction-profile focused \
  --workspace /absolute/path/to/fixtures 'Read the documents and summarize them'
```

For the focused coding experiment, use a built `hick` binary:

```sh
PYTHONPATH="$PWD/apps/agent/src" apps/agent/.venv/bin/python -m hickory_agent_worker.local_experiment \
  --hick target/debug/hick --model qwen3.5:latest \
  --instruction-profile focused --output target/local-agent/my-run
```

The runner compares temperatures 1.0 and 0.2 with append and document-handoff
contexts on three small tasks. Use `--repetitions 2` to use seeds 42 and 43 and
reverse mode order in the second repetition. `--tasks ledger`, `--modes handoff`,
`--temperatures 0.2`, `--context-tokens 8192` and `--output-tokens 768` bound a
smaller run. A result directory resumes only with identical configuration;
interrupted attempts are preserved separately. Requests, replies, candidate
revisions, actual check receipts and `report.json` remain there. These are local
development fixtures, not production review or evidence of frontier-model parity.
Candidate code runs in Monty with no tool callback. Hickory reconstructs candidate
bytes in disposable directories. Public examples guide repair; held-out examples
are scored only after the agent stops. Input mutation is non-passing.

For this experiment's state, inspect its `report.json` and per-case `result.json`.
`ollama ps` shows residency and allocated context. Neither is a regression check.
Broad regressions and their status monitoring remain a separate maintainer task.

## Configure local inference through the hosted agent

The existing compatible client accepts `OPENAI_BASE_URL` as the **complete** URL,
for example `http://127.0.0.1:11434/v1/chat/completions`. Use provider `openai`, the
installed model id, and a nonempty local placeholder key. Launch Hickory with
these environment values; settings do not currently expose a local endpoint form.
The standalone development client's `--base-url` instead appends that route.
Native Ollama settings above apply to the independent client; hosted compatible
requests depend on the endpoint's supported fields and server context allocation.

Host controls (all optional):

| Variable | Default | Meaning |
| --- | --- | --- |
| `HICKORY_AGENT_INFERENCE_TIMEOUT_SECS` | 180 | Per-request wall time; 1–14400 seconds |
| `HICKORY_AGENT_RUN_TIMEOUT_SECS` | 3600 | Whole-run wall time; 1–86400 seconds |
| `HICKORY_AGENT_CONTEXT_BYTES` | 700000 | Explicit request byte guard; 1024–700000 |
| `HICKORY_AGENT_DOCUMENT_HANDOFF` | true | Replace exploration only after current acceptance |
| `HICKORY_AGENT_CONTEXT_LAYOUT` | structured | `interleaved` places revision-matched captured evidence beside selected source |
| `HICKORY_AGENT_INSTRUCTION_PROFILE` | full | `focused` selects shorter capability-based guidance; private predictions retain full policy |
| `HICKORY_AGENT_MAX_OUTPUT_TOKENS` | 16384 | Compatible-client output cap; 1–131072 |
| `HICKORY_AGENT_TEMPERATURE` | endpoint default | Optional finite value from 0 to 2 |
| `HICKORY_AGENT_TOP_P` | endpoint default | Optional finite value from 0 to 1 |
| `HICKORY_AGENT_TOP_K` | endpoint default | Optional integer from 1 to 1000 |
| `HICKORY_AGENT_REASONING_EFFORT` | endpoint default | `none`, `low`, `medium` or `high`, if supported |

Malformed host controls fail before inference. Larger output limits do not expand
Monty snippet or transport budgets. A token-limit response is retained as evidence
but no action in it executes; the agent is asked to split the unfinished step.
The focused profile is opt-in because small fixture results do not establish
coverage of all production tasks. Every profile retains independent tool approval
and host completion checks.

See [the document-loop and context-placement study](local-agent-study.md) for the separate hypotheses, experimental commands and limits.
