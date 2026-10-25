<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/debugging/browser-debugging-runs-edited-source.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `ad297de0b33540d7ebda8fc15750474b42a892c07a43389a9f2fb0bf29c691c7`.

<book:entry path="docs/guarantees/debugging/browser-debugging-runs-edited-source.md">
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


Test verification amendment (2026-10-13, Codex): a pre-commit full-suite run
exposed a hardware/load-dependent timeout in the loop-limit witness, which
executed two million real interpreter steps within Vitest’s five-second test
budget. The witness now runs a real loop, pauses and continues it, then places
the fixture’s accrued step counter one step below the declared two-million
limit. One step remains runnable and the next emits the explicit limit failure
and exit code 1. This is a white-box fixture setup with public event assertions;
it checks the boundary rather than interpreter throughput. The private counter
name is a test dependency. Production interpreter code and its limit are
unchanged. `just test-browser-core` passed its Rust parser/materialization
checks and all 29 frontend tests, including all seven interpreter cases. The
full commit hook is being rerun with isolated browser artifact directories.


Browser-run isolation (2026-10-13): native and embedding Playwright configurations
now allocate separate per-process artifact directories under `test-results/`.
The embedding JSON report lives in that run’s directory. `HICKORY_E2E_OUTPUT_DIR`
can supply an explicit directory. Concurrent default runs cannot erase another
process’s traces; callers overriding the directory own its exclusivity. This
addresses two hook failures caused by missing trace network files during context
close while another browser run used the same output directory.
</book:entry>

</book:doc>
