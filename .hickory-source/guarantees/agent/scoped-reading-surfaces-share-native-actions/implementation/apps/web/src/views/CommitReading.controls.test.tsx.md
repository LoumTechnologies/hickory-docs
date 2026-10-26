<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/CommitReading.controls.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-ee73e4c07840"></a>
## `apps/web/src/views/CommitReading.controls.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Captured reading surfaces share owned native actions](../../../../../contract.md).

<a id="f-ee73e4c07840-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-ee73e4c07840-expression_statement">
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { api, request } from "../api/client";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { CommitReading } from "./CommitReading";
vi.mock("../api/client", () => ({ api: { gitReading: vi.fn() }, request: vi.fn() }));
</storycode:copy>
<a id="f-ee73e4c07840-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-ee73e4c07840-expression_statement-2">
vi.mock("./DocumentReading", () => ({ DocumentComparison: (props: { projects?: unknown }) => <output aria-label="Loaded comparison contexts">{JSON.stringify(props.projects ?? null)}</output> }));
</storycode:copy>
<a id="f-ee73e4c07840-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-ee73e4c07840-expression_statement-3">
afterEach(() => { cleanup(); vi.clearAllMocks(); });
</storycode:copy>
<a id="f-ee73e4c07840-record"></a>
### `record`

<storycode:copy id="f-ee73e4c07840-record">
const record = (sha: string) => ({ sha, parent: "parent-" + sha, message: "message", files: [{ path: "note.md", from: "note.md", status: "modified", binary: false, before: "old", after: "new" }] });
</storycode:copy>
<a id="f-ee73e4c07840-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-ee73e4c07840-expression_statement-4">
// Guarantee: docs/guarantees/agent/scoped-reading-surfaces-share-native-actions.md
it("uses the native context loader for agents, refuses a late replaced commit and retains retry after failure", async () => {
  vi.mocked(api.gitReading).mockImplementation(async sha => record(sha));
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const ui = (sha: string) => <UiActionContext.Provider value={registry}><CommitReading sha={sha} /></UiActionContext.Provider>;
  const rendered = render(ui("first"));
  await screen.findByRole("button", { name: "Load complete revision contexts" });
  const action = () => registry.groups().groups.flatMap(group => registry.observe(group).actions).find(action => action.label === "Load complete commit revision contexts")!;
  const resolvers: ((value: unknown) => void)[] = [];
  vi.mocked(request).mockImplementation(() => new Promise(resolve => resolvers.push(resolve)));
  const descriptor = action();
  let pending!: ReturnType<typeof registry.act>;
  await act(async () => { pending = registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: descriptor.handle, arguments: {}, request_id: "first" }); });
  expect(request).toHaveBeenCalledTimes(2);
  expect(request).toHaveBeenNthCalledWith(1, "POST", "/api/revision-projects", { kind: "git", revision: "parent-first" });
  expect(screen.getByRole("button", { name: "Load complete revision contexts" })).toHaveProperty("disabled", true);
  rendered.rerender(ui("second"));
  await waitFor(() => expect(screen.getByText(/second · read-only/)).toBeTruthy());
  await act(async () => { resolvers.forEach(resolve => resolve({ files: [{ path: "old-first", source: "old" }] })); expect(await pending).toMatchObject({ outcome: { status: "unavailable" } }); });
  expect(screen.getAllByLabelText("Loaded comparison contexts").every(node => node.textContent === "null")).toBe(true);
  expect(await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: descriptor.handle, arguments: {}, request_id: "old" })).toMatchObject({ status: "unavailable" });
  vi.mocked(request).mockRejectedValueOnce(new Error("context unavailable")).mockResolvedValue({ files: [] });
  await act(async () => fireEvent.click(screen.getByRole("button", { name: "Load complete revision contexts" })));
  expect(await screen.findByRole("alert")).toHaveProperty("textContent", "context unavailable");
  vi.mocked(request).mockResolvedValue({ files: [{ path: "second.md", source: "retained" }] });
  await act(async () => fireEvent.click(screen.getByRole("button", { name: "Load complete revision contexts" })));
  expect(screen.queryByRole("alert")).toBeNull();
  expect(screen.getAllByLabelText("Loaded comparison contexts").some(node => node.textContent?.includes("second.md"))).toBe(true);
});
</storycode:copy>

### Assemble `apps/web/src/views/CommitReading.controls.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/CommitReading.controls.test.tsx"><storycode:paste select="#f-ee73e4c07840-expression_statement"/><storycode:paste select="#f-ee73e4c07840-expression_statement-2"/><storycode:paste select="#f-ee73e4c07840-expression_statement-3"/><storycode:paste select="#f-ee73e4c07840-record"/><storycode:paste select="#f-ee73e4c07840-expression_statement-4"/></storycode:file>

</storycode:doc>
