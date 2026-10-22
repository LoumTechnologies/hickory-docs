# A comparison keeps current code editable

Given a literate reading or persistent document, comparing it with a Git
revision leaves the current code in the main editor's actual buffer. Added lines
update as it is edited. Removed lines are shown by a minus marker in the gutter. Clicking it opens a
floating previous-version preview with an explicit restore action on editable
comparisons. No historical rows take space in the text. Removed bytes enter the
current buffer only through explicit restoration. They do not affect saved text or language-server coordinates.

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


Verification update (2026-10-16): `comparison.ts` uses gutter markers and a
CodeMirror tooltip. Editing, changing comparison and Escape dismiss the preview;
removed text remains outside the buffer. `comparison.test.ts` covers the marker,
unchanged current bytes and explicit restoration. Review and browser-flow tests
now open the gutter preview instead of expanding an inserted historical row.


### Shared removed-source disclosure and restore — 2026-10-22

The actual gutter/tooltip now shares marker metadata, open/close, bounded
historical-byte reading and restore actions. Historical body bytes remain absent
from discovery, even while the explicit preview is open; reading is separately
requested in pages of at most 8192 bytes. Closing or changing its source,
selection, comparison or read-only owner retires its children and old handles.
A retained native restore callback cannot apply after retirement. Current source
and base identities, live editability and the actual post-filter buffer decide
restore acceptance. Model restoration uses input.uiAgentRestore and does not
claim save, human approval or verification.

LLM review is partial. Three existing comparison cases and two actual
CodeMirror/registry flows passed, challenging Unicode paging, closed/stale reads,
actor origin, stale native button callbacks and readonly historical restoration.
Targeted TypeScript passed. The handoff records exact dependency hashes and the
recursive-field inference correction. Human judgment and current packaged/native
visual acceptance are absent. Automatic historical disclosure, stale restoration
or restored bytes treated as verified output would falsify the stated scope.
