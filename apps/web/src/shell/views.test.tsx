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
import { cleanup, render, waitFor } from "@testing-library/react";
import type { EditorView } from "@codemirror/view";
import { installMockHandler } from "../api/client";
import type { OutputFile, SourceEdit } from "../api/types";
import { GeneratedFileView } from "./views";

afterEach(cleanup);

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
