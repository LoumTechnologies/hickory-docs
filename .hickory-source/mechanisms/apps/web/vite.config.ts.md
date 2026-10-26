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
import react from "@vitejs/plugin-react";

// Dev-server env (no @types/node dependency; Vite runs under Node).
const env: Record<string, string | undefined> =
  (globalThis as { process?: { env: Record<string, string | undefined> } }).process?.env ?? {};

</storycode:copy>
<a id="f-eea049695b01-export_statement"></a>
### `export_statement`

<storycode:copy id="f-eea049695b01-export_statement">
export default defineConfig({
  plugins: [react()],
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
        // The API's Port Zero domain (scripts/dev.sh sets this). Falling back
        // to a localhost port keeps a bare `npm run dev` working without the
        // daemon. `ws: true` matters: the whole live-sync layer is WebSocket.
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
});
</storycode:copy>

### Assemble `apps/web/vite.config.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/vite.config.ts"><storycode:paste select="#f-eea049695b01-env"/><storycode:paste select="#f-eea049695b01-export_statement"/></storycode:file>

</storycode:doc>
