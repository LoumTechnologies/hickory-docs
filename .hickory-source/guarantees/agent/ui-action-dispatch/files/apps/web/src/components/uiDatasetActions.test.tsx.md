<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/uiDatasetActions.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md)

Captured SHA-256: `3b48f633d1cb6eb46bc383e1e0ead2199e3426a3d9cdeb9ffcb08576a19d4fc4`.

<book:entry path="apps/web/src/components/uiDatasetActions.test.tsx">
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions, type UiResult, type UiValue } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { DatasetPanel } from "./DatasetPanel";
import { ReadOnlyTable, type NotebookTable } from "./ReadOnlyTable";
afterEach(() => { cleanup(); vi.unstubAllGlobals(); });
const table: NotebookTable = { kind: "table", schema: [{ name: "Name", dtype: "str" }, { name: "Cost", dtype: "int" }], rows: [["PRIVATE λ", 2], ["Other", 1]], offset: 0, has_more: true, total: 200 };
function setup() {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {}); let n = 0;
  return { registry, invoke: async (parent: string, label: string, args: Record<string, UiValue>) => {
    const action = registry.observe(parent).actions.find(a => a.label === label)!;
    let result: UiResult | undefined;
    await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action.handle, arguments: args, request_id: String(++n) }); });
    return result as UiResult;
  } };
}
function decode(reading: UiResult) {
  if (reading.status !== "result" || reading.outcome.status !== "completed") throw Error("Expected bytes");
  return JSON.parse(new TextDecoder().decode(new Uint8Array((reading.outcome.value as { bytes: number[] }).bytes)));
}
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("reads exact visible values, sorts and filters the loaded page without a dataset write", async () => {
  const { registry, invoke } = setup();
  render(<UiActionContext.Provider value={registry}><ReadOnlyTable table={table} /></UiActionContext.Provider>);
  const parent = registry.groups().groups[0];
  expect(JSON.stringify(registry.observe(parent))).not.toContain("PRIVATE λ");
  await invoke(parent, "Sort read-only table page", { column: 1, descending: false });
  let reading = decode(await invoke(parent, "Read visible table page bytes", { offset: 0, limit: 8192 }));
  expect(reading.rows).toEqual([["Other", 1], ["PRIVATE λ", 2]]);
  await invoke(parent, "Filter read-only table page", { filter: "λ" });
  reading = decode(await invoke(parent, "Read visible table page bytes", { offset: 0, limit: 8192 }));
  expect(reading.rows).toEqual([["PRIVATE λ", 2]]); expect(reading.loaded_rows).toBe(2); expect(reading.has_more).toBe(true);
  expect(await invoke(parent, "Sort read-only table page", { column: 99, descending: true })).toMatchObject({ outcome: { status: "unavailable" } });
  fireEvent.change(screen.getByRole("textbox"), { target: { value: "Other" } });
  expect(decode(await invoke(parent, "Read visible table page bytes", { offset: 0, limit: 8192 })).rows).toEqual([["Other", 1]]);
});
it("requires explicit S3 loading and binds paging to the actually returned preview", async () => {
  const fetcher = vi.fn(async (url: string) => ({ ok: true, json: async () => new URL(url, "http://local").searchParams.get("offset") === "100" ? { ...table, rows: [["Next page", 3]], offset: 100, has_more: false } : table }));
  vi.stubGlobal("fetch", fetcher);
  const { registry, invoke } = setup();
  render(<UiActionContext.Provider value={registry}><DatasetPanel src="s3://bucket/data.parquet" format="parquet" path="doc.md" /></UiActionContext.Provider>);
  const parent = registry.groups().groups.find(g => g.startsWith("dataset:"))!;
  expect(fetcher).not.toHaveBeenCalled();
  await invoke(parent, "Load dataset from S3", {});
  await screen.findByText("PRIVATE λ");
  expect(fetcher).toHaveBeenCalledWith(expect.stringContaining("limit=100"), expect.anything());
  expect(registry.observe(parent).actions.some(a => a.label === "Load dataset from S3")).toBe(false);
  const old = registry.observe(parent).actions.find(a => a.label === "Next dataset page")!;
  await invoke(parent, "Next dataset page", {});
  await screen.findByText("Next page");
  expect(screen.queryByText("PRIVATE λ")).toBeNull();
  expect(registry.observe(parent).actions.find(a => a.label === "Next dataset page")!.enabled).toBe(false);
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "old-page" })).toMatchObject({ status: "unavailable" });
  const reading = decode(await invoke(parent, "Read dataset preview status bytes", { offset: 0, limit: 8192 }));
  expect(reading).toMatchObject({ page: 2, source_offset: 100, rows: 1, has_more: false });
});
</book:entry>

</book:doc>
