import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { MOCK } from "./api/client";
import { setSharedRealtime } from "./api/realtime";
import { applyEditorAppearance, loadEditorAppearance } from "./lib/editorAppearance";
import { applyStoredTheme } from "./lib/theme";
import { loadHickLang } from "./editor/hickLang";
import { loadKeymap } from "./lib/keymap";
import "./styles.css";

// The index.html inline script already themed the first paint; re-applying
// here keeps any embedding that skips that script (tests, the desktop
// webview's saved page) on the same contract.
applyStoredTheme();
applyEditorAppearance(loadEditorAppearance());

async function boot() {
  // The parser is WebAssembly and parses synchronously once loaded; the
  // editor must not mount before it is here.
  await loadHickLang();
  // The keymap is read once, before any editor is built: editors take their
  // bindings at construction (lib/keymap.ts).
  await loadKeymap();
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
