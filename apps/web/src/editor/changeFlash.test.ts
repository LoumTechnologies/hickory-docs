// @vitest-environment jsdom
//
// The fading change-flash: marks on text that changed because of an edit
// somewhere else. What matters: the marks appear where asked, follow the
// text through further edits, come off on their own once the fade is over —
// and NEVER appear for content identical to the buffer, which is how the
// user's own just-typed edit round-tripping through the server stays silent.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { EditorState, type Extension } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { FLASH_MS, changeFlashField, flashSpans, syncAndFlash } from "./changeFlash";

function editor(doc: string, extensions: Extension = []) {
  return new EditorView({
    parent: document.body,
    state: EditorState.create({ doc, extensions }),
  });
}

/** The flashed slices of the buffer, in order. */
function flashed(view: EditorView): string[] {
  const field = view.state.field(changeFlashField, false);
  if (!field) return [];
  const out: string[] = [];
  const cursor = field.iter();
  while (cursor.value) {
    out.push(view.state.doc.sliceString(cursor.from, cursor.to));
    cursor.next();
  }
  return out;
}

let views: EditorView[] = [];
const track = (v: EditorView) => (views.push(v), v);

beforeEach(() => vi.useFakeTimers());
afterEach(() => {
  for (const v of views.splice(0)) v.destroy();
  vi.useRealTimers();
});

describe("flashSpans", () => {
  it("marks the given ranges, installing the field on first use", () => {
    const view = track(editor("hello woven world"));
    expect(view.state.field(changeFlashField, false)).toBeUndefined();
    flashSpans(view, [{ from: 6, to: 11 }]);
    expect(flashed(view)).toEqual(["woven"]);
    expect(view.dom.querySelector(".cm-change-flash")?.textContent).toBe("woven");
  });

  it("drops collapsed and empty ranges without dispatching", () => {
    const view = track(editor("abc"));
    flashSpans(view, [{ from: 1, to: 1 }]);
    flashSpans(view, []);
    expect(view.state.field(changeFlashField, false)).toBeUndefined();
  });

  it("maps the marks through edits made while they fade", () => {
    const view = track(editor("one two three"));
    flashSpans(view, [{ from: 4, to: 7 }]); // "two"
    view.dispatch({ changes: { from: 0, insert: "zero " } });
    expect(flashed(view)).toEqual(["two"]);
  });

  it("removes its marks once the fade is over", () => {
    const view = track(editor("fading text"));
    flashSpans(view, [{ from: 0, to: 6 }]);
    expect(flashed(view)).toEqual(["fading"]);
    vi.advanceTimersByTime(FLASH_MS + 1);
    expect(flashed(view)).toEqual([]);
  });

  it("expires each batch alone: an older flash going out never takes a newer one with it", () => {
    const view = track(editor("first second"));
    flashSpans(view, [{ from: 0, to: 5 }]);
    vi.advanceTimersByTime(FLASH_MS / 2);
    flashSpans(view, [{ from: 6, to: 12 }]);
    vi.advanceTimersByTime(FLASH_MS / 2 + 1); // first batch expires
    expect(flashed(view)).toEqual(["second"]);
    vi.advanceTimersByTime(FLASH_MS); // second batch expires
    expect(flashed(view)).toEqual([]);
  });

  it("survives the view being destroyed mid-fade", () => {
    const view = editor("short lived");
    flashSpans(view, [{ from: 0, to: 5 }]);
    view.destroy();
    expect(() => vi.advanceTimersByTime(FLASH_MS + 1)).not.toThrow();
  });
});

describe("syncAndFlash", () => {
  it("updates the buffer to the incoming content and flashes what arrived", () => {
    const view = track(editor("line one\nline two\n"));
    const changed = syncAndFlash(view, "line one\nline 2 now\nline three\n");
    expect(changed).toBe(true);
    expect(view.state.doc.toString()).toBe("line one\nline 2 now\nline three\n");
    // Every flashed slice is text the sync itself introduced.
    expect(flashed(view).join("")).toContain("2 now");
  });

  it("flashes nothing for content identical to the buffer — the own-edit round-trip", () => {
    const view = track(editor("typed right here\n"));
    const changed = syncAndFlash(view, "typed right here\n");
    expect(changed).toBe(false);
    expect(flashed(view)).toEqual([]);
  });

  it("does not read as a user edit, so a save loop keyed on input events stays quiet", () => {
    let userEdits = 0;
    const view = track(
      editor("before\n", [
        EditorView.updateListener.of((u) => {
          // The same guard OutputEditorPane uses before calling onLocalEdit.
          if (
            u.docChanged &&
            u.transactions.some((t) => t.isUserEvent("input") || t.isUserEvent("delete"))
          ) {
            userEdits += 1;
          }
        }),
      ]),
    );
    syncAndFlash(view, "after\n");
    expect(view.state.doc.toString()).toBe("after\n");
    expect(userEdits).toBe(0);
  });

  it("keeps the cursor where the user left it when the change is elsewhere", () => {
    const view = track(editor("alpha\nbeta\ngamma\n"));
    view.dispatch({ selection: { anchor: 3 } }); // inside "alpha"
    syncAndFlash(view, "alpha\nbeta\ngamma\ndelta\n");
    expect(view.state.selection.main.head).toBe(3);
  });
});
