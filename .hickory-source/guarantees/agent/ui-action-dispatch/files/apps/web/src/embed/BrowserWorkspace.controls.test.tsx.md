<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/embed/BrowserWorkspace.controls.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md)

Captured SHA-256: `8f7320564f70c6edfb9977070def78911c57d00daccf1f06b35adf610db1b832`.

<book:entry path="apps/web/src/embed/BrowserWorkspace.controls.test.tsx">
// Guarantees: docs/guarantees/agent/ui-action-dispatch.md
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { BrowserWorkspace } from "./BrowserWorkspace";
import { createMemoryStorage, type WorkspaceStorage } from "./storage";
import { openSyncedStorage } from "./syncedStorage";
import { UiActionContext } from "../lib/uiActionBridge";
import { UiActions, type UiValue } from "../lib/uiActions";
vi.mock("./DebuggableDocument", () => ({ DebuggableDocument: ({ source, onChange }: { source: string; onChange: (change: { source: string }) => void }) => <textarea aria-label="Fixture editor" value={source} onChange={event => onChange({ source: event.target.value })} /> }));
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
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
</book:entry>

</book:doc>
