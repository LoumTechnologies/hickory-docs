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

// Publish the read-only Simple Browser entry at the static host's root.
// Keep index.html in the source tree as the live workspace entry.
function emitAsIndex(): Plugin {
  return {
    name: "hickory:emit-as-index",
    enforce: "post",
    generateBundle(_options, bundle) {
      for (const [fileName, chunk] of Object.entries(bundle)) {
        if (fileName === "home.html") {
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
      input: { home: resolve(import.meta.dirname, "home.html"), source: resolve(import.meta.dirname, "source.html"), ci: resolve(import.meta.dirname, "ci.html"), snapshot: resolve(import.meta.dirname, "snapshot.html"), site: resolve(import.meta.dirname, "site.html"), ...(process.env.HICKORY_SITE_ONLY === "1" ? {} : {
        embed: resolve(import.meta.dirname, "embed.html"), iframe: resolve(import.meta.dirname, "iframe.html"),
      }) },
    },
  },
});
