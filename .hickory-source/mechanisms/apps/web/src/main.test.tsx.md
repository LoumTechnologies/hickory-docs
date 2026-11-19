<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/main.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-059a51f9194b"></a>
## `apps/web/src/main.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-059a51f9194b-boot"></a>
### `boot`

<storycode:copy id="f-059a51f9194b-boot">
import staticHtml from "../index.html?raw";
import type { ReactNode } from "react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { act, screen } from "@testing-library/react";

const boot = vi.hoisted(() => ({ parser: vi.fn(), keys: vi.fn(), policy: vi.fn() }));
</storycode:copy>
<a id="f-059a51f9194b-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-059a51f9194b-expression_statement">
vi.mock("./api/client", () => {
  class ApiError extends Error { constructor(readonly status: number) { super(); } }
  return { MOCK: false, ApiError, request: boot.policy };
});
</storycode:copy>
<a id="f-059a51f9194b-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-059a51f9194b-expression_statement-2">
vi.mock("./components/PrivatePredictionCoach", () => ({ validatePrivatePredictionState: vi.fn() }));
</storycode:copy>
<a id="f-059a51f9194b-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-059a51f9194b-expression_statement-3">
vi.mock("./api/realtime", () => ({ setSharedRealtime: vi.fn() }));
</storycode:copy>
<a id="f-059a51f9194b-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-059a51f9194b-expression_statement-4">
vi.mock("./lib/editorAppearance", () => ({ applyEditorAppearance: vi.fn(), loadEditorAppearance: vi.fn() }));
</storycode:copy>
<a id="f-059a51f9194b-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-059a51f9194b-expression_statement-5">
vi.mock("./lib/theme", () => ({ applyStoredTheme: vi.fn() }));
</storycode:copy>
<a id="f-059a51f9194b-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-059a51f9194b-expression_statement-6">
vi.mock("./editor/hickLang", () => ({ loadHickLang: boot.parser }));
</storycode:copy>
<a id="f-059a51f9194b-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-059a51f9194b-expression_statement-7">
vi.mock("./lib/keymap", () => ({ loadKeymap: boot.keys }));
</storycode:copy>
<a id="f-059a51f9194b-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-059a51f9194b-expression_statement-8">
vi.mock("./lib/uiActionBridge", () => ({ UiActionsProvider: ({ children }: { children: ReactNode }) => children }));
</storycode:copy>
<a id="f-059a51f9194b-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-059a51f9194b-expression_statement-9">
vi.mock("./App", () => ({ App: () => <main>Retained workspace</main> }));

</storycode:copy>
<a id="f-059a51f9194b-loadStaticPage"></a>
### `loadStaticPage`

<storycode:copy id="f-059a51f9194b-loadStaticPage">
function loadStaticPage() {
  const page = new DOMParser().parseFromString(staticHtml, "text/html");
  document.body.innerHTML = page.body.innerHTML;
}
</storycode:copy>
<a id="f-059a51f9194b-expression_statement-10"></a>
### `expression_statement`

<storycode:copy id="f-059a51f9194b-expression_statement-10">
beforeEach(async () => {
  const { ApiError } = await import("./api/client");
  boot.policy.mockRejectedValue(new ApiError(404, "Ordinary workspace"));
  boot.keys.mockResolvedValue(undefined);
});
afterEach(() => { document.body.innerHTML = ""; vi.resetModules(); vi.unstubAllGlobals(); vi.resetAllMocks(); });

</storycode:copy>
<a id="f-059a51f9194b-expression_statement-11"></a>
### `expression_statement`

<storycode:copy id="f-059a51f9194b-expression_statement-11">
it("mounts the ordinary workspace before its parser and saved keymap arrive", async () => {
  loadStaticPage();
  expect(screen.getByRole("status").textContent).toBe("Opening Hickory Docs…");
  let ready!: () => void;
  boot.parser.mockImplementation(() => new Promise<void>(resolve => { ready = resolve; }));
  let keymapReady!: () => void;
  boot.keys.mockImplementation(() => new Promise<void>(resolve => { keymapReady = resolve; }));
  await act(async () => { await import("./main"); });
  const workspace = await screen.findByRole("main");
  await act(async () => { ready(); keymapReady(); });
  expect(screen.getByRole("main")).toBe(workspace);
  expect(screen.queryByRole("status")).toBeNull();
});

</storycode:copy>
<a id="f-059a51f9194b-expression_statement-12"></a>
### `expression_statement`

<storycode:copy id="f-059a51f9194b-expression_statement-12">
it("keeps the ordinary workspace mounted when the parser fails", async () => {
  loadStaticPage();
  boot.parser.mockRejectedValue(new Error("parser dependency unavailable"));
  boot.keys.mockResolvedValue(undefined);
  await act(async () => { await import("./main"); });
  expect((await screen.findByRole("main")).textContent).toBe("Retained workspace");
  expect(screen.queryByRole("alert")).toBeNull();
  expect(screen.queryByRole("status")).toBeNull();
});

</storycode:copy>
<a id="f-059a51f9194b-expression_statement-13"></a>
### `expression_statement`

<storycode:copy id="f-059a51f9194b-expression_statement-13">
it("updates the actual static opening paint with pending preparation progress", async () => {
  loadStaticPage();
  const { ApiError } = await import("./api/client");
  let finish!: () => void;
  boot.policy.mockImplementation(() => new Promise((_, reject) => { finish = () => reject(new ApiError(404, "Ordinary workspace")); }));
  boot.parser.mockResolvedValue(undefined);
  await act(async () => { await import("./main"); });
  let ready=false;
  vi.stubGlobal("fetch",vi.fn(async()=>new Response(JSON.stringify(ready ? {ready:true} : {
    ready:false,phase:"weaving",completed:2,total:5,document:"sessions/large.md",elapsed_ms:20000,stage_elapsed_ms:17000,
  }))));
  const {waitForWorkspace}=await import("./api/workspaceStartup");
  let waiting!:Promise<void>;
  await act(async()=>{waiting=waitForWorkspace();await new Promise(resolve=>setTimeout(resolve,0));});
  expect(screen.getByRole("status").textContent).toContain("2 of 5 documents finished");
  expect(screen.getByRole("status").textContent).toContain("still pending, not complete");
  expect(screen.queryByRole("main")).toBeNull();
  expect(boot.parser).not.toHaveBeenCalled();
  ready=true;
  await act(async()=>{await waiting;finish();});
  expect((await screen.findByRole("main")).textContent).toBe("Retained workspace");
});
</storycode:copy>

### Assemble `apps/web/src/main.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/main.test.tsx"><storycode:paste select="#f-059a51f9194b-boot"/><storycode:paste select="#f-059a51f9194b-expression_statement"/><storycode:paste select="#f-059a51f9194b-expression_statement-2"/><storycode:paste select="#f-059a51f9194b-expression_statement-3"/><storycode:paste select="#f-059a51f9194b-expression_statement-4"/><storycode:paste select="#f-059a51f9194b-expression_statement-5"/><storycode:paste select="#f-059a51f9194b-expression_statement-6"/><storycode:paste select="#f-059a51f9194b-expression_statement-7"/><storycode:paste select="#f-059a51f9194b-expression_statement-8"/><storycode:paste select="#f-059a51f9194b-expression_statement-9"/><storycode:paste select="#f-059a51f9194b-loadStaticPage"/><storycode:paste select="#f-059a51f9194b-expression_statement-10"/><storycode:paste select="#f-059a51f9194b-expression_statement-11"/><storycode:paste select="#f-059a51f9194b-expression_statement-12"/><storycode:paste select="#f-059a51f9194b-expression_statement-13"/></storycode:file>

</storycode:doc>
