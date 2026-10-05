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
  base: "./",
  worker: { format: "iife" },
  build: {
    outDir: "dist-site",
    emptyOutDir: true,
    rollupOptions: {
      input: { site: resolve(import.meta.dirname, "site.html"), embed: resolve(import.meta.dirname, "embed.html"), iframe: resolve(import.meta.dirname, "iframe.html") },
    },
  },
});
