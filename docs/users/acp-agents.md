# Use Codex in Hickory Docs

For engineers who already use Codex or Claude and want the same agent inside
Hickory's document workspace.

Open a document, then open **Agent** from the Welcome page. Choose **Codex**
in the Agent selector. If its adapter is missing, click **Install Codex
adapter**. Installation requires Node.js and npm on this machine.

An existing Codex login is reused. Otherwise choose **ChatGPT** to sign in
through Codex's browser flow, then return to Hickory. Hickory does not need a
provider API key for this connection. The adapter owns authentication.

Choose the model, reasoning effort, and access mode in the pane. These choices
come from the adapter, including the models available to your account. Send:

> Read this document and its generated code using the hick tools. Change the
> greeting from hello to welcome, then verify the result.

The pane streams the answer and folds reasoning separately. Tool details and
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
