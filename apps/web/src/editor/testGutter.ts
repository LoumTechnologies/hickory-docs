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

const RUST_ATTRIBUTE = /^\s*#\[\s*(?:tokio::|async_std::)?test\b/;
const RUST_FN = /^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)/;
const JS_CASE = /^\s*(?:it|test)(?:\.(?:only|skip|each))?\s*\(\s*(["'`])((?:\\.|(?!\1).)*)\1/;
const PY_DEF = /^\s*(?:async\s+)?def\s+(test_[A-Za-z0-9_]*)\s*\(/;
const CS_ATTRIBUTE = /^\s*\[\s*(?:Fact|Theory|Test|TestMethod|TestCase)\b/;
const CS_METHOD = /^\s*(?:public\s+|private\s+|internal\s+|static\s+|async\s+|virtual\s+|override\s+)*[A-Za-z_<>\[\],. ]+?\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/;
const GO_FUNC = /^\s*func\s+(Test[A-Za-z0-9_]*)\s*\(\s*\w+\s+\*testing\.T\s*\)/;

/** Every test in `text`, for `language`. Empty for a language with no shape
 * this editor recognises — which is not an error, it is a gutter with
 * nothing in it. */
export function findTests(language: string, text: string): TestMark[] {
  const lines = text.split("\n");
  const marks: TestMark[] = [];
  const attributed = (attribute: RegExp, definition: RegExp) => {
    for (let i = 0; i < lines.length; i++) {
      if (!attribute.test(lines[i])) continue;
      // The name is on the next line that defines something; attributes
      // stack, so skip other attributes and blank lines on the way.
      for (let j = i + 1; j < lines.length && j <= i + 6; j++) {
        const line = lines[j];
        if (line.trim() === "" || /^\s*(#\[|\[)/.test(line)) continue;
        const found = definition.exec(line);
        if (found) marks.push({ line: j + 1, name: found[1] });
        break;
      }
    }
  };
  switch (language) {
    case "rust":
      attributed(RUST_ATTRIBUTE, RUST_FN);
      break;
    case "csharp":
      attributed(CS_ATTRIBUTE, CS_METHOD);
      break;
    case "python":
      lines.forEach((line, i) => {
        const found = PY_DEF.exec(line);
        if (found) marks.push({ line: i + 1, name: found[1] });
      });
      break;
    case "go":
      lines.forEach((line, i) => {
        const found = GO_FUNC.exec(line);
        if (found) marks.push({ line: i + 1, name: found[1] });
      });
      break;
    case "typescript":
    case "javascript":
    case "tsx":
    case "jsx":
    case "typescriptreact":
    case "javascriptreact":
      lines.forEach((line, i) => {
        const found = JS_CASE.exec(line);
        if (found) marks.push({ line: i + 1, name: found[2] });
      });
      break;
    default:
      break;
  }
  return marks;
}

const setTestMarks = StateEffect.define<TestMark[]>();

const testMarksField = StateField.define<TestMark[]>({
  create: () => [],
  update(marks, tr) {
    for (const effect of tr.effects) if (effect.is(setTestMarks)) marks = effect.value;
    return marks;
  },
});

class RunMarker extends GutterMarker {
  constructor(
    private readonly mark: TestMark,
    private readonly onRun: (mark: TestMark) => void,
  ) {
    super();
  }
  eq(other: RunMarker) {
    return other.mark.name === this.mark.name && other.mark.line === this.mark.line;
  }
  toDOM() {
    const el = document.createElement("span");
    el.className = "cm-test-run";
    el.textContent = "▶";
    el.dataset.tip = `Run ${this.mark.name}`;
    el.setAttribute("role", "button");
    // The mark answers its own click, rather than the gutter resolving a
    // line from the pointer's height: the mark knows which test it is.
    el.addEventListener("mousedown", (event) => {
      event.preventDefault();
      this.onRun(this.mark);
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
function markerSet(state: EditorState, onRun: (mark: TestMark) => void) {
  const builder = new RangeSetBuilder<GutterMarker>();
  const marks = [...state.field(testMarksField)].sort((a, b) => a.line - b.line);
  for (const mark of marks) {
    if (mark.line < 1 || mark.line > state.doc.lines) continue;
    const at = state.doc.line(mark.line).from;
    builder.add(at, at, new RunMarker(mark, onRun));
  }
  return builder.finish();
}

export interface TestGutterOptions {
  /** The file's language id, which decides what a test looks like. */
  language: string;
  /** Run this one. */
  onRun: (mark: TestMark) => void;
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
    view.dispatch({ effects: setTestMarks.of(marks) });
  };
  return [
    testMarksField,
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
      markers: (view) => markerSet(view.state, options.onRun),
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
