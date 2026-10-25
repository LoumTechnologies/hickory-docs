<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/main.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [The Editor Does Not Wait For Workspace Preparation](../../../../../../authoring/the-editor-does-not-wait-for-workspace-preparation/contract.md)
- [Every Shortcut Is A Setting](../../../../../../editor-intelligence/every-shortcut-is-a-setting/contract.md)
- [Keep a prediction separate from the answer it is tested against](../../../../contract.md)

Captured SHA-256: `4113a8fa8972f7db15cabc20101c4457c9a36854d8bb425dadc4c932ad123687`.

<book:entry path="apps/web/src/main.tsx">
import { startupNotice, subscribeStartup } from "./api/workspaceStartup";
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

// Keep the static opening paint informative while policy/bootstrap requests
// wait on ownership preparation. React takes ownership only after bootstrap.
const stopOpeningNotice = subscribeStartup(() => {
  const status = document.querySelector<HTMLElement>("#root > .boot-status[role=status]");
  if (status) status.textContent = startupNotice() ?? "Opening Hickory Docs…";
});
async function boot() {
  // Fetch the application, parser and keymap together. A static App import
  // would delay these requests until the whole editor bundle had been parsed.
  // Editors need the parser and their configured bindings before mounting.
  if (!MOCK) {
    try {
      const privateSession = await request<PrivatePredictionState>("GET", "/api/prediction/state");
      validatePrivatePredictionState(privateSession);
      const [{ NativeWorkflowTour }] = await Promise.all([import("./components/NativeWorkflowTour"), loadHickLang()]);
      stopOpeningNotice();
      createRoot(document.getElementById("root")!).render(<StrictMode><UiActionsProvider><NativeWorkflowTour privateSession={privateSession} /></UiActionsProvider></StrictMode>);
      return;
    } catch (error) {
      // Only an explicit absent capability may select the ordinary workspace.
      // Network/protocol failures must not mount its independent context readers.
      if (!(error instanceof ApiError) || ![404, 422].includes(error.status)) {
        stopOpeningNotice();
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
  stopOpeningNotice();
  createRoot(document.getElementById("root")!).render(
    <StrictMode>
      <UiActionsProvider><App /></UiActionsProvider>
    </StrictMode>,
  );
}

void boot().catch(() => {
  stopOpeningNotice();
  createRoot(document.getElementById("root")!).render(
    <p className="boot-status" role="alert">Hickory Docs could not open. Reload this window to try again.</p>,
  );
});
</book:entry>

</book:doc>
