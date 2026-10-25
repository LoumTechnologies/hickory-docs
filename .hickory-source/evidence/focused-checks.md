<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# Inspect the checks that actually ran

For: reviewers distinguishing executed witnesses from retained requirements and historical verification claims.

Seventeen focused witnesses passed against the integrated generated source on 2026-10-25: eight Rust checks and nine Vitest assertions. Each invocation was required to execute exactly one named passing case. Other cases in the same frontend file were skipped, not reported as covered.

| Witness | Requirement | Source |
|---|---|---|
| `closure_constructs_atomic_endpoints_and_can_be_reorganized` | `literate-projects/markdown-constructs-the-complete-project.md` | [project.rs](../shared/files/crates/hick-lang/tests/project.rs.md) |
| `refuses_unordered_transforms_hidden_inputs_and_processes` | `literate-projects/markdown-constructs-the-complete-project.md` | [project.rs](../shared/files/crates/hick-lang/tests/project.rs.md) |
| `inventory_checks_binary_modes_symlinks_and_portable_paths` | `literate-projects/complete-inventory-is-byte-exact.md` | [project.rs](../shared/files/crates/hick-lang/tests/project.rs.md) |
| `patch_preserves_bom_crlf_unicode_and_untouched_origins` | `literate-projects/replay-is-exact-and-atomic.md` | [project.rs](../shared/files/crates/hick-lang/tests/project.rs.md) |
| `newline_markers_add_delete_rename_and_failure_are_atomic` | `literate-projects/replay-is-exact-and-atomic.md` | [project.rs](../shared/files/crates/hick-lang/tests/project.rs.md) |
| `maps_edit_within_one_span` | `execution/output-lineage-round-trips-byte-for-byte.md` | [lib.rs](../shared/files/crates/hickory-lineage/src/lib.rs.md) |
| `rejects_synthetic_overlap_with_offending_range` | `execution/output-lineage-round-trips-byte-for-byte.md` | [lib.rs](../shared/files/crates/hickory-lineage/src/lib.rs.md) |
| `parse_text_content_with_special_chars` | `language/prefix-only-parsing-no-escaping.md` | [tests.rs](../shared/files/crates/hick-lang/src/lib/tests/tests.rs.md) |
| `waits for an unfinished recovery write and never rewrites explicitly discarded work` | `authoring/unsaved-work-survives-closing-the-app.md` | [drafts.lifecycle.test.tsx](../guarantees/authoring/unsaved-work-survives-closing-the-app/files/apps/web/src/lib/drafts.lifecycle.test.tsx.md) |
| `keeps an inactive view open when its attempted Save fails` | `authoring/unsaved-work-survives-closing-the-app.md` | [useUnsavedLifecycle.test.tsx](../shared/files/apps/web/src/views/useUnsavedLifecycle.test.tsx.md) |
| `serializes overlapping saves so each diff sees the previous result` | `authoring/an-output-edit-lands-in-its-document.md` | [outputSave.test.ts](../shared/files/apps/web/src/lib/outputSave.test.ts.md) |
| `does not discard generated edits typed while an earlier save is settling` | `authoring/an-output-edit-lands-in-its-document.md` | [outputSave.test.ts](../shared/files/apps/web/src/lib/outputSave.test.ts.md) |
| `installs a missing debugger and resumes without another click` | `debugging/a-missing-debugger-is-a-button.md` | [DebugStrip.test.tsx](../shared/files/apps/web/src/debug/DebugStrip.test.tsx.md) |
| `does not start a session after setup is dismissed` | `debugging/a-missing-debugger-is-a-button.md` | [DebugStrip.test.tsx](../shared/files/apps/web/src/debug/DebugStrip.test.tsx.md) |
| `preserves Unicode source coordinates, duplicate passages and omitted ranges while pins bypass folds` | `agent/expanded-reading-projection.md` | [contextProjection.test.ts](../guarantees/agent/expanded-reading-projection/files/apps/web/src/lib/contextProjection.test.ts.md) |
| `retains fold intent across editor unmounts and refuses stale source coordinates` | `agent/expanded-reading-projection.md` | [contextProjection.test.ts](../guarantees/agent/expanded-reading-projection/files/apps/web/src/lib/contextProjection.test.ts.md) |
| `disables both buttons while the document's run is in flight` | `editor-intelligence/the-toolbar-uses-words-an-ide-user-knows.md` | [workspaceTabs.test.tsx](../shared/files/apps/web/src/views/workspaceTabs.test.tsx.md) |

Exact commands, observed output and cited-source hashes are retained in the outer focused-results.json record. Cited-file hashes are not a complete transitive source/environment closure. Frontend checks used the integrated source with already installed local dependencies; temporary links and cache files were cleaned up.

## Interpret the results within their scope

These checks exercise core reconstruction and byte mapping; failed saves and discard ordering; serialized generated saves; selected-context coordinates and stale folds; debugger prerequisite setup and dismissal; and disabled actions during a run. They do not certify every clause, native window destruction, real adapter downloads, paid-provider behavior or released-package acceptance.

Three separate paragraph-ledger controls rejected a removed interval, a changed hash and an incorrect destination. This checks accounting of original text, not semantic equivalence of a replacement. LLM implementation review, executed checks and human acceptance remain separate.

</book:doc>
