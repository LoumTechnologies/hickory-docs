# The homepage opens with a live, paused debugger

Given a developer visiting the homepage, when its static assets load, then
one short Markdown document opens with a real browser debug session paused
at its return statement. The breakpoint, selected frame, highlighted line
and variables come from executing that document's code.

Given the paused example, when the developer steps or continues, then the
interpreter advances and produces its actual output. When they edit, then a
visible caret follows their typing and the old session is marked stale;
restarting executes the edited source. Ordinary rerenders do not restart it.

Given this page, then the header identifies Hickory Docs as downloadable
software running on the person's machine. The browser example uses static
assets without a native engine, authenticated remote engine or account.

---

Last LLM verification:
- Date: 2026-10-05
- Reviewer: Codex
- Result: verified
- Evidence: `BrowserDebugDemo` supplies a nine-line example and opts into
  `DebuggableDocument.startPausedAt`. `EditingSurface` draws selections and
  the embed supplies contrasting cursor colours. The production build omits
  host fixtures and the homepage omits the previous long demos.
- Test coverage: `apps/web/e2e/browserEmbedding.spec.ts` homepage flow checks
  initial pause without a click, real locals, step out, editing, caret colour,
  restart, language switching and mobile overflow in three browsers.
  `LandingView.test.tsx` checks the download message precedes the demo.
- Caveat: asset download time depends on the visitor's connection. The UI
  shows startup progress until the interpreter reaches the breakpoint.
