import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
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
  // Fetch the application, parser and keymap together. A static App import
  // would delay these requests until the whole editor bundle had been parsed.
  // Editors need the parser and their configured bindings before mounting.
  const [{ App }] = await Promise.all([import("./App"), loadHickLang(), loadKeymap()]);
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
