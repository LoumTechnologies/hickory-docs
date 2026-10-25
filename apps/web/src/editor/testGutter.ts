import { testGutterActions, type TestScan } from "./testGutterActions";
// A run mark beside every test, in the gutter.
//
// Finding a test is a line-shaped question with a per-language answer —
// `#[test]` above a `fn`, `it("…")`, `def test_…`, `[Fact]` above a method,
// `func TestX(t *testing.T)` — and the editor answers it by looking, the way
// the breakpoint gutter decides which lines can hold one. A click runs THAT
// test, in that ecosystem's own runner, in a terminal session named after it
// (see crates/hickory-cli/src/serve/test_run.rs); the terminal opens beside
// the file, and its output is the runner's own.
//
// Detection is textual and deliberately so: a language server knows what is
// in scope but has no opinion about what is a test, and a test runner's own
// discovery would mean running it to draw a gutter.

import { RangeSetBuilder, StateEffect, StateField } from "@codemirror/state";
import type { EditorState, Extension } from "@codemirror/state";
import { EditorView, GutterMarker, ViewPlugin, gutter } from "@codemirror/view";
import type { ViewUpdate } from "@codemirror/view";

/** One test, on the line its name is written on (1-based). */
export interface TestMark {
  line: number;
  name: string;
}

export { findTests } from "../plugins/languages/tests";
import { findTests } from "../plugins/languages/tests";

const setTestMarks = StateEffect.define<TestScan>();

const testMarksField = StateField.define<TestScan>({
  create: () => ({ marks: [], source: null }),
  update(marks, tr) {
    for (const effect of tr.effects) if (effect.is(setTestMarks)) marks = effect.value;
    return marks;
  },
});

class RunMarker extends GutterMarker {
  constructor(
    private readonly mark: TestMark,
    private readonly onRun: (view: EditorView, mark: TestMark) => void,
    private readonly disabled: boolean,
  ) {
    super();
  }
  eq(other: RunMarker) {
    return other.mark.name === this.mark.name && other.mark.line === this.mark.line && other.onRun === this.onRun && other.disabled === this.disabled;
  }
  toDOM(view: EditorView) {
    const el = document.createElement("span");
    el.className = "cm-test-run";
    el.textContent = "▶";
    el.dataset.tip = `Run ${this.mark.name}`;
    el.setAttribute("role", "button");
    el.setAttribute("aria-label", `Run ${this.mark.name}`);
    el.setAttribute("aria-disabled", String(this.disabled));
    // The mark answers its own click, rather than the gutter resolving a
    // line from the pointer's height: the mark knows which test it is.
    el.addEventListener("mousedown", (event) => {
      event.preventDefault();
      if (!this.disabled) this.onRun(view, this.mark);
    });
    return el;
  }
}

const SPACER = new (class extends GutterMarker {
  toDOM() {
    const el = document.createElement("span");
    el.className = "cm-test-run cm-test-run--spacer";
    el.textContent = "▶";
    return el;
  }
})();

/** Marks at their lines' starts, clamped to the document. */
function markerSet(state: EditorState, onRun: (view: EditorView, mark: TestMark) => void, disabled: boolean) {
  const builder = new RangeSetBuilder<GutterMarker>();
  const marks = [...state.field(testMarksField).marks].sort((a, b) => a.line - b.line);
  for (const mark of marks) {
    if (mark.line < 1 || mark.line > state.doc.lines) continue;
    const at = state.doc.line(mark.line).from;
    builder.add(at, at, new RunMarker(mark, onRun, disabled));
  }
  return builder.finish();
}

export interface TestGutterOptions {
  /** Owned raw file serialization; used only when it matches the captured editor text. */
  serializedSource?: () => string | undefined;
  /** The file's language id, which decides what a test looks like. */
  language: string;
  /** Run this one. */
  onRun: (mark: TestMark, expectedSourceHash: string, current: () => boolean) => unknown | Promise<unknown>;
  onError?: (reason: string) => void;
}

const RESCAN_MS = 300;

/** The languages `findTests` has a shape for. */
const HAS_TESTS = new Set([
  "rust",
  "csharp",
  "python",
  "go",
  "typescript",
  "javascript",
  "tsx",
  "jsx",
  "typescriptreact",
  "javascriptreact",
]);

/** The gutter, and the scan that fills it after the text settles. */
export function testGutter(options: TestGutterOptions): Extension[] {
  // A language with no test shape gets no gutter at all, rather than an
  // empty column taking width from every file that is not a test.
  if (!HAS_TESTS.has(options.language)) return [];
  const scan = (view: EditorView) => {
    const marks = findTests(options.language, view.state.doc.toString());
    view.dispatch({ effects: setTestMarks.of({ marks, source: view.state.doc }) });
  };
  const controls = testGutterActions(options, testMarksField);
  const nativeRun = (view: EditorView, mark: TestMark) => { void controls.run(view, mark, new AbortController().signal); };
  return [
    testMarksField, controls.extensions,
    ViewPlugin.fromClass(
      class {
        private timer: ReturnType<typeof setTimeout> | null = null;
        constructor(view: EditorView) {
          // After construction, not during: dispatching inside a plugin's
          // constructor is an update inside an update.
          this.timer = setTimeout(() => scan(view), 0);
        }
        update(update: ViewUpdate) {
          if (!update.docChanged) return;
          if (this.timer) clearTimeout(this.timer);
          this.timer = setTimeout(() => scan(update.view), RESCAN_MS);
        }
        destroy() {
          if (this.timer) clearTimeout(this.timer);
        }
      },
    ),
    gutter({
      class: "cm-test-gutter",
      markers: (view) => markerSet(view.state, nativeRun, controls.pending() || view.state.field(testMarksField).source !== view.state.doc),
      initialSpacer: () => SPACER,
    }),
    EditorView.theme({
      ".cm-test-gutter": { width: "1.1em" },
      ".cm-test-run": {
        display: "inline-block",
        width: "1.1em",
        textAlign: "center",
        fontSize: "0.7em",
        color: "var(--ok)",
        cursor: "pointer",
        opacity: "0.85",
      },
      ".cm-test-run:hover": { opacity: "1" },
      ".cm-test-run--spacer": { visibility: "hidden" },
    }),
  ];
}
