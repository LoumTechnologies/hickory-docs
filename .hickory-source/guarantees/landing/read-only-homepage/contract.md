<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The homepage opens a read-only Simple Browser document

For maintainers updating the public introduction to Hickory Docs.

A visitor to `/` sees the introductory Markdown document and its file tree in
the Simple Browser UI. The published bytes are read-only. Text is available
while the parser loads and links can lead to the demos, published source, and
downloads without a running engine.

## Update the introduction

Edit [home.md](home.md). Vite bundles the authored document as text, so a copy
change only needs that file in the commit. Run `just preview-browser-embedding`
and open the local address it prints to preview the site. Push the commit to
`master` to run **Deploy Site**, which packages and publishes to Cloudflare
Pages. The previous interactive examples are served at `/site.html`.

## Mount the browser document

Give the shared Simple editor a published, transient buffer. This uses the
same permission and transaction checks as read-only recorded messages, and
keeps live language and launch services out of the public reader. The document
mounts from its bundled text and the shared rich renderer attaches separately.
The [static source viewer's link decorations](../../../mechanisms/apps/web/src/embed/sourceLinks.ts.md)
turn public prose links into native anchors. The homepage resolves same-origin
HTTP paths and HTTPS URLs.

<storycode:copy id="site-home-reader">
import { createRoot } from "react-dom/client";
import homeSource from "../../../../.hickory-source/guarantees/landing/read-only-homepage/home.md?raw";
import { Buffer, Buffers } from "./buffers";
import { Document } from "./Document";
import { RichOwners } from "./RichInline";
import "./simple.css";

function browserLink(href: string) {
  try {
    const url = new URL(href, window.location.href);
    return url.protocol === "https:" || (url.protocol === "http:" && url.origin === window.location.origin)
      ? url.href : null;
  } catch {
    return null;
  }
}

const buffers = new Buffers(() => {});
const file = { name: "home.md", path: "home.md", dir: false, generated_by: "published site" };
const buffer = new Buffer(file, homeSource, "", buffers.changed, true);
buffers.values.set(file.path, buffer);

createRoot(document.getElementById("root")!).render(
  <main className="site-home">
    <aside className="group" aria-label="Files">
      <div className="tabs"><span>Files</span></div>
      <div className="file-tree"><ul><li aria-current="true">{file.name}</li></ul></div>
    </aside>
    <section className="group" aria-label={file.name}>
      <div className="tabs"><div className="tab" data-active="true"><span>{file.name}</span></div></div>
      <div className="pane-with-status">
        <Document buffer={buffer} />
        <footer className="pane-status" aria-label="Document status">Read-only</footer>
      </div>
    </section>
    <RichOwners buffers={buffers} browserLink={browserLink} />
  </main>,
);
</storycode:copy>

<storycode:file path="apps/web/src/simple/main.site.tsx"><storycode:paste select="#site-home-reader"/></storycode:file>

## Serve the same document at the domain root

[Vite's site build](../../../mechanisms/apps/web/vite.site.config.ts.md)
publishes this HTML entry as `index.html`. The entry's layout uses the Simple
Browser tabs, file tree, editor, and status row, with a narrow file column on
small screens. The app's own entry and the demo page have independent bundles.

<storycode:entry path="apps/web/home.html">
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <meta name="color-scheme" content="light dark" />
    <title>Hickory Docs</title>
    <meta name="description" content="Explore Hickory Docs, an IDE for literate Markdown." />
    <style>
      main.site-home {
        grid-template: minmax(0, 1fr) / clamp(76px, 21vw, 200px) minmax(0, 1fr);
      }
      .site-home > aside { border-inline-end: 1px solid color-mix(in srgb, CanvasText 20%, Canvas); }
      .site-home .tabs span { padding: 5px 8px; }
      .site-home .file-tree [aria-current] { background: color-mix(in srgb, Highlight 15%, Canvas); padding: 4px; }
      .site-home .editor-cm-host .cm-content { max-width: 84ch; }
      .site-home .editor-cm-host .cm-scroller { line-height: 1.55; }
      .site-home .source-prose-link { color: LinkText; }
    </style>
  </head>
  <body>
    <div id="root"><p role="status">Opening home.md…</p></div>
    <script type="module" src="/src/simple/main.site.tsx"></script>
  </body>
</html>
</storycode:entry>

## Verify the served page

The [browser witness](../../../mechanisms/apps/web/e2e/browserEmbedding.spec.ts.md#f-31019d3567c7-homepage-reader)
holds the parser request while checking the published source, attempts to type
in the editor, and confirms that the same editor remains mounted when rich
presentation becomes available. The deploy workflow runs this witness on
Chromium, Firefox, and WebKit alongside the demo checks.

</storycode:doc>
