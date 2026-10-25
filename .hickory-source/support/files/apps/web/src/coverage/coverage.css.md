<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/coverage/coverage.css

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `c0d8edf910cfcd8ccb466373f274d8a445fe2b9b8506604eb17fc5a368e117af`.

<book:entry path="apps/web/src/coverage/coverage.css">
.coverage-backdrop { position: fixed; inset: 0; z-index: 1200; background: #0007; display: grid; place-items: center; padding: 2vh 2vw; }
.coverage-panel { background: var(--bg, #fff); color: var(--fg, #222); border: 1px solid var(--border, #aaa); border-radius: 8px; width: min(1300px, 96vw); max-height: 96vh; overflow: auto; padding: 18px; box-shadow: 0 12px 50px #0005; }
.coverage-panel header { display: flex; justify-content: space-between; align-items: center; }
.coverage-panel h2, .coverage-panel h3 { margin: 8px 0; }
.coverage-panel p { max-width: 100ch; }
.coverage-controls { display: flex; flex-wrap: wrap; align-items: end; gap: 8px; margin: 12px 0; }
.coverage-panel label { display: flex; flex-direction: column; gap: 3px; font-size: 12px; }
.coverage-panel input, .coverage-panel select, .coverage-panel textarea { min-height: 30px; padding: 4px; color: inherit; background: var(--bg, #fff); border: 1px solid var(--border, #aaa); border-radius: 4px; }
.coverage-panel details { padding: 6px 0; }
.coverage-panel summary { cursor: pointer; }
.coverage-columns { display: grid; grid-template-columns: minmax(180px, 240px) minmax(0, 1fr); gap: 18px; margin-top: 12px; }
.coverage-run { display: block; width: 100%; text-align: left; margin: 6px 0; padding: 8px; color: inherit; background: transparent; border: 1px solid var(--border, #aaa); border-radius: 4px; overflow-wrap: anywhere; }
.coverage-run[aria-pressed=true] { border-color: var(--accent, #568); background: color-mix(in srgb, var(--accent, #568) 10%, transparent); }
.coverage-panel table { width: 100%; border-collapse: collapse; font-size: 12px; }
.coverage-panel td, .coverage-panel th { padding: 5px; border-bottom: 1px solid var(--border, #ddd); text-align: left; }
.coverage-link { color: var(--accent, #268); cursor: pointer; border: 0; background: transparent; text-align: left; overflow-wrap: anywhere; }
.coverage-panel pre { white-space: pre-wrap; overflow-wrap: anywhere; max-height: 300px; overflow: auto; font-size: 12px; }
@media (max-width: 650px) { .coverage-columns { grid-template-columns: 1fr; } }
</book:entry>

</book:doc>
