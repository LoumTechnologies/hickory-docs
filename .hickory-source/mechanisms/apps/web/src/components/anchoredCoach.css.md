<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/anchoredCoach.css

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-99921a163259"></a>
## `apps/web/src/components/anchoredCoach.css`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/components/anchoredCoach.css">
.anchored-coach { position: fixed; z-index: 120; width: min(400px, calc(100vw - 24px)); max-height: min(65vh, 620px); overflow: auto; padding: 1rem; border: 1px solid var(--border, #888); border-radius: .5rem; background: var(--bg, #fff); color: var(--fg, #222); box-shadow: 0 8px 28px #0003; }
.anchored-coach > header { display: flex; align-items: center; justify-content: space-between; gap: .5rem; }
.anchored-coach .workflow-tour--app { position: static; inset: auto; z-index: auto; max-width: none; max-height: none; overflow: visible; overflow-wrap: anywhere; background: transparent; box-shadow: none; border: 0; padding: 0; margin: 0; }
.anchored-coach .workflow-tour--app button { max-width: 100%; white-space: normal; overflow-wrap: anywhere; }
.anchored-coach pre { white-space: pre-wrap; overflow-wrap: anywhere; max-height: 12rem; overflow: auto; }
.anchored-coach .workflow-tour-controls { display: flex; flex-wrap: wrap; gap: .4rem; }
@media (prefers-reduced-motion: reduce) { .anchored-coach { animation: none; transition: none; scroll-behavior: auto; } }
</storycode:entry>

</storycode:doc>
