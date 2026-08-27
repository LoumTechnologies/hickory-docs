// @vitest-environment jsdom
//
// Ctrl+←/→ under both definitions of "word". The commands themselves are
// CodeMirror's; what is ours — and what these assert — is that the SETTING
// picks between them, that it is consulted at keypress rather than captured
// at mount, and that the chords are the two `standardKeymap` already owns, so
// the override actually shadows the default instead of sitting beside it.

import { afterEach, describe, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { standardKeymap } from "@codemirror/commands";

import { wordMotionBindings } from "./wordMotion";
import { saveWordMotion, WORD_MOTION_KEY } from "../lib/wordMotion";

const [LEFT, RIGHT] = wordMotionBindings;

let views: EditorView[] = [];
afterEach(() => {
  for (const v of views.splice(0)) v.destroy();
  localStorage.removeItem(WORD_MOTION_KEY);
});

/** A view with the caret at `at`, tracked for teardown. */
function editor(doc: string, at: number) {
  const view = new EditorView({
    parent: document.body,
    state: EditorState.create({ doc, selection: { anchor: at } }),
  });
  views.push(view);
  return view;
}

/** Where the caret ends up after one press. */
const press = (view: EditorView, binding: typeof LEFT, shift = false): number => {
  (shift ? binding.shift! : binding.run!)(view);
  return view.state.selection.main.head;
};

describe("what Ctrl+arrow counts as a word", () => {
  it("crosses a whole identifier under the default", () => {
    saveWordMotion("word");
    const view = editor("XMLHttpRequest done", 0);
    expect(press(view, RIGHT)).toBe("XMLHttpRequest".length);
  });

  it("stops on the case humps under subwords", () => {
    saveWordMotion("subword");
    const view = editor("XMLHttpRequest done", 0);
    // XML | Http | Request — the run of capitals belongs to the word that
    // follows it, which is what makes this readable on an acronym.
    expect(press(view, RIGHT)).toBe("XML".length);
    expect(press(view, RIGHT)).toBe("XMLHttp".length);
    expect(press(view, RIGHT)).toBe("XMLHttpRequest".length);
  });

  it("stops on PascalCase and snake_case going both ways", () => {
    saveWordMotion("subword");
    const forward = editor("ParseResult parse_result", 0);
    expect(forward.state.doc.sliceString(0, press(forward, RIGHT))).toBe("Parse");

    const back = editor("ParseResult", "ParseResult".length);
    expect(press(back, LEFT)).toBe("Parse".length);

    const snake = editor("parse_result", 0);
    expect(press(snake, RIGHT)).toBe("parse".length);
  });

  it("extends the selection by the same unit when Shift is held", () => {
    saveWordMotion("subword");
    const view = editor("ParseResult", 0);
    press(view, RIGHT, true);
    const { from, to } = view.state.selection.main;
    expect(view.state.doc.sliceString(from, to)).toBe("Parse");
  });

  // The whole reason the choice is read inside the command rather than baked
  // into the extension list: a person changes this BECAUSE the arrow key just
  // did the wrong thing, and presses it again to check — in the editor that
  // is already open.
  it("follows a change of setting without rebuilding the editor", () => {
    saveWordMotion("word");
    const view = editor("ParseResult", 0);
    expect(press(view, RIGHT)).toBe("ParseResult".length);

    view.dispatch({ selection: { anchor: 0 } });
    saveWordMotion("subword");
    expect(press(view, RIGHT)).toBe("Parse".length);
  });

  it("binds exactly the chords the default keymap uses for this", () => {
    // If these ever drift apart the override silently stops overriding —
    // both bindings would be live on different keys.
    const defaults = standardKeymap.filter((b) => b.key?.endsWith("-ArrowLeft") || b.key?.endsWith("-ArrowRight"));
    for (const ours of wordMotionBindings) {
      const match = defaults.find((b) => b.key === ours.key);
      expect(match, `standardKeymap still binds ${ours.key}`).toBeTruthy();
      expect(match!.mac).toBe(ours.mac);
    }
  });
});
