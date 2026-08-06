// @vitest-environment jsdom
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { DocumentEditor, matchExecBlock } from "./DocumentEditor";
import { LocalRealtime } from "../api/realtime";
import { CLI_BLOCKS } from "../mock/mockData";
import type { ExecBlock } from "../api/types";

afterEach(cleanup);

const execBlocks = CLI_BLOCKS.filter((b): b is ExecBlock => b.kind === "exec");

describe("DocumentEditor (WYSIWYG over raw source)", () => {
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
    // Decorations only — the tag syntax stays visible, byte for byte.
    expect(content!.textContent).toContain('<hick:file path="a.py" language="python">');
    expect(content!.textContent).toContain("# Title");
    // The exec cell got its panel widget below the block.
    await waitFor(() => expect(container.querySelector(".cm-cell-panel")).toBeTruthy());
    // The file block got its path chip.
    expect(container.querySelector(".cm-file-chip")?.textContent).toContain("a.py");
    realtime.close();
  });

  it("renders the cell panel (status + Run) through the widget portal", async () => {
    const realtime = new LocalRealtime();
    const onRun = vi.fn();
    const source = '<hick:exec container="shell" image="debian:12">\nhickory --version\n</hick:exec>\n';
    render(
      <DocumentEditor
        docId="d1"
        initialSource={source}
        realtime={realtime}
        execBlocks={[execBlocks[0]]}
        runningCells={new Set()}
        onRunCell={onRun}
      />,
    );
    await waitFor(() => expect(screen.getByRole("button", { name: "Run" })).toBeTruthy());
    expect(screen.getByText("ok")).toBeTruthy();
    screen.getByRole("button", { name: "Run" }).click();
    expect(onRun).toHaveBeenCalledWith("cli-version");
    realtime.close();
  });

  it("renders a hick:container declaration as an environment card (tag stays visible)", async () => {
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
    await waitFor(() => expect(container.querySelector(".env-card")).toBeTruthy());
    const card = container.querySelector(".env-card")!;
    expect(card.textContent).toContain("shell");
    expect(card.textContent).toContain("alpine:3.20");
    // Decoration-only: the tag source itself is still in the document text.
    expect(container.querySelector(".cm-content")!.textContent).toContain(
      '<hick:container name="shell" image="alpine:3.20" />',
    );
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

  it("never corrupts text: decorations leave the document unchanged on malformed docs", () => {
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
    const lines = Array.from(container.querySelectorAll(".cm-line")).map(
      (el) => el.textContent ?? "",
    );
    expect(lines.join("\n")).toBe(source);
    realtime.close();
  });
});

describe("matchExecBlock", () => {
  it("prefers span overlap, falls back to ordinal", () => {
    const blocks = execBlocks;
    expect(matchExecBlock({ span: blocks[1].span, index: 0 }, blocks)?.id).toBe(blocks[1].id);
    expect(matchExecBlock({ span: [99990, 99999], index: 1 }, blocks)?.id).toBe(blocks[1].id);
  });
});
