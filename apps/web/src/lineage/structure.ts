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

/** Stable id for a reference site. */
export function referenceId(path: string, line: number): string {
  return `${path}@ref:${line}`;
}

export interface StructuralAddition {
  nodes: LineageNode[];
  links: Link[];
  /** Names that matched more than one definition, for reporting. */
  ambiguous: Map<string, number>;
}

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
    if (!model.files.has(file.path)) continue;
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
    if (!model.files.has(link.from_path) || !model.files.has(link.to_path)) continue;
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
