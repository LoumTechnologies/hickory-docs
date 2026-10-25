<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/authoring/print-requests-do-not-attest-physical-output.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `3f1a3eb7347edc424615a9eddc3968ab424b45e30ecccbec4c058f273c09e5c8`.

<book:entry path="docs/guarantees/authoring/print-requests-do-not-attest-physical-output.md">
# Shared print requests do not attest physical output

Given a focused printable buffer, when the human or agent requests Print, both
invoke the same full-buffer print callback. The title comes from the active tab
in the focused pane, rather than another pane's active tab. An absent editor is
unavailable; a platform callback error remains an error. A returned callback does
not establish that a dialog appeared, was confirmed rather than cancelled, or
produced physical output. Printer selection is browser/OS-owned, beyond what the
page's print API reports; the registered command states this limitation before
invocation and returns its actual callback result afterward.

The printable text is the entire captured buffer, including lines omitted from
CodeMirror's virtualized DOM. Text is assigned as text, never interpreted as
HTML. The temporary print element and title are cleaned up even if printing
throws. This does not guarantee power-loss recovery, paper delivery or OS dialog
automation, nor does it infer human approval from a model request.

## Verification — 2026-10-20

Origin: #72 same-handler parity and truthful platform request/result boundaries.
The earlier agent-only refusal to invoke Print was an application policy, not a
measured OS inability. That blanket refusal was removed; actual printer selection
and physical outcomes remain explicitly unobservable through this callback.

LLM implementation review: partially verified. Inspected
`workspaceMenuActions::runWorkspaceMenu`, `uiWorkspaceMenuActions`,
`WorkspaceView::printFocusedBuffer` and `printing::printText`. The focused-pane
lookup replaces an earlier first-visible-pane title lookup. No native dialog
completion or physical print witness is claimed.

Executed checks: seven focused workspace-menu cases and six print-text cases
passed. Tests compare both actors' callback invocation, false/missing-editor and
thrown platform outcomes, first/last lines of a 500-line buffer, literal script
text, title restoration and cleanup on failure. They use mock print callbacks;
they do not drive a printer or validate browser/WebKit dialog timing. Targeted
TypeScript checking included WorkspaceView; whitespace/file-length checks passed.

Human judgment: not recorded. Supported native package/provider/released/deployed
witnesses for #72 remain absent. The OS boundary is tracked here and in the command
prerequisites/outcome; an action receipt is not print success evidence. Changed
source dependencies invalidate this checkpoint.

Evidence dependencies (SHA-256):

- `apps/web/src/views/workspaceMenuActions.ts`: `9d514e8bdb542cee510c9e2e4844fb792c1642755333c61813ac2747dc40aa9b`
- `apps/web/src/views/uiWorkspaceMenuActions.ts`: `43f1e6c6675795a54e441afbc54cc7de5cc3b5b0a87b0233e3e0f4d327978368`
- `apps/web/src/views/WorkspaceView.tsx`: `ee5f671d02093adfb5a7d475a83eb9a0986ff7a6f1ae7451ed14e5ae1aea4090`
- `apps/web/src/lib/printing.ts`: `c0345d6ab51e12faf06bc156c4fd6f2986489a0a4fd40be3e432689721733d2e`
</book:entry>

</book:doc>
