# A comparison keeps current code editable

Given a literate reading or persistent document, comparing it with a Git
revision leaves the current code in the main editor's actual buffer. Added lines
update as it is edited. Removed historical lines are labelled read-only,
expandable decorations and enter the current buffer only through explicit
restoration. They do not affect saved text or language-server coordinates.

An optional historical target is read-only. Reading either revision performs no
checkout or repository write. Exact provenance maps current code into its real
file for language and debugger features, including reordered fragments and
Unicode. Ambiguous origins and prose receive no guessed coordinates. Debugging
requires saved code; historical execution uses a worktree.

---

Last LLM verification:

- Date: 2026-10-03
- Reviewer: Codex
- Result: partially verified
- Evidence: `serve/revision.rs`, `editor/comparison.ts`, `editor/DocumentEditor.tsx`,
  `views/RepresentationTools.tsx`, and `lsp/representationMapping.ts`.
- Test coverage: revision HTTP tests in `tests/literate_views.rs`; actual
  CodeMirror edit/restore tests in `editor/comparison.test.ts`; exact Unicode,
  reordered-fragment and ambiguity checks in `lsp/representationMapping.test.ts`;
  language formatting refusal across separated fragments and stale asynchronous
  answers in `lsp/cmLspFeatures.test.ts`; browser workflow in
  `e2e/literate-editor.spec.ts`.
- Limits: historical correspondence can require a fresh reading. New mapping
  tests do not launch every discoverable language server or debugger adapter.
