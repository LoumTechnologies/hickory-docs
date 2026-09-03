# A Breakpoint Can Carry A Condition

Given a breakpoint in a document or a plain file, when it is right-clicked or
alt-clicked in the gutter, then a panel opens on that line offering three
things — a **condition**, a **hit count**, and a **log message** — and what is
typed reaches the adapter on the next `setBreakpoints`, or on the next run if
nothing is running.

A breakpoint carrying any of the three is drawn as the wedge rather than the
dot, and its tooltip says which — *Stops when i > 10*, *Hit count > 5*, *Logs
"…" and continues* — so the condition can be read without opening anything. A
reason it could not bind still wins the tooltip, because that is the more
urgent fact.

A condition is **trimmed by the setter, not by the caller**: three spaces is
not a condition, and an adapter handed one reports a parse error about code
nobody wrote. An empty field clears. Only fields that were set go on the wire.

**A breakpoint that moves keeps its condition.** The adapter binds to the next
executable line and reports `moved_to`; the match is made against the status
*before* the move is applied, where both the requested and the bound line are
still in hand. A breakpoint that slid one line down and silently became
unconditional would stop on every pass — the opposite of what was asked for.

A pane with no way to set a condition falls back to an ordinary toggle rather
than opening a panel that could do nothing, and leaves the right-click to the
browser.

## Why

All three have been carried by `hick_dap::Breakpoint` since the debugger was
written, forwarded to the adapter under the adapter's own capability flags,
and deserialized by the server from the wire. Nothing could set one. The hook
sent `{ line }` and dropped the rest; `debugStateEffects` hardcoded
`conditional: false`, which made the gutter's `.cm-bp-conditional` styling —
already written, already themed — unreachable.

So this is not a new capability. It is a surface over one that was complete
and had no door, which is the cheapest kind of work there is and the easiest
kind to leave undone forever. Conditional breakpoints and logpoints are among
the most-used debugger features in the JetBrains pack this product is being
asked to replace; a debugger without them makes a person add a print
statement, which changes the file they were debugging.

The gesture is tested as a **pure intent** (`gutterIntent`) rather than by
dispatching events, for the same reason `gutterAction` is a pure exported
function: a gutter's `domEventHandlers` never fire under jsdom, which has no
layout to resolve a pointer's height against. Discovering that is what stopped
this from shipping with tests that dispatched clicks, observed nothing, and
passed anyway.

---

Last LLM verification:
- Date: 2026-09-03
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `apps/web/src/debug/useDebugger.ts` — `HeldBreakpoint`,
  `followMoves` (which now carries conditions across a move), `onTheWire`
  (which omits unset fields), and `setBreakpointCondition`;
  `apps/web/src/debug/cmDebug.ts` — `breakpointTip`, `gutterIntent`,
  `editBreakpoint`, `openBreakpointEditor`, and `debugStateEffects` computing
  `conditional`; `apps/web/src/debug/client.ts` — `log_message` on
  `DebugBreakpoint`; both panes wire `onSetBreakpointCondition`. The server
  half was already present: `crates/hick-dap/src/session.rs` sends
  `condition`, `hitCondition` and `logMessage` under
  `supportsConditionalBreakpoints` / `supportsHitConditionalBreakpoints`.
- Test coverage: `useDebugger.test.ts` — "sends a condition on the wire, and
  only what was set", "carries a hit count and a log message too", "clears a
  condition when it is emptied" (which found the untrimmed-whitespace defect),
  "keeps the condition when the adapter moves the breakpoint", "keeps a
  condition set before the program runs, and starts with it";
  `cmDebug.test.ts` — the `conditional`-is-computed test, the wedge, the
  tooltip, redraw-on-condition-change, and the nine gesture and panel cases.
- Caveat requiring LLM review: no test drives a real adapter with a condition
  and asserts the program stopped only when it held. The wire contents are
  asserted and the Rust side is covered by `hick-dap`'s own tests, but the
  end-to-end claim — *this condition actually gated this stop* — is verified
  by reading, not by running. That is the same gap `a-plain-file-has-the-same-debugger.md`
  closed for stepping, and it should be closed the same way.
