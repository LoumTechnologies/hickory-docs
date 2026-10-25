<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/authoring/settings-preserves-the-open-workspace.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `a0ff90336c0efeb57a25563bfd418d2e561c63feffeb0e22bb456ff76a9faf3d`.

<book:entry path="docs/guarantees/authoring/settings-preserves-the-open-workspace.md">
# Settings preserves the open workspace

Given open documents, generated files, unnamed drafts or an agent conversation,
when Settings is opened and closed, then the same workspace and editor instances
remain mounted. Draft bytes, selections, undo history and open tabs survive the
round trip. Back and Escape return to the route from which Settings was opened,
without replaying the startup introduction or opening another unnamed buffer.

While Settings occupies the window, workspace keyboard actions do not close or
rearrange hidden tabs. Appearance changes apply to the retained workspace;
closing Settings refreshes the server's window title and recovery preference.
No execution or agent authorization policy changes.

Closing the window from Settings returns to the workspace before its existing
unsaved-work decision is shown, so the decision is visible and cancellation
preserves the editors.

---

Verification (2026-10-13, Codex):

- Implementation: `apps/web/src/App.tsx`, `views/SettingsView.tsx`,
  `views/useWorkspaceAppearance.ts`, `lib/workspaceVisibility.ts`,
  `shell/CommandBar.tsx`, `shell/ShellView.tsx`, and `views/WorkspaceView.tsx`.
- Regression evidence: `App.test.tsx` retains the same CodeMirror instance and
  draft selection; `useWorkspaceAppearance.test.tsx` checks live local preferences
  and the server refresh. `e2e/workflow-polish.spec.ts` checks real editor identity,
  bytes, selection and tabs through Settings, including Escape and Cmd/Ctrl+W.
- Observed trigger: the installed macOS app's Settings → Back path replaced an
  existing project workspace with a recovered unnamed draft.
- Limits: browser checks use a real native engine in a disposable project.
  They do not prove every native menu, every preference, or every platform.
</book:entry>

</book:doc>
