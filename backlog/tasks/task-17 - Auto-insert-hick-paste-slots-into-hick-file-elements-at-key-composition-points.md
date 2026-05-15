---
id: TASK-17
title: 'Auto-insert hick:paste slots into hick:file elements at key composition points'
status: To Do
assignee: []
created_date: '2026-05-15 12:45'
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
- [ ] #1 New hick:file elements get paste slots inserted automatically on creation
- [ ] #2 Slot names are language-aware (function names, class names, etc.) using Tree-sitter
- [ ] #3 Slot names are stable: renaming an unrelated function does not change existing slot names
- [ ] #4 hick paste-slots <file> CLI command inserts slots into an existing hick:file element
- [ ] #5 PasteSlotRegistry indexes all slots and is queryable by file path and symbol name
- [ ] #6 Unknown/binary file types get header + footer slots only
- [ ] #7 Inserting paste slots does not change the rendered output of hick run (verified by hick equiv)
- [ ] #8 cargo test --workspace passes
<!-- AC:END -->
