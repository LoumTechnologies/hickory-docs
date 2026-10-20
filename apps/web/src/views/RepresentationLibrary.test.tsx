// @vitest-environment jsdom
import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { representations, type Representation } from "../api/representations";
import { OPEN_REPRESENTATION } from "../lib/openRepresentation";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { RepresentationLibrary } from "./RepresentationLibrary";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("shares source selection and actual create/discard results without exposing source in its catalogue", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const view: Representation = { id: "actual-view", revision: "r1", backing: { kind: "files", paths: ["code.py"] }, source: "PRIVATE SOURCE", files: [{ path: "code.py", content: "PRIVATE CODE", hash: "h", provenance: [] }], explanation_stale: false };
  vi.spyOn(representations, "list").mockResolvedValue({ views: [] });
  const create = vi.spyOn(representations, "create").mockRejectedValueOnce(new Error("source missing")).mockResolvedValue(view);
  const discard = vi.spyOn(representations, "discard").mockRejectedValueOnce(new Error("storage unavailable")).mockResolvedValue({ ok: true });
  const navigated = vi.fn(); window.addEventListener(OPEN_REPRESENTATION, navigated);
  try {
    render(<UiActionContext.Provider value={registry}><RepresentationLibrary /></UiActionContext.Provider>);
    await waitFor(() => expect((screen.getByRole("button", { name: "Refresh views" }) as HTMLButtonElement).disabled).toBe(false));
    const actions = () => registry.groups().groups.flatMap(group => registry.observe(group).actions);
    let id = 0;
    const run = async (label: string, args = {}) => { let reply: unknown; await act(async () => { reply = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: actions().find(a => a.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return reply; };
    await run("Set literate library source paths", { value: " code.py \nother.py\n" });
    expect((screen.getByLabelText("Literate source files") as HTMLTextAreaElement).value).toContain("other.py");
    expect(await run("Create and open literate view from source paths")).toMatchObject({ status: "result", outcome: { status: "unavailable", reason: "source missing" } });
    expect(navigated).not.toHaveBeenCalled();
    expect(await run("Create and open literate view from source paths")).toMatchObject({ status: "result", outcome: { status: "completed", value: { created_view: "actual-view", revision: "r1", navigation_requested: true, verified: false } } });
    expect(create).toHaveBeenLastCalledWith({ kind: "files", paths: ["code.py", "other.py"] });
    expect(navigated.mock.calls[0][0].detail.id).toBe("actual-view");
    const catalogue = await run("Read literate view catalogue bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
    const text = new TextDecoder().decode(new Uint8Array(catalogue.outcome.value.bytes));
    expect(text).toContain("actual-view"); expect(text).not.toContain("PRIVATE");
    expect(await run("Discard local literate view actual-view")).toMatchObject({ status: "result", outcome: { status: "unavailable", reason: "storage unavailable" } });
    expect(screen.getByRole("button", { name: "code.py" })).toBeTruthy();
    expect(await run("Discard local literate view actual-view")).toMatchObject({ status: "result", outcome: { status: "completed", value: { discarded_view: "actual-view", host_reply: '{"ok":true}' } } });
    expect(discard).toHaveBeenCalledTimes(2);
    expect(actions().some(a => a.label === "Open literate view actual-view")).toBe(false);
  } finally { window.removeEventListener(OPEN_REPRESENTATION, navigated); }
});
