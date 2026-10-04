# Use Codex in Hickory Docs

For engineers who already use Codex or Claude and want the same agent inside
Hickory's document workspace.

Open a document, then open **Agent** from the Welcome page. Choose **Codex**
in the Agent selector. **Hickory (built-in)** is Hickory's own agent;
**Installed ACP agents** lists adapters detected on this machine. If Codex's
adapter is missing, choose **Codex — install adapter**, then click **Install Codex
adapter**. Installation requires Node.js and npm on this machine.

The Codex CLI and the Codex ACP adapter are separate executables. Having
`codex` installed alone does not make it an ACP agent. **Refresh agents** checks
again after an installation or a change in **Settings → Agents**; returning to
the window also refreshes detection. Custom configured ACP commands appear in
the same selector.

Switching agents starts a new thread and keeps previous conversations in
**Tree**. Your explicit choice is remembered in this browser for documents
without a conversation; reopening a conversation uses the agent that ran it.

An existing Codex login is reused. Otherwise choose **ChatGPT** to sign in
through Codex's browser flow, then return to Hickory. Hickory does not need a
provider API key for this connection. The adapter owns authentication.

Choose the model, reasoning effort, and access mode in the pane. These choices
come from the adapter, including the models available to your account. Send:

> Read this document and its generated code using the hick tools. Change the
> greeting from hello to welcome, then verify the result.

Choose **Document edits → Review** to inspect changes and click **Accept change**
or **Reject change**. Choose **Auto-accept** to apply them as they arrive. The
choice belongs to this conversation and is remembered when you reopen it;
new conversations start with Review. Change the setting between turns.

For a current note, including an untitled draft, simply ask:

> Change the heading of this document to “Meeting notes”.

The agent receives live editor tools. A proposed change opens beside your work
in a document review tab, with additions and removals shown in the literate
editor. Read it there, then choose **Accept change** or **Reject change**.
Accepting an untitled edit updates its buffer;
use Save when you want a file. Typing during review can make a proposal stale,
in which case it is refused and the agent needs to read the note again.

The Agent pane is a live literate document. The agent's text is protected while
it writes, and your editable **Your response** region is at the bottom. Enter
adds a line; **Send** or Cmd/Ctrl+Enter submits it. The pane streams the answer
and folds reasoning separately. Tool details and
permission choices appear while it works. **Stop** cancels the turn; a process
that ignores cancellation is terminated after three seconds. **Reconnect**
recovers a disconnected adapter. **New thread** starts a separate conversation.

Each conversation is saved under the workspace's `sessions/` folder as a
Hickory session document. Open the session link to inspect the conversation,
reasoning, tool activity, permission decisions, and Hickory read/write evidence.
Returning to a saved conversation after restarting loads the adapter's own
session when it supports loading. An adapter's saved context must still exist
on this machine; the Hickory record remains readable independently.

Codex supports **rewind here** and `/rewind`: the next turn forks the adapter's
context at the selected recorded response. Rewind preserves the abandoned
branch in the same session file. Other adapters offer rewind only when Hickory
can identify an exact fork point. **New thread** works for every adapter.

## Claude and custom agents

Choose **Claude Agent** to install its ACP adapter. It uses its own account
and permission choices. An adapter that requires terminal authentication must
be signed in through its own CLI before reconnecting.

**Settings → Agents** stores executable commands and argument arrays locally.
For example, add this entry to the displayed list:

```json
{
  "id": "my-agent",
  "name": "My ACP agent",
  "command": "/absolute/path/to/my-agent",
  "args": ["--acp"]
}
```

Save, then choose the agent in the pane. Commands are launched directly in the
workspace directory. For a Node script, use the Node executable as `command`
and the script path as the first argument. Reconnect after changing a command.

## What the connection records

Hickory automatically gives the adapter its `hick` MCP tools. Code edits made
with `edit_output` follow lineage back into the source document and reach the
open editor. The adapter's ordinary commands run under its own sandbox and
access mode. They do not run in Hickory's cell executor.

Use Hickory's tools for document and generated-code edits. Native shell edits
are reported only to the extent the adapter emits activity; Hickory cannot
invent lineage or tool evidence for an unreported edit. Generic ACP file writes
refuse generated files and direct the agent to `edit_output`.

Streamed text is checkpointed for interrupted-turn recovery. These checkpoints
are removed after the complete assistant message is saved. ACP activity stays
recorded as evidence; shell commands are never converted into executable cells.

The pane does not estimate ACP subscription spend. Billing and any usage limits
belong to the agent's provider. The built-in **Hickory** agent continues to use
Hickory's provider-key settings.

## Native workspace preview on macOS

A build containing **Hickory Workspace** can route an agent's native file
operations through Hickory's document engine. Enable its File System Extension
in System Settings → General → Login Items & Extensions, then enable the native
workspace for that agent in Hickory Settings, save, and start a new thread.
It uses Apple's user-space FSKit; no macFUSE download or Recovery-mode setup
is required. This preview currently targets macOS 26 or later.

Generated-file saves are checked and carried back to their document; stale,
invalid, or noneditable saves are refused. Native file accesses are recorded
separately from text shown to the model. The mount does not restrict the agent
from explicitly accessing other host paths. Some tools need filesystem features
this preview does not yet implement. Signed activation and a mounted Codex
workflow remain unverified; ordinary ACP mode is available in the meantime.

In **History** or **Story**, choose **Read commit** to read its full message
above literate comparisons of the changed files. This is a read-only view;
opening it leaves your current files and branch in place.
