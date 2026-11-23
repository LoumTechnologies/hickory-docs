import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

// Dev-server env (no @types/node dependency; Vite runs under Node).
const env: Record<string, string | undefined> =
  (globalThis as { process?: { env: Record<string, string | undefined> } }).process?.env ?? {};

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
});
