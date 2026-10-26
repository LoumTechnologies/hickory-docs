<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/main.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-44decff65943"></a>
## `apps/web/src/main.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The Editor Does Not Wait For Workspace Preparation](../../../../../../authoring/the-editor-does-not-wait-for-workspace-preparation/contract.md), [Every Shortcut Is A Setting](../../../../../../editor-intelligence/every-shortcut-is-a-setting/contract.md), [Keep a prediction separate from the answer it is tested against](../../../../contract.md).

<a id="f-44decff65943-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-44decff65943-expression_statement">
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
</storycode:copy>
<a id="f-44decff65943-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-44decff65943-expression_statement-2">
applyEditorAppearance(loadEditorAppearance());

</storycode:copy>
<a id="f-44decff65943-stopOpeningNotice"></a>
### `stopOpeningNotice`

Keep the static opening paint informative while policy/bootstrap requests
wait on ownership preparation. React takes ownership only after bootstrap.

<storycode:copy id="f-44decff65943-stopOpeningNotice">
// Keep the static opening paint informative while policy/bootstrap requests
// wait on ownership preparation. React takes ownership only after bootstrap.
const stopOpeningNotice = subscribeStartup(() => {
  const status = document.querySelector<HTMLElement>("#root > .boot-status[role=status]");
  if (status) status.textContent = startupNotice() ?? "Opening Hickory Docs…";
});
</storycode:copy>
<a id="f-44decff65943-boot"></a>
### `boot`

<storycode:copy id="f-44decff65943-boot">
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

</storycode:copy>
<a id="f-44decff65943-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-44decff65943-expression_statement-3">
void boot().catch(() => {
  stopOpeningNotice();
  createRoot(document.getElementById("root")!).render(
    <p className="boot-status" role="alert">Hickory Docs could not open. Reload this window to try again.</p>,
  );
});
</storycode:copy>

### Assemble `apps/web/src/main.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/main.tsx"><storycode:paste select="#f-44decff65943-expression_statement"/><storycode:paste select="#f-44decff65943-expression_statement-2"/><storycode:paste select="#f-44decff65943-stopOpeningNotice"/><storycode:paste select="#f-44decff65943-boot"/><storycode:paste select="#f-44decff65943-expression_statement-3"/></storycode:file>

</storycode:doc>
