<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Inspect the checks that actually ran

For: reviewers distinguishing executed witnesses from retained requirements and historical verification claims.

Seventeen focused witnesses passed against the integrated generated source on 2026-10-25: eight Rust checks and nine Vitest assertions. Each invocation was required to execute exactly one named passing case. Other cases in the same frontend file were skipped, not reported as covered.

| Witness | Requirement | Source |
|---|---|---|
| `closure_constructs_atomic_endpoints_and_can_be_reorganized` | `literate-projects/markdown-constructs-the-complete-project.md` | [project.rs](../mechanisms/crates/hick-lang/tests/project.rs.md#output-005c1e9344c7) |
| `refuses_unordered_transforms_hidden_inputs_and_processes` | `literate-projects/markdown-constructs-the-complete-project.md` | [project.rs](../mechanisms/crates/hick-lang/tests/project.rs.md#output-005c1e9344c7) |
| `inventory_checks_binary_modes_symlinks_and_portable_paths` | `literate-projects/complete-inventory-is-byte-exact.md` | [project.rs](../mechanisms/crates/hick-lang/tests/project.rs.md#output-005c1e9344c7) |
| `patch_preserves_bom_crlf_unicode_and_untouched_origins` | `literate-projects/replay-is-exact-and-atomic.md` | [project.rs](../mechanisms/crates/hick-lang/tests/project.rs.md#output-005c1e9344c7) |
| `newline_markers_add_delete_rename_and_failure_are_atomic` | `literate-projects/replay-is-exact-and-atomic.md` | [project.rs](../mechanisms/crates/hick-lang/tests/project.rs.md#output-005c1e9344c7) |
| `maps_edit_within_one_span` | `execution/output-lineage-round-trips-byte-for-byte.md` | [lib.rs](../mechanisms/crates/hickory-lineage/src/lib.rs.md#output-bc43b0352ff3) |
| `rejects_synthetic_overlap_with_offending_range` | `execution/output-lineage-round-trips-byte-for-byte.md` | [lib.rs](../mechanisms/crates/hickory-lineage/src/lib.rs.md#output-bc43b0352ff3) |
| `parse_text_content_with_special_chars` | `language/prefix-only-parsing-no-escaping.md` | [tests.rs](../mechanisms/crates/hick-lang/src/lib/tests/tests.rs.md#output-e91f067d75eb) |
| `waits for an unfinished recovery write and never rewrites explicitly discarded work` | `authoring/unsaved-work-survives-closing-the-app.md` | [drafts.lifecycle.test.tsx](../guarantees/authoring/unsaved-work-survives-closing-the-app/contract.md#output-06cfd4483cf0) |
| `keeps an inactive view open when its attempted Save fails` | `authoring/unsaved-work-survives-closing-the-app.md` | [useUnsavedLifecycle.test.tsx](../mechanisms/apps/web/src/views/useUnsavedLifecycle.ts.md#output-18ff98d49ede) |
| `serializes overlapping saves so each diff sees the previous result` | `authoring/an-output-edit-lands-in-its-document.md` | [outputSave.test.ts](../mechanisms/apps/web/src/lib/outputSave.ts.md#output-5dd719c6fb43) |
| `does not discard generated edits typed while an earlier save is settling` | `authoring/an-output-edit-lands-in-its-document.md` | [outputSave.test.ts](../mechanisms/apps/web/src/lib/outputSave.ts.md#output-5dd719c6fb43) |
| `installs a missing debugger and resumes without another click` | `debugging/a-missing-debugger-is-a-button.md` | [DebugStrip.test.tsx](../mechanisms/apps/web/src/debug/DebugStrip.tsx.md#output-1063c352c210) |
| `does not start a session after setup is dismissed` | `debugging/a-missing-debugger-is-a-button.md` | [DebugStrip.test.tsx](../mechanisms/apps/web/src/debug/DebugStrip.tsx.md#output-1063c352c210) |
| `preserves Unicode source coordinates, duplicate passages and omitted ranges while pins bypass folds` | `agent/expanded-reading-projection.md` | [contextProjection.test.ts](../guarantees/agent/expanded-reading-projection/contract.md#output-cd54a8e364bd) |
| `retains fold intent across editor unmounts and refuses stale source coordinates` | `agent/expanded-reading-projection.md` | [contextProjection.test.ts](../guarantees/agent/expanded-reading-projection/contract.md#output-cd54a8e364bd) |
| `disables both buttons while the document's run is in flight` | `editor-intelligence/the-toolbar-uses-words-an-ide-user-knows.md` | [workspaceTabs.test.tsx](../mechanisms/apps/web/src/views/workspaceTabs.tsx.md#output-5370130a675d) |

Exact commands, observed output and cited-source hashes are retained in the outer focused-results.json record. Cited-file hashes are not a complete transitive source/environment closure. Frontend checks used the integrated source with already installed local dependencies; temporary links and cache files were cleaned up.

## Interpret the results within their scope

These checks exercise core reconstruction and byte mapping; failed saves and discard ordering; serialized generated saves; selected-context coordinates and stale folds; debugger prerequisite setup and dismissal; and disabled actions during a run. They do not certify every clause, native window destruction, real adapter downloads, paid-provider behavior or released-package acceptance.

Three separate paragraph-ledger controls rejected a removed interval, a changed hash and an incorrect destination. This checks accounting of original text, not semantic equivalence of a replacement. LLM implementation review, executed checks and human acceptance remain separate.


## Agent pane refinement — 2026-10-31

For maintainers reviewing contextual approval and conversation space, the following
focused checks passed locally with the installed web dependencies and confined
Python runtime. These are execution observations, not acceptance of every agent
contract or a broad regression result.

- `npm --prefix apps/web test -- src/components/AgentChanges.test.tsx src/components/ChatDock.test.tsx src/components/ContextUpdates.test.tsx src/components/AcpControls.test.tsx src/views/WorkspaceChat.test.tsx`: 64 passing tests. These cover empty review/context UI, proposal decisions, agent self-authorization refusal, mandatory private transport, context revision/race handling, provider/model identity, approval reset and hidden context selection.
- `npm --prefix apps/web run typecheck`: passed.
- With `HICKORY_AGENT_TEST_PYTHON` set to `apps/agent/.venv/bin/python`, `cargo test -p hickory-cli --test serve_python_agent python_turn_requires_review_streams_receipts_and_rehydrates_its_branch -- --exact`: passed. The real confined runtime waits for the current decision, retains future approval in its session, applies a later proposal, restores the saved choice and starts a new conversation in Review.
- `cargo test -p hickory-cli --test serve_acp acp_untitled_edits_use_tools_and_wait_for_client_review -- --exact`: passed. Authorizing future edits at a pending review leaves the current proposal pending; Accept/Reject and consumed-decision refusal still work.
- `cargo test -p hickory-cli --lib private_acp_managed_tools_require_review_and_refuse_unscoped_delivery`: one passing safety case.
- `bash scripts/test-python-agent-ui.sh`: one passing Chromium flow using the scripted provider and real confined worker, through runtime configuration, edit review, reload/recovery and cancellation. The empty-pane screenshot was visually inspected. The stale assertion for unrun-check wording was updated to the supervisor's existing receipt text. Later ACP model compatibility and approval-reset wiring were checked in the focused component tests above.
- Source reconstruction, `behavior.py check`, `behavior.py audit` and `git diff --check` passed during this change. Reconstruction establishes source/output agreement; the audit retains the existing explicit explanation gaps.

Paid providers, other native hosts and broad regressions were not exercised. The
changed context gutter's decisions and lifecycle were exercised in component tests;
its changed-reference popup was not driven against a paid-provider browser session.

</book:doc>
