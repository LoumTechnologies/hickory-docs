<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/vite.config.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-eea049695b01"></a>
## `apps/web/vite.config.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-eea049695b01-env"></a>
### `env`

<storycode:copy id="f-eea049695b01-env">
import { defineConfig } from "vitest/config";
import type { Plugin } from "vite";
import react from "@vitejs/plugin-react";
import { execFileSync } from "node:child_process";

// Dev-server env (no @types/node dependency; Vite runs under Node).
const env: Record<string, string | undefined> =
  (globalThis as { process?: { env: Record<string, string | undefined> } }).process?.env ?? {};

</storycode:copy>
<a id="f-eea049695b01-export_statement"></a>
### `export_statement`

Vite embeds the UI stamp when it starts a development server or bundles an
explicitly opted-in desktop preview. A page that survives an engine restart
keeps its own identity. If hot replacement changes the stamp, invalidate
the virtual module and reload the page together with its updated modules.
The production bundle receives an empty stamp.

<storycode:copy id="dev-browser-build-stamp">
function devBuildStamp(): string {
  try {
    const git = (...args: string[]) => execFileSync("git", ["--no-optional-locks", "-C", "../..", ...args], {
      encoding: "utf8", stdio: ["ignore", "pipe", "ignore"],
    }).trim();
    if (git("rev-parse", "--show-prefix")) return "unknown";
    const commit = git("rev-parse", "--verify", "HEAD");
    const dirty = git("status", "--porcelain=v1", "--untracked-files=normal");
    return git("rev-parse", "--verify", "HEAD") === commit ? commit + (dirty ? "+" : "") : "unknown";
  } catch {
    return "unknown";
  }
}

</storycode:copy>

<storycode:copy id="dev-browser-stamp-module">
function devBuildModule(enabled: boolean): Plugin {
  const name = "virtual:hickory-dev-build";
  const id = "\0" + name;
  let stamp = enabled ? devBuildStamp() : "";
  return {
    name: "hickory-dev-build",
    resolveId(source) { if (source === name) return id; },
    load(source) { if (source === id) return `export default ${JSON.stringify(stamp)};`; },
    handleHotUpdate({ modules, server }) {
      if (!enabled || modules.length === 0) return;
      const nextStamp = devBuildStamp();
      if (stamp === nextStamp) return;
      stamp = nextStamp;
      const module = server.moduleGraph.getModuleById(id);
      if (module) server.moduleGraph.invalidateModule(module);
      server.ws.send({ type: "full-reload" });
    },
  };
}

</storycode:copy>

<storycode:copy id="f-eea049695b01-export_statement">
export default defineConfig(({ command, mode }) => ({
  plugins: [react(), devBuildModule(
    mode !== "test" && !env.HICKORY_VERSION && (command === "serve" || env.HICKORY_DEV_BUILD === "1"),
  )],
  server: {
    // Unset = let Vite pick (its default, incrementing past anything already
    // bound). Vite has no port-0 support — `!configPort` treats 0 as "unset"
    // — so this, not 0, is how the dev server stops needing an agreed port.
    // Port Zero publishes whatever it lands on at a stable
    // `*.portzero.local` name. A real WEB_PORT still pins it by hand.
    port: Number(env.WEB_PORT ?? 0) || undefined,
    // IPv4 loopback, explicitly. Vite's default binds `[::1]` ONLY, and the
    // Port Zero forwarder always dials `127.0.0.1:<port>` — so the default
    // leaves the tunnel connecting to nothing (curl reports an empty reply).
    // Not `true`/`0.0.0.0`: that would publish the dev server to the LAN,
    // which the tunnel does not need.
    host: "127.0.0.1",
    // Vite 6 answers 403 to any Host header it does not recognise, which is
    // every request arriving through a Port Zero tunnel. The leading dot
    // matches `portzero.local` itself and every subdomain of it.
    allowedHosts: [".portzero.local"],
    proxy: {
      "/api": {
        // The Simple launcher supplies the workspace host. Plain Vite can
        // proxy an independently started engine at the configured port.
        target: env.HICKORY_API_ORIGIN ?? `http://localhost:${env.HICKORY_SERVER_PORT ?? 8080}`,
        changeOrigin: true,
        ws: true,
      },
    },
  },
  test: {
    // Keep interpreter and editor timing checks stable on development machines
    // while native build checks are running alongside the frontend suite.
    maxWorkers: 2,
    environment: "jsdom",
    include: ["src/**/*.test.{ts,tsx}"],
    // Shims for the DOM measurements jsdom does not implement — without them
    // CodeMirror throws from inside a rAF, which reports as an unhandled
    // error beside a green suite rather than as a failure. See test-setup.ts.
    setupFiles: ["src/test-setup.ts"],
  },
}));
</storycode:copy>

### Assemble `apps/web/vite.config.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/vite.config.ts"><storycode:paste select="#f-eea049695b01-env"/><storycode:paste select="#dev-browser-build-stamp"/><storycode:paste select="#dev-browser-stamp-module"/><storycode:paste select="#f-eea049695b01-export_statement"/></storycode:file>

</storycode:doc>
