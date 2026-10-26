<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/vite.site.config.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-2cea88e563a6"></a>
## `apps/web/vite.site.config.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Drawn Diagram Is Document Text](../../../guarantees/authoring/a-drawn-diagram-is-document-text/contract.md), [A viewer snapshot keeps recorded results historical](../../../guarantees/embedding/a-viewer-snapshot-keeps-recorded-results-historical/contract.md).

<a id="f-2cea88e563a6-revision"></a>
### `revision`

Build config for the **static marketing site**.
A second config rather than a multi-page build, because the two outputs go
to different places and are deployed by different things: this one to
hickorydocs.com, the default one into the desktop app. Sharing a source tree
is deliberate — the demos on the site are live demos of the editor, so they
legitimately import the same CodeMirror and ribbon code the app does. What
is NOT shared is what each build reaches for at runtime: the site has no API
client at all.

<storycode:copy id="f-2cea88e563a6-revision">
// Build config for the **static marketing site**.
//
// A second config rather than a multi-page build, because the two outputs go
// to different places and are deployed by different things: this one to
// hickorydocs.com, the default one into the desktop app. Sharing a source tree
// is deliberate — the demos on the site are live demos of the editor, so they
// legitimately import the same CodeMirror and ribbon code the app does. What
// is NOT shared is what each build reaches for at runtime: the site has no API
// client at all.
import { defineConfig, type Plugin } from "vite";
import react from "@vitejs/plugin-react";
import { resolve } from "node:path";
import { execFileSync } from "node:child_process";
function revision(): string {
  try { return process.env.GITHUB_SHA ?? execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim(); }
  catch { return ""; }
}

</storycode:copy>
<a id="f-2cea88e563a6-emitAsIndex"></a>
### `emitAsIndex`

Emit the entry as `index.html`.
Vite names an HTML output after its input, so a `site.html` entry produces
`dist-site/site.html` — and a static host serves `index.html` at `/`, so
the deployed domain would 404 at its own front door. The entry is not
simply *called* `index.html` because that name belongs to the desktop app's
build, which is what `vite dev` and Tauri expect to find.

<storycode:copy id="f-2cea88e563a6-emitAsIndex">
/// Emit the entry as `index.html`.
///
/// Vite names an HTML output after its input, so a `site.html` entry produces
/// `dist-site/site.html` — and a static host serves `index.html` at `/`, so
/// the deployed domain would 404 at its own front door. The entry is not
/// simply *called* `index.html` because that name belongs to the desktop app's
/// build, which is what `vite dev` and Tauri expect to find.
function emitAsIndex(): Plugin {
  return {
    name: "hickory:emit-as-index",
    enforce: "post",
    generateBundle(_options, bundle) {
      for (const [fileName, chunk] of Object.entries(bundle)) {
        if (fileName === "site.html") {
          delete bundle[fileName];
          chunk.fileName = "index.html";
          bundle["index.html"] = chunk;
        }
      }
    },
  };
}

</storycode:copy>
<a id="f-2cea88e563a6-export_statement"></a>
### `export_statement`

<storycode:copy id="f-2cea88e563a6-export_statement">
export default defineConfig({
  plugins: [react(), emitAsIndex()],
  define: { "import.meta.env.VITE_SOURCE_REVISION": JSON.stringify(revision()) },
  base: "./",
  worker: { format: "iife" },
  build: {
    outDir: "dist-site",
    emptyOutDir: true,
    rollupOptions: {
      input: { snapshot: resolve(import.meta.dirname, "snapshot.html"), site: resolve(import.meta.dirname, "site.html"), ...(process.env.HICKORY_SITE_ONLY === "1" ? {} : {
        embed: resolve(import.meta.dirname, "embed.html"), iframe: resolve(import.meta.dirname, "iframe.html"),
      }) },
    },
  },
});
</storycode:copy>

### Assemble `apps/web/vite.site.config.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/vite.site.config.ts"><storycode:paste select="#f-2cea88e563a6-revision"/><storycode:paste select="#f-2cea88e563a6-emitAsIndex"/><storycode:paste select="#f-2cea88e563a6-export_statement"/></storycode:file>

</storycode:doc>
