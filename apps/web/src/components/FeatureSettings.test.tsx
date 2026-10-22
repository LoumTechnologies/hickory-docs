// docs/guarantees/agent/acp-agents-are-first-class.md
import { afterEach, expect, it } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { FeatureSettings } from "./FeatureSettings";
afterEach(cleanup);
it("opens outside a clipped pane, keeps controls usable, and dismisses locally", () => {
  const { container } = render(<div style={{ overflow: "hidden" }}><FeatureSettings label="Agent settings"><label>Agent<select aria-label="Agent"><option>Codex</option></select></label></FeatureSettings></div>);
  const gear = screen.getByRole("button", { name: "Agent settings" });
  expect(screen.queryByRole("region")).toBeNull();
  fireEvent.click(gear);
  const panel = screen.getByRole("region", { name: "Agent settings" });
  expect(container.contains(panel)).toBe(false);
  fireEvent.pointerDown(screen.getByLabelText("Agent"));
  expect(screen.getByRole("region")).toBe(panel);
  fireEvent.keyDown(panel, { key: "Escape" });
  expect(screen.queryByRole("region")).toBeNull();
  fireEvent.click(gear);
  fireEvent.pointerDown(document.body);
  expect(screen.queryByRole("region")).toBeNull();
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("waits for a real child preference before Escape unmounts it, without weakening retirement", async () => {
  const { act, waitFor } = await import("@testing-library/react");
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const { AgentPicker } = await import("./AgentPicker");
  const changes: string[] = [];
  let release!: () => void;
  const registry = new UiActions({ workspace: "project", window: "settings" }, () => true, receipt => receipt.stage === "requested" && receipt.action.label === "Select conversation agent" ? new Promise<void>(resolve => { release = resolve; }) : Promise.resolve());
  render(<UiActionContext.Provider value={registry}><FeatureSettings label="Agent settings" owner="doc"><AgentPicker scope="doc" backend="python" agents={[{ id: "python", name: "Python", available: true }, { id: "external", name: "External", available: true }] as never} disabled={false} detecting={false} error={null} onChange={value => changes.push(value)} onRefresh={() => {}} /></FeatureSettings></UiActionContext.Provider>);
  fireEvent.click(screen.getByRole("button", { name: "Agent settings" }));
  fireEvent.change(screen.getByRole("combobox", { name: "Agent" }), { target: { value: "external" } });
  await waitFor(() => expect(release).toBeTypeOf("function"));
  fireEvent.keyDown(screen.getByRole("combobox", { name: "Agent" }), { key: "Escape" });
  expect(screen.getByRole("region", { name: "Agent settings" })).toBeTruthy();
  expect(changes).toEqual([]);
  await act(async () => { release(); });
  await waitFor(() => expect(screen.queryByRole("region", { name: "Agent settings" })).toBeNull());
  expect(changes).toEqual(["external"]);
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("excludes the visibility command itself while waiting for a pending human preference", async () => {
  const { act, waitFor } = await import("@testing-library/react");
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const { AgentPicker } = await import("./AgentPicker");
  let release!: () => void;
  const changed: string[] = [];
  const registry = new UiActions({ workspace: "project", window: "settings-close" }, () => true, receipt => receipt.stage === "requested" && receipt.action.label === "Select conversation agent" ? new Promise<void>(resolve => { release = resolve; }) : Promise.resolve());
  render(<UiActionContext.Provider value={registry}><FeatureSettings label="Agent settings" owner="doc"><AgentPicker scope="doc" backend="python" agents={[{ id: "python", name: "Python", available: true }, { id: "external", name: "External", available: true }] as never} disabled={false} detecting={false} error={null} onChange={value => changed.push(value)} onRefresh={() => {}} /></FeatureSettings></UiActionContext.Provider>);
  fireEvent.click(screen.getByRole("button", { name: "Agent settings" }));
  fireEvent.change(screen.getByRole("combobox", { name: "Agent" }), { target: { value: "external" } });
  await waitFor(() => expect(release).toBeTypeOf("function"));
  const close = registry.observe("feature-settings").actions[0];
  let closing!: ReturnType<typeof registry.act>;
  await act(async () => { closing = registry.act(registry.scope, { id: "local-user", kind: "human" }, { handle: close.handle, arguments: { open: false }, request_id: "close-settings" }); });
  expect(screen.getByRole("region", { name: "Agent settings" })).toBeTruthy();
  let result: unknown;
  await act(async () => { release(); result = await closing; });
  expect(result).toMatchObject({ status: "result", outcome: { status: "completed", value: { open: false } } });
  expect(changed).toEqual(["external"]);
  expect(screen.queryByRole("region", { name: "Agent settings" })).toBeNull();
});
