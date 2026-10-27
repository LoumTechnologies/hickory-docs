<mobile:doc xmlns:mobile="http://www.hickorydocs.com/1.0">
# Build the mobile surface independently

For maintainers building the phone UI. Its entry point imports shared editor code, not the desktop workspace bootstrap or native server. The same page runs as an explicitly labeled browser preview for focused interaction checks.

## Mobile assets

The separate output directory prevents a mobile build from replacing desktop assets. Development uses the Tauri dev URL; production embeds relative assets and the existing WASM parser.

<mobile:copy id="mobile-web-build-mobile-assets">
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  base: "./",
  server: { host: "127.0.0.1", port: 1420, strictPort: true },
  build: { outDir: "dist-mobile", rollupOptions: { input: "mobile.html" } },
});
</mobile:copy>

Assemble `apps/web/vite.mobile.config.ts` in the order explained above.

<mobile:file path="apps/web/vite.mobile.config.ts"><mobile:paste select="#mobile-web-build-mobile-assets"/></mobile:file>

## Load shared parser

Load the existing Rust/WASM parser before mounting the shared Hickory editor decorations. A bootstrap failure is shown as an error rather than leaving an empty screen.

<mobile:copy id="mobile-web-build-load-shared-parser">
import { createRoot } from "react-dom/client";
import { loadHickLang } from "../editor/hickLang";
import { MobileApp } from "./MobileApp";
import "./mobile.css";

void loadHickLang().then(() => {
  createRoot(document.getElementById("root")!).render(<MobileApp />);
}).catch((error: unknown) => {
  const root = document.getElementById("root")!;
  root.setAttribute("role", "alert");
  root.textContent = `Hickory Docs could not open: ${String(error)}`;
});
</mobile:copy>

Assemble `apps/web/src/mobile/main.tsx` in the order explained above.

<mobile:file path="apps/web/src/mobile/main.tsx"><mobile:paste select="#mobile-web-build-load-shared-parser"/></mobile:file>

</mobile:doc>
