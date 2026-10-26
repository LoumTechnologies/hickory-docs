<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Test Runs From The Line It Is Written On

For: Authors running documents on their own machine and reviewers assessing their side effects.

Given a plain or generated file in a language whose tests have a shape the
editor knows — `#[test]` above a `fn`, `it("…")` or `test("…")`, `def test_…`,
`[Fact]`/`[Test]` above a method, `func TestX(t *testing.T)` — when the file
is open, then a run mark sits in the gutter on each test's line; and when the
mark is clicked, then that one test runs in its ecosystem's own runner
(`cargo test <name>`, `npx vitest run <file> -t <name>` or jest, `python3 -m
pytest <file>::<name>`, `dotnet test --filter`, `go test -run '^<name>$'`),
in the nearest directory that owns the file, as a terminal session named
`test: <name>` that opens beside the file and reports finished or failed the
way every session does. A file with no manifest above it, or a runner the
package does not name, is refused in words that say what is missing.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/execution/a-test-runs-from-the-line-it-is-written-on.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-77ab0d2581f9"></a>
## `apps/web/src/editor/testGutter.controls.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Test Runs From The Line It Is Written On](contract.md).

<a id="f-77ab0d2581f9-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-77ab0d2581f9-expression_statement">
// Guarantee: docs/guarantees/execution/a-test-runs-from-the-line-it-is-written-on.md
import { act, cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { UiActions, type UiValue } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useEditorActions } from "./uiEditorActions";
import { testGutter, type TestGutterOptions, type TestMark } from "./testGutter";
import { sha256 } from "../lib/reviews";
vi.mock("../lib/reviews", () => ({ sha256: vi.fn(async () => "a".repeat(64)) }));
</storycode:copy>
<a id="f-77ab0d2581f9-views"></a>
### `views`

<storycode:copy id="f-77ab0d2581f9-views">
const views: EditorView[] = [];
</storycode:copy>
<a id="f-77ab0d2581f9-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-77ab0d2581f9-expression_statement-2">
afterEach(() => { cleanup(); for (const view of views.splice(0)) { const host = view.dom.parentElement; view.destroy(); host?.remove(); } vi.mocked(sha256).mockReset().mockResolvedValue("a".repeat(64)); });
</storycode:copy>
<a id="f-77ab0d2581f9-source"></a>
### `source`

<storycode:copy id="f-77ab0d2581f9-source">
const source = "#[test]\nfn adds() {}\n";
</storycode:copy>
<a id="f-77ab0d2581f9-setup"></a>
### `setup`

<storycode:copy id="f-77ab0d2581f9-setup">
async function setup(onRun: TestGutterOptions["onRun"], text = source, language = "rust", serializedSource?: TestGutterOptions["serializedSource"]) {
  const host = document.body.appendChild(document.createElement("div"));
  const view = new EditorView({ parent: host, state: EditorState.create({ doc: text, extensions: testGutter({ language, onRun, serializedSource }) }) }); views.push(view);
  const registry = new UiActions({ workspace: "tests", window: "one" }, () => true, async () => {});
  function Bridge() { useEditorActions(view, "tests"); return null; }
  render(<UiActionContext.Provider value={registry}><Bridge /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  await waitFor(() => expect(actions().find(action => action.label === "Run test at editor line")?.enabled).toBe(true));
  let id = 0;
  const run = async (label: string, args: Record<string, UiValue> = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "acp:gutter", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return result; };
  const status = async () => { const reply = await run("Read test gutter launch status bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } }; return JSON.parse(new TextDecoder().decode(new Uint8Array(reply.outcome.value.bytes))); };
  return { view, registry, actions, run, status };
}
</storycode:copy>
<a id="f-77ab0d2581f9-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-77ab0d2581f9-expression_statement-3">
it("excludes duplicate launches, keeps status readable while pending and reports admission rather than a passing test", async () => {
  let resolve!: (value: unknown) => void;
  const launch = vi.fn((_mark: TestMark, _hash: string, _current: () => boolean) => new Promise<unknown>(done => { resolve = done; }));
  const { view, registry, actions, run, status } = await setup(launch);
  let pending!: Promise<unknown>;
  await act(async () => { pending = registry.act(registry.scope, { id: "acp:gutter", kind: "agent" }, { handle: actions().find(action => action.label === "Run test at editor line")!.handle, arguments: { line: 2 }, request_id: "launch" }); });
  await waitFor(() => expect(launch).toHaveBeenCalledTimes(1));
  expect(launch.mock.calls[0][0]).toEqual({ line: 2, name: "adds" });
  expect(launch.mock.calls[0][1]).toBe("a".repeat(64));
  expect(await status()).toMatchObject({ pending: true, test_passed: null });
  expect(await run("Run test at editor line", { line: 2 })).toMatchObject({ status: "disabled" });
  const marker = view.dom.querySelector('.cm-test-run[role="button"]')!;
  expect(marker.getAttribute("aria-disabled")).toBe("true");
  await act(async () => marker.dispatchEvent(new MouseEvent("mousedown", { bubbles: true })));
  expect(launch).toHaveBeenCalledTimes(1);
  await act(async () => resolve({ id: "terminal", title: "test: adds", source_hash: "a".repeat(64), source_checked_at_admission: true }));
  expect(await pending).toMatchObject({ outcome: { status: "completed", value: { terminal: { id: "terminal" }, source_checked_at_admission: true, source_frozen: false, test_passed: null } } });
  expect(await status()).toMatchObject({ pending: false, terminal: { id: "terminal" } });
  expect(view.state.doc.toString()).toBe(source);
});
</storycode:copy>
<a id="f-77ab0d2581f9-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-77ab0d2581f9-expression_statement-4">
it("refuses a changed source before dispatch and records cancellation after an admitted launch", async () => {
  let hashDone!: (hash: string) => void;
  vi.mocked(sha256).mockImplementationOnce(() => new Promise(done => { hashDone = done; }));
  const launch = vi.fn();
  const { view, registry, actions } = await setup(launch);
  let pending!: Promise<unknown>;
  await act(async () => { pending = registry.act(registry.scope, { id: "acp:gutter", kind: "agent" }, { handle: actions().find(action => action.label === "Run test at editor line")!.handle, arguments: { line: 2 }, request_id: "changed" }); });
  await waitFor(() => expect(hashDone).toBeTypeOf("function"));
  await act(async () => view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: "#[test]\nfn changed() {}\n" } }));
  await act(async () => hashDone("a".repeat(64)));
  expect(await pending).toMatchObject({ outcome: { status: "unavailable" } });
  expect(launch).not.toHaveBeenCalled();

  let resolve!: (value: unknown) => void;
  const admitted = vi.fn((_mark: TestMark, _hash: string, _current: () => boolean) => new Promise<unknown>(done => { resolve = done; }));
  const second = await setup(admitted);
  await act(async () => { pending = second.registry.act(second.registry.scope, { id: "acp:gutter", kind: "agent" }, { handle: second.actions().find(action => action.label === "Run test at editor line")!.handle, arguments: { line: 2 }, request_id: "cancelled" }); });
  await waitFor(() => expect(admitted).toHaveBeenCalledTimes(1));
  expect(second.registry.cancel(second.registry.scope, { id: "acp:gutter", kind: "agent" }, "cancelled")).toBe(true);
  await act(async () => resolve({ id: "possibly-running", title: "test: adds", source_hash: "a".repeat(64), source_checked_at_admission: true }));
  expect(await pending).not.toMatchObject({ outcome: { status: "completed" } });
  expect(await second.status()).toMatchObject({ cancellation_requested: true, terminal: { id: "possibly-running" } });
});
</storycode:copy>
<a id="f-77ab0d2581f9-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-77ab0d2581f9-expression_statement-5">
it("does not turn an empty named marker into a whole-file run or trust an older unguarded host", async () => {
  const launch = vi.fn();
  const empty = await setup(launch, 'it("", () => {});\n', "typescript");
  expect(await empty.run("Run test at editor line", { line: 1 })).toMatchObject({ outcome: { status: "requires_input" } });
  expect(launch).not.toHaveBeenCalled();
  const oldHost = await setup(async () => ({ id: "legacy", title: "test: adds" }));
  expect(await oldHost.run("Run test at editor line", { line: 2 })).toMatchObject({ outcome: { status: "requires_input" } });
  expect(await oldHost.status()).toMatchObject({ terminal: { id: "legacy" }, source_checked_at_admission: false, test_passed: null });
});

</storycode:copy>
<a id="f-77ab0d2581f9-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-77ab0d2581f9-expression_statement-6">
it("fingerprints matching owned CRLF bytes and refuses malformed Unicode", async () => {
  const raw = source.replaceAll("\n", "\r\n");
  const launch = vi.fn(async () => ({ id: "crlf", title: "test: adds", source_hash: "a".repeat(64), source_checked_at_admission: true }));
  const actual = await setup(launch, raw, "rust", () => raw);
  expect(actual.view.state.doc.toString()).toBe(source);
  await actual.run("Run test at editor line", { line: 2 });
  expect(sha256).toHaveBeenLastCalledWith(raw);
  const malformed = await setup(launch, source + "\ud800");
  expect(await malformed.run("Run test at editor line", { line: 2 })).toMatchObject({ outcome: { status: "requires_input" } });
  expect(launch).toHaveBeenCalledTimes(1);
});
</storycode:copy>

### Assemble `apps/web/src/editor/testGutter.controls.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/testGutter.controls.test.tsx"><storycode:paste select="#f-77ab0d2581f9-expression_statement"/><storycode:paste select="#f-77ab0d2581f9-views"/><storycode:paste select="#f-77ab0d2581f9-expression_statement-2"/><storycode:paste select="#f-77ab0d2581f9-source"/><storycode:paste select="#f-77ab0d2581f9-setup"/><storycode:paste select="#f-77ab0d2581f9-expression_statement-3"/><storycode:paste select="#f-77ab0d2581f9-expression_statement-4"/><storycode:paste select="#f-77ab0d2581f9-expression_statement-5"/><storycode:paste select="#f-77ab0d2581f9-expression_statement-6"/></storycode:file>

<a id="output-87d28cb20f24"></a>
## `apps/web/src/editor/testGutter.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Test Runs From The Line It Is Written On](contract.md).

<a id="f-87d28cb20f24-expression_statement"></a>
### `expression_statement`

Protects docs/guarantees/execution/a-test-runs-from-the-line-it-is-written-on.md

<storycode:copy id="f-87d28cb20f24-expression_statement">
// Protects docs/guarantees/execution/a-test-runs-from-the-line-it-is-written-on.md
import { describe, expect, it, vi } from "vitest";
vi.mock("../lib/reviews", () => ({ sha256: async () => "0".repeat(64) }));
</storycode:copy>
<a id="f-87d28cb20f24-import_statement"></a>
### `import_statement`

<storycode:copy id="f-87d28cb20f24-import_statement">
import { EditorState } from "@codemirror/state";
</storycode:copy>
<a id="f-87d28cb20f24-import_statement-2"></a>
### `import_statement`

<storycode:copy id="f-87d28cb20f24-import_statement-2">
import { EditorView } from "@codemirror/view";
</storycode:copy>
<a id="f-87d28cb20f24-import_statement-3"></a>
### `import_statement`

<storycode:copy id="f-87d28cb20f24-import_statement-3">
import { findTests, testGutter } from "./testGutter";

</storycode:copy>
<a id="f-87d28cb20f24-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-87d28cb20f24-expression_statement-2">
describe("finding tests by their shape", () => {
  it("names a Rust test by the fn under its attribute, attributes stacked or not", () => {
    const text = [
      "#[test]",
      "fn adds() {}",
      "",
      "#[tokio::test(flavor = \"multi_thread\")]",
      "#[ignore]",
      "async fn waits() {}",
      "fn not_a_test() {}",
    ].join("\n");
    expect(findTests("rust", text)).toEqual([
      { line: 2, name: "adds" },
      { line: 6, name: "waits" },
    ]);
  });

  it("names a vitest or jest case by its string, whichever quote it used", () => {
    const text = 'describe("x", () => {\n  it("renders the thing", () => {});\n  test.skip(`later`, () => {});\n});\n';
    expect(findTests("typescript", text)).toEqual([
      { line: 2, name: "renders the thing" },
      { line: 3, name: "later" },
    ]);
  });

  it("names python, go and C# tests too", () => {
    expect(findTests("python", "def helper():\n    pass\n\ndef test_sum():\n    pass\n")).toEqual([
      { line: 4, name: "test_sum" },
    ]);
    expect(findTests("go", "func TestSum(t *testing.T) {}\nfunc helper() {}\n")).toEqual([
      { line: 1, name: "TestSum" },
    ]);
    expect(findTests("csharp", "[Fact]\npublic void Adds()\n{\n}\n")).toEqual([
      { line: 2, name: "Adds" },
    ]);
  });

  it("finds nothing in a language it has no shape for, which is not an error", () => {
    expect(findTests("markdown", "# it(\"looks like one\")")).toEqual([]);
  });
});

</storycode:copy>
<a id="f-87d28cb20f24-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-87d28cb20f24-expression_statement-3">
describe("the gutter", () => {
  it("draws a run mark on each test's line and runs it on click", async () => {
    const runs: string[] = [];
    const view = new EditorView({
      parent: document.body,
      state: EditorState.create({
        doc: "#[test]\nfn adds() {}\n",
        extensions: testGutter({ language: "rust", onRun: (mark) => runs.push(mark.name) }),
      }),
    });
    // The scan is deferred past construction.
    await new Promise((resolve) => setTimeout(resolve, 20));
    const marks = view.dom.querySelectorAll(".cm-test-run:not(.cm-test-run--spacer)");
    expect(marks).toHaveLength(1);
    expect((marks[0] as HTMLElement).dataset.tip).toBe("Run adds");
    marks[0].dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
    await vi.waitFor(() => expect(runs).toEqual(["adds"]));
    view.destroy();
  });
});
</storycode:copy>

### Assemble `apps/web/src/editor/testGutter.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/testGutter.test.ts"><storycode:paste select="#f-87d28cb20f24-expression_statement"/><storycode:paste select="#f-87d28cb20f24-import_statement"/><storycode:paste select="#f-87d28cb20f24-import_statement-2"/><storycode:paste select="#f-87d28cb20f24-import_statement-3"/><storycode:paste select="#f-87d28cb20f24-expression_statement-2"/><storycode:paste select="#f-87d28cb20f24-expression_statement-3"/></storycode:file>

<a id="output-09b8956c0f0c"></a>
## `apps/web/src/editor/testGutter.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Test Runs From The Line It Is Written On](contract.md).

<a id="f-09b8956c0f0c-TestMark"></a>
### `TestMark`

<storycode:copy id="f-09b8956c0f0c-TestMark">
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

</storycode:copy>
<a id="f-09b8956c0f0c-export_statement"></a>
### `export_statement`

<storycode:copy id="f-09b8956c0f0c-export_statement">
export { findTests } from "../plugins/languages/tests";
</storycode:copy>
<a id="f-09b8956c0f0c-import_statement"></a>
### `import_statement`

<storycode:copy id="f-09b8956c0f0c-import_statement">
import { findTests } from "../plugins/languages/tests";

</storycode:copy>
<a id="f-09b8956c0f0c-setTestMarks"></a>
### `setTestMarks`

<storycode:copy id="f-09b8956c0f0c-setTestMarks">
const setTestMarks = StateEffect.define<TestScan>();

</storycode:copy>
<a id="f-09b8956c0f0c-testMarksField"></a>
### `testMarksField`

<storycode:copy id="f-09b8956c0f0c-testMarksField">
const testMarksField = StateField.define<TestScan>({
  create: () => ({ marks: [], source: null }),
  update(marks, tr) {
    for (const effect of tr.effects) if (effect.is(setTestMarks)) marks = effect.value;
    return marks;
  },
});

</storycode:copy>
<a id="f-09b8956c0f0c-RunMarker"></a>
### `RunMarker`

<storycode:copy id="f-09b8956c0f0c-RunMarker">
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

</storycode:copy>
<a id="f-09b8956c0f0c-SPACER"></a>
### `SPACER`

<storycode:copy id="f-09b8956c0f0c-SPACER">
const SPACER = new (class extends GutterMarker {
  toDOM() {
    const el = document.createElement("span");
    el.className = "cm-test-run cm-test-run--spacer";
    el.textContent = "▶";
    return el;
  }
})();

</storycode:copy>
<a id="f-09b8956c0f0c-markerSet"></a>
### `markerSet`

Marks at their lines' starts, clamped to the document.

<storycode:copy id="f-09b8956c0f0c-markerSet">
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

</storycode:copy>
<a id="f-09b8956c0f0c-TestGutterOptions"></a>
### `TestGutterOptions`

<storycode:copy id="f-09b8956c0f0c-TestGutterOptions">
export interface TestGutterOptions {
  /** Owned raw file serialization; used only when it matches the captured editor text. */
  serializedSource?: () => string | undefined;
  /** The file's language id, which decides what a test looks like. */
  language: string;
  /** Run this one. */
  onRun: (mark: TestMark, expectedSourceHash: string, current: () => boolean) => unknown | Promise<unknown>;
  onError?: (reason: string) => void;
}

</storycode:copy>
<a id="f-09b8956c0f0c-RESCAN_MS"></a>
### `RESCAN_MS`

<storycode:copy id="f-09b8956c0f0c-RESCAN_MS">
const RESCAN_MS = 300;

</storycode:copy>
<a id="f-09b8956c0f0c-HAS_TESTS"></a>
### `HAS_TESTS`

<storycode:copy id="f-09b8956c0f0c-HAS_TESTS">
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

</storycode:copy>
<a id="f-09b8956c0f0c-testGutter"></a>
### `testGutter`

The gutter, and the scan that fills it after the text settles.

<storycode:copy id="f-09b8956c0f0c-testGutter">
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
</storycode:copy>

### Assemble `apps/web/src/editor/testGutter.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/testGutter.ts"><storycode:paste select="#f-09b8956c0f0c-TestMark"/><storycode:paste select="#f-09b8956c0f0c-export_statement"/><storycode:paste select="#f-09b8956c0f0c-import_statement"/><storycode:paste select="#f-09b8956c0f0c-setTestMarks"/><storycode:paste select="#f-09b8956c0f0c-testMarksField"/><storycode:paste select="#f-09b8956c0f0c-RunMarker"/><storycode:paste select="#f-09b8956c0f0c-SPACER"/><storycode:paste select="#f-09b8956c0f0c-markerSet"/><storycode:paste select="#f-09b8956c0f0c-TestGutterOptions"/><storycode:paste select="#f-09b8956c0f0c-RESCAN_MS"/><storycode:paste select="#f-09b8956c0f0c-HAS_TESTS"/><storycode:paste select="#f-09b8956c0f0c-testGutter"/></storycode:file>

<a id="output-0b3e4dd81785"></a>
## `crates/hickory-cli/tests/test_run.rs`

Related contracts: [A Test Runs From The Line It Is Written On](contract.md).

<a id="f-0b3e4dd81785-a_rust_test_runs_in_a_terminal_named_after_it"></a>
### `a_rust_test_runs_in_a_terminal_named_after_it`

One test, run from the gutter, through the endpoint, to a terminal that
finishes.
Protects docs/guarantees/execution/a-test-runs-from-the-line-it-is-written-on.md

<storycode:copy id="f-0b3e4dd81785-a_rust_test_runs_in_a_terminal_named_after_it">
//! One test, run from the gutter, through the endpoint, to a terminal that
//! finishes.
//!
//! Protects docs/guarantees/execution/a-test-runs-from-the-line-it-is-written-on.md

use std::time::{Duration, Instant};

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

#[tokio::test]
async fn a_rust_test_runs_in_a_terminal_named_after_it() {
    if std::process::Command::new("cargo")
        .arg("--version")
        .output()
        .map(|o| !o.status.success())
        .unwrap_or(true)
    {
        eprintln!("SKIPPED: no cargo on this machine");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(
        root.join("notes.md"),
        "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"notes.md\">\n# Notes\n</hick:doc>\n",
    )
    .unwrap();
    std::fs::create_dir_all(root.join("lib/src")).unwrap();
    std::fs::write(
        root.join("lib/Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(
        root.join("lib/src/lib.rs"),
        "pub fn add(a: u32, b: u32) -> u32 { a + b }\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn adds() { assert_eq!(super::add(1, 2), 3); }\n}\n".replace('\n', "\r\n"),
    )
    .unwrap();

    let prepared = prepare(ServeOptions {
        target: root.join("notes.md"),
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .expect("session prepares");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    let base = format!("http://127.0.0.1:{port}");
    let client = reqwest::Client::new();

    // A path outside the folder, and a language with no runner, are refused.
    let refused = client
        .post(format!("{base}/api/tests/run"))
        .json(&json!({ "path": "../x.rs", "name": "adds", "language": "rust" }))
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status().as_u16(), 400);
    let refused = client
        .post(format!("{base}/api/tests/run"))
        .json(&json!({ "path": "lib/src/lib.rs", "name": "adds", "language": "haskell" }))
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status().as_u16(), 422);

    let source_hash = format!(
        "{:x}",
        Sha256::digest(std::fs::read(root.join("lib/src/lib.rs")).unwrap())
    );
    for (expected, status) in [("0".repeat(64), 409), ("invalid".into(), 400)] {
        let refused = client.post(format!("{base}/api/tests/run")).json(&json!({"path":"lib/src/lib.rs","name":"adds","language":"rust","expected_source_hash":expected})).send().await.unwrap();
        assert_eq!(refused.status().as_u16(), status);
    }
    #[cfg(unix)]
    {
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(
            outside.path().join("escape.rs"),
            "#[test] fn escaped() {}\n",
        )
        .unwrap();
        std::os::unix::fs::symlink(
            outside.path().join("escape.rs"),
            root.join("lib/src/escape.rs"),
        )
        .unwrap();
        let refused = client
            .post(format!("{base}/api/tests/run"))
            .json(&json!({"path":"lib/src/escape.rs","name":"escaped","language":"rust"}))
            .send()
            .await
            .unwrap();
        assert_eq!(refused.status().as_u16(), 422);
    }
    let empty: Value = client
        .get(format!("{base}/api/terminals"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        empty["sessions"].as_array().unwrap().is_empty(),
        "Refused admission must not start a terminal"
    );
    let response = client
        .post(format!("{base}/api/tests/run"))
        .json(&json!({ "path": "lib/src/lib.rs", "name": "adds", "language": "rust", "expected_source_hash":source_hash }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);
    let session: Value = response.json().await.unwrap();
    assert_eq!(session["title"], "test: adds");
    assert_eq!(session["source_hash"], source_hash);
    assert_eq!(session["source_checked_at_admission"], true);
    assert_eq!(session["source_frozen"], false);
    assert!(
        session["cwd"].as_str().unwrap().ends_with("lib"),
        "ran in the wrong directory: {session}"
    );
    let id = session["id"].as_str().unwrap().to_string();

    // The session finishes: cargo compiled the crate and ran the test. Its
    // state comes from the terminal registry, the same one the dock reads.
    let deadline = Instant::now() + Duration::from_secs(180);
    loop {
        let listing: Value = client
            .get(format!("{base}/api/terminals"))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let found = listing["sessions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["id"] == id)
            .cloned()
            .expect("the session is listed");
        let state = found["state"].as_str().unwrap_or("");
        if state == "finished" {
            // Finished because cargo ran the test, not because something
            // exited early: the runner's own last line says so.
            let preview = found["preview"].as_str().unwrap_or("");
            assert!(
                preview.contains("test result") || preview.contains("1 passed"),
                "the session finished without cargo's verdict: {found}"
            );
            break;
        }
        assert_ne!(state, "failed", "the test run failed: {found}");
        assert!(
            Instant::now() < deadline,
            "the test run did not finish: {found}"
        );
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/test_run.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/test_run.rs"><storycode:paste select="#f-0b3e4dd81785-a_rust_test_runs_in_a_terminal_named_after_it"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/OutputEditorPane.tsx](../../../mechanisms/apps/web/src/components/OutputEditorPane.tsx.md#output-2571b0682164)
- [apps/web/src/components/PlainFilePane.tsx](../../../mechanisms/apps/web/src/components/PlainFilePane.tsx.md#output-2f891a49445f)
- [apps/web/src/editor/testGutter.ts](contract.md#output-09b8956c0f0c)
- [apps/web/src/lib/revealLine.ts](../../../mechanisms/apps/web/src/lib/revealLine.ts.md#output-bca672428db0)
- [apps/web/src/views/WorkspaceView.tsx](../../../mechanisms/apps/web/src/views/WorkspaceView.tsx.md#output-47afe3575948)

### Tests and fixtures

- [apps/web/src/editor/testGutter.controls.test.tsx](contract.md#output-77ab0d2581f9)
- [apps/web/src/editor/testGutter.test.ts](contract.md#output-87d28cb20f24)
- [crates/hickory-cli/src/serve/test_run.rs](../../../mechanisms/crates/hickory-cli/src/serve/test_run.rs.md#output-c99ee9641f1f)
- [crates/hickory-cli/tests/test_run.rs](contract.md#output-0b3e4dd81785)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
