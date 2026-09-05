import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { MOCK } from "./api/client";
import { setSharedRealtime } from "./api/realtime";
import { applyStoredTheme } from "./lib/theme";
import { loadHickLang } from "./editor/hickLang";
import "./styles.css";

// The index.html inline script already themed the first paint; re-applying
// here keeps any embedding that skips that script (tests, the desktop
// webview's saved page) on the same contract.
applyStoredTheme();

async function boot() {
  // The parser is WebAssembly and parses synchronously once loaded; the
  // editor must not mount before it is here.
  await loadHickLang();
  if (MOCK) {
    const { installMockApi, mockRealtime } = await import("./mock/mockApi");
    installMockApi();
    setSharedRealtime(mockRealtime);
  }
  createRoot(document.getElementById("root")!).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
}

void boot();
