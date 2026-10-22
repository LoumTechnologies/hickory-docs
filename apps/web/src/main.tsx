import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { MOCK, ApiError, request } from "./api/client";
import { validatePrivatePredictionState, type PrivatePredictionState } from "./components/PrivatePredictionCoach";
import { setSharedRealtime } from "./api/realtime";
import { applyEditorAppearance, loadEditorAppearance } from "./lib/editorAppearance";
import { applyStoredTheme } from "./lib/theme";
import { loadHickLang } from "./editor/hickLang";
import { loadKeymap } from "./lib/keymap";
import "./styles.css";
import { UiActionsProvider } from "./lib/uiActionBridge";

// The index.html inline script already themed the first paint; re-applying
// here keeps any embedding that skips that script (tests, the desktop
// webview's saved page) on the same contract.
applyStoredTheme();
applyEditorAppearance(loadEditorAppearance());

async function boot() {
  // Fetch the application, parser and keymap together. A static App import
  // would delay these requests until the whole editor bundle had been parsed.
  // Editors need the parser and their configured bindings before mounting.
  if (!MOCK) {
    try {
      const privateSession = await request<PrivatePredictionState>("GET", "/api/prediction/state");
      validatePrivatePredictionState(privateSession);
      const [{ NativeWorkflowTour }] = await Promise.all([import("./components/NativeWorkflowTour"), loadHickLang()]);
      createRoot(document.getElementById("root")!).render(<StrictMode><UiActionsProvider><NativeWorkflowTour privateSession={privateSession} /></UiActionsProvider></StrictMode>);
      return;
    } catch (error) {
      // Only an explicit absent capability may select the ordinary workspace.
      // Network/protocol failures must not mount its independent context readers.
      if (!(error instanceof ApiError) || ![404, 422].includes(error.status)) {
        createRoot(document.getElementById("root")!).render(<p role="alert">Workspace policy could not be established. Reload this window to retry.</p>);
        return;
      }
    }
  }
  const [{ App }] = await Promise.all([import("./App"), loadHickLang(), loadKeymap()]);
  if (MOCK) {
    const { installMockApi, mockRealtime } = await import("./mock/mockApi");
    installMockApi();
    setSharedRealtime(mockRealtime);
  }
  createRoot(document.getElementById("root")!).render(
    <StrictMode>
      <UiActionsProvider><App /></UiActionsProvider>
    </StrictMode>,
  );
}

void boot();
