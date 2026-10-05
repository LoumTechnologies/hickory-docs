# Browser debugging runs edited source

Given a document containing a literal JavaScript or TypeScript file using the
advertised ES5 function/statement subset,
when the visitor explicitly starts Debug,
then a worker executes that frozen source with a real step-capable interpreter.
Breakpoints stop before executable statements, stepping can enter and leave a
user function, and inspected variables come from live scopes. Editing and
restarting changes the result. A source edit marks a live session as belonging
to the previous revision and removes its marker from the new bytes.

Watches are bounded read-only expressions; inspection never calls a getter or
user function. Unsupported syntax and unavailable APIs fail explicitly. The
interpreter has no page/storage/network host bridge and dynamic code is disabled.
Stop terminates the worker, including a running loop. Execution and output have
limits. After the runner assets load, restart and inspection work offline.
Debugging output is not persisted as verified execution evidence.

---

Last LLM verification:

- Date: 2026-10-04
- Reviewer: Codex
- Result: verified
- Evidence: `apps/web/src/debug/browser/{compile,runtime,inspect,worker,transport}.ts`;
  `embed/DebuggableDocument.tsx`; homepage `BrowserDebugDemo.tsx`. The transport
  reuses `DebugClient`, `useDebuggerOver`, `DebugStrip` and `cmDebug` events/UI.
- Test coverage: `runtime.test.ts` checks native-engine result parity, real frames,
  closures, edited values, getter/call refusals, network/page/dynamic-code refusal,
  exceptions, syntax/type errors and loop limits. Production static-site
  Playwright flows in Chromium/Firefox/WebKit verify JS and TS breakpoints,
  steps, watches, edited output, offline restart, Stop and homepage execution.
- Limits: ES5 plus TypeScript type annotations; use `var`. No imports, async,
  classes, arrow functions, block-scoped declarations, DOM/network APIs, conditional
  breakpoints, reverse execution or arbitrary instruction jumps. Worker isolation
  alone is not a security sandbox; the interpreter's restricted bridge is the
  execution boundary. Memory is bounded by browser resources, not a hard heap quota.
