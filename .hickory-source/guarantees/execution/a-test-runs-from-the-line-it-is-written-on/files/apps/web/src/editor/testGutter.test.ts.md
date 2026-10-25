<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/editor/testGutter.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Test Runs From The Line It Is Written On](../../../../../contract.md)

Captured SHA-256: `7d920ad779bcfb3ff8e54857031ef8494a826a9a2d2fbdfd01227ed0249f7661`.

<book:entry path="apps/web/src/editor/testGutter.test.ts">
// Protects docs/guarantees/execution/a-test-runs-from-the-line-it-is-written-on.md
import { describe, expect, it, vi } from "vitest";
vi.mock("../lib/reviews", () => ({ sha256: async () => "0".repeat(64) }));
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { findTests, testGutter } from "./testGutter";

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
</book:entry>

</book:doc>
