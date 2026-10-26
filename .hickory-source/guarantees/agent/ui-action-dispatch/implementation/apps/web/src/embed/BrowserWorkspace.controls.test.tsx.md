<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/embed/BrowserWorkspace.controls.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-4bb8f636db00"></a>
## `apps/web/src/embed/BrowserWorkspace.controls.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-4bb8f636db00-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-4bb8f636db00-expression_statement">
// Guarantees: docs/guarantees/agent/ui-action-dispatch.md
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { BrowserWorkspace } from "./BrowserWorkspace";
import { createMemoryStorage, type WorkspaceStorage } from "./storage";
import { openSyncedStorage } from "./syncedStorage";
import { UiActionContext } from "../lib/uiActionBridge";
import { UiActions, type UiValue } from "../lib/uiActions";
vi.mock("./DebuggableDocument", () => ({ DebuggableDocument: ({ source, onChange }: { source: string; onChange: (change: { source: string }) => void }) => <textarea aria-label="Fixture editor" value={source} onChange={event => onChange({ source: event.target.value })} /> }));
</storycode:copy>
<a id="f-4bb8f636db00-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-4bb8f636db00-expression_statement-2">
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
</storycode:copy>
<a id="f-4bb8f636db00-setup"></a>
### `setup`

<storycode:copy id="f-4bb8f636db00-setup">
function setup(storage = createMemoryStorage()) {
  const registry = new UiActions({ workspace: "browser", window: "fixture" }, () => true, async () => {});
  const wrap = (storage: WorkspaceStorage) => <UiActionContext.Provider value={registry}><BrowserWorkspace storage={storage} /></UiActionContext.Provider>;
  const app = render(wrap(storage));
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  const begin = (label: string, args: Record<string, UiValue> = {}) => {
    const action = actions().find(action => action.label === label)!; expect(action, label).toBeDefined();
    return registry.act(registry.scope, { id: "parent", kind: "agent" }, { handle: action.handle, arguments: args, request_id: crypto.randomUUID() });
  };
  const run = async (label: string, args: Record<string, UiValue> = {}) => { let result: unknown; await act(async () => { result = await begin(label, args); }); return result; };
  return { storage, registry, app, wrap, actions, begin, run };
}
</storycode:copy>
<a id="f-4bb8f636db00-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-4bb8f636db00-expression_statement-3">
it("shares the real discard decision, import and save without an implicit document overwrite", async () => {
  const { run, begin, storage } = setup();
  await act(async () => {});
  let pending!: ReturnType<typeof begin>;
  await act(async () => { pending = begin("Import browser note text", { name: "import.md", text: "# Imported draft" }); });
  expect(screen.getByRole("dialog")).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: /^Cancel$/ }));
  await act(async () => { expect(await pending).toMatchObject({ outcome: { value: { cancelled: true, imported: false } } }); });
  expect(screen.getByLabelText("Fixture editor")).toHaveProperty("value", "# A browser note\n");
  await act(async () => { pending = begin("Import browser note text", { name: "import.md", text: "# Imported draft" }); });
  fireEvent.click(screen.getByRole("button", { name: "Discard and continue" }));
  await act(async () => { expect(await pending).toMatchObject({ outcome: { value: { imported: true, saved: false } } }); });
  expect(await storage.read("notes/import.md")).toBeNull();
  expect(await run("Save browser note")).toMatchObject({ outcome: { value: { saved: true, durability: "memory" } } });
  expect(new TextDecoder().decode((await storage.read("notes/import.md"))!.bytes)).toBe("# Imported draft");
});
</storycode:copy>
<a id="f-4bb8f636db00-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-4bb8f636db00-expression_statement-4">
it("preserves newer text while saving the captured bytes and refuses an atomic conflicting import", async () => {
  const storage = createMemoryStorage(), write = storage.write.bind(storage);
  let finish!: () => void;
  vi.spyOn(storage, "write").mockImplementation(async (...args) => { await new Promise<void>(resolve => { finish = resolve; }); return write(...args); });
  const { begin, run } = setup(storage); await act(async () => {});
  let pending!: ReturnType<typeof begin>;
  await act(async () => { pending = begin("Save browser note"); });
  fireEvent.change(screen.getByLabelText("Fixture editor"), { target: { value: "# Newer draft" } });
  await act(async () => { finish(); expect(await pending).toMatchObject({ outcome: { value: { saved: true, current_draft_matches: false } } }); });
  expect(screen.getByLabelText("Fixture editor")).toHaveProperty("value", "# Newer draft");
  expect(new TextDecoder().decode((await storage.read("notes/note.md"))!.bytes)).toBe("# A browser note\n");
  const bundle = { version: 1, files: [{ path: "other.md", base64: btoa("# Other") }, { path: "notes/note.md", base64: btoa("# Replacement") }] };
  expect(await run("Import browser workspace JSON", { json: JSON.stringify(bundle) })).toMatchObject({ outcome: { status: "unavailable" } });
  expect(await storage.read("other.md")).toBeNull();
  expect(screen.getByLabelText("Fixture editor")).toHaveProperty("value", "# Newer draft");
});
</storycode:copy>
<a id="f-4bb8f636db00-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-4bb8f636db00-expression_statement-5">
it("retires a replaced storage owner and prevents its late read from replacing the new draft", async () => {
  const old = createMemoryStorage(), next = createMemoryStorage();
  await old.write("old.md", new TextEncoder().encode("# Old storage"), null);
  let finish!: (value: Awaited<ReturnType<typeof old.read>>) => void;
  vi.spyOn(old, "read").mockImplementation(() => new Promise(resolve => { finish = resolve; }));
  const { app, wrap, actions, registry } = setup(old);
  await waitFor(() => expect(finish).toBeDefined());
  const handle = actions().find(action => action.label === "Save browser note")!.handle;
  app.rerender(wrap(next));
  await act(async () => { finish({ path: "old.md", bytes: new TextEncoder().encode("# Old storage"), revision: "old" }); });
  expect(screen.getByLabelText("Fixture editor")).toHaveProperty("value", "# A browser note\n");
  await act(async () => { expect(await registry.act(registry.scope, { id: "parent", kind: "agent" }, { handle, arguments: {}, request_id: "old-owner" })).toMatchObject({ status: "unavailable" }); });
  expect(await next.read("old.md")).toBeNull();
});

</storycode:copy>
<a id="f-4bb8f636db00-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-4bb8f636db00-expression_statement-6">
it("discloses retained sync evidence and refuses a resolution after the remote revision changes", async () => {
  const local = createMemoryStorage(), remote = createMemoryStorage(), bytes = (text: string) => new TextEncoder().encode(text);
  const base = await remote.write("note.md", bytes("# Base"), null);
  const storage = await openSyncedStorage(local, remote, "fixture"), file = (await storage.read("note.md"))!;
  await storage.write("note.md", bytes("# Ours"), file.revision);
  const theirs = await remote.write("note.md", bytes("# Theirs"), base.revision);
  await expect(storage.sync()).rejects.toThrow();
  const { run, actions } = setup(storage);
  await waitFor(() => expect(screen.getByLabelText("Fixture editor")).toHaveProperty("value", "# Ours"));
  expect(actions().some(action => action.label === "Read disclosed browser sync conflict bytes")).toBe(false);
  expect(await run("Review browser sync conflicts")).toMatchObject({ outcome: { value: { conflicts_disclosed: 1 } } });
  expect(actions().some(action => action.label === "Read disclosed browser sync conflict bytes")).toBe(true);
  await run("Set browser conflict visibility: note.md", { open: false });
  expect(actions().some(action => action.label === "Resolve browser conflict note.md: local")).toBe(false);
  expect(actions().some(action => action.label === "Read disclosed browser sync conflict bytes")).toBe(false);
  await run("Set browser conflict visibility: note.md", { open: true });
  await remote.write("note.md", bytes("# Newer remote"), theirs.revision);
  expect(await run("Resolve browser conflict note.md: local")).toMatchObject({ outcome: { status: "unavailable" } });
  expect(new TextDecoder().decode((await remote.read("note.md"))!.bytes)).toBe("# Newer remote");
  expect(screen.getByLabelText("Fixture editor")).toHaveProperty("value", "# Ours");
});
</storycode:copy>

### Assemble `apps/web/src/embed/BrowserWorkspace.controls.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/BrowserWorkspace.controls.test.tsx"><storycode:paste select="#f-4bb8f636db00-expression_statement"/><storycode:paste select="#f-4bb8f636db00-expression_statement-2"/><storycode:paste select="#f-4bb8f636db00-setup"/><storycode:paste select="#f-4bb8f636db00-expression_statement-3"/><storycode:paste select="#f-4bb8f636db00-expression_statement-4"/><storycode:paste select="#f-4bb8f636db00-expression_statement-5"/><storycode:paste select="#f-4bb8f636db00-expression_statement-6"/></storycode:file>

</storycode:doc>
