<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/SurfaceInfo.css

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-e2d76d492c26"></a>
## `apps/web/src/components/SurfaceInfo.css`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/components/SurfaceInfo.css">
.surface-info { flex-shrink: 0; min-width: 0; padding: .35rem .65rem; font-size: .78rem; color: var(--fg-muted); border-bottom: 1px solid var(--border); }
.surface-info > summary { display: flex; align-items: center; gap: .5rem; cursor: pointer; list-style: none; white-space: nowrap; }
.surface-info > summary::-webkit-details-marker { display: none; }
.surface-info > summary::before { content: "›"; flex-shrink: 0; }
.surface-info[open] > summary::before { transform: rotate(90deg); }
.surface-info strong { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; color: var(--fg); font-weight: 500; }
.surface-info > summary > span { flex-shrink: 0; }
.surface-info > p { margin: .5rem 0 0; overflow-wrap: anywhere; }
</storycode:entry>

</storycode:doc>
