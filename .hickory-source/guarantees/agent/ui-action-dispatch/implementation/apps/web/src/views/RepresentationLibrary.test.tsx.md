<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/RepresentationLibrary.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b21809301432"></a>
## `apps/web/src/views/RepresentationLibrary.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-b21809301432-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-b21809301432-expression_statement">
// @vitest-environment jsdom
import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { representations, type Representation } from "../api/representations";
import { OPEN_REPRESENTATION } from "../lib/openRepresentation";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { RepresentationLibrary } from "./RepresentationLibrary";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
</storycode:copy>
<a id="f-b21809301432-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-b21809301432-expression_statement-2">
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
</storycode:copy>

### Assemble `apps/web/src/views/RepresentationLibrary.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/RepresentationLibrary.test.tsx"><storycode:paste select="#f-b21809301432-expression_statement"/><storycode:paste select="#f-b21809301432-expression_statement-2"/></storycode:file>

</storycode:doc>
