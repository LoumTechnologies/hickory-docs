<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/embed/debug.css

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-bb81b5c30b9b"></a>
## `apps/web/src/embed/debug.css`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

The existing debugger palette and layout remain together below; their internal reasoning boundaries still need editorial review. The caret override is explained separately because it must win independently of stylesheet loading order.

<storycode:copy id="browser-debug-base-style">
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
</storycode:copy>


## Keep the debugger caret tied to its own foreground

Static multi-entry builds can change the order of shared stylesheets. The generic
embed cursor rule must not replace this debugger's warm foreground with the
standalone viewer's palette. Scope the native and drawn caret overrides through
both containers so they win regardless of CSS chunk order. The existing focused
homepage browser test checks the drawn caret's exact contrasting color and then
edits and restarts the paused program.

<storycode:copy id="browser-debug-caret-style">
.hickory-browser-debug .hickory-embed .cm-content { caret-color: var(--fg); }
.hickory-browser-debug .hickory-embed .cm-cursor, .hickory-browser-debug .hickory-embed .cm-dropCursor { border-left-color: var(--fg); }
</storycode:copy>
## Keep retained run identities inside the viewport

A retained run's UUID can make a native select wider than the mobile debugger.
Allow its grid item to shrink, wrap the adjacent source fingerprint, and constrain the select itself; clipping the page
would hide controls rather than keep them usable. The homepage's existing 390px
browser assertion exercises this after running and restarting the program.

<storycode:copy id="browser-debug-history-width">
.hickory-browser-debug section[aria-label="Runtime run history"] { min-width: 0; overflow-wrap: anywhere; }
.hickory-browser-debug section[aria-label="Runtime run history"] label { display: block; }
.hickory-browser-debug section[aria-label="Runtime run history"] select { max-width: 100%; box-sizing: border-box; }
</storycode:copy>
<storycode:file path="apps/web/src/embed/debug.css"><storycode:paste select="#browser-debug-base-style"/><storycode:paste select="#browser-debug-caret-style"/><storycode:paste select="#browser-debug-history-width"/></storycode:file>

## Focused execution evidence

On 2026-10-26, `browserEmbedding.spec.ts`'s `homepage loads genuinely paused,
edits with a visible caret and restarts` flow passed in local Chromium against
the rebuilt static site. It exercised the exact caret color, edits, restart,
program output and 390px viewport assertion. Local Firefox could not start
because its expected binary was absent; WebKit lacked a system library. No local
cross-browser success or broad regression coverage is claimed.

</storycode:doc>
