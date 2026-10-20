import { act, cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { renderMathInto } from "../editor/mathRender";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { MathPanel } from "./MathPanel";
vi.mock("../editor/mathRender", () => ({ renderMathInto: vi.fn() }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("promotes only the latest rendered equation and exposes actual source/fallback status explicitly", async () => {
  const completions = new Map<string, () => void>();
  vi.mocked(renderMathInto).mockImplementation((target, source) => new Promise(resolve => { completions.set(source, () => { target.textContent = `Rendered ${source}`; if (source === "new λ") { target.classList.add("cm-math--unparsed"); target.dataset.tip = "Engine unavailable"; } resolve(); }); }));
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const content = (source: string) => <UiActionContext.Provider value={registry}><MathPanel source={source} /></UiActionContext.Provider>;
  const view = render(content("old private equation"));
  view.rerender(content("new λ"));
  await act(async () => { completions.get("new λ")!(); });
  expect(screen.getByTestId("math-panel").textContent).toBe("Rendered new λ");
  await act(async () => { completions.get("old private equation")!(); });
  expect(screen.getByTestId("math-panel").textContent).toBe("Rendered new λ");
  const parent = registry.groups().groups[0];
  const action = registry.observe(parent).actions[0];
  expect(JSON.stringify(action)).not.toContain("new λ");
  const reading = await registry.act(registry.scope, { id: "acp:session", kind: "agent" }, { handle: action.handle, arguments: { offset: 0, limit: 8192 }, request_id: "read" });
  if (reading.status !== "result" || reading.outcome.status !== "completed") throw Error("Expected bytes");
  expect(JSON.parse(new TextDecoder().decode(new Uint8Array((reading.outcome.value as { bytes: number[] }).bytes)))).toMatchObject({ source: "new λ", rendering: { status: "source_fallback", error: "Engine unavailable" } });
});
