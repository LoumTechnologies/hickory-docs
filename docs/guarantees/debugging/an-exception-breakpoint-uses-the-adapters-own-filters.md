# An Exception Breakpoint Uses The Adapter's Own Filters

Given a debug session, when the adapter reports
`exceptionBreakpointFilters`, then the strip draws one switch per filter, in
the adapter's own words, and toggling one sends `setExceptionBreakpoints`
with the ids that are on. An adapter reporting none draws nothing.

The choice is **kept across runs and re-applied to the next session**: "stop
on uncaught exceptions" is a standing preference, the way a breakpoint is,
not a property of one run.

The **id** goes on the wire and the **label** is what a person reads. Neither
is invented here: `raised` and `uncaught` are Python's words, a JVM's are its
own, and a list written into this product would be wrong for the third
adapter.

## Why

`hick_dap::Session::set_exception_breakpoints` has existed as long as the
debugger, `Capabilities` has parsed `exceptionBreakpointFilters` from
`initialize` all along, and the web client's `DebugCapabilities` already
carried `exception_filters`. Nothing connected them: the server bridge had no
request for it at all — the word "exception" did not appear in
`debug_bridge.rs` — so the engine could do it, the client knew the filters
existed, and no path ran between them.

That is the same shape as the conditional breakpoints found on 2026-09-03:
capability with no door. It is worth naming as a pattern rather than as two
incidents, because the way both were found was the same — reading what the
engine can do and asking which of it a person can reach.

Stopping where an exception is thrown is the difference between reading a
stack trace and standing in the frame that produced it, which is most of why
a debugger beats a print statement.

---

Last LLM verification:
- Date: 2026-09-04
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `crates/hickory-cli/src/serve/debug_bridge.rs` —
  `Request::SetExceptionBreakpoints`, its handler, and its name in
  `describe()`; `apps/web/src/debug/client.ts` —
  `setExceptionBreakpoints`; `apps/web/src/debug/useDebugger.ts` —
  `exceptionFilters`, `toggleExceptionFilter`, and the re-apply in the
  `started` handler (through a ref, because that handler is not re-created
  per render); `apps/web/src/debug/DebugStrip.tsx` — the switches, drawn from
  `capabilities.exception_filters` and never from a list here.
- Test coverage: `DebugStrip.test.tsx` — "offers the adapter's own filters,
  in the adapter's own words", "draws nothing for an adapter that has none",
  "shows which are on, and toggles by id" (which asserts the id rather than
  the label reaches the callback); `useDebugger.test.ts` — "sends the filters
  that are on, by id" and "keeps them across runs and re-applies them to the
  next session".
- Caveat requiring LLM review: no live test throws an exception and asserts
  the program stopped on it. The wire contents are asserted and
  `set_exception_breakpoints` is the adapter's own request, but the
  end-to-end claim is verified by reading. **Set-variable remains
  unreachable** for a different reason and is not fixed here: the whole path
  exists — `Session::set_variable`, the bridge's `SetVariable`, the client's
  `setVariable` — and there is nowhere to put it, because variables are drawn
  as faded values at the ends of lines rather than in a pane. Changing one
  needs a variables view, which is its own missing JetBrains feature rather
  than a missing button.
