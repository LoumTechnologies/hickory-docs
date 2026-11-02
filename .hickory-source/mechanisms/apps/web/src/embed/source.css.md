<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Keep source navigation readable beside code

For engineers maintaining the public source browser.

Give the rendered tutorial the main column. Keep the files pane quiet and scrollable on the left, and place only compact breadcrumbs above the document. Remove the embed border so the reader sees the document itself rather than a second framed application. On narrow screens, retain a small files column and wrap paths.

<storycode:copy id="public-source-apps-web-src-embed-source-css">
.source-browser { display: grid; grid-template-columns: 17rem minmax(0, 1fr); min-height: 100vh; }
.source-files { position: sticky; top: 0; height: 100vh; overflow: auto; padding: 1.5rem .75rem; border-right: 1px solid color-mix(in srgb, currentColor 12%, transparent); font-size: .85rem; }
.source-files ul { list-style: none; padding: 0 0 0 .8rem; margin: .25rem 0; }
.source-files nav > ul { padding-left: 0; }
.source-files li { margin: .2rem 0; }
.source-files a { display: block; padding: .2rem .35rem; border-radius: .2rem; text-decoration: none; overflow-wrap: anywhere; }
.source-files a[aria-current="page"] { background: color-mix(in srgb, currentColor 12%, transparent); font-weight: 600; }
.source-files summary { padding: .2rem 0; cursor: pointer; overflow-wrap: anywhere; }
.source-document { min-width: 0; padding: 1.25rem clamp(1rem, 4vw, 4rem) 4rem; }
.source-breadcrumbs { font-size: .8rem; opacity: .75; margin-bottom: 1.5rem; }
.source-breadcrumbs ol { display: flex; flex-wrap: wrap; align-items: baseline; gap: .35rem; list-style: none; padding: 0; margin: 0; }
.source-breadcrumbs li { overflow-wrap: anywhere; }
.source-breadcrumbs li + li::before { content: "/"; margin-right: .35rem; opacity: .5; }
.source-breadcrumbs a { text-decoration: none; }
.source-document .hickory-embed { border: 0; border-radius: 0; background: transparent; }
.source-document .cm-editor { border: 0; border-radius: 0; background: transparent; }
.source-document .cm-content { padding: 0 0 2rem; }
.source-document .source-prose-link { color: var(--embed-accent); text-decoration: underline; cursor: pointer; }
@media (max-width: 700px) {
  .source-browser { grid-template-columns: 7rem minmax(0, 1fr); }
  .source-files { padding: 1rem .25rem; font-size: .7rem; }
  .source-files ul { padding-left: .4rem; }
  .source-document { padding: 1rem .75rem 3rem; }
}
</storycode:copy>

<storycode:file path="apps/web/src/embed/source.css"><storycode:paste select="#public-source-apps-web-src-embed-source-css"/></storycode:file>
</storycode:doc>
