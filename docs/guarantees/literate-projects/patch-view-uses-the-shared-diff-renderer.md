# Patch elements use the shared diff renderer

Given a saved executable patch element, when the document view renders its
block, then the element registry selects the patch view and draws its raw unified
diff through the same DiffView used for existing changes. The context labels
identify New as the completed project and Old as the scope input. Rendering
does not apply a patch, execute a cell, build code or launch a native tool.

---

Last LLM verification:

- Date: 2026-10-12
- Reviewer: Codex
- Result: verified for the rendered code-block presentation.
- Originating requirement: `docs/specs/freeform/literate-changesets.md`, registry
  seam, raw patch grammar, shared comparison renderer and explicit context labels.
- Evidence: `hick-literate/src/patch_element.rs`, `render.rs`; frontend registry
  `apps/web/src/elements/index.ts` and `patch/view.tsx`, existing `DiffView.tsx`.
- Witnesses and challenges: `patch/view.test.tsx` exercises the actual WASM parser,
  inert ordinary fences, the shared additions/deletions renderer and pure project
  evaluation with exact Unicode/CRLF bytes.
- Assumptions and exclusions: this is currently a patch code block; editable
  inline/side-by-side old/new project comparisons and native action selectors
  are pending. The context label does not claim a successful debug launch.
- Executed checks: recorded in `implementation-evidence.md`.
- Dependencies: paths above plus generated parser; regeneration or UI changes
  invalidate this review.
- Human judgment: not recorded.
