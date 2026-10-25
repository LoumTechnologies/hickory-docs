<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/agent/the-agent-sees-the-open-editors.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `95de37d6ead137366e8f26ae0e1173f2b8b83a5aa4e00c04848b37ba95db147d`.

<book:entry path="docs/guarantees/agent/the-agent-sees-the-open-editors.md">
> 2026-10-18 issue #52 amendment: complete open buffers remain edit snapshots.
> Workspace model input now uses explicit selected reading scope; full text is
> not automatically sent. The contract below describes legacy clients without
> a projection. See [selected reading scope](expanded-reading-projection.md).

# The agent sees the open editors

> 2026-10-18 amendment: the built-in loop is Python, not Rust. Branch data and
> evidence persist through the existing session/UI; Python constructs model
> context. Hosted completion requires a current accepted changeset and reports
> runtime checks unrun. Old Rust-runtime evidence below is historical. See
> [the migration audit](../audits/2026-10-18-python-agent-ownership.md).


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
appear in the answer for the built-in agent. ACP agents instead use live-buffer
editing tools and the conversation review policy described in
`conversation-edits-use-the-client-review-policy.md`. Context snapshots are recorded as inert JSON in the `.md`
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
  WebSocket streaming, and fresh context on a continuing turn. It starts with
  `prepare_without_folder` and an empty document index, matching an initial
  untitled window rather than depending on a saved primary document.
- Additional checks: `workspace_agent_reads_a_folder_without_a_primary_document_and_recovers`
  covers folder reads and server restart hydration.
  `serve_acp::acp_connects_to_workspace_with_untitled_context_and_resumes`
  covers ACP context recording and restart. Desktop `serves_one_origin` covers
  a real folderless window server with editor APIs and `folder_open: false`.
- Caveats: no live paid-provider call or native desktop interaction is needed for
  these tests. ACP adapter filesystem tools retain their own permissions and
  scope; context delivery does not grant them additional filesystem access.

Amended 2026-10-07: the recorded editor snapshots yield context links to their
named files and supplied line ranges. They include unsaved bytes, identify
snapshot input explicitly, and do not claim that the current disk or editor
still has those bytes. Untitled buffers yield no fictitious file target.
`session_receipts::tests::both_recorders_link_supplied_buffers_using_explicit_untitled_identities`
checks both native and ACP envelopes. `agent_context::describe` asks agents to
cite supporting passages and acknowledge missing support for workspace claims.

Untitled buffers with recorded ids now have editor-only provenance addresses
(`hickory-buffer://<window>/<id>`), and the prompt specifies that citation syntax. Their
context connects to final answer paragraphs even without a model citation.
They remain unsaved; these addresses are not filesystem paths.
</book:entry>

</book:doc>
