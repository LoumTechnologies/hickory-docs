// @vitest-environment jsdom
//
// The generated pane's half of the two-way edit flow:
//  - a fresh copy arriving from the session (a re-weave) is reconciled into
//    the live buffer and the changed text flashes;
//  - content identical to the buffer — the round-trip of what was typed right
//    here — flashes nothing;
//  - while the pane's own edits are unsent, an incoming stale copy is NOT
//    adopted (it predates the keystrokes and would eat them);
//  - a successful save reports the server's `source_edits`, which is what the
//    owning document's editor flashes.

import { afterEach, describe, expect, it } from "vitest";
import { act, cleanup, render, waitFor } from "@testing-library/react";
import type { EditorView } from "@codemirror/view";
import { installMockHandler } from "../api/client";
import type { OutputFile, SourceEdit } from "../api/types";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { GeneratedFileView } from "./views";
import { viewBuffers } from "../lib/viewBuffers";

afterEach(async () => {
  cleanup();
  for (const id of viewBuffers.ids()) {
    await viewBuffers.get(id)?.discard();
    viewBuffers.forget(id);
  }
});

const file = (content: string): OutputFile => ({
  path: "src/hello.py",
  language: "python",
  content,
  provenance: [],
});

/** Mock server: serves `initial`, applies output edits, records them. */
function serve(initial: string) {
  const state = { content: initial, editBodies: [] as unknown[] };
  installMockHandler(async (method, path, body) => {
    if (method === "GET" && path.includes("/outputs/file")) return file(state.content);
    if (method === "POST" && path.endsWith("/outputs/edit")) {
      state.editBodies.push(body);
      const edits: SourceEdit[] = [
        { doc_path: "hello.md", span: [10, 15], text: "typed" },
      ];
      return { source_edits: edits, applied: true };
    }
    throw new Error(`unexpected ${method} ${path}`);
  });
  return state;
}

function mount(liveFile: OutputFile | null, onSourceEdits?: (e: SourceEdit[]) => void) {
  let view: EditorView | null = null;
  const utils = render(
    <GeneratedFileView
      docId="d1"
      path="src/hello.py"
      liveFile={liveFile}
      onSourceEdits={onSourceEdits}
      onReady={(target) => {
        view = target?.view ?? null;
      }}
    />,
  );
  return { ...utils, view: () => view };
}

const rerenderWith = (
  utils: ReturnType<typeof mount>,
  liveFile: OutputFile | null,
  onSourceEdits?: (e: SourceEdit[]) => void,
) =>
  utils.rerender(
    <GeneratedFileView
      docId="d1"
      path="src/hello.py"
      liveFile={liveFile}
      onSourceEdits={onSourceEdits}
      onReady={() => undefined}
    />,
  );

describe("GeneratedFileView live updates", () => {
  // Guarantee: docs/guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file.md.
  it("retains unsent text across a tab switch and saves it through the workspace", async () => {
    const server = serve("x = 1\n");
    const first = mount(null);
    await waitFor(() => expect(first.view()).not.toBeNull());
    act(() => first.view()!.dispatch({ changes: { from: 4, to: 5, insert: "2" }, userEvent: "input" }));
    first.unmount();
    const second = mount(null);
    await waitFor(() => expect(second.view()?.state.doc.toString()).toBe("x = 2\n"));
    await act(async () => expect(await viewBuffers.get("d1:src/hello.py")!.save()).toBe(true));
    expect(server.editBodies).toHaveLength(1);
    expect(viewBuffers.get("d1:src/hello.py")?.dirty).toBe(false);
  });

  it("holds retained text when the generated file changed while the tab was inactive", async () => {
    const server = serve("x = 1\n");
    const first = mount(null);
    await waitFor(() => expect(first.view()).not.toBeNull());
    act(() => first.view()!.dispatch({ changes: { from: 4, to: 5, insert: "2" }, userEvent: "input" }));
    first.unmount();
    server.content = "x = 3\n";
    const second = mount(null);
    await waitFor(() => expect(second.view()?.state.doc.toString()).toBe("x = 2\n"));
    await act(async () => expect(await viewBuffers.get("d1:src/hello.py")!.save()).toBe(false));
    expect(server.editBodies).toHaveLength(0);
    expect(second.container.textContent).toContain("changed while this buffer was inactive");
    await act(async () => viewBuffers.get("d1:src/hello.py")!.discard());
    expect(second.view()?.state.doc.toString()).toBe("x = 3\n");
  });

  it("does not duplicate an in-flight reverse edit after remounting", async () => {
    let finish!: () => void;
    let writes = 0;
    let remote = "x = 1\n";
    installMockHandler(async (method, path) => {
      if (method === "GET" && path.includes("/outputs/file")) return file(remote);
      if (method === "POST" && path.endsWith("/outputs/edit")) {
        writes++;
        await new Promise<void>(resolve => { finish = resolve; });
        remote = "x = 2\n";
        return { source_edits: [], applied: true };
      }
      throw new Error(`unexpected ${method} ${path}`);
    });
    const first = mount(null);
    await waitFor(() => expect(first.view()).not.toBeNull());
    act(() => first.view()!.dispatch({ changes: { from: 4, to: 5, insert: "2" }, userEvent: "input" }));
    const saving = viewBuffers.get("d1:src/hello.py")!.save();
    await waitFor(() => expect(writes).toBe(1));
    first.unmount();
    const second = mount(null);
    await waitFor(() => expect(second.view()?.state.doc.toString()).toBe("x = 2\n"));
    await act(async () => expect(await viewBuffers.get("d1:src/hello.py")!.save()).toBe(false));
    await act(async () => { finish(); await saving; });
    expect(writes).toBe(1);
    expect(viewBuffers.get("d1:src/hello.py")?.dirty).toBe(true);
    await act(async () => viewBuffers.get("d1:src/hello.py")!.discard());
    expect(second.view()?.state.doc.toString()).toBe("x = 2\n");
  });

  it("reconciles an incoming re-weave into the buffer and flashes the change", async () => {
    serve("print('one')\n");
    const utils = mount(null);
    await waitFor(() =>
      expect(utils.container.querySelector(".cm-content")?.textContent).toContain("print('one')"),
    );

    rerenderWith(utils, file("print('two')\n"));
    await waitFor(() =>
      expect(utils.container.querySelector(".cm-content")?.textContent).toContain("print('two')"),
    );
    // The arrived text carries the flash.
    const flash = [...utils.container.querySelectorAll(".cm-change-flash")]
      .map((el) => el.textContent)
      .join("");
    expect(flash).toContain("two");
  });

  it("flashes nothing when the incoming content matches the buffer (own-edit round-trip)", async () => {
    serve("print('same')\n");
    const utils = mount(null);
    await waitFor(() =>
      expect(utils.container.querySelector(".cm-content")?.textContent).toContain("print('same')"),
    );

    rerenderWith(utils, file("print('same')\n"));
    expect(utils.container.querySelector(".cm-change-flash")).toBeNull();
  });

  it("keeps unsent keystrokes when a stale copy arrives, then reports the save's source edits", async () => {
    serve("print('base')\n");
    const sourceEdits: SourceEdit[][] = [];
    const onSourceEdits = (e: SourceEdit[]) => sourceEdits.push(e);
    const utils = mount(null, onSourceEdits);
    await waitFor(() => expect(utils.view()).not.toBeNull());
    const view = utils.view()!;

    // The user types; the save is debounced, so for a moment the edit exists
    // only in this buffer.
    view.dispatch({
      changes: { from: 7, to: 11, insert: "typed" },
      userEvent: "input",
    });
    expect(view.state.doc.toString()).toBe("print('typed')\n");

    // A stale re-weave (woven before the keystrokes) must not be adopted.
    rerenderWith(utils, file("print('base')\n"), onSourceEdits);
    expect(view.state.doc.toString()).toBe("print('typed')\n");
    expect(utils.container.querySelector(".cm-change-flash")).toBeNull();

    // The debounced save lands and the server answers with where in the
    // document the edit resolved — the hook the document-editor flash hangs on.
    await waitFor(() => expect(sourceEdits).toHaveLength(1), { timeout: 3000 });
    expect(sourceEdits[0]).toEqual([{ doc_path: "hello.md", span: [10, 15], text: "typed" }]);
  }, 10000);
});

describe("removed chrome stays removed", () => {
  it("renders no explanatory hint and no provenance footer", async () => {
    serve("x = 1\n");
    const utils = mount(null);
    await waitFor(() =>
      expect(utils.container.querySelector(".cm-content")?.textContent).toContain("x = 1"),
    );
    expect(utils.container.textContent).not.toContain("Edit freely");
    expect(utils.container.querySelector(".generated-view__lineage")).toBeNull();
    expect(utils.container.querySelector(".output-pane-status")).toBeNull();
  });
});

// Guarantee: docs/guarantees/authoring/the-editor-keeps-its-space.md
it("reads the actual generated-file loading/failure path before its editor mounts", async () => {
  let fail!: (cause: Error) => void;
  installMockHandler(async (method, path) => {
    if (method === "GET" && path.includes("/outputs/file")) return new Promise((_resolve, reject) => { fail = reject; });
    if (method === "GET" && path.endsWith("/diverged")) return { diverged: {} };
    throw new Error(`Unexpected ${method} ${path}`);
  });
  const registry = new UiActions({ workspace: "p", window: "generated-loading" }, () => true, async () => {});
  const mounted = render(<UiActionContext.Provider value={registry}><GeneratedFileView docId="load-owner" path="missing.py" bufferId="loading-status-only" /></UiActionContext.Provider>);
  let id = 0;
  const read = async () => {
    const action = registry.groups().groups.flatMap(group => registry.observe(group, 64).actions).find(action => action.label === "Read generated file load and save status bytes")!;
    let reply: any;
    await act(async () => { reply = await registry.act(registry.scope, { id: "acp:status", kind: "agent" }, { handle: action.handle, arguments: { offset: 0, limit: 8192 }, request_id: String(++id) }); });
    return JSON.parse(new TextDecoder().decode(new Uint8Array(reply.outcome.value.bytes)));
  };
  expect(await read()).toMatchObject({ loading: true, loaded: false, message: "Loading missing.py…", source_read: false });
  await act(async () => fail(new Error("Generated file is unavailable")));
  await waitFor(() => expect(mounted.getByText("Generated file is unavailable")).toBeTruthy());
  expect(await read()).toMatchObject({ loading: false, error: "Generated file is unavailable", source_read: false });
  expect(mounted.container.querySelector(".cm-editor")).toBeNull();
});
