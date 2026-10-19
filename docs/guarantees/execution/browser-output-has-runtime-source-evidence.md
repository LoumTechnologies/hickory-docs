# Browser output carries actual runtime source and occurrence evidence

Given a browser JS/TS run with a captured document digest, console output is
recorded in hick-observe v1 run/event shapes. Emission anchors come from the live
interpreter CallExpression, independently of paused-line state. TypeScript
source maps and literal-file segments map to exact UTF-8 document anchors;
unmapped code keeps block-only attribution. Exact anchors identify the emitting
statement location, not a fabricated whole-statement span across source fragments.

Events preserve raw UTF-8 chunks and receive order. `console.log/warn/error`
append a newline; the explicitly supported `console.write` helper does not.
Multiline/partial output, CRLF and terminal bytes are retained. Actual JavaScript
call-state identities distinguish repeated, nested and recursive invocations;
calling locations remain separate from the logger's emitting anchor. Text
equality does not establish occurrence identity. Top-level output uses one
explicit program invocation.

Captured document bytes/revision remain immutable after editor changes. Runtime
exceptions produce typed observations separately from console.error text.
Throwing and host-observed unhandled/unknown states remain distinct; caught-site
mapping is unavailable and remains null. Runtime observations do not enable
page/network/dynamic-code access or change existing stepping semantics.

Worker events are incremental; source/history snapshots are not retransmitted
for each write. Evidence caps at 4096 events with explicit truncation, independently
of the existing 64 KiB text-output and two-million-step limits. Cancel/restart/
dispose preserves cancelled status for a running captured record. No authored
Markdown output or persisted hick:output behavior is removed.

---

Last LLM verification:

- Date: 2026-10-18
- Reviewer: Codex
- Result: partially verified
- Originating requirement: #55/#47 and execution-observation-provenance.md.
- Evidence: BrowserRun.observe/location/invocation, compileProgram.sourceByteAt,
  createBrowserDebugger and DebuggableDocument.onRuntime.
- Assumptions: js-interpreter call-state identity is an invocation occurrence;
  native console callback executes before that state is removed. Its captured
  stack is inspected directly, not reconstructed from log text.
- Witnesses: runtime.test.ts JS/TS recursion, multiline Unicode/CRLF partial
  writes, real caller anchors, typed exceptions, stepping and confinement tests;
  browserEmbedding source-linked flows in Chromium/Firefox/WebKit.
- Challenges: recursive equal text, logger/caller separation, no paused state,
  subsequent edits and output consisting solely of empty writes.
- Coverage boundary: this provider covers existing ES5/typed-ES5 interpreter
  syntax, not arbitrary browser execution, concurrent threads or Node streams.
  Caught exceptions retain throwing observation with catching unknown; native
  host/runtime errors may lack a throwing source location. Captures without a
  source digest intentionally lack new runtime records. Browser run persistence
  and run/invocation selection belong to #57; this exposes a reusable callback.
- Dependencies: implementation/test files above in this commit; changes invalidate
  this review. No provider policy or model correctness claim is made.
- Executed checks: final eleven runtime tests and six serial three-engine
  browser flows passed after incremental transport and bounded-event safety
  refinement. Production site build (including TypeScript) and diff checks passed.
- Human judgment: not recorded.
