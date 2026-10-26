<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lineage/structure.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b8a05f59d8ae"></a>
## `apps/web/src/lineage/structure.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-b8a05f59d8ae-definitionId"></a>
### `definitionId`

Fold structural navigation into the lineage model.
The server finds definitions and references with tree-sitter and matches
them by name (crates/hick-structure). This turns that into nodes and links
of kind `structural`, so navigation shares one picture with provenance
instead of living in a second panel that has to be kept in step.
Two rules keep it honest:
* A definition becomes a node only when the browser has that file, so a
link can never point at something the reader cannot open.
* A name matching several definitions produces several links, each
carrying how many candidates there were. The view can then say "one of
three" rather than picking one and looking certain.

<storycode:copy id="f-b8a05f59d8ae-definitionId">
// Fold structural navigation into the lineage model.
//
// The server finds definitions and references with tree-sitter and matches
// them by name (crates/hick-structure). This turns that into nodes and links
// of kind `structural`, so navigation shares one picture with provenance
// instead of living in a second panel that has to be kept in step.
//
// Two rules keep it honest:
//
//  * A definition becomes a node only when the browser has that file, so a
//    link can never point at something the reader cannot open.
//  * A name matching several definitions produces several links, each
//    carrying how many candidates there were. The view can then say "one of
//    three" rather than picking one and looking certain.

import type { StructureResponse } from "../api/types";
import type { LineageModel, LineageNode, Link } from "./model";

/** Stable id for a definition — the file and the line it starts on. */
export function definitionId(path: string, line: number): string {
  return `${path}@def:${line}`;
}

</storycode:copy>
<a id="f-b8a05f59d8ae-referenceId"></a>
### `referenceId`

<storycode:copy id="f-b8a05f59d8ae-referenceId">
/** Stable id for a reference site. */
export function referenceId(path: string, line: number): string {
  return `${path}@ref:${line}`;
}

</storycode:copy>
<a id="f-b8a05f59d8ae-StructuralAddition"></a>
### `StructuralAddition`

<storycode:copy id="f-b8a05f59d8ae-StructuralAddition">
export interface StructuralAddition {
  nodes: LineageNode[];
  links: Link[];
  /** Names that matched more than one definition, for reporting. */
  ambiguous: Map<string, number>;
}

</storycode:copy>
<a id="f-b8a05f59d8ae-structuralAddition"></a>
### `structuralAddition`

<storycode:copy id="f-b8a05f59d8ae-structuralAddition">
export function structuralAddition(
  model: LineageModel,
  structure: StructureResponse,
): StructuralAddition {
  const nodes: LineageNode[] = [];
  const links: Link[] = [];
  const ambiguous = new Map<string, number>();
  const seen = new Set<string>();

  const add = (node: LineageNode) => {
    if (seen.has(node.id)) return;
    seen.add(node.id);
    nodes.push(node);
  };

  const defs = new Map<string, { name: string; kind: string; end: number }>();
  for (const file of structure.files) {
    if (!model.files.has(file.path) || model.files.get(file.path)?.preview) continue;
    for (const def of file.definitions) {
      defs.set(definitionId(file.path, def.start_line), {
        name: def.name,
        kind: def.kind,
        end: def.end_line,
      });
    }
  }

  for (const link of structure.links) {
    // Both ends must be files the browser can show; a link into a file it
    // does not have is a dead end dressed as navigation.
    if (!model.files.has(link.from_path) || !model.files.has(link.to_path)
      || model.files.get(link.from_path)?.preview || model.files.get(link.to_path)?.preview) continue;
    const target = defs.get(definitionId(link.to_path, link.to_line));
    if (!target) continue;

    const fromId = referenceId(link.from_path, link.from_line);
    const toId = definitionId(link.to_path, link.to_line);

    add({
      id: fromId,
      file: link.from_path,
      startLine: link.from_line,
      endLine: link.from_line,
      label: `${link.name}()`,
      kind: "reference",
    });
    add({
      id: toId,
      file: link.to_path,
      startLine: link.to_line,
      endLine: target.end,
      label: target.name,
      kind: target.kind,
    });
    links.push({ from: fromId, to: toId, kind: "structural" });
    if (link.candidates > 1) ambiguous.set(link.name, link.candidates);
  }

  return { nodes, links, ambiguous };
}

</storycode:copy>
<a id="f-b8a05f59d8ae-withStructure"></a>
### `withStructure`

A copy of `model` with structural nodes and links merged in.

<storycode:copy id="f-b8a05f59d8ae-withStructure">
/** A copy of `model` with structural nodes and links merged in. */
export function withStructure(
  model: LineageModel,
  structure: StructureResponse,
): { model: LineageModel; ambiguous: Map<string, number> } {
  const addition = structuralAddition(model, structure);
  const nodes = new Map(model.nodes);
  for (const node of addition.nodes) {
    // Provenance nodes win a collision: a generated run is a stronger claim
    // about a range than a name match at the same line.
    if (!nodes.has(node.id)) nodes.set(node.id, node);
  }
  return {
    model: { ...model, nodes, links: [...model.links, ...addition.links] },
    ambiguous: addition.ambiguous,
  };
}
</storycode:copy>

### Assemble `apps/web/src/lineage/structure.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lineage/structure.ts"><storycode:paste select="#f-b8a05f59d8ae-definitionId"/><storycode:paste select="#f-b8a05f59d8ae-referenceId"/><storycode:paste select="#f-b8a05f59d8ae-StructuralAddition"/><storycode:paste select="#f-b8a05f59d8ae-structuralAddition"/><storycode:paste select="#f-b8a05f59d8ae-withStructure"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-cd85845ac6f7"></a>
## `apps/web/src/lineage/structure.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-cd85845ac6f7-DOC"></a>
### `DOC`

<storycode:copy id="f-cd85845ac6f7-DOC">
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

</storycode:copy>
<a id="f-cd85845ac6f7-model"></a>
### `model`

<storycode:copy id="f-cd85845ac6f7-model">
function model() {
  return buildModel([
    { path: "notes.md", source: DOC, outputs: weaveOutputs(DOC, "notes.md") },
  ]);
}

</storycode:copy>
<a id="f-cd85845ac6f7-STRUCTURE"></a>
### `STRUCTURE`

The shape crates/hick-structure returns, with `app.py`'s banner line
accounted for (the weaver writes one, so definitions start at line 1).

<storycode:copy id="f-cd85845ac6f7-STRUCTURE">
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

</storycode:copy>
<a id="f-cd85845ac6f7-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-cd85845ac6f7-expression_statement">
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
</storycode:copy>

### Assemble `apps/web/src/lineage/structure.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lineage/structure.test.ts"><storycode:paste select="#f-cd85845ac6f7-DOC"/><storycode:paste select="#f-cd85845ac6f7-model"/><storycode:paste select="#f-cd85845ac6f7-STRUCTURE"/><storycode:paste select="#f-cd85845ac6f7-expression_statement"/></storycode:file>


</storycode:doc>
