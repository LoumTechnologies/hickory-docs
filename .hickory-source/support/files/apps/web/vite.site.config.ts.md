<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/vite.site.config.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Drawn Diagram Is Document Text](../../../../guarantees/authoring/a-drawn-diagram-is-document-text/contract.md)
- [A viewer snapshot keeps recorded results historical](../../../../guarantees/embedding/a-viewer-snapshot-keeps-recorded-results-historical/contract.md)

Captured SHA-256: `ff62ef1f48662ae24d5decc9b596b873cda17bb07f449cf42b3d3cbc090aae41`.

<book:entry path="apps/web/vite.site.config.ts">
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
</book:entry>

</book:doc>
