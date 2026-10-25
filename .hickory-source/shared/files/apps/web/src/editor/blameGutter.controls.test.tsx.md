<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/editor/blameGutter.controls.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Authorship display accepts non-Git providers](../../../../../../guarantees/lineage/authorship-display-accepts-non-git-providers/contract.md)
- [The Blame Column Is Optional And Honest](../../../../../../guarantees/lineage/the-blame-column-is-optional-and-honest/contract.md)

Captured SHA-256: `71f6848ef77b902dae7fd8dd38ffca6c3fdd67c6c6098d0813d1994db1ff7684`.

<book:entry path="apps/web/src/editor/blameGutter.controls.test.tsx">
// Guarantees: docs/guarantees/lineage/the-blame-column-is-optional-and-honest.md
// docs/guarantees/lineage/authorship-display-accepts-non-git-providers.md
import { act, cleanup, render } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { UiActions, type UiValue } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useEditorActions } from "./uiEditorActions";
import { blameGutter, setBlame, setBlameShown, setAuthorshipShown, type LineBlame } from "./blameGutter";
let view: EditorView | undefined;
afterEach(() => { cleanup(); const host = view?.dom.parentElement; view?.destroy(); host?.remove(); });
it("reads the real visible gutter/color metadata progressively and retires stale attribution on edit", async () => {
  const host = document.body.appendChild(document.createElement("div"));
  const current = new EditorView({ parent: host, state: EditorState.create({ doc: "private body\nsecond", extensions: blameGutter() }) }); view = current;
  const registry = new UiActions({ workspace: "p", window: "attribution" }, () => true, async () => {});
  function Bridge() { useEditorActions(current, "attribution"); return null; }
  render(<UiActionContext.Provider value={registry}><Bridge /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args: Record<string, UiValue>) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "acp:reader", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return result; };
  const detail = "Read displayed attribution detail bytes", page = "Read displayed attribution line page";
  expect(actions().some(action => action.label === page)).toBe(false);
  const entry: LineBlame & { text: string } = { line: 1, author: "Ada Lovelace", email: "ada@example.com", commit: "abc", summary: "retained metadata", time: 1, uncommitted: false, origin: "agent", provider: "Installed provider", text: "NEVER INCLUDE PROVIDER SOURCE" };
  await act(async () => current.dispatch({ effects: [setBlame.of([entry]), setBlameShown.of(true), setAuthorshipShown.of(true)] }));
  expect(JSON.stringify(actions())).not.toContain("retained metadata");
  expect(JSON.stringify(actions())).not.toContain("private body");
  expect(await run(page, { first_line: 1, count: 2 })).toMatchObject({ outcome: { value: { lines: [{ line: 1, gutter_present: true, authorship_present: true, origin: "agent" }, { line: 2, gutter_present: false }], source_read: false } } });
  const reply = await run(detail, { line: 1, offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  const text = new TextDecoder().decode(new Uint8Array(reply.outcome.value.bytes));
  expect(text).not.toContain(entry.text);
  expect(JSON.parse(text)).toMatchObject({ gutter: { tooltip: current.dom.querySelector<HTMLElement>(".cm-blame-entry")!.dataset.tip, provider: "Installed provider" }, authorship: { origin: "agent" } });
  const old = actions().find(action => action.label === detail)!;
  await act(async () => current.dispatch({ changes: { from: 0, insert: "new " } }));
  expect(current.dom.querySelector(".cm-blame-entry")).toBeNull();
  expect(await registry.act(registry.scope, { id: "acp:reader", kind: "agent" }, { handle: old.handle, arguments: { line: 1, offset: 0, limit: 8192 }, request_id: "stale" })).not.toMatchObject({ outcome: { status: "completed" } });
  expect(await run(detail, { line: 1, offset: 0, limit: 8192 })).toMatchObject({ outcome: { status: "unavailable" } });
  await act(async () => current.dispatch({ effects: [setBlame.of([{ ...entry, summary: "X".repeat(9000) }]), setBlameShown.of(false)] }));
  const bounded = await run(detail, { line: 1, offset: 0, limit: 64 }) as { outcome: { value: { bytes: number[]; next: number } } };
  expect(bounded.outcome.value.bytes).toHaveLength(64);
  expect(bounded.outcome.value.next).toBe(64);
  await act(async () => current.dispatch({ effects: setAuthorshipShown.of(false) }));
  expect(actions().some(action => action.label === detail)).toBe(false);
});
</book:entry>

</book:doc>
