import staticHtml from "../index.html?raw";
import type { ReactNode } from "react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { act, screen } from "@testing-library/react";

const boot = vi.hoisted(() => ({ parser: vi.fn(), keys: vi.fn(), policy: vi.fn() }));
vi.mock("./api/client", () => {
  class ApiError extends Error { constructor(readonly status: number) { super(); } }
  return { MOCK: false, ApiError, request: boot.policy };
});
vi.mock("./components/PrivatePredictionCoach", () => ({ validatePrivatePredictionState: vi.fn() }));
vi.mock("./api/realtime", () => ({ setSharedRealtime: vi.fn() }));
vi.mock("./lib/editorAppearance", () => ({ applyEditorAppearance: vi.fn(), loadEditorAppearance: vi.fn() }));
vi.mock("./lib/theme", () => ({ applyStoredTheme: vi.fn() }));
vi.mock("./editor/hickLang", () => ({ loadHickLang: boot.parser }));
vi.mock("./lib/keymap", () => ({ loadKeymap: boot.keys }));
vi.mock("./lib/uiActionBridge", () => ({ UiActionsProvider: ({ children }: { children: ReactNode }) => children }));
vi.mock("./App", () => ({ App: () => <main>Retained workspace</main> }));

function loadStaticPage() {
  const page = new DOMParser().parseFromString(staticHtml, "text/html");
  document.body.innerHTML = page.body.innerHTML;
}
beforeEach(async () => {
  const { ApiError } = await import("./api/client");
  boot.policy.mockRejectedValue(new ApiError(404, "Ordinary workspace"));
  boot.keys.mockResolvedValue(undefined);
});
afterEach(() => { document.body.innerHTML = ""; vi.resetModules(); vi.unstubAllGlobals(); vi.resetAllMocks(); });

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

it("keeps the ordinary workspace mounted when the parser fails", async () => {
  loadStaticPage();
  boot.parser.mockRejectedValue(new Error("parser dependency unavailable"));
  boot.keys.mockResolvedValue(undefined);
  await act(async () => { await import("./main"); });
  expect((await screen.findByRole("main")).textContent).toBe("Retained workspace");
  expect(screen.queryByRole("alert")).toBeNull();
  expect(screen.queryByRole("status")).toBeNull();
});

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
