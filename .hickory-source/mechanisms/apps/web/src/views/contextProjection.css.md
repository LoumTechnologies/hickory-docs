<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/contextProjection.css

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-3595d8ea22ae"></a>
## `apps/web/src/views/contextProjection.css`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/views/contextProjection.css">
.workspace-chat { display: flex; flex-direction: column; min-height: 0; height: 100%; overflow: hidden; }
.workspace-chat > details { flex: none; max-height: 40%; overflow: auto; padding: 0.5rem; border-bottom: 1px solid var(--border); }
.workspace-chat > details pre { white-space: pre-wrap; overflow-wrap: anywhere; }
.workspace-chat > .chat-dock { flex: 1; min-height: 0; }
</storycode:entry>

</storycode:doc>
