<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/runtimeLog.css

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f67fb95dafff"></a>
## `apps/web/src/debug/runtimeLog.css`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/debug/runtimeLog.css">
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
</storycode:entry>

</storycode:doc>
