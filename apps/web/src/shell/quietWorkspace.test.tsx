// Contract: .hickory-source/guarantees/shell/quiet-inline-workspace/contract.md
import { useRef } from "react";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { codexUsage, agentIdentity, deepseekWindow } from "../lib/agentStatus";
import { ProvenanceToggles } from "./ProvenanceToggles";
import { WorkspaceStatusExtras } from "../environments/WorkspaceStatusExtras";
import { AgentStatusBar, useAgentStatus } from "./AgentStatus";
import { BlankWelcome } from "../views/BlankWelcome";
import { FileToolMenu } from "./FileToolMenu";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import type { ProvenanceLayer } from "../lib/provenanceLayers";
import { request } from "../api/client";
import { MENU_EVENT } from "../lib/menuBridge";
vi.mock("../api/client", () => ({ request: vi.fn(), MOCK: false }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });

it("labels only painted families and leaves disabled preferences reachable separately", () => {
  const props = { layers: new Set(["lineage", "context"] as const), onToggle: vi.fn() };
  const ui = render(<ProvenanceToggles {...props} displayed={new Set(["context"])} />);
  expect(screen.queryByText("Lineage")).toBeNull();
  expect(screen.getByText("Context").tagName).toBe("SPAN");
  expect(screen.queryByRole("button")).toBeNull();
  ui.rerender(<ProvenanceToggles {...props} displayed={new Set()} />);
  expect(screen.queryByText("Context")).toBeNull();
  ui.rerender(<ProvenanceToggles {...props} />);
  fireEvent.click(screen.getByRole("button", { name: "Declared" }));
  expect(props.onToggle).toHaveBeenCalledWith("declared");
});

it("a healthy workspace has no environment or graph chrome", () => {
  const onEnvironments = vi.fn();
  const props = { provenance: { layers: new Set<ProvenanceLayer>(), displayed: new Set<ProvenanceLayer>(), onToggle: vi.fn() }, environmentCount: 0, onEnvironments };
  const ui = render(<WorkspaceStatusExtras {...props} />);
  expect(screen.queryByRole("button")).toBeNull();
  ui.rerender(<WorkspaceStatusExtras {...props} environmentCount={2} />);
  fireEvent.click(screen.getByRole("button", { name: "2 dependency problems" }));
  expect(onEnvironments).toHaveBeenCalledTimes(1);
});

it("preserves quota durations, real zeroes and unknown reset counts", () => {
  const now = Date.parse("2026-11-04T20:00:00Z"), future = now / 1000 + 100;
  const data = { rateLimits: { primary: { usedPercent: 25, windowDurationMins: 300, resetsAt: future }, secondary: { usedPercent: 0, windowDurationMins: 10080, resetsAt: future } }, rateLimitResetCredits: { availableCount: 0 } };
  expect(codexUsage(data, now)).toMatchObject({ windows: [{ label: "5h", used: 25 }, { label: "Week", used: 0 }], resets: 0 });
  expect(codexUsage(null, now)).toMatchObject({ windows: [], resets: null });
  expect(codexUsage({ rateLimits: { primary: { usedPercent: NaN, windowDurationMins: 300, resetsAt: future } } }, now).windows).toEqual([]);
  expect(codexUsage(data, future * 1000).windows).toEqual([]);
  expect(codexUsage({ rateLimits: { primary: { usedPercent: 10, windowDurationMins: 60, resetsAt: future } } }, now).windows[0].label).toBe("60m");
});

it("uses reported model, effort and speed, never model-name guesses", () => {
  expect(agentIdentity("codex", "", "ignored", { backend: "codex", ready: true, configOptions: [
    { id: "model", name: "Model", type: "select", currentValue: "m" },
    { id: "reasoning_effort", name: "Effort", type: "select", currentValue: "high" },
    { id: "fast_mode", name: "Speed", type: "boolean", currentValue: false },
  ] }, "Codex")).toBe("Codex via ACP · m · high · Default speed");
  expect(agentIdentity("claude", "", "ignored", null)).toBe("claude via ACP · Model not reported");
});

it("counts to the actual next DeepSeek transition across lunch, weekends and China holidays", () => {
  const at = (time: string) => deepseekWindow(Date.parse(time));
  expect(at("2026-11-04T00:59:59Z")).toEqual({ peak: false, changesAt: Date.parse("2026-11-04T01:00:00Z") });
  expect(at("2026-11-04T01:00:00Z")).toEqual({ peak: true, changesAt: Date.parse("2026-11-04T04:00:00Z") });
  expect(at("2026-11-04T04:00:00Z")).toEqual({ peak: false, changesAt: Date.parse("2026-11-04T06:00:00Z") });
  expect(at("2026-11-06T10:00:00Z")).toEqual({ peak: false, changesAt: Date.parse("2026-11-09T01:00:00Z") });
  expect(at("2026-10-01T01:00:00Z")).toEqual({ peak: false, changesAt: Date.parse("2026-10-08T01:00:00Z") });
  // A makeup working Saturday is still off-peak under the vendor's weekend rule.
  expect(at("2026-10-10T01:00:00Z")?.peak).toBe(false);
  expect(at("2027-01-01T01:00:00Z")).toBeNull();
});

function Conversation({ name, hidden = false, owner = name }: { name: string; hidden?: boolean; owner?: string }) {
  const node = useRef<HTMLElement>(null);
  useAgentStatus({ owner, identity: `${name} via ACP · model`, backend: name, provider: "", node, agents: [] }, true);
  return <section ref={node} hidden={hidden}><button>{name} conversation</button></section>;
}
it("follows the selected visible Agent pane and removes status when its owner closes", async () => {
  const ui = render(<><Conversation name="Claude" /><Conversation name="Grok" /><AgentStatusBar /></>);
  fireEvent.pointerDown(screen.getByRole("button", { name: "Claude conversation" }));
  expect(screen.getByText("Claude via ACP · model", { selector: "summary" })).toBeTruthy();
  ui.rerender(<><Conversation name="Claude" hidden /><Conversation name="Grok" /><AgentStatusBar /></>);
  await waitFor(() => expect(screen.getByText("Grok via ACP · model", { selector: "summary" })).toBeTruthy());
  ui.rerender(<AgentStatusBar />);
  expect(screen.queryByText(/via ACP/)).toBeNull();
  expect(request).not.toHaveBeenCalled();
});

it("retires status settings handles when the conversation owner changes with the same model", async () => {
  const registry = new UiActions({ workspace: "test", window: "test" }, () => true, async () => {});
  const ui = (owner: string) => <UiActionContext.Provider value={registry}><Conversation name="Claude" owner={owner} /><AgentStatusBar /></UiActionContext.Provider>;
  const rendered = render(ui("first-document"));
  await act(async () => fireEvent.click(screen.getByText("Claude via ACP · model", { selector: "summary" })));
  const settings = () => registry.groups().groups.flatMap(group => registry.observe(group).actions).find(action => action.label === "Change visible agent settings");
  await waitFor(() => expect(settings()).toBeTruthy());
  const old = settings()!;
  rendered.rerender(ui("second-document"));
  expect(await registry.act(registry.scope, { kind: "agent", id: "test" }, { handle: old.handle, arguments: {}, request_id: "retired-owner" })).toMatchObject({ status: "stale" });
});

it("discloses file tools on request and refuses handles bound to a previous file", async () => {
  const registry = new UiActions({ workspace: "test", window: "test" }, () => true, async () => {});
  const open = vi.fn();
  const ui = (name: string) => <UiActionContext.Provider value={registry}><FileToolMenu name={name} document readRevision={() => name} onOpen={open} /></UiActionContext.Provider>;
  const rendered = render(ui("a.md"));
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group).actions);
  expect(actions().some(action => action.label === "Review this file")).toBe(false);
  await act(async () => fireEvent.click(screen.getByLabelText("Tools for a.md")));
  const review = actions().find(action => action.label === "Review this file")!;
  await act(async () => { await registry.act(registry.scope, { kind: "agent", id: "test" }, { handle: review.handle, arguments: {}, request_id: "review-a" }); });
  expect(open).toHaveBeenCalledWith("review");
  rendered.rerender(ui("b.md"));
  expect(await registry.act(registry.scope, { kind: "agent", id: "test" }, { handle: review.handle, arguments: {}, request_id: "old-a" })).toMatchObject({ status: "stale" });
  await act(async () => fireEvent.click(screen.getByLabelText("Tools for b.md")));
  expect(actions().some(action => action.label === "Review this file")).toBe(false);
});

it("requests real native pickers and creates a new unsaved file from the blank window", async () => {
  vi.mocked(request).mockResolvedValue({ native: true });
  const menu = vi.fn(); window.addEventListener(MENU_EVENT, menu);
  try {
    render(<BlankWelcome />);
    await act(async () => fireEvent.click(screen.getByRole("button", { name: "Open File…" })));
    expect(request).toHaveBeenCalledWith("POST", "/api/window/open-file", {});
    await act(async () => fireEvent.click(screen.getByRole("button", { name: "Open Folder…" })));
    expect(request).toHaveBeenLastCalledWith("POST", "/api/window/open-folder", {});
    await act(async () => fireEvent.click(screen.getByRole("button", { name: "New File" })));
    expect((menu.mock.calls[0][0] as CustomEvent).detail).toBe("new");
  } finally { window.removeEventListener(MENU_EVENT, menu); }
});
