<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/runtimeLog.css

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `c5377f4f952a98c16d56495bd7170d9309153597ee51b81a43c9f599100bb6ed`.

<book:entry path="apps/web/src/debug/runtimeLog.css">
.runtime-log-panes { position: relative; display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); gap: 3rem; }
.runtime-log-panes > div { min-width: 0; }
.runtime-log-panes .cm-editor { max-height: 22rem; border: 1px solid currentColor; }
.runtime-log-panes .cm-scroller { overflow: auto; }
.runtime-log-panes .ribbon-layer { position: absolute; inset: 0; width: 100%; height: 100%; pointer-events: none; overflow: visible; z-index: 3; }
.runtime-log-panes .ribbon, .runtime-log-panes .ribbon-brace-hit { pointer-events: stroke; }
.runtime-log-panes .ribbon-brace-hit { stroke: transparent; stroke-width: 12px; fill: none; }
.ribbon.ribbon-family-execution { fill: var(--accent, currentColor); opacity: .35; }
.ribbon-brace.ribbon-family-execution, .ribbon-brace-link.ribbon-family-execution { stroke: var(--accent, currentColor); stroke-dasharray: 10 2 2 2; fill: none; stroke-width: 2px; }
.ribbon-group--execution .ribbon-connector { fill: var(--accent, currentColor); }
@media (max-width: 600px) { .runtime-log-panes { gap: 1rem; } }
</book:entry>

</book:doc>
