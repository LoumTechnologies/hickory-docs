<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/embed/debug.css

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-bb81b5c30b9b"></a>
## `apps/web/src/embed/debug.css`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

This literal construction preserves embedded document syntax and exact file bytes. Its internal reasoning boundaries still need editorial review.

<storycode:entry path="apps/web/src/embed/debug.css">
.hickory-browser-debug { --warn: #ebbe69; --accent-2: #98c4a1; --danger: #ef9b8e; --ok: #98c4a1; --fg: #e4ded5; --fg-muted: #b2b9b0; --border: #4c514b; --bg: #202522; --bg-elevated: #282e29; color: var(--fg); background: var(--bg); border-radius: 8px; padding: 12px; }
.hickory-browser-debug button, .hickory-browser-debug select, .hickory-browser-debug input { background: var(--bg-elevated); color: var(--fg); border: 1px solid var(--border); border-radius: 4px; padding: 4px 8px; cursor: pointer; }
.hickory-browser-debug button:disabled { opacity: .4; cursor: default; }
.hickory-browser-debug .browser-debug-actions, .hickory-browser-debug .debug-strip { display: flex; align-items: center; flex-wrap: wrap; gap: 8px; padding-bottom: 10px; }
.hickory-browser-debug .debug-strip__error { color: var(--danger); }
.hickory-browser-debug .cm-paused-line { background: #594b24; }
.hickory-browser-debug .cm-debug-value { color: #98c4a1; padding-left: 12px; opacity: .8; }
.hickory-browser-debug .browser-debug-inspection ul { list-style: none; padding-left: 16px; }
.hickory-browser-debug .browser-debug-output { min-height: 50px; white-space: pre-wrap; overflow-wrap: anywhere; border-top: 1px solid var(--border); padding-top: 12px; }
.hickory-browser-debug .cm-scroller { max-height: 480px; }
.hickory-browser-debug .cm-breakpoint-gutter { min-width: 18px; background: #2d352e; cursor: pointer; }
.hickory-browser-debug .cm-bp, .hickory-browser-debug .cm-bp-ghost { display: inline-block; width: 9px; height: 9px; margin-top: 5px; border-radius: 50%; }
.hickory-browser-debug .cm-bp { background: var(--danger); }
.hickory-browser-debug .cm-breakpoint-gutter:hover .cm-bp-ghost { background: #84564e; }
.hickory-browser-debug .cm-bp-broken { background: transparent; border: 1px solid var(--danger); }
.hickory-browser-debug .cm-paused-arrow { color: var(--warn); }
</storycode:entry>

</storycode:doc>
