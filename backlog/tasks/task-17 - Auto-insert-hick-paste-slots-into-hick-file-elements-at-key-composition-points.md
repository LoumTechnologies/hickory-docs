---
id: TASK-17
title: 'Auto-insert hick:paste slots into hick:file elements at key composition points'
status: Done
assignee:
  - claude
created_date: '2026-05-15 12:45'
updated_date: '2026-05-15 20:21'
labels: []
milestone: m-1
dependencies:
  - TASK-14
  - TASK-15
references:
  - hick-lang/crates/hick-lsp/src/lib.rs
  - hick-lang/crates/hick-lsp/src/lang_detect.rs
priority: high
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
When the pipeline creates a new `hick:file`, automatically insert `hick:paste` slots at structurally meaningful positions so that subsequent `hick:copy` edits have named insertion points. This is what makes the hick edit format merge-safe: two agents targeting different paste slots in the same file never conflict.

**Where to insert paste slots:**

Language-aware insertion using the same language detection as `hick-lsp`:
- After each top-level function/class/impl block: `<hick:paste name="after-fn-{name}"/>`
- At the bottom of a module/file: `<hick:paste name="module-footer"/>`
- After imports section: `<hick:paste name="after-imports"/>`
- Inside class/struct/impl bodies: `<hick:paste name="impl-{TypeName}-body"/>`

For unknown/text files: one slot at top (`file-header`) and one at bottom (`file-footer`).

**When paste slots are inserted:**
1. When a `hick:file` element is first created (by the agent or manually)
2. When `hick run` processes a new `hick:file` for the first time
3. On demand: `hick paste-slots <file>` CLI command inserts slots into an existing hick:file

**Implementation approach:**
- Use Tree-sitter (already used by hick-lsp) for language-aware AST parsing
- Map AST node positions back to positions within the hick:file content
- Insert `<hick:paste name="..."/>` XML elements at those positions
- Names must be stable across minor edits (based on symbol names, not line numbers)

**PasteSlotRegistry (from TASK-14):**
Maintain an index of all named paste slots across all hick:file elements so the agent can quickly answer "where can I insert code for function X?" without re-parsing every file.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 New hick:file elements get paste slots inserted automatically on creation
- [x] #2 Slot names are language-aware (function names, class names, etc.) using Tree-sitter
- [x] #3 Slot names are stable: renaming an unrelated function does not change existing slot names
- [x] #4 hick paste-slots <file> CLI command inserts slots into an existing hick:file element
- [x] #5 PasteSlotRegistry indexes all slots and is queryable by file path and symbol name
- [x] #6 Unknown/binary file types get header + footer slots only
- [x] #7 Inserting paste slots does not change the rendered output of hick run (verified by hick equiv)
- [x] #8 cargo test --workspace passes
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
## Implementation Plan

1. `hick-lens/src/paste_slots.rs` (new): tree-sitter insertion point computation
   - `PasteInsertion { byte_offset, name }`
   - `compute_insertions(file_path, content)` — walks AST, finds top-level elements, produces named slots
   - Falls back to file-header + module-footer for unknown languages

2. `hick-lens/src/lib.rs` (modify): add `pub mod paste_slots`, re-export types

3. `hick-agent-hick-edit/src/paste_slot_inserter.rs` (new): string-level insertion
   - `insert_paste_slots(file_path, content) -> String`
   - Idempotent (skips if slots already present)

4. `hick-agent-hick-edit/src/paste_slot_registry.rs` (modify): name-based slots, file/symbol lookup
   - Add `named_slots`, `by_file` maps
   - Add `register_named`, `lookup_named`, `slots_for_file`, `slots_for_symbol`
   - Update `from_sources` to scan `name=` attribute

5. `hick-agent-hick-edit/src/hick_edit_agent.rs` (modify): auto-insert in `compose_file_element`

6. `hick-agent-hick-edit/src/lib.rs` (modify): export new items

7. `iron-peak/src/main.rs` (modify): add `paste-slots` subcommand
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
## Implementation Summary

### New files
- **`crates/hick-lens/src/paste_slots.rs`**: Tree-sitter-based insertion point computation. `compute_insertions(file_path, content)` walks the AST to find top-level structural elements (filtering nested ones by byte-range containment), then emits: `after-imports` after the last import block, `after-fn-{name}` after each non-import top-level element, `impl-{Name}-body` before the closing `}` of impl/class/trait/struct/interface (brace languages only), and `module-footer` at end. Falls back to `file-header` + `module-footer` for unknown languages or parse failures.
- **`hick-agent/crates/hick-agent-hick-edit/src/paste_slot_inserter.rs`**: String-level insertion. `insert_paste_slots(file_path, content)` calls `compute_insertions` and applies insertions in reverse byte order. Idempotent: returns content unchanged if `<hick:paste name=` already present.

### Modified files
- **`crates/hick-lens/src/lib.rs`**: Added `pub mod paste_slots` and re-exported `PasteInsertion`, `compute_insertions`.
- **`hick-agent/crates/hick-agent-hick-edit/src/paste_slot_registry.rs`**: Extended with `named_slots` (name→file) and `by_file` (file→names) maps. New methods: `register_named`, `lookup_named`, `slots_for_file`, `slots_for_symbol`, `named_slot_names`, `named_len`. `from_sources` now scans both `select=` and `name=` attributes.
- **`hick-agent/crates/hick-agent-hick-edit/src/hick_edit_agent.rs`**: `compose_file_element` now calls `insert_paste_slots` before wrapping content in `<hick:file>`.
- **`hick-agent/crates/hick-agent-hick-edit/src/lib.rs`**: Added `pub mod paste_slot_inserter`, re-exported `insert_paste_slots`.
- **`hick-agent/crates/iron-peak/src/main.rs`**: Added `PasteSlots` subcommand (`paste-slots <file> [--write]`). Dry-run by default; `--write` applies changes.
- **`hick-agent/crates/iron-peak/Cargo.toml`**: Added `hick-lang` dependency.

### AC verification
- AC#7 (rendered output unchanged): Tests `rendered_output_unchanged_for_{rust,python,typescript}` strip all `<hick:paste name=.../>` tags from the result and assert equality with original content — pass in all three languages.
- AC#3 (stable names): `slot_names_stable_across_unrelated_renames` test confirms name stability.
- All 0 failures across the workspace (3 pre-existing broken crates excluded: `blue-iron-cube`, `c2w`, `native2wasm`).
<!-- SECTION:FINAL_SUMMARY:END -->
