<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/environments/environments.css

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-8535923e02aa"></a>
## `apps/web/src/environments/environments.css`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/environments/environments.css">
.environment-panel, .environment-notice { padding: 12px; background: var(--bg); color: var(--fg); border: 1px solid var(--border); }
.environment-panel header { display: flex; justify-content: space-between; gap: 12px; }
.environment-finding p { margin: 6px 0; }
.environment-directory { opacity: .7; overflow-wrap: anywhere; }
.environment-finding code, .environment-finding pre { white-space: pre-wrap; overflow-wrap: anywhere; }
.environment-finding button, .environment-finding a { margin: 4px 10px 4px 0; }
.environment-panel summary { cursor: pointer; overflow-wrap: anywhere; }
.environment-notice { max-height: 220px; overflow: auto; }
</storycode:entry>

</storycode:doc>
