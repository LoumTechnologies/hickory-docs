// @vitest-environment jsdom
import { cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { SplitView } from "./SplitView";
import { installMockApi } from "../mock/mockApi";
import { WEAVE_SOURCE } from "../mock/mockData";
import { LocalRealtime, setSharedRealtime } from "../api/realtime";

afterEach(cleanup);

describe("SplitView (document | ribbons | output)", () => {
  it("mounts the editor left, the woven output right, and the ribbon layer", async () => {
    installMockApi();
    const realtime = new LocalRealtime();
    // SplitView also opens a live room for the active output file
    // (useOutputRealtime); registering the shared bus is what main.tsx does
    // for VITE_MOCK=1, and is this test's seam for the same behavior.
    setSharedRealtime(realtime);
    const { container } = render(
      <SplitView
        docId="d3"
        docPath="docs/weave-demo.hick"
        docSource={WEAVE_SOURCE}
        editorKey="d3:0"
        realtime={realtime}
        onChange={() => undefined}
        selectSpan={null}
        execBlocks={[]}
        runningCells={new Set()}
        onRunCell={() => undefined}
      />,
    );
    // Left: the WYSIWYG editor over the raw source.
    await waitFor(() =>
      expect(
        container.querySelector(".split-left .cm-content")?.textContent ?? "",
      ).toContain('<hick:copy id="load">'),
    );
    // Right: the woven output file (single file → path label, no tabs).
    await waitFor(() =>
      expect(
        container.querySelector(".split-output-editor .cm-content")?.textContent ?? "",
      ).toContain("def load_runs(path):"),
    );
    // Middle: the generated-file tree, with the open file selected. `src/` is
    // a directory node and `latency.py` the file node the ribbons pass through.
    const dirs = [...container.querySelectorAll(".tree-dir .tree-name")].map((e) => e.textContent);
    expect(dirs).toContain("src/");
    const active = container.querySelector(".tree-file.on");
    expect(active?.getAttribute("data-path")).toBe("src/latency.py");
    expect(active?.textContent).toContain("latency.py");
    // The SVG ribbon overlay is mounted (geometry is zero-sized in jsdom,
    // but the layer and its per-ribbon paths derive from real provenance).
    expect(container.querySelector(".ribbon-layer")).toBeTruthy();
    realtime.close();
  });
});
