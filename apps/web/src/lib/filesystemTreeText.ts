import type { FileNode } from "../api/types";

export const TREE_INDENT = 2;

export interface FilesystemTextEntry {
  path: string;
  name: string;
  dir: boolean;
  depth: number;
}

export interface FilesystemTextEdit extends FilesystemTextEntry {
  desiredPath: string;
}

export interface FilesystemTreeReconciliation {
  renames: { path: string; to: string }[];
  creates: { path: string; dir: boolean }[];
  deletes: string[];
}

export function filesystemTextEntries(nodes: readonly FileNode[], depth = 0): FilesystemTextEntry[] {
  return nodes.flatMap((node) => [
    { path: node.path, name: node.name, dir: node.dir, depth },
    ...(node.dir ? filesystemTextEntries(node.children ?? [], depth + 1) : []),
  ]);
}

export function filesystemTreeText(entries: readonly FilesystemTextEntry[]): string {
  return entries.map((entry) => `${" ".repeat(entry.depth * TREE_INDENT)}${entry.name}${entry.dir ? "/" : ""}`).join("\n");
}

export function parseFilesystemTreeText(
  text: string,
  original: readonly FilesystemTextEntry[],
): { edits?: FilesystemTextEdit[]; error?: string } {
  const lines = text.split("\n");
  if (lines.length !== original.length) {
    return { error: "Creating and deleting entries from the Files buffer is not implemented yet; restore one line per existing entry." };
  }
  const parents: { depth: number; path: string }[] = [];
  const edits: FilesystemTextEdit[] = [];
  for (let index = 0; index < lines.length; index += 1) {
    const raw = lines[index];
    const spaces = raw.length - raw.trimStart().length;
    if (spaces % TREE_INDENT !== 0) return { error: `Line ${index + 1}: indentation must use two spaces per level.` };
    const depth = spaces / TREE_INDENT;
    const expected = original[index];
    const shown = raw.slice(spaces);
    const dir = shown.endsWith("/");
    const name = (dir ? shown.slice(0, -1) : shown).trimEnd();
    if (!name || name.includes("/") || name.includes("\\") || name === "." || name === "..") {
      return { error: `Line ${index + 1}: ${JSON.stringify(name)} is not one filesystem name.` };
    }
    if (dir !== expected.dir) return { error: `Line ${index + 1}: the trailing slash marks a folder and cannot be added or removed.` };
    if (depth > 0 && !parents[depth - 1]) return { error: `Line ${index + 1}: indentation has no parent folder at level ${depth}.` };
    parents.splice(depth);
    const parent = depth === 0 ? "" : parents[depth - 1].path;
    const desiredPath = parent ? `${parent}/${name}` : name;
    const edit = { ...expected, name, depth, desiredPath };
    edits.push(edit);
    if (dir) parents[depth] = { depth, path: desiredPath };
  }
  return { edits };
}

interface DesiredLine {
  raw: string;
  name: string;
  dir: boolean;
  depth: number;
  desiredPath: string;
}

function parseDesiredLines(text: string): { lines?: DesiredLine[]; error?: string } {
  const rawLines = text.split("\n");
  const parents: string[] = [];
  const lines: DesiredLine[] = [];
  for (let index = 0; index < rawLines.length; index += 1) {
    const raw = rawLines[index];
    const spaces = raw.length - raw.trimStart().length;
    if (spaces % TREE_INDENT !== 0) return { error: `Line ${index + 1}: indentation must use two spaces per level.` };
    const depth = spaces / TREE_INDENT;
    const shown = raw.slice(spaces);
    const dir = shown.endsWith("/");
    const name = (dir ? shown.slice(0, -1) : shown).trimEnd();
    if (!name || name.includes("/") || name.includes("\\") || name === "." || name === "..") {
      return { error: `Line ${index + 1}: ${JSON.stringify(name)} is not one filesystem name.` };
    }
    if (depth > 0 && !parents[depth - 1]) return { error: `Line ${index + 1}: indentation has no parent folder at level ${depth}.` };
    parents.splice(depth);
    const parent = depth === 0 ? "" : parents[depth - 1];
    const desiredPath = parent ? `${parent}/${name}` : name;
    lines.push({ raw, name, dir, depth, desiredPath });
    if (dir) parents[depth] = desiredPath;
  }
  return { lines };
}

function subsequencePositions(needles: readonly string[], haystack: readonly string[]): number[] | null {
  const positions: number[] = [];
  let at = 0;
  for (const needle of needles) {
    while (at < haystack.length && haystack[at] !== needle) at += 1;
    if (at === haystack.length) return null;
    positions.push(at);
    at += 1;
  }
  return positions;
}

/** Read the edited buffer conservatively. A save may rename/move existing
 * entries, add entries, or remove entries. Combining those classes makes line
 * identity ambiguous without hidden ids, so it is refused and can be saved as
 * two understandable acts. */
export function reconcileFilesystemTreeText(
  text: string,
  original: readonly FilesystemTextEntry[],
): { reconciliation?: FilesystemTreeReconciliation; error?: string } {
  const originalLines = filesystemTreeText(original).split("\n");
  const desired = parseDesiredLines(text);
  if (!desired.lines) return { error: desired.error };
  const wantedLines = desired.lines.map((line) => line.raw);

  if (wantedLines.length === originalLines.length) {
    const parsed = parseFilesystemTreeText(text, original);
    if (!parsed.edits) return { error: parsed.error };
    return { reconciliation: { renames: filesystemTreeOperations(parsed.edits), creates: [], deletes: [] } };
  }

  if (wantedLines.length > originalLines.length) {
    const existingAt = subsequencePositions(originalLines, wantedLines);
    if (!existingAt) return { error: "Create new lines in a separate save from renames or moves." };
    const existing = new Set(existingAt);
    return {
      reconciliation: {
        renames: [],
        creates: desired.lines.filter((_line, index) => !existing.has(index)).map((line) => ({ path: line.desiredPath, dir: line.dir })),
        deletes: [],
      },
    };
  }

  const keptAt = subsequencePositions(wantedLines, originalLines);
  if (!keptAt) return { error: "Delete lines in a separate save from renames or moves." };
  const kept = new Set(keptAt);
  const removed = original.filter((_entry, index) => !kept.has(index));
  const removedDirs = removed.filter((entry) => entry.dir).map((entry) => entry.path);
  const deletes = removed
    .filter((entry) => !removedDirs.some((dir) => entry.path !== dir && entry.path.startsWith(`${dir}/`)))
    .map((entry) => entry.path);
  return { reconciliation: { renames: [], creates: [], deletes } };
}

/** Operations are parent-first. A child carried unchanged by a parent rename
 * needs no second filesystem call. */
export function filesystemTreeOperations(edits: readonly FilesystemTextEdit[]): { path: string; to: string }[] {
  const moved: { from: string; to: string }[] = [];
  const operations: { path: string; to: string }[] = [];
  for (const edit of edits) {
    let source = edit.path;
    for (const parent of moved) {
      if (source.startsWith(`${parent.from}/`)) source = `${parent.to}${source.slice(parent.from.length)}`;
    }
    if (source === edit.desiredPath) continue;
    operations.push({ path: source, to: edit.desiredPath });
    if (edit.dir) moved.push({ from: edit.path, to: edit.desiredPath });
  }
  return operations;
}
