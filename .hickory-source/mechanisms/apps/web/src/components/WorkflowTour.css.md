<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/WorkflowTour.css

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-88f12b5ed940"></a>
## `apps/web/src/components/WorkflowTour.css`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/components/WorkflowTour.css">
.workflow-tour-launcher { margin:.5rem 1rem; }
.workflow-tour { margin:1rem auto; padding:1rem; border:1px solid var(--border); border-radius:8px; max-width:72rem; max-height:80vh; overflow:auto; }
.workflow-tour h2 { margin:.4rem 0; }
.workflow-tour textarea { display:block; width:100%; min-height:5rem; }
.workflow-tour pre { white-space:pre-wrap; overflow-wrap:anywhere; }
.workflow-tour-controls { display:flex; flex-wrap:wrap; gap:.6rem; margin:1rem 0; }

.workflow-tour--app { position:fixed; inset:3rem 1rem auto; z-index:140; background:var(--bg-raised); color:var(--fg); box-shadow:var(--shadow); }

.workflow-tour .hickory-browser-debug .cm-scroller { max-height:20rem; }
</storycode:entry>

</storycode:doc>
