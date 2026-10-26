<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/MarginActions.css

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-3128cbfa8dc0"></a>
## `apps/web/src/components/MarginActions.css`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/components/MarginActions.css">
.comparison-editor--compact { position: relative; padding-right: 4rem; }
.cm-rendered.cm-rendered-patch { overflow: visible; }
.cm-rendered-patch:has(.margin-actions[open]) { z-index: 10; }
.comparison-editor--compact .comparison-side { position: relative; }
.margin-actions { position: absolute; right: 0; top: 0; z-index: 5; }
.margin-actions > summary {
  display: grid; place-items: center; width: 1.75rem; height: 1.75rem;
  list-style: none; cursor: pointer; color: var(--fg-muted); border-radius: .25rem;
}
.margin-actions > summary::-webkit-details-marker { display: none; }
.margin-actions > summary:hover, .margin-actions[open] > summary {
  color: var(--fg); background: var(--bg-sunken);
}
.margin-actions[open] { z-index: 10; }
.margin-actions__panel {
  position: absolute; top: 0; right: 2rem;
  width: min(22rem, 70vw); max-height: 60vh; overflow: auto;
  padding: .75rem; border: 1px solid var(--border); border-radius: .35rem;
  background: var(--bg); color: var(--fg); box-shadow: 0 4px 16px #0003;
  font-family: system-ui, sans-serif; font-size: .85rem;
}
.margin-actions__panel label { display: flex; align-items: center; gap: .5rem; margin: .5rem 0; }
.margin-actions__panel select { min-width: 0; max-width: 100%; }
.comparison-editor--compact .comparison-side > .margin-actions { right: -2rem; top: 0; }
.comparison-editor--compact .comparison-panes--side-by-side { gap: 2rem; }
</storycode:entry>

</storycode:doc>
