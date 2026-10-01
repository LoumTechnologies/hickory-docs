// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import { installMockApi } from "./mockApi";
import { WEAVE_SOURCE } from "./mockData";
import { mapEditsToSource, weaveOutputs, SyntheticRangeViolation } from "../lib/weave";
import { api, ApiError } from "../api/client";
import type { SyntheticRangeError } from "../api/types";
import { byteLength, byteToChar, charToByte } from "../lib/offsets";

/** Slice a string by UTF-8 byte range (both source and content offsets are
 * bytes on the wire; WEAVE_SOURCE and the woven banner contain em dashes, so
 * bytes and chars genuinely differ). */
function byteSlice(text: string, start: number, end: number): string {
  return text.slice(byteToChar(text, start), byteToChar(text, end));
}

describe("mock weaver", () => {
  it("weaves two hick:copy slots into one output file with real provenance", () => {
    const files = weaveOutputs(WEAVE_SOURCE, "docs/weave-demo.md");
    expect(files.map((f) => f.path)).toEqual(["src/latency.py"]);
    const file = files[0];
    expect(file.language).toBe("python");
    expect(file.content).toContain("def load_runs(path):");
    expect(file.content).toContain("def summarise(runs):");
    expect(file.content).toContain('if __name__ == "__main__":');

    // Provenance tiles the whole content (in bytes) with no gaps or overlaps.
    let pos = 0;
    for (const p of file.provenance) {
      expect(p.start).toBe(pos);
      expect(p.end).toBeGreaterThan(p.start);
      pos = p.end;
    }
    expect(pos).toBe(byteLength(file.content));

    // Every non-synthetic range's output text equals its source-span text.
    for (const p of file.provenance) {
      if (p.origin.kind === "synthetic") continue;
      expect(byteSlice(file.content, p.start, p.end)).toBe(
        byteSlice(WEAVE_SOURCE, p.origin.span[0], p.origin.span[1]),
      );
    }

    const kinds = file.provenance.map((p) => p.origin.kind);
    expect(kinds).toContain("synthetic"); // woven banner
    expect(kinds.filter((k) => k === "paste")).toHaveLength(2); // both slots
    expect(kinds).toContain("literal"); // the entry point
  });

  it("maps an output edit inside a pasted slot back to the copy's source span", () => {
    const file = weaveOutputs(WEAVE_SOURCE, "docs/weave-demo.md")[0];
    const at = charToByte(file.content, file.content.indexOf("statistics.median"));
    const edits = [{ start: at, end: at + "statistics.median".length, text: "statistics.mean" }];
    const sourceEdits = mapEditsToSource(file, edits);
    expect(sourceEdits).toHaveLength(1);
    expect(sourceEdits[0].doc_path).toBe("docs/weave-demo.md");
    expect(byteSlice(WEAVE_SOURCE, sourceEdits[0].span[0], sourceEdits[0].span[1])).toBe(
      "statistics.median",
    );
    expect(sourceEdits[0].text).toBe("statistics.mean");
  });

  it("rejects edits overlapping synthetic ranges with the offending range", () => {
    const file = weaveOutputs(WEAVE_SOURCE, "docs/weave-demo.md")[0];
    const synthetic = file.provenance.find((p) => p.origin.kind === "synthetic")!;
    expect(() =>
      mapEditsToSource(file, [{ start: synthetic.start, end: synthetic.start + 5, text: "nope" }]),
    ).toThrow(SyntheticRangeViolation);
    try {
      mapEditsToSource(file, [{ start: synthetic.start, end: synthetic.start + 5, text: "nope" }]);
    } catch (e) {
      expect((e as SyntheticRangeViolation).range).toEqual({
        start: synthetic.start,
        end: synthetic.end,
      });
    }
  });
});

describe("mock /outputs lineage round trip (through the typed client)", () => {
  it("GET outputs → GET file → POST edit → source updated → rewoven output reflects the edit", async () => {
    installMockApi();
    const outputs = await api.outputs("d3");
    expect(outputs.files).toEqual([{ path: "src/latency.py", language: "python" }]);

    const before = await api.outputFile("d3", "src/latency.py");
    expect(before.provenance.length).toBeGreaterThan(3);

    const at = charToByte(before.content, before.content.indexOf("load_runs(path)"));
    const res = await api.editOutput("d3", "src/latency.py", [
      { start: at, end: at + "load_runs(path)".length, text: "load_runs(path, *, strict=True)" },
    ]);
    expect(res.applied).toBe(true);
    expect(res.source_edits).toHaveLength(1);

    // The doc source now contains the edited text at the mapped span.
    const doc = await api.doc("d3");
    expect(doc.source).toContain("def load_runs(path, *, strict=True):");

    // Re-weaving from the edited source reproduces the edited output.
    const after = await api.outputFile("d3", "src/latency.py");
    expect(after.content).toContain("def load_runs(path, *, strict=True):");
    // Provenance still tiles the new content.
    let pos = 0;
    for (const p of after.provenance) {
      expect(p.start).toBe(pos);
      pos = p.end;
    }
    expect(pos).toBe(byteLength(after.content));
  });

  it("returns a 422 ApiError with the offending range for synthetic edits", async () => {
    installMockApi();
    const file = await api.outputFile("d3", "src/latency.py");
    const synthetic = file.provenance.find((p) => p.origin.kind === "synthetic")!;
    let caught: unknown = null;
    try {
      await api.editOutput("d3", "src/latency.py", [
        { start: synthetic.start, end: synthetic.start + 3, text: "x" },
      ]);
    } catch (e) {
      caught = e;
    }
    expect(caught).toBeInstanceOf(ApiError);
    const err = caught as ApiError;
    expect(err.status).toBe(422);
    const body = err.body as SyntheticRangeError;
    expect(body.range).toEqual({ start: synthetic.start, end: synthetic.end });
  });
});
