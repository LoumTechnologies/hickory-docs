import { describe, expect, it } from "vitest";

import { weaveOutputs } from "../lib/weave";
import { buildModel } from "./build";
import { structuralAddition, withStructure } from "./structure";
import { walk, type LinkKind } from "./model";
import type { StructureResponse } from "../api/types";

const DOC = `# Code

<hick:file path="app.py" language="python">
def load(path):
    return open(path).read()

def summarise(path):
    return len(load(path))
</hick:file>
`;

function model() {
  return buildModel([
    { path: "notes.md", source: DOC, outputs: weaveOutputs(DOC, "notes.md") },
  ]);
}

// The shape crates/hick-structure returns, with `app.py`'s banner line
// accounted for (the weaver writes one, so definitions start at line 1).
const STRUCTURE: StructureResponse = {
  files: [
    {
      path: "app.py",
      language: "python",
      definitions: [
        { name: "load", kind: "function", start_line: 1, end_line: 2 },
        { name: "summarise", kind: "function", start_line: 4, end_line: 5 },
      ],
      references: [{ name: "load", line: 5 }],
    },
  ],
  links: [
    {
      from_path: "app.py",
      from_line: 5,
      to_path: "app.py",
      to_line: 1,
      name: "load",
      candidates: 1,
    },
  ],
};

describe("folding structural navigation into the model", () => {
  it("adds a reference node and a definition node, linked", () => {
    const { nodes, links } = structuralAddition(model(), STRUCTURE);
    expect(links).toHaveLength(1);
    expect(links[0].kind).toBe("structural");
    const ids = nodes.map((n) => n.id);
    expect(ids).toContain("app.py@ref:5");
    expect(ids).toContain("app.py@def:1");
    // A definition node spans its body, so the brace around it means
    // something.
    const def = nodes.find((n) => n.id === "app.py@def:1")!;
    expect(def.endLine).toBeGreaterThan(def.startLine);
  });

  it("navigates from the call to the definition", () => {
    const { model: merged } = withStructure(model(), STRUCTURE);
    const kinds = new Set<LinkKind>(["paste", "structural", "asserted"]);
    expect([...walk(merged.links, kinds, "app.py@ref:5", "down")]).toContain("app.py@def:1");
  });

  it("keeps structural links separable from provenance", () => {
    const { model: merged } = withStructure(model(), STRUCTURE);
    // With only computed provenance enabled, the name match is not a path.
    const paste = new Set<LinkKind>(["paste"]);
    expect(walk(merged.links, paste, "app.py@ref:5", "down").size).toBe(0);
    // And the provenance links still work with structural switched off.
    expect(merged.links.some((l) => l.kind === "paste")).toBe(true);
  });

  it("drops a link into a file the browser does not have", () => {
    // Navigation that cannot be followed is worse than no navigation.
    const elsewhere: StructureResponse = {
      files: STRUCTURE.files,
      links: [
        {
          from_path: "app.py",
          from_line: 5,
          to_path: "vendor/other.py",
          to_line: 3,
          name: "load",
          candidates: 1,
        },
      ],
    };
    expect(structuralAddition(model(), elsewhere).links).toHaveLength(0);
  });

  it("reports a name that could mean more than one definition", () => {
    const ambiguousStructure: StructureResponse = {
      files: [
        {
          path: "app.py",
          language: "python",
          definitions: [
            { name: "render", kind: "function", start_line: 1, end_line: 2 },
            { name: "render", kind: "function", start_line: 4, end_line: 5 },
          ],
          references: [{ name: "render", line: 7 }],
        },
      ],
      links: [
        { from_path: "app.py", from_line: 7, to_path: "app.py", to_line: 1, name: "render", candidates: 2 },
        { from_path: "app.py", from_line: 7, to_path: "app.py", to_line: 4, name: "render", candidates: 2 },
      ],
    };
    const { links, ambiguous } = structuralAddition(model(), ambiguousStructure);
    expect(links).toHaveLength(2);
    expect(ambiguous.get("render")).toBe(2);
  });
});
