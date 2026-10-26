<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/DiffView.css

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a4151550e3f6"></a>
## `apps/web/src/components/DiffView.css`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/components/DiffView.css">
.diff-breakpoints { display: inline-flex; width: 3rem; vertical-align: baseline; }
.diff-breakpoints > span { display: inline-block; width: 1.5rem; }
.diff-breakpoint { border: 0; padding: 0; width: 1.5rem; background: transparent; color: var(--fg-muted); font: inherit; cursor: pointer; }
.diff-breakpoint:hover, .diff-breakpoint[aria-pressed="true"] { color: var(--fail); }
.diff-breakpoint:focus-visible { outline: 1px solid var(--accent-2); }
</storycode:entry>

</storycode:doc>
