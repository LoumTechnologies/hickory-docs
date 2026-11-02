<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Keep source navigation readable beside code

For engineers maintaining the public source browser.

Give the rendered tutorial the main column. Keep the files pane quiet and scrollable on the left, and place only compact breadcrumbs above the document. Remove the embed border so the reader sees the document itself rather than a second framed application. On narrow screens, collapse the files pane behind a 44-pixel Files button so the document keeps its full reading width. Constrain the editor content to the reading column and wrap long code lines, including unbroken identifiers. Otherwise a max-content code line widens the shared content flex item and clips prose even when the page itself reports no horizontal overflow. Captured graphs retain their own horizontal scrollers. Anchor the nonmodal symbol inspector to the bottom with safe-area padding and a bounded height; add document padding while it is open so the last line remains reachable.

Use the app’s compact monospace file buffer and shared folding, with one line per filename and horizontal scrolling for long paths. On touch devices give each row and fold control a 44-pixel target; on desktop keep the app’s denser line spacing. The tree owns its own scroll viewport so remembered navigation state is independent of the document.

Bind the embed palette to the shared document tokens rather than its standalone dark defaults. Code payload surfaces, selection, syntax and the inspector must change along with the page background; otherwise a light page retains dark code islands or low-contrast text. Scope these bindings to the published reader so other embed hosts retain control of their presentation.

<storycode:copy id="public-source-apps-web-src-embed-source-css">
.source-browser { display: grid; grid-template-columns: 17rem minmax(0, 1fr); min-height: 100vh; }
.source-files { position: sticky; top: 0; height: 100dvh; box-sizing: border-box; padding: .5rem .25rem; border-right: 1px solid var(--border); }
.source-files nav, .source-tree, .source-tree .cm-editor { height: 100%; }
.source-tree .cm-scroller { overflow: auto; font-size: .8rem; }
.source-tree .cm-line { white-space: pre; line-height: 1.65; }
.source-tree a { color: inherit; text-decoration: none; }
.source-tree a:hover { text-decoration: underline; }
.source-tree .source-tree-directory { color: var(--fg-muted); cursor: pointer; }
.source-tree .source-tree-current { background: var(--select-bg); color: var(--fg); }
.source-tree .cm-gutterElement { cursor: pointer; }
@media (pointer: coarse) {
  .source-tree .cm-line, .source-tree .cm-gutterElement { line-height: 44px; }
}
.source-document { min-width: 0; padding: 1.25rem clamp(1rem, 4vw, 4rem) 4rem; }
.source-breadcrumbs { font-size: .8rem; opacity: .75; margin-bottom: 1.5rem; }
.source-breadcrumbs ol { display: flex; flex-wrap: wrap; align-items: baseline; gap: .35rem; list-style: none; padding: 0; margin: 0; }
.source-breadcrumbs li { overflow-wrap: anywhere; }
.source-breadcrumbs li + li::before { content: "/"; margin-right: .35rem; opacity: .5; }
.source-breadcrumbs a { text-decoration: none; }
.source-document .hickory-embed {
  --embed-bg: var(--bg);
  --embed-raised: var(--bg-raised);
  --embed-fg: var(--fg);
  --embed-muted: var(--fg-muted);
  --embed-border: var(--border);
  --embed-accent: var(--accent);
  --embed-teal: var(--accent-2);
  --embed-string: var(--tok-string);
  --embed-literal: var(--tok-literal);
  border: 0; border-radius: 0; background: transparent; }
.source-document .hickory-embed .cm-file-line, .source-document .hickory-embed .cm-cell-line { background: var(--bg-sunken); }
.source-document .hickory-embed .cm-focused .cm-selectionBackground { background: var(--select-bg); }
.source-document .hickory-embed .tok-keyword { color: var(--tok-keyword); }
.source-document .hickory-embed .tok-typeName { color: var(--tok-type); }
.source-document .cm-editor { border: 0; border-radius: 0; background: transparent; }
.source-document .cm-content { padding: 0 0 2rem; min-width: 0; width: 100%; }
.source-document .cm-code-line { width: auto; min-width: 0; white-space: pre-wrap; overflow-wrap: anywhere; }
.source-document .source-prose-link { color: var(--embed-accent); text-decoration: underline; cursor: pointer; }
.source-files-toggle { display: none; }
.source-document pre { max-width: 100%; overflow: auto; }
.source-document .cm-scroller { max-width: 100%; }
.source-inspector { position: fixed; z-index: 30; right: 1rem; bottom: 1rem; width: min(32rem, calc(100% - 2rem)); max-height: 45dvh; overflow: auto; overscroll-behavior: contain; padding: 1rem; border: 1px solid color-mix(in srgb, currentColor 30%, transparent); border-radius: .75rem; background: var(--bg-raised); color: var(--fg); box-shadow: 0 4px 24px #0003; overflow-wrap: anywhere; }
.source-inspector header { display: flex; align-items: center; justify-content: space-between; gap: 1rem; }
.source-inspector header strong { min-width: 0; }
.source-inspector button, .source-inspector a { min-height: 44px; display: flex; align-items: center; }
.source-inspector summary { min-height: 44px; line-height: 44px; cursor: pointer; }
.source-inspector button { padding: .5rem .75rem; }
.source-inspector ul { padding-left: 1.25rem; }
.source-inspector pre { white-space: pre-wrap; overflow-wrap: anywhere; }
.source-document.has-inspector { padding-bottom: 50dvh; }
@media (max-width: 700px) {
  .source-browser { display: block; }
  .source-files-toggle { display: block; position: sticky; top: 0; z-index: 20; min-height: 44px; padding: .5rem 1rem; margin: .5rem; background: Canvas; color: CanvasText; border: 1px solid; border-radius: .35rem; }
  .source-files { display: none; position: static; height: 60dvh; max-height: 60dvh; padding: .5rem; border-right: 0; border-bottom: 1px solid; font-size: .9rem; }
  .source-files.is-open { display: block; }
  .source-document { padding: 1rem .75rem 3rem; }
  .source-inspector { right: 0; bottom: 0; width: 100%; max-height: 45dvh; box-sizing: border-box; border-radius: .75rem .75rem 0 0; padding: .75rem .75rem max(.75rem, env(safe-area-inset-bottom)); }
}
</storycode:copy>

<storycode:file path="apps/web/src/embed/source.css"><storycode:paste select="#public-source-apps-web-src-embed-source-css"/></storycode:file>
</storycode:doc>
