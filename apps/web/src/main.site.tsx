// Entry point for the **static marketing site** (`site.html`).
//
// It renders one page and mounts no API client: the site has no server to talk
// to, and the demos on it run entirely in the browser. The desktop app's entry
// is `main.tsx`; the two never load each other's code.
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { LandingView } from "./views/LandingView";
import { applyStoredTheme } from "./lib/theme";
import { loadHickLang } from "./editor/hickLang";
import "./styles.css";

applyStoredTheme();

// The demos run the real parser in the browser — the same `hick-lang` the
// product ships, as WebAssembly — so a document the site shows is drawn the
// way the app draws it.
void loadHickLang().then(() => {
  createRoot(document.getElementById("root")!).render(
    <StrictMode>
      <LandingView />
    </StrictMode>,
  );
});
