# The agent sees the open editors

Given a window containing untitled notes, saved documents, generated files, or
ordinary text files, with or without an open folder,
when the user sends an agent message,
then the pane is available and includes the current contents of every open
editor tab, including unsaved edits and buffers whose files no longer exist.
Untitled documents have names and no disk path. The last focused editor is
identified even while focus is in the agent pane.

The conversation belongs to the workspace and stays the same while the user
switches editors or saves an untitled note. Each message refreshes its editor
snapshots. The pane names the included editors and open folder. Only an explicitly
opened folder is folder context; a single file's parent is an execution base.
The built-in agent can list and read that folder through the existing `read_file`
tool, without requiring a primary document. Existing document editing tools are
available for a focused indexed document whose snapshot matches disk.

Sending context does not save a buffer, manufacture a filename for an untitled
note, or overwrite disk with unsaved text. Suggested changes to unsaved buffers
appear in the answer. Context snapshots are recorded as inert JSON in the `.md`
conversation record, for both the built-in agent and ACP adapters.

---

Last LLM verification:

- Date: 2026-10-03
- Reviewer: Codex
- Result: verified by implementation review and automated regression checks.
- Evidence: `WorkspaceChat.editorContext`, `PlainFilePane.onSource`,
  `WorkspaceView`'s buffer references and focus tracking; generated editor
  snapshot callbacks in `GeneratedFileView` and `GeneratedTabBody`; `agent_context::describe`
  and `primary`; `agent::run_turn`; ACP `Client::prompt`; `AgentConfig`'s
  `session_subject` and `folder_context`; `tools/read_file.rs`; workspace socket
  run subscriptions independent of document rooms.
- Tests: `WorkspaceChat.test.tsx` covers the startup composer, fresh unsaved
  contents, outside files, inactive tabs, and missing files.
  `serve_agent::workspace_agent_receives_unsaved_buffers_without_saving_them`
  checks actual model input, disk preservation, recorded context, workspace
  WebSocket streaming, and fresh context on a continuing turn.
- Additional checks: `workspace_agent_reads_a_folder_without_a_primary_document_and_recovers`
  covers folder reads and server restart hydration.
  `serve_acp::acp_connects_to_workspace_with_untitled_context_and_resumes`
  covers ACP context recording and restart. Desktop `serves_one_origin` covers
  a real folderless window server with editor APIs and `folder_open: false`.
- Caveats: no live paid-provider call or native desktop interaction is needed for
  these tests. ACP adapter filesystem tools retain their own permissions and
  scope; context delivery does not grant them additional filesystem access.
