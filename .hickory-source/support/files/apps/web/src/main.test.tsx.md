<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/main.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `3af71020490f2ef73fb2dc6d75098adc923896bd4feb70e335aeee0acf15d7a3`.

<book:entry path="apps/web/src/main.test.tsx">
import staticHtml from "../index.html?raw";
import type { ReactNode } from "react";
import { afterEach, expect, it, vi } from "vitest";
import { act, screen } from "@testing-library/react";

const boot = vi.hoisted(() => ({ parser: vi.fn(), keys: vi.fn() }));
vi.mock("./api/client", () => {
  class ApiError extends Error { status = 404; }
  return { MOCK: false, ApiError, request: vi.fn(() => Promise.reject(new ApiError())) };
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
afterEach(() => { document.body.innerHTML = ""; vi.resetModules(); vi.unstubAllGlobals(); });

it("paints static opening status before the actual entrypoint's dependencies resolve, then mounts the workspace", async () => {
  loadStaticPage();
  expect(screen.getByRole("status").textContent).toBe("Opening Hickory Docs…");
  let ready!: () => void;
  boot.parser.mockImplementation(() => new Promise<void>(resolve => { ready = resolve; }));
  boot.keys.mockResolvedValue(undefined);
  await act(async () => { await import("./main"); });
  expect(screen.getByRole("status").textContent).toBe("Opening Hickory Docs…");
  expect(screen.queryByRole("main")).toBeNull();
  await act(async () => { ready(); });
  expect(screen.getByRole("main").textContent).toBe("Retained workspace");
  expect(screen.queryByRole("status")).toBeNull();
});

it("replaces static opening status with an actionable alert when the actual bootstrap dependency fails", async () => {
  loadStaticPage();
  boot.parser.mockRejectedValue(new Error("parser dependency unavailable"));
  boot.keys.mockResolvedValue(undefined);
  await act(async () => { await import("./main"); });
  expect(screen.getByRole("alert").textContent).toBe("Hickory Docs could not open. Reload this window to try again.");
  expect(screen.queryByRole("status")).toBeNull();
  expect(screen.queryByRole("main")).toBeNull();
});

it("updates the actual static opening paint with pending preparation progress", async () => {
  loadStaticPage();
  let finish!: () => void;
  boot.parser.mockImplementation(() => new Promise<void>(resolve => {finish=resolve;}));
  boot.keys.mockResolvedValue(undefined);
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
  ready=true;
  await act(async()=>{await waiting;finish();});
  expect(screen.getByRole("main").textContent).toBe("Retained workspace");
});
</book:entry>

</book:doc>
