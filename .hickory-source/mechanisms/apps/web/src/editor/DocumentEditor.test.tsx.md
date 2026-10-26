<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/DocumentEditor.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-71bbcfe029bb"></a>
## `apps/web/src/editor/DocumentEditor.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [A Diagram Names What Proves It](../../../../../guarantees/authoring/a-diagram-names-what-proves-it/contract.md), [A Fence Becomes A Cell That Runs](../../../../../guarantees/authoring/a-fence-becomes-a-cell-that-runs/contract.md), [A Literate File Opens Rendered, And Its Source Is One Click Away](../../../../../guarantees/authoring/a-literate-file-opens-rendered/contract.md), [An Ingested Scaffold Opens As A Tree](../../../../../guarantees/authoring/an-ingested-scaffold-opens-as-a-tree/contract.md), [Markdown Paragraphs Unwrap By Default](../../../../../guarantees/authoring/paragraphs-unwrap-by-default/contract.md), [The editor keeps its space](../../../../../guarantees/authoring/the-editor-keeps-its-space/contract.md), [The Gutters Never Skip A Number](../../../../../guarantees/authoring/the-gutters-never-skip-a-number/contract.md), [Undo Only Undoes What This Client Typed](../../../../../guarantees/editor-intelligence/undo-never-destroys-unowned-content/contract.md), [A run result is distinct from what was checked](../../../../../guarantees/execution/a-run-result-is-distinct-from-what-was-checked/contract.md), [Document-View Styling Never Mutates the Source](../../../../../guarantees/language/document-view-decorations-never-mutate-source/contract.md).

<a id="f-71bbcfe029bb-mermaidRender"></a>
### `mermaidRender`

<storycode:copy id="f-71bbcfe029bb-mermaidRender">
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
// @vitest-environment jsdom
import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { EditorView as EditorViewType } from "@codemirror/view";
import { DocumentEditor, assertionStates, matchExecBlock } from "./DocumentEditor";
import { LocalRealtime } from "../api/realtime";
import { CLI_BLOCKS } from "../mock/mockData";
import { parseHickDoc } from "./hickDoc";
import { ICON_SIZE } from "../lib/cardRail";
import type { ExecBlock } from "../api/types";
import { undo } from "@codemirror/commands";
import { saveUnwrapParagraphs, UNWRAP_PARAGRAPHS_KEY } from "../lib/unwrapParagraphs";

// The real engine wants a live browser; these tests are about what the editor
// HANDS the panel, not what the panel draws.
const mermaidRender = vi.fn();
</storycode:copy>
<a id="f-71bbcfe029bb-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-71bbcfe029bb-expression_statement">
vi.mock("mermaid", () => ({
  default: {
    initialize: vi.fn(),
    render: (...args: unknown[]) => mermaidRender(...args),
  },
}));

</storycode:copy>
<a id="f-71bbcfe029bb-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-71bbcfe029bb-expression_statement-2">
afterEach(cleanup);

</storycode:copy>
<a id="f-71bbcfe029bb-execBlocks"></a>
### `execBlocks`

<storycode:copy id="f-71bbcfe029bb-execBlocks">
const execBlocks = CLI_BLOCKS.filter((b): b is ExecBlock => b.kind === "exec");

</storycode:copy>
<a id="f-71bbcfe029bb-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-71bbcfe029bb-expression_statement-3">
describe("DocumentEditor (WYSIWYG over raw source)", () => {
  // Guarantee: docs/guarantees/authoring/the-editor-keeps-its-space.md
  it("offers Save on an unnamed cell's Run rail without changing source bytes", async () => {
    const source = '<hick:container name="shell" image="alpine:3.20" />\n<hick:exec container="shell">echo hello</hick:exec>\n';
    const onRun = vi.fn(), save = vi.fn();
    const listener = (event: Event) => save((event as CustomEvent).detail);
    window.addEventListener("hickory-menu", listener);
    const realtime = new LocalRealtime();
    try {
      const rendered = render(<DocumentEditor docId="untitled" initialSource={source} realtime={realtime} execBlocks={[]} runningCells={new Set()} onRunCell={onRun} />);
      const button = await screen.findByRole("button", { name: /save this document to enable running/ });
      act(() => button.click());
      expect(save).toHaveBeenCalledWith("save");
      expect(onRun).not.toHaveBeenCalled();
      const view = (window as unknown as { __hickoryView: EditorViewType }).__hickoryView;
      expect(view.state.doc.toString()).toBe(source);
      rendered.unmount();
    } finally { realtime.close(); window.removeEventListener("hickory-menu", listener); }
  });
  it("shows new shell cell output below editable source and updates it as the run grows", async () => {
    const realtime = new LocalRealtime();
    const initialSource = "# Notes\n";
    const source = '<hick:exec container="main">\ndotnet new console blah\n</hick:exec>\n';
    const props = { docId: "inline-stdout", initialSource, realtime, execBlocks: [] as ExecBlock[], runningCells: new Set<string>(), onRunCell: vi.fn() };
    const { container, rerender } = render(<DocumentEditor {...props} />);
    await act(async () => { await Promise.resolve(); });
    const view = (window as unknown as { __hickoryView: EditorViewType }).__hickoryView;
    await act(async () => { view.dispatch({ changes: { from: view.state.doc.length, insert: source } }); });
    expect(container.querySelector("[data-testid='watch-terminal']")).toBeNull();
    const block: ExecBlock = { kind: "exec", id: "console", container: "main", command: "dotnet new console blah", span: [initialSource.length, initialSource.length + source.trimEnd().length], status: "ok", transcript: [{ t: 0, kind: "out", data: "Created console app\n" }] };
    rerender(<DocumentEditor {...props} execBlocks={[block]} runningCells={new Set([block.id])} />);
    await waitFor(() => expect(container.querySelector(".xterm-rows")?.textContent).toContain("Created console app"));
    const output = container.querySelector(".cm-exec-output-frame")!;
    expect(output.previousElementSibling?.textContent).toContain("</hick:exec>");
    expect(container.querySelector("[data-testid='cell-command']")).toBeNull();
    rerender(<DocumentEditor {...props} execBlocks={[{ ...block, transcript: [...(block.transcript ?? []), { t: 1, kind: "out", data: "Restore succeeded\n" }] }]} />);
    await waitFor(() => expect(output.textContent).toContain("Restore succeeded"));
    expect(view.state.doc.toString()).toBe(initialSource + source);
    realtime.close();
  });

  // Guarantee: docs/guarantees/authoring/paragraphs-unwrap-by-default.md
  it("reflows synced prose through the live room and reports the source edit, with Undo and opt-out", async () => {
    const source = "A wrapped paragraph\ncontinues here.\n";
    const onChange = vi.fn();
    const realtime = new LocalRealtime();
    let view: EditorViewType | null = null;
    localStorage.removeItem(UNWRAP_PARAGRAPHS_KEY);
    const props = {
      docId: "unwrap", initialSource: source, realtime, execBlocks: [],
      runningCells: new Set<string>(), onRunCell: () => undefined, onChange,
      onViewReady: (ready: EditorViewType | null) => { view = ready; },
    };
    const mounted = render(<DocumentEditor {...props} />);
    try {
      await waitFor(() => expect(view?.state.doc.toString()).toBe("A wrapped paragraph continues here.\n"));
      expect(onChange).toHaveBeenLastCalledWith("A wrapped paragraph continues here.\n");
      await act(async () => { expect(undo(view!)).toBe(true); });
      expect(view!.state.doc.toString()).toBe(source);
      mounted.unmount();
      realtime.close();
      saveUnwrapParagraphs(false);
      const offRealtime = new LocalRealtime();
      const off = render(<DocumentEditor {...props} realtime={offRealtime} />);
      try {
        await waitFor(() => expect(view?.state.doc.toString()).toBe(source));
      } finally {
        off.unmount();
        offRealtime.close();
      }
    } finally {
      mounted.unmount();
      realtime.close();
      localStorage.removeItem(UNWRAP_PARAGRAPHS_KEY);
    }
  });
  it("mounts one CodeMirror instance whose text IS the raw source", async () => {
    const realtime = new LocalRealtime();
    // Compact so every line is inside jsdom's zero-height viewport estimate.
    const source =
      '# Title\n<hick:file path="a.py" language="python">\nx = 1\n</hick:file>\n<hick:exec container="shell">\nls\n</hick:exec>\n';
    const { container } = render(
      <DocumentEditor
        docId="d3"
        initialSource={source}
        realtime={realtime}
        execBlocks={[]}
        runningCells={new Set()}
        onRunCell={() => undefined}
      />,
    );
    const content = container.querySelector(".cm-content");
    expect(content).toBeTruthy();
    // Seeding happens after the (immediately-resolved) sync promise.
    await waitFor(() =>
      expect(content!.textContent).toContain('<hick:file path="a.py" language="python">'),
    );
    expect(content!.textContent).toContain("# Title");
    // The exec cell got a rail icon, NOT a card in the text.
    await waitFor(() => expect(container.querySelector(".cm-card-rail__icon")).toBeTruthy());
    // …and the rail actually PLACED it. This assertion is the reason the
    // rail's `place()` is reachable at all in a test: it used to throw on
    // `CSS.escape`, which jsdom does not ship, from inside a measure
    // callback — an unhandled error printed beside a green suite, so the
    // whole placement path ran nowhere. See lib/attrSelector.ts.
    await waitFor(() =>
      expect(
        container.querySelector<HTMLElement>(".cm-card-rail__icon")!.style.top,
      ).not.toBe(""),
    );
    // The file block got its path chip, inline on the line it opens.
    expect(container.querySelector(".cm-file-chip")?.textContent).toContain("a.py");
    realtime.close();
  });

  it("stacks two crowded icons rather than drawing them on top of each other", async () => {
    const realtime = new LocalRealtime();
    // Two cells on adjacent lines want the same pixels — in jsdom every line
    // measures zero, so they want *exactly* the same pixel, which is the
    // crowding case `stackIcons` exists for.
    const source =
      '<hick:exec container="shell">\nls\n</hick:exec>\n' +
      '<hick:exec container="shell">\npwd\n</hick:exec>\n';
    const { container } = render(
      <DocumentEditor
        docId="rail"
        initialSource={source}
        realtime={realtime}
        execBlocks={[]}
        runningCells={new Set()}
        onRunCell={() => undefined}
      />,
    );
    // Each cell carries several verbs, so the rail holds more icons than
    // cells — every one of them wants the same pixel here.
    await waitFor(() =>
      expect(
        container.querySelectorAll(".cm-card-rail__icon").length,
      ).toBeGreaterThan(1),
    );
    await waitFor(() => {
      const tops = [
        ...container.querySelectorAll<HTMLElement>(".cm-card-rail__icon"),
      ].map((el) => Number.parseFloat(el.style.top));
      expect(tops.every((t) => Number.isFinite(t))).toBe(true);
      // Monotonic and non-overlapping: an icon is pushed DOWN, never up.
      for (let i = 1; i < tops.length; i++) {
        expect(tops[i] - tops[i - 1]).toBeGreaterThanOrEqual(ICON_SIZE);
      }
    });
    realtime.close();
  });

  // Protects docs/guarantees/authoring/the-gutters-never-skip-a-number.md
  it("puts no unnumbered row in the document: every row is a line or a fold", async () => {
    const realtime = new LocalRealtime();
    // One of every widget that used to be a block widget.
    const source =
      '<hick:container name="shell" image="alpine:3.20" />\n' +
      '<hick:file path="a.py" language="python">\nx = 1\n</hick:file>\n' +
      '<hick:copy id="hdr">\nq\n</hick:copy>\n' +
      '<hick:when test="!with-r">\nprose\n</hick:when>\n' +
      '<hick:feature name="with-r" description="R section" />\n' +
      '<hick:exec container="shell">\nls\n</hick:exec>\n';
    const { container } = render(
      <DocumentEditor
        docId="dRows"
        initialSource={source}
        realtime={realtime}
        execBlocks={[]}
        runningCells={new Set()}
        onRunCell={() => undefined}
      />,
    );
    await waitFor(() => expect(container.querySelector(".cm-file-chip")).toBeTruthy());
    // Every annotation sits INSIDE a `.cm-line` — a row CodeMirror numbers.
    // A block widget is a child of `.cm-content` instead, which is exactly
    // the row the gutter cannot label.
    for (const selector of [
      ".cm-file-chip",
      ".cm-frag-chip",
      ".cm-hick-banner-when",
      ".cm-hick-banner-feature",
      ".cm-env-inline",
    ]) {
      const el = container.querySelector(selector);
      expect(el, `${selector} is rendered`).toBeTruthy();
      expect(el!.closest(".cm-line"), `${selector} sits on a numbered line`).toBeTruthy();
    }
    // Every screen row is either a document line or a rendered block standing
    // in for a run of lines — a fold, which the gutter has always been
    // allowed to step over. Nothing else may occupy a row.
    const rows = Array.from(container.querySelectorAll(".cm-content > *"));
    expect(rows.length).toBeGreaterThan(0);
    for (const row of rows) {
      const kind = row.classList.contains("cm-line")
        ? "line"
        // The fold is `.cm-rendered` inside a `.cm-rendered-frame`, whose only
        // job is to hold the gap around it as padding CodeMirror can measure.
        : row.classList.contains("cm-rendered-frame") && row.querySelector(".cm-rendered")
          ? "fold"
          : row.className;
      expect(kind, "row is a line or a rendered fold").not.toBe(row.className);
    }
    realtime.close();
  });

  // Protects docs/guarantees/authoring/a-literate-file-opens-rendered.md
  it("puts every one of a cell's verbs on the rail, and none in the card", async () => {
    const realtime = new LocalRealtime();
    const onRun = vi.fn();
    const source = '<hick:exec container="shell" image="debian:12">\nhick --version\n</hick:exec>\n';
    const { container } = render(
      <DocumentEditor
        docId="d1"
        initialSource={source}
        realtime={realtime}
        execBlocks={[execBlocks[0]]}
        runningCells={new Set()}
        onRunCell={onRun}
      />,
    );
    // Reader-friendly on open: the result is on screen without being asked
    // for, and the command it ran is shown in place of the tags.
    await waitFor(() => expect(container.querySelector(".cm-rendered-exec")).toBeTruthy());
    expect(screen.getByTestId("cell-command").textContent).toContain("$ hick --version");
    expect(screen.getByText("Recorded · run succeeded")).toBeTruthy();
    // The card is something to READ: not one clickable thing inside it.
    expect(container.querySelector(".cm-rendered-exec button")).toBeNull();
    // The tags are folded away, not deleted: the document still holds them.
    const view = (window as unknown as { __hickoryView?: EditorViewType }).__hickoryView!;
    expect(view.state.doc.toString()).toBe(source);

    // Run is the rail's own icon now, and running is what clicking it does.
    const run = container.querySelector<HTMLButtonElement>(".cm-card-rail__act-run")!;
    expect(run.getAttribute("aria-label")).toContain("click to run it");
    // A verb does not latch, so it reports no pressed state.
    expect(run.getAttribute("aria-pressed")).toBeNull();
    run.click();
    expect(onRun).toHaveBeenCalledWith("cli-version");

    // The source icon is the way back, and says which way it goes.
    const src = () => container.querySelector<HTMLButtonElement>(".cm-card-rail__act-source")!;
    expect(src().getAttribute("aria-pressed")).toBe("true");
    expect(src().getAttribute("aria-label")).toContain("click for the source");
    src().click();
    await waitFor(() => expect(container.querySelector(".cm-rendered-exec")).toBeNull());
    expect(container.querySelector(".cm-content")!.textContent).toContain(
      '<hick:exec container="shell" image="debian:12">',
    );

    // And back again.
    container.querySelector<HTMLButtonElement>(".cm-card-rail__act-source")!.click();
    await waitFor(() => expect(container.querySelector(".cm-rendered-exec")).toBeTruthy());
    realtime.close();
  });

  it("keeps a block you are still typing as source", async () => {
    // Render-on-open is for reading a file, not for fighting the caret: a
    // cell written after the document opened stays as text.
    const realtime = new LocalRealtime();
    const { container } = render(
      <DocumentEditor
        docId="dNew"
        initialSource={"# Notes\n"}
        realtime={realtime}
        execBlocks={[]}
        runningCells={new Set()}
        onRunCell={() => undefined}
      />,
    );
    await waitFor(() => expect(container.querySelector(".cm-content")).toBeTruthy());
    const view = (window as unknown as { __hickoryView?: EditorViewType }).__hickoryView!;
    view.dispatch({
      changes: {
        from: view.state.doc.length,
        insert: '<hick:exec container="a">\nls\n</hick:exec>\n',
      },
      userEvent: "input.type",
    });
    await waitFor(() => expect(container.querySelector(".cm-card-rail__exec")).toBeTruthy());
    expect(container.querySelector(".cm-rendered-exec")).toBeNull();
    realtime.close();
  });

  it("annotates a hick:container declaration inline — no unnumbered card row", async () => {
    const realtime = new LocalRealtime();
    const source =
      '<hick:container name="shell" image="alpine:3.20" />\n<hick:exec container="shell">\nls\n</hick:exec>\n';
    const { container } = render(
      <DocumentEditor
        docId="dEnv"
        initialSource={source}
        realtime={realtime}
        execBlocks={[]}
        runningCells={new Set()}
        onRunCell={() => undefined}
      />,
    );
    // The environment note is an INLINE widget at the end of the declaration
    // line: every screen row keeps a line number. A block-widget card was a
    // row the gutter could not number.
    await waitFor(() => expect(container.querySelector(".cm-env-inline")).toBeTruthy());
    const slot = container.querySelector(".cm-env-inline")!;
    expect(slot.closest(".cm-line"), "inline on a real document line").toBeTruthy();
    expect(container.querySelector(".cm-env-card")).toBeNull();
    // The declaration itself is the document text — name and image live on
    // numbered lines, not in chrome.
    expect(container.querySelector(".cm-content")!.textContent).toContain(
      '<hick:container name="shell" image="alpine:3.20" />',
    );
    realtime.close();
  });

  it("colours a C# file block and an XML one, in the document itself", async () => {
    // The question a reader actually asks: opening scaffolding.md, is the
    // `Program.cs` that `dotnet new` wrote coloured like code? Until the
    // registry learned C# and XML it was flat grey text, whatever the
    // document said the file was.
    const realtime = new LocalRealtime();
    const source =
      '<hick:file path="Program.cs">\n// hello\nConsole.WriteLine("Hello");\n</hick:file>\n';
    const { container } = render(
      <DocumentEditor
        docId="dCs"
        initialSource={source}
        realtime={realtime}
        execBlocks={[]}
        runningCells={new Set()}
        onRunCell={() => undefined}
      />,
    );
    await waitFor(() => expect(container.querySelector(".tok-comment")).toBeTruthy());
    expect(container.querySelector(".tok-comment")?.textContent).toBe("// hello");
    expect(
      Array.from(container.querySelectorAll(".tok-string")).map((n) => n.textContent),
    ).toContain('"Hello"');
    realtime.close();
  });

  it("renders fragment chips, when banners, paste chips and embedded highlighting", async () => {
    const realtime = new LocalRealtime();
    const source =
      '<hick:copy id="hdr" class="analysis-py">\ndef f():\n    return 1\n</hick:copy>\n' +
      '<hick:file path="analysis.py">\n<hick:paste select=".analysis-py" />\n</hick:file>\n' +
      '<hick:when test="!with-r">\n## Gated heading\n</hick:when>\n';
    const { container } = render(
      <DocumentEditor
        docId="dFrag"
        initialSource={source}
        realtime={realtime}
        execBlocks={[]}
        runningCells={new Set()}
        onRunCell={() => undefined}
      />,
    );
    await waitFor(() => expect(container.querySelector(".cm-frag-chip")).toBeTruthy());
    const chip = container.querySelector(".cm-frag-chip")!;
    expect(chip.textContent).toContain("#hdr");
    expect(chip.textContent).toContain(".analysis-py");
    expect(chip.textContent).toContain("python"); // inferred from analysis.py
    const banner = container.querySelector(".cm-hick-banner-when");
    expect(banner?.textContent).toContain("!with-r");
    expect(container.querySelector(".cm-paste-chip")).toBeTruthy();
    // Embedded python highlighting inside the copy body (decoration-only).
    const keyword = container.querySelector(".tok-keyword");
    expect(keyword?.textContent).toBe("def");
    // Nested prose inside hick:when still styles as a heading.
    expect(container.querySelector(".cm-md-h2")?.textContent).toContain("Gated heading");
    // And the raw source is untouched.
    expect(container.querySelector(".cm-content")!.textContent).toContain(
      '<hick:copy id="hdr" class="analysis-py">',
    );
    realtime.close();
  });

  // Protects docs/guarantees/authoring/a-fence-becomes-a-cell-that-runs.md
  it("turns a prose fence into an exec cell, replacing exactly the fence", async () => {
    const realtime = new LocalRealtime();
    const source =
      '<hick:container name="py" image="python:3.12" />\n\n' +
      "Run this:\n\n```python\nprint(1)\n```\n\ndone\n";
    const { container } = render(
      <DocumentEditor
        docId="dFence"
        initialSource={source}
        realtime={realtime}
        execBlocks={[]}
        runningCells={new Set()}
        onRunCell={() => undefined}
      />,
    );
    // The fence gets its own rail icon.
    await waitFor(() =>
      expect(container.querySelector(".cm-card-rail__fence")).toBeTruthy(),
    );
    container.querySelector<HTMLButtonElement>(".cm-card-rail__fence")!.click();

    // The container it will run in was taken from the document.
    await waitFor(() => expect(screen.getByRole("button", { name: "Convert" })).toBeTruthy());
    expect((screen.getByLabelText(/Container/) as HTMLSelectElement).value).toBe("py");
    screen.getByRole("button", { name: "Convert" }).click();

    const view = (window as unknown as { __hickoryView?: EditorViewType }).__hickoryView!;
    await waitFor(() => expect(view.state.doc.toString()).toContain("<hick:exec"));
    const text = view.state.doc.toString();
    // The fence is gone, its program is intact, and the prose around it is
    // byte-for-byte where it was.
    expect(text).not.toContain("```");
    expect(text).toContain("python3 - <<'EOF'\nprint(1)\nEOF");
    expect(text.startsWith('<hick:container name="py" image="python:3.12" />\n\nRun this:\n\n')).toBe(
      true,
    );
    expect(text.endsWith("\n\ndone\n")).toBe(true);
    // And the new cell has taken the fence's place on the rail.
    await waitFor(() => expect(container.querySelector(".cm-card-rail__exec")).toBeTruthy());
    expect(container.querySelector(".cm-card-rail__fence")).toBeNull();
    realtime.close();
  });

  it.each([false, true])("converts current fence contents after editing (panel already open: %s)", async (openFirst) => {
    const realtime = new LocalRealtime();
    const source = '<hick:container name="shell" image="alpine:3.20" />\n\nHello:\n\n```shell\necho old\n```\n';
    const mounted = render(<DocumentEditor docId={`edited-fence-${openFirst}`} initialSource={source} realtime={realtime}
      execBlocks={[]} runningCells={new Set()} onRunCell={() => undefined} />);
    try {
      const icon = () => mounted.container.querySelector<HTMLButtonElement>(".cm-card-rail__fence")!;
      await waitFor(() => expect(icon()).toBeTruthy());
      if (openFirst) {
        await act(async () => icon().click());
        await screen.findByRole("button", { name: "Convert" });
      }
      const view = (window as unknown as { __hickoryView: EditorViewType }).__hickoryView;
      const start = view.state.doc.toString().indexOf("echo old");
      await act(async () => view.dispatch({ changes: { from: start, to: start + "echo old".length, insert: "dotnet new console blah" } }));
      // Language edits must also refresh the label and converter.
      const language = view.state.doc.toString().indexOf("```shell") + 3;
      await act(async () => view.dispatch({ changes: { from: language, to: language + 5, insert: "python" } }));
      await waitFor(() => expect(icon().getAttribute("aria-label")).toContain("python"));
      if (!openFirst) await act(async () => icon().click());
      await screen.findByRole("button", { name: "Convert" });
      expect(mounted.container.querySelector(".insert-menu__preview-text")?.textContent).toContain("python3 - <<'EOF'\ndotnet new console blah\nEOF");
      await act(async () => screen.getByRole("button", { name: "Convert" }).click());
      await waitFor(() => expect(view.state.doc.toString()).toContain("python3 - <<'EOF'\ndotnet new console blah\nEOF"));
      expect(view.state.doc.toString()).not.toContain("```");
      expect(screen.queryByRole("dialog")).toBeNull();
    } finally { mounted.unmount(); realtime.close(); }
  });

  it("closes the fence converter when its source disappears", async () => {
    const realtime = new LocalRealtime();
    const mounted = render(<DocumentEditor docId="deleted-fence" initialSource={"Hello:\n\n```shell\necho hello\n```\n"}
      realtime={realtime} execBlocks={[]} runningCells={new Set()} onRunCell={() => undefined} />);
    try {
      const icon = await screen.findByRole("button", { name: /Code fence.*make it a cell/ });
      await act(async () => icon.click());
      await screen.findByRole("button", { name: "Convert" });
      const view = (window as unknown as { __hickoryView: EditorViewType }).__hickoryView;
      await act(async () => view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: "Hello:\n" } }));
      await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
      expect(view.state.doc.toString()).toBe("Hello:\n");
    } finally { mounted.unmount(); realtime.close(); }
  });

  it("draws every prose fence as code, even when Hick gives it no special meaning", async () => {
    const realtime = new LocalRealtime();
    const source =
      "A JSON example is documentation, not a cell.\n\n```json\n{\"enabled\": true}\n```\n";
    const { container } = render(
      <DocumentEditor
        docId="dMarkdownFence"
        initialSource={source}
        realtime={realtime}
        execBlocks={[]}
        runningCells={new Set()}
        onRunCell={() => undefined}
      />,
    );
    await waitFor(() => expect(container.querySelector(".cm-fence-line")).toBeTruthy());
    expect(container.querySelectorAll(".cm-fence-first")).toHaveLength(1);
    expect(container.querySelectorAll(".cm-fence-last")).toHaveLength(1);
    expect(
      container.querySelector(".tok-property, .tok-propertyName, .tok-string")?.textContent,
    ).toBe('"enabled"');
    // The syntax remains Markdown source, not a newly-created Hick element.
    expect(container.querySelector(".cm-content")?.textContent).toContain("```json");
    realtime.close();
  });

  it("never corrupts text: decorations leave the document unchanged on malformed docs", async () => {
    const realtime = new LocalRealtime();
    const source = "broken <hick:exec container=\"shell\">\nno close tag, raw < and ** unbalanced";
    const { container } = render(
      <DocumentEditor
        docId="dX"
        initialSource={source}
        realtime={realtime}
        execBlocks={[]}
        runningCells={new Set()}
        onRunCell={() => undefined}
      />,
    );
    await waitFor(() => {
      const lines = Array.from(container.querySelectorAll(".cm-line")).map(
        (el) => el.textContent ?? "",
      );
      expect(lines.join("\n")).toBe(source);
    });
    realtime.close();
  });
});

</storycode:copy>
<a id="f-71bbcfe029bb-expression_statement-4"></a>
### `expression_statement`

Protects docs/guarantees/authoring/a-diagram-names-what-proves-it.md — the
panel under a proved diagram must not look like the panel under a broken one.

<storycode:copy id="f-71bbcfe029bb-expression_statement-4">
// Protects docs/guarantees/authoring/a-diagram-names-what-proves-it.md — the
// panel under a proved diagram must not look like the panel under a broken one.
describe("live diagram assertions", () => {
  const source =
    '<hick:exec container="shell" id="row-count">\nwc -l data.csv\n</hick:exec>\n' +
    '<hick:diagram renderer="mermaid" asserts="#row-count">\nflowchart TD\n  a --> b\n</hick:diagram>\n';
  const execAt = (span: [number, number], status: string): ExecBlock => ({
    kind: "exec",
    id: "shell:1",
    container: "shell",
    command: "wc -l data.csv",
    span,
    status: status as ExecBlock["status"],
  });

  it("maps each asserts id through the cell that carries it to its run state", () => {
    const structure = parseHickDoc(source);
    const exec = structure.blocks.find((b) => b.name === "exec")!;
    const span: [number, number] = [exec.from, exec.to];
    const states = (status: string, running = new Set<string>()) =>
      assertionStates(structure, [execAt(span, status)], ["row-count"], running);
    expect(states("ok")).toEqual([{ id: "row-count", state: "passing" }]);
    expect(states("failed")).toEqual([{ id: "row-count", state: "failing" }]);
    expect(states("unrecorded")).toEqual([{ id: "row-count", state: "unknown" }]);
    // A run in flight is not a verdict.
    expect(states("ok", new Set(["shell:1"]))).toEqual([{ id: "row-count", state: "unknown" }]);
    // An id no cell carries stays unknown rather than borrowing a neighbour.
    expect(assertionStates(structure, [execAt(span, "ok")], ["renamed-away"], new Set())).toEqual([
      { id: "renamed-away", state: "unknown" },
    ]);
  });

  it("tells the reader, in the document, when a named cell now fails", async () => {
    mermaidRender.mockResolvedValue({ svg: "<svg></svg>" });
    const realtime = new LocalRealtime();
    const failing = execAt([0, source.indexOf("</hick:exec>") + "</hick:exec>".length], "failed");
    const { container } = render(
      <DocumentEditor
        docId="dDiagLive"
        initialSource={source}
        realtime={realtime}
        execBlocks={[failing]}
        runningCells={new Set()}
        onRunCell={() => undefined}
      />,
    );
    await waitFor(() => expect(container.querySelector(".diagram-assertions")).toBeTruthy());
    await waitFor(() =>
      expect(container.querySelector(".diagram-assertions")!.textContent).toContain(
        "out of date",
      ),
    );
    realtime.close();
  });

  it("draws a derived diagram from the server's resolved body, not the paste tag", async () => {
    mermaidRender.mockResolvedValue({ svg: "<svg></svg>" });
    const realtime = new LocalRealtime();
    const derived =
      '<hick:copy id="edges">\nflowchart TD\n  a --> b\n</hick:copy>\n' +
      '<hick:diagram renderer="mermaid">\n<hick:paste select="#edges" />\n</hick:diagram>\n';
    const from = derived.indexOf("<hick:diagram");
    const { container } = render(
      <DocumentEditor
        docId="dDiagDerived"
        initialSource={derived}
        realtime={realtime}
        execBlocks={[]}
        diagramBlocks={[
          {
            kind: "diagram",
            renderer: "mermaid",
            body: "flowchart TD\n  a --> b\n",
            asserts: [],
            span: [from, derived.length - 1],
          },
        ]}
        runningCells={new Set()}
        onRunCell={() => undefined}
      />,
    );
    await waitFor(() => expect(container.querySelector(".diagram-panel")).toBeTruthy());
    await waitFor(() => expect(mermaidRender).toHaveBeenCalled());
    const drawn = mermaidRender.mock.calls.at(-1)![1] as string;
    expect(drawn).toContain("a --> b");
    expect(drawn).not.toContain("hick:paste");
    realtime.close();
  });
});

</storycode:copy>
<a id="f-71bbcfe029bb-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-71bbcfe029bb-expression_statement-5">
describe("keys inside a rendered widget", () => {
  // Regression: the widgets are portalled INSIDE CodeMirror's DOM, so a
  // Delete pressed while arranging a diagram (or working a table) bubbled
  // natively into the editor's keymap and erased document text at a caret
  // nobody was looking at.
  it("never reach the buffer: Delete in a widget deletes nothing from the text", async () => {
    const realtime = new LocalRealtime();
    const source =
      '<hick:exec container="shell">\nls\n</hick:exec>\n\nprose that must survive\n';
    const { container } = render(
      <DocumentEditor
        docId="dKeys"
        initialSource={source}
        realtime={realtime}
        execBlocks={[]}
        runningCells={new Set()}
        onRunCell={() => undefined}
      />,
    );
    await waitFor(() => expect(container.querySelector(".cm-rendered-exec")).toBeTruthy());
    const view = (window as unknown as { __hickoryView?: EditorViewType }).__hickoryView!;
    // A caret parked in the prose, the way one is after any earlier edit.
    view.dispatch({ selection: { anchor: view.state.doc.length - 2 } });

    // The canvas pane carries a tabindex and takes the focus when clicked,
    // which is exactly what convinces CodeMirror it should act on keys.
    const widget = container.querySelector(".cm-rendered-exec") as HTMLElement;
    widget.tabIndex = 0;
    widget.focus();
    for (const key of ["Backspace", "Delete"]) {
      widget.dispatchEvent(new KeyboardEvent("keydown", { key, bubbles: true }));
    }
    expect(view.state.doc.toString()).toBe(source);
    realtime.close();
  });
});

</storycode:copy>
<a id="f-71bbcfe029bb-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-71bbcfe029bb-expression_statement-6">
describe("a generated picture is a door back to its source", () => {
  it("clicking the image lands the caret on the block that writes the file", async () => {
    const realtime = new LocalRealtime();
    const source =
      '<hick:file path="pic.svg" doc-hidden="true">\n<svg></svg>\n</hick:file>\n\nprose\n\n![the picture](pic.svg)\n';
    const { container } = render(
      <DocumentEditor
        docId="dImg"
        initialSource={source}
        realtime={realtime}
        execBlocks={[]}
        runningCells={new Set()}
        onRunCell={() => undefined}
        path="cards.md"
      />,
    );
    await waitFor(() => expect(container.querySelector(".cm-md-image img")).toBeTruthy());
    const img = container.querySelector(".cm-md-image img") as HTMLElement;
    img.click();
    const view = (window as unknown as { __hickoryView?: EditorViewType }).__hickoryView!;
    expect(view.state.selection.main.head).toBe(source.indexOf("<hick:file"));
  });
});

</storycode:copy>
<a id="f-71bbcfe029bb-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-71bbcfe029bb-expression_statement-7">
describe("matchExecBlock", () => {
  it("prefers span overlap, falls back to ordinal", () => {
    const blocks = execBlocks;
    expect(matchExecBlock({ span: blocks[1].span, index: 0 }, blocks)?.id).toBe(blocks[1].id);
    expect(matchExecBlock({ span: [99990, 99999], index: 1 }, blocks)?.id).toBe(blocks[1].id);
  });
});

</storycode:copy>
<a id="f-71bbcfe029bb-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-71bbcfe029bb-expression_statement-8">
describe("undo safety", () => {
  // Regression: a few Ctrl+Z presses used to erase the entire document.
  // CodeMirror's own history treated the CRDT's initial sync — "the document
  // appeared" — as an undoable local edit, so undoing past it deleted
  // everything, and the deletion synced to every other client. Undo must come
  // from the CRDT, which only tracks what this client actually typed.
  it("never erases content this client did not type", async () => {
    const realtime = new LocalRealtime();
    const source = "# Title\n\nsome prose\n";
    const { container } = render(
      <DocumentEditor
        docId="undo-1"
        initialSource={source}
        realtime={realtime}
        execBlocks={[]}
        runningCells={new Set()}
        onRunCell={() => undefined}
      />,
    );
    const content = container.querySelector(".cm-content") as HTMLElement;
    await waitFor(() => expect(content.textContent).toContain("some prose"));
    const before = content.textContent;

    // Drive the real key binding rather than a command import, so the test
    // fails if the keymap is ever wired back to CodeMirror's history.
    for (let i = 0; i < 10; i++) {
      content.dispatchEvent(
        new KeyboardEvent("keydown", { key: "z", ctrlKey: true, bubbles: true }),
      );
    }

    expect(content.textContent).toBe(before);
    expect(content.textContent).toContain("some prose");

    // …and undo must still undo what this client DID type, or the fix would
    // have traded a destructive undo for a useless one.
    const view = (window as unknown as { __hickoryView?: EditorViewType }).__hickoryView!;
    view.dispatch({
      changes: { from: 0, insert: "TYPED" },
      userEvent: "input.type",
    });
    expect(view.state.doc.toString()).toContain("TYPED");
    content.dispatchEvent(
      new KeyboardEvent("keydown", { key: "z", ctrlKey: true, bubbles: true }),
    );
    expect(view.state.doc.toString()).not.toContain("TYPED");
    expect(view.state.doc.toString()).toContain("some prose");
    realtime.close();
  });
});

</storycode:copy>
<a id="f-71bbcfe029bb-expression_statement-9"></a>
### `expression_statement`

Protects docs/guarantees/authoring/an-ingested-scaffold-opens-as-a-tree.md

<storycode:copy id="f-71bbcfe029bb-expression_statement-9">
// Protects docs/guarantees/authoring/an-ingested-scaffold-opens-as-a-tree.md
describe("an ingested scaffold, mounted", () => {
  const SOURCE = [
    "# Owning what a scaffolder wrote",
    "",
    '<hick:exec container="sdk" mount="project:out">',
    '<hick:copy id="scaffold">dotnet new webapi -o out</hick:copy>',
    '<hick:ingested from="#scaffold" sha256="9f2c" at="2026-09-01" files="3" skipped="6">',
    '<hick:file path="service/Controllers/HomeController.cs">using Microsoft.AspNetCore.Mvc;',
    "",
    "public class HomeController : ControllerBase { }",
    "</hick:file>",
    '<hick:file path="service/Controllers/WeatherController.cs">using Microsoft.AspNetCore.Mvc;',
    "",
    "public class WeatherController : ControllerBase { }",
    "</hick:file>",
    '<hick:file path="service/Program.cs">var builder = WebApplication.CreateBuilder(args);',
    "builder.Build().Run();",
    "</hick:file>",
    "</hick:ingested>",
    "</hick:exec>",
    "",
    "And now your four lines.",
    "",
  ].join("\n");

  const mount = () => {
    const realtime = new LocalRealtime();
    const rendered = render(
      <DocumentEditor
        docId="ingest"
        initialSource={SOURCE}
        realtime={realtime}
        execBlocks={[]}
        runningCells={new Set()}
        onRunCell={() => undefined}
      />,
    );
    return { realtime, ...rendered };
  };

  it("opens with each file folded to its own tag line and a counted placeholder", async () => {
    const { container, realtime } = mount();
    const content = container.querySelector(".cm-content")!;
    await waitFor(() =>
      expect(content.textContent).toContain("Owning what a scaffolder wrote"),
    );
    // The folds land in a microtask after the first non-empty document.
    await waitFor(() =>
      expect(container.querySelectorAll(".cm-hick-fold-counted").length).toBe(3),
    );
    const labels = [...container.querySelectorAll(".cm-hick-fold-counted")].map(
      (el) => el.textContent,
    );
    expect(labels).toEqual(["3 lines", "3 lines", "2 lines"]);

    // Every path is still on screen — that visible line per file IS the tree.
    for (const path of [
      "service/Controllers/HomeController.cs",
      "service/Controllers/WeatherController.cs",
      "service/Program.cs",
    ]) {
      expect(content.textContent).toContain(path);
    }
    // …and the bodies are not.
    expect(content.textContent).not.toContain("public class HomeController");
    expect(content.textContent).not.toContain("builder.Build().Run();");
    // The prose on the other side of the scaffold is where it always was.
    expect(content.textContent).toContain("And now your four lines.");
    realtime.close();
  });

  it("leaves the bytes exactly as they were — a fold is not an edit", async () => {
    const { container, realtime } = mount();
    await waitFor(() =>
      expect(container.querySelectorAll(".cm-hick-fold-counted").length).toBe(3),
    );
    const view = (window as unknown as { __hickoryView?: EditorViewType })
      .__hickoryView!;
    expect(view.state.doc.toString()).toBe(SOURCE);
    realtime.close();
  });
});

</storycode:copy>
<a id="f-71bbcfe029bb-expression_statement-10"></a>
### `expression_statement`

<storycode:copy id="f-71bbcfe029bb-expression_statement-10">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("agent document input shares the live room callback and respects a later read-only owner", async () => {
  const realtime = new LocalRealtime();
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const source = "# Title\n\nfirst\n", onChange = vi.fn();
  let view: EditorViewType | null = null;
  const element = (readOnly: boolean) => <UiActionContext.Provider value={registry}><DocumentEditor docId="ui-source-edit" initialSource={source} realtime={realtime} readOnly={readOnly} preserveBytes execBlocks={[]} runningCells={new Set()} onRunCell={vi.fn()} onChange={onChange} onViewReady={next => { view = next; }} /></UiActionContext.Provider>;
  const mounted = render(element(false));
  try {
    await waitFor(() => expect(view?.state.doc.toString()).toBe(source));
    const action = () => registry.groups().groups.flatMap(g => registry.observe(g).actions).find(a => a.label === "Replace document editor range")!;
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action().handle, arguments: { from: 9, to: 14, text: "changed" }, request_id: "edit" }); });
    expect(result).toMatchObject({ status: "result", outcome: { value: { requested_applied: true, approved: false, verified: false } } });
    expect(onChange).toHaveBeenCalledWith("# Title\n\nchanged\n");
    mounted.rerender(element(true));
    expect(action().enabled).toBe(false);
    expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action().handle, arguments: { from: 9, to: 16, text: "wrong" }, request_id: "readonly" })).toMatchObject({ status: "disabled" });
    act(() => { view!.dispatch({ changes: { from: 9, to: 16, insert: "wrong" }, userEvent: "input" }); });
    expect(view!.state.doc.toString()).toBe("# Title\n\nchanged\n");
  } finally { mounted.unmount(); realtime.close(); }
});

</storycode:copy>
<a id="f-71bbcfe029bb-expression_statement-11"></a>
### `expression_statement`

<storycode:copy id="f-71bbcfe029bb-expression_statement-11">
it("refuses a fence conversion against a changed live document before React refreshes the preview", async () => {
  const source = '<hick:container name="py" image="python:3.12" />\n\n```python\nprint(1)\n```\n';
  const realtime = new LocalRealtime();
  const view = render(<DocumentEditor docId="fence-stale" initialSource={source} realtime={realtime} execBlocks={[]} runningCells={new Set()} onRunCell={() => {}} />);
  try {
    const rail = await screen.findByRole("button", { name: /make it a cell/i });
    act(() => rail.click());
    const convert = await screen.findByRole("button", { name: "Convert" });
    const editor = (window as unknown as { __hickoryView: EditorViewType }).__hickoryView;
    act(() => {
      const from = editor.state.doc.toString().indexOf("print(1)");
      editor.dispatch({ changes: { from, to: from + 8, insert: "print(2)" }, userEvent: "input" });
      convert.click(); // The old DOM callback still closes over the prior preview.
    });
    expect(editor.state.doc.toString()).toContain("```python\nprint(2)");
    expect(editor.state.doc.toString()).not.toContain("<hick:exec");
    await waitFor(() => expect(document.querySelector(".insert-menu__preview-text")?.textContent).toContain("print(2)"));
    act(() => screen.getByRole("button", { name: "Convert" }).click());
    expect(editor.state.doc.toString()).toContain("<hick:exec");
    expect(editor.state.doc.toString()).toContain("print(2)");
  } finally { view.unmount(); realtime.close(); }
});

</storycode:copy>
<a id="f-71bbcfe029bb-expression_statement-12"></a>
### `expression_statement`

<storycode:copy id="f-71bbcfe029bb-expression_statement-12">
it("keeps an edited CSV fence bound to its real span and shares dataset promotion", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const source = 'Before\n\n```csv\nname,count\nNorth,2\n```\n\nAfter\n';
  const realtime = new LocalRealtime();
  const view = render(<UiActionContext.Provider value={registry}><DocumentEditor docId="fence-table-owned" initialSource={source} realtime={realtime} execBlocks={[]} runningCells={new Set()} onRunCell={() => {}} /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, {
    handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id),
  }); }); return result; };
  try {
    const rail = await screen.findByRole("button", { name: /make it a cell/i });
    act(() => rail.click());
    await screen.findByTestId("table-panel");
    const editor = (window as unknown as { __hickoryView: EditorViewType }).__hickoryView;
    await run("Write table cell contents", { row: 1, column: 1, value: "12345" });
    expect(editor.state.doc.toString()).toContain("North,12345");
    expect(screen.getByTestId("table-panel")).toBeTruthy();
    await run("Write table cell contents", { row: 1, column: 1, value: "678" });
    expect(editor.state.doc.toString()).toContain("North,678");
    await run("Set fence dataset path", { path: "data/north.csv" });
    expect((screen.getByLabelText(/file to write the csv to/i) as HTMLInputElement).value).toBe("data/north.csv");
    expect(await run("Promote fence to table element")).toMatchObject({ status: "result", outcome: { value: { source_changed: true, file_written: false } } });
    expect(editor.state.doc.toString()).toBe('Before\n\n<hick:table path="data/north.csv">\nname,count\nNorth,678\n</hick:table>\n\nAfter\n');
  } finally { view.unmount(); realtime.close(); }
});
</storycode:copy>

### Assemble `apps/web/src/editor/DocumentEditor.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/DocumentEditor.test.tsx"><storycode:paste select="#f-71bbcfe029bb-mermaidRender"/><storycode:paste select="#f-71bbcfe029bb-expression_statement"/><storycode:paste select="#f-71bbcfe029bb-expression_statement-2"/><storycode:paste select="#f-71bbcfe029bb-execBlocks"/><storycode:paste select="#f-71bbcfe029bb-expression_statement-3"/><storycode:paste select="#f-71bbcfe029bb-expression_statement-4"/><storycode:paste select="#f-71bbcfe029bb-expression_statement-5"/><storycode:paste select="#f-71bbcfe029bb-expression_statement-6"/><storycode:paste select="#f-71bbcfe029bb-expression_statement-7"/><storycode:paste select="#f-71bbcfe029bb-expression_statement-8"/><storycode:paste select="#f-71bbcfe029bb-expression_statement-9"/><storycode:paste select="#f-71bbcfe029bb-expression_statement-10"/><storycode:paste select="#f-71bbcfe029bb-expression_statement-11"/><storycode:paste select="#f-71bbcfe029bb-expression_statement-12"/></storycode:file>

</storycode:doc>
