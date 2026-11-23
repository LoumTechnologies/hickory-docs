<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Files use a familiar sidebar tree

For: authors opening and organizing files in a workspace.

Files shows a hierarchical sidebar: a single click opens a text file, folders
have disclosure arrows, and expanded folders persist per workspace. Arrow keys
move through visible rows; Right expands or enters a folder, Left collapses or
returns to its parent, and Enter activates a row. Show Files focuses the tree.
F2 prepares a rename; Delete requests confirmation before removing anything.
Context menus provide new file, new folder, rename, move, copy and delete.
Ordinary typing never edits filenames or queues filesystem operations.

Rows retain paths for provenance and show unsaved-buffer indicators. Folder
contents appear independently of language services and provenance loading.
Filesystem writes continue through the existing authoritative endpoint and
its permission, containment and overwrite checks. Nested terminals and issues
remain reachable beneath expanded directories.

Requirement decision: the user's November 2026 request for a conventional file
sidebar supersedes the editable-buffer, text-selection and Dired interaction
requirements. The former [detail 1](requirements/01.md),
[detail 2](requirements/02.md) and [original capture](../../../support/files/docs/guarantees/authoring/the-workspace-tree-edits-like-an-editor.md.md)
remain historical context, not current sidebar requirements.

The focused FolderTreePane tests cover disclosure, activation, keyboard focus,
persistence, context actions and deletion confirmation. The browser witness
below exercises the visible tree against a real filesystem endpoint.

## Historical detail

These included captures retain the former requirements for comparison only;
the sidebar contract above supersedes their filename-buffer interactions.

<storycode:include file="requirements/01.md" />
<storycode:include file="requirements/02.md" />

## Verification on 2026-11-06

TypeScript checking and all 48 focused FolderTreePane/treeNavigation tests passed.
Source reconstruction check and ownership audit passed. The updated native
browser flow could not execute: cached Playwright Chromium aborts during macOS
MachPortRendezvous bootstrap; the connected browser host is unavailable too.
Browser appearance and the real-server flow remain unverified in this session.
Run `UV_OFFLINE=1 bash scripts/test-spatial-ui.sh --sidebar` on a machine that can
launch Chromium. The fixture contains disposable files and cleans itself up.

## Implementation

<a id="output-a743bd990aba"></a>
## `apps/web/src/lib/filesystemTreeText.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Filesystem Acts Keep Dired's Safety](../the-tree-is-a-dired/contract.md), [The Workspace Tree Is An Editor Buffer](contract.md).

<a id="f-a743bd990aba-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-a743bd990aba-expression_statement">
import { describe, expect, it } from "vitest";

import {
  filesystemTextEntries,
  filesystemTreeOperations,
  filesystemTreeText,
  parseFilesystemTreeText,
  reconcileFilesystemTreeText,
} from "./filesystemTreeText";

// Guarantee: docs/guarantees/authoring/the-workspace-tree-edits-like-an-editor.md
describe("the filesystem as significant-whitespace text", () => {
  const nodes = [{ name: "src", path: "src", dir: true, children: [
    { name: "one.ts", path: "src/one.ts", dir: false },
  ] }, { name: "notes.txt", path: "notes.txt", dir: false }];

  it("renders hierarchy as ordinary indented lines", () => {
    expect(filesystemTreeText(filesystemTextEntries(nodes))).toBe("src/\n  one.ts\nnotes.txt");
  });

  it("turns names and indentation into filesystem destinations", () => {
    const original = filesystemTextEntries(nodes);
    const parsed = parseFilesystemTreeText("source/\n  first.ts\n  notes.txt", original);
    expect(filesystemTreeOperations(parsed.edits!)).toEqual([
      { path: "src", to: "source" },
      { path: "source/one.ts", to: "source/first.ts" },
      { path: "notes.txt", to: "source/notes.txt" },
    ]);
  });

  it("treats reordered unchanged entries as presentation, not filesystem edits", () => {
    const result = reconcileFilesystemTreeText(
      "notes.txt\nsrc/\n  one.ts",
      filesystemTextEntries(nodes),
    );
    expect(result.reconciliation).toEqual({ renames: [], creates: [], deletes: [] });
  });

  it("matches unchanged reordered entries before finding the one real rename", () => {
    const result = reconcileFilesystemTreeText(
      "notes.txt\nsrc/\n  first.ts",
      filesystemTextEntries(nodes),
    );
    expect(result.reconciliation?.renames).toEqual([
      { path: "src/one.ts", to: "src/first.ts" },
    ]);
  });

  it("keeps reordered row identity through a column-style prefix edit", () => {
    const original = filesystemTextEntries([
      { name: "one.txt", path: "one.txt", dir: false },
      { name: "two.txt", path: "two.txt", dir: false },
    ]);
    const result = reconcileFilesystemTreeText("renamed-two.txt\nrenamed-one.txt", original);
    expect(result.reconciliation?.renames).toEqual([
      { path: "two.txt", to: "renamed-two.txt" },
      { path: "one.txt", to: "renamed-one.txt" },
    ]);
  });

  it("refuses malformed or type-changing structure", () => {
    const original = filesystemTextEntries(nodes);
    expect(parseFilesystemTreeText("src/\n one.ts\nnotes.txt", original).error).toContain("two spaces");
    expect(parseFilesystemTreeText("src\n  one.ts\nnotes.txt", original).error).toContain("trailing slash");
    expect(parseFilesystemTreeText("folder\n  child.txt", filesystemTextEntries([
      { name: "one.txt", path: "one.txt", dir: false },
      { name: "two.txt", path: "two.txt", dir: false },
    ])).error).toContain("Folder lines must end in /");
    expect(parseFilesystemTreeText("one.txt\n\n", filesystemTextEntries([
      { name: "one.txt", path: "one.txt", dir: false },
      { name: "two.txt", path: "two.txt", dir: false },
    ])).error).toContain("Line 2 is blank");
  });

  it("ignores one terminal newline rather than treating it as a blank file", () => {
    const original = filesystemTextEntries(nodes);
    expect(reconcileFilesystemTreeText("src/\n  one.ts\nnotes.txt\n", original).reconciliation)
      .toEqual({ renames: [], creates: [], deletes: [] });
  });

  it("turns inserted lines into parent-first creates", () => {
    const result = reconcileFilesystemTreeText(
      "src/\n  one.ts\n  fixtures/\n    sample.json\nnotes.txt",
      filesystemTextEntries(nodes),
    );
    expect(result.reconciliation?.creates).toEqual([
      { path: "src/fixtures", dir: true },
      { path: "src/fixtures/sample.json", dir: false },
    ]);
  });

  it("creates a folder and moves an unchanged uniquely named file into it in one apply", () => {
    const original = filesystemTextEntries([
      { name: "test53.py", path: "test53.py", dir: false },
    ]);
    const result = reconcileFilesystemTreeText("test 1/\n  test53.py", original);
    expect(result.reconciliation).toEqual({
      renames: [{ path: "test53.py", to: "test 1/test53.py" }],
      creates: [{ path: "test 1", dir: true }],
      deletes: [],
    });
  });

  it("collapses a removed folder subtree into one reviewed delete", () => {
    const result = reconcileFilesystemTreeText("notes.txt", filesystemTextEntries(nodes));
    expect(result.reconciliation?.deletes).toEqual(["src"]);
  });

  it("refuses a new entry combined with a renamed existing one", () => {
    const result = reconcileFilesystemTreeText("source/\n  one.ts\n  new.ts\nnotes.txt", filesystemTextEntries(nodes));
    expect(result.error).toContain("Cannot create entries and rename existing ones in the same Apply");
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/filesystemTreeText.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/filesystemTreeText.test.ts"><storycode:paste select="#f-a743bd990aba-expression_statement"/></storycode:file>

<a id="output-cd4772c3dc30"></a>
## `apps/web/src/lib/filesystemTreeText.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The Workspace Tree Is An Editor Buffer](contract.md).

<a id="f-cd4772c3dc30-TREE_INDENT"></a>
### `TREE_INDENT`

<storycode:copy id="f-cd4772c3dc30-TREE_INDENT">
import type { FileNode } from "../api/types";

export const TREE_INDENT = 2;

</storycode:copy>
<a id="f-cd4772c3dc30-FilesystemTextEntry"></a>
### `FilesystemTextEntry`

<storycode:copy id="f-cd4772c3dc30-FilesystemTextEntry">
export interface FilesystemTextEntry {
  path: string;
  name: string;
  dir: boolean;
  depth: number;
}

</storycode:copy>
<a id="f-cd4772c3dc30-FilesystemTextEdit"></a>
### `FilesystemTextEdit`

<storycode:copy id="f-cd4772c3dc30-FilesystemTextEdit">
export interface FilesystemTextEdit extends FilesystemTextEntry {
  desiredPath: string;
}

</storycode:copy>
<a id="f-cd4772c3dc30-FilesystemTreeReconciliation"></a>
### `FilesystemTreeReconciliation`

<storycode:copy id="f-cd4772c3dc30-FilesystemTreeReconciliation">
export interface FilesystemTreeReconciliation {
  renames: { path: string; to: string }[];
  creates: { path: string; dir: boolean }[];
  deletes: string[];
}

</storycode:copy>
<a id="f-cd4772c3dc30-matchExistingLines"></a>
### `matchExistingLines`

<storycode:copy id="f-cd4772c3dc30-matchExistingLines">
function matchExistingLines(
  desired: readonly DesiredLine[],
  original: readonly FilesystemTextEntry[],
): { matches?: (number | null)[]; error?: string } {
  const unmatched = new Set(original.map((_entry, index) => index));
  const matches = desired.map((line) => {
    const exact = original.findIndex((entry, index) =>
      unmatched.has(index) && entry.path === line.desiredPath && entry.dir === line.dir);
    if (exact >= 0) unmatched.delete(exact);
    return exact >= 0 ? exact : null;
  });

  // An unchanged name is enough to identify a moved row when it is unique on
  // both sides. This is the useful unambiguous case: make `folder/`, then
  // indent an existing `file.txt` under it in one Apply. Renames deliberately
  // stay out: without ids, a new `report.txt` and a renamed `notes.txt` are
  // indistinguishable from each other.
  for (let index = 0; index < matches.length; index += 1) {
    if (matches[index] !== null) continue;
    const line = desired[index];
    const candidates = [...unmatched].filter((originalIndex) =>
      original[originalIndex].dir === line.dir && original[originalIndex].name === line.name);
    if (candidates.length === 1) {
      matches[index] = candidates[0];
      unmatched.delete(candidates[0]);
    }
  }
  return { matches };
}

</storycode:copy>
<a id="f-cd4772c3dc30-filesystemBufferLines"></a>
### `filesystemBufferLines`

CodeMirror commonly leaves one terminal newline after pressing Enter. It
does not name a filesystem entry; blank lines elsewhere remain invalid.

<storycode:copy id="f-cd4772c3dc30-filesystemBufferLines">
/** CodeMirror commonly leaves one terminal newline after pressing Enter. It
 * does not name a filesystem entry; blank lines elsewhere remain invalid. */
function filesystemBufferLines(text: string): string[] {
  const lines = text.split("\n");
  if (lines.length > 1 && lines[lines.length - 1] === "") lines.pop();
  return lines;
}

</storycode:copy>
<a id="f-cd4772c3dc30-visibleNameSimilarity"></a>
### `visibleNameSimilarity`

<storycode:copy id="f-cd4772c3dc30-visibleNameSimilarity">
function visibleNameSimilarity(left: string, right: string): number {
  let prefix = 0;
  while (prefix < left.length && prefix < right.length && left[prefix] === right[prefix]) prefix += 1;
  let suffix = 0;
  while (suffix < left.length - prefix
    && suffix < right.length - prefix
    && left[left.length - 1 - suffix] === right[right.length - 1 - suffix]) suffix += 1;
  return prefix + suffix;
}

</storycode:copy>
<a id="f-cd4772c3dc30-filesystemTextEntries"></a>
### `filesystemTextEntries`

<storycode:copy id="f-cd4772c3dc30-filesystemTextEntries">
export function filesystemTextEntries(nodes: readonly FileNode[], depth = 0): FilesystemTextEntry[] {
  return nodes.flatMap((node) => [
    { path: node.path, name: node.name, dir: node.dir, depth },
    ...(node.dir ? filesystemTextEntries(node.children ?? [], depth + 1) : []),
  ]);
}

</storycode:copy>
<a id="f-cd4772c3dc30-filesystemTreeText"></a>
### `filesystemTreeText`

<storycode:copy id="f-cd4772c3dc30-filesystemTreeText">
export function filesystemTreeText(entries: readonly FilesystemTextEntry[]): string {
  return entries.map((entry) => `${" ".repeat(entry.depth * TREE_INDENT)}${entry.name}${entry.dir ? "/" : ""}`).join("\n");
}

</storycode:copy>
<a id="f-cd4772c3dc30-parseFilesystemTreeText"></a>
### `parseFilesystemTreeText`

<storycode:copy id="f-cd4772c3dc30-parseFilesystemTreeText">
export function parseFilesystemTreeText(
  text: string,
  original: readonly FilesystemTextEntry[],
): { edits?: FilesystemTextEdit[]; error?: string } {
  const rawLines = filesystemBufferLines(text);
  if (rawLines.length !== original.length) {
    return { error: "Creating and deleting entries from the Files buffer is not implemented yet; restore one line per existing entry." };
  }
  const desiredDirectories = rawLines.filter((line) => line.trimEnd().endsWith("/")).length;
  const originalDirectories = original.filter((entry) => entry.dir).length;
  if (desiredDirectories !== originalDirectories) {
    return { error: "The trailing slash marks a folder and cannot be added or removed." };
  }
  const desired = parseDesiredLines(text);
  if (!desired.lines) return { error: desired.error };

  // Lines have no hidden ids. Match their strongest visible identity first,
  // independent of row order, then pair the remaining rows by kind for real
  // name and indentation edits.
  const matched = matchExistingLines(desired.lines, original);
  const matches = matched.matches!;
  const unmatched = new Set(original.map((_entry, index) => index));
  for (const match of matches) if (match !== null) unmatched.delete(match);
  for (let index = 0; index < matches.length; index += 1) {
    if (matches[index] !== null) continue;
    const line = desired.lines[index];
    const candidate = [...unmatched]
      .filter((originalIndex) => original[originalIndex].dir === line.dir)
      .sort((left, right) =>
        visibleNameSimilarity(line.name, original[right].name)
        - visibleNameSimilarity(line.name, original[left].name))[0];
    if (candidate === undefined) {
      return { error: `Line ${index + 1}: the trailing slash marks a folder and cannot be added or removed.` };
    }
    matches[index] = candidate;
    unmatched.delete(candidate);
  }

  return { edits: desired.lines.map((line, index) => ({
    ...original[matches[index]!],
    name: line.name,
    depth: line.depth,
    desiredPath: line.desiredPath,
  })) };
}

</storycode:copy>
<a id="f-cd4772c3dc30-DesiredLine"></a>
### `DesiredLine`

<storycode:copy id="f-cd4772c3dc30-DesiredLine">
interface DesiredLine {
  raw: string;
  name: string;
  dir: boolean;
  depth: number;
  desiredPath: string;
}

</storycode:copy>
<a id="f-cd4772c3dc30-parseDesiredLines"></a>
### `parseDesiredLines`

<storycode:copy id="f-cd4772c3dc30-parseDesiredLines">
function parseDesiredLines(text: string): { lines?: DesiredLine[]; error?: string } {
  const rawLines = filesystemBufferLines(text);
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
    if (!name) {
      return { error: `Line ${index + 1} is blank. Remove the empty line; every Files-pane line must name a file or end in / for a folder.` };
    }
    if (name.includes("/") || name.includes("\\") || name === "." || name === "..") {
      return { error: `Line ${index + 1}: ${JSON.stringify(name)} is not one filesystem name.` };
    }
    if (depth > 0 && !parents[depth - 1]) {
      return { error: `Line ${index + 1}: indentation has no parent folder at level ${depth}. Folder lines must end in / (for example, test folder 1/).` };
    }
    parents.splice(depth);
    const parent = depth === 0 ? "" : parents[depth - 1];
    const desiredPath = parent ? `${parent}/${name}` : name;
    lines.push({ raw, name, dir, depth, desiredPath });
    if (dir) parents[depth] = desiredPath;
  }
  return { lines };
}

</storycode:copy>
<a id="f-cd4772c3dc30-subsequencePositions"></a>
### `subsequencePositions`

<storycode:copy id="f-cd4772c3dc30-subsequencePositions">
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

</storycode:copy>
<a id="f-cd4772c3dc30-reconcileFilesystemTreeText"></a>
### `reconcileFilesystemTreeText`

Read the edited buffer conservatively. A save may rename/move existing
entries, add entries, or remove entries. A new folder plus an unchanged,
uniquely named file beneath it is identifiable without ids; other
create-and-rename combinations remain separate acts.

<storycode:copy id="f-cd4772c3dc30-reconcileFilesystemTreeText">
/** Read the edited buffer conservatively. A save may rename/move existing
 * entries, add entries, or remove entries. A new folder plus an unchanged,
 * uniquely named file beneath it is identifiable without ids; other
 * create-and-rename combinations remain separate acts. */
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
    const matched = matchExistingLines(desired.lines, original);
    const matches = matched.matches!;
    if (new Set(matches.filter((match): match is number => match !== null)).size !== original.length) {
      return {
        error: "Cannot create entries and rename existing ones in the same Apply. Rename first, then create the new entry and Apply again.",
      };
    }
    return {
      reconciliation: {
        renames: filesystemTreeOperations(desired.lines.flatMap((line, index) => {
          const match = matches[index];
          return match === null ? [] : [{
            ...original[match],
            name: line.name,
            depth: line.depth,
            desiredPath: line.desiredPath,
          }];
        })),
        creates: desired.lines.filter((_line, index) => matches[index] === null).map((line) => ({ path: line.desiredPath, dir: line.dir })),
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

</storycode:copy>
<a id="f-cd4772c3dc30-filesystemTreeOperations"></a>
### `filesystemTreeOperations`

Operations are parent-first. A child carried unchanged by a parent rename
needs no second filesystem call.

<storycode:copy id="f-cd4772c3dc30-filesystemTreeOperations">
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
</storycode:copy>

### Assemble `apps/web/src/lib/filesystemTreeText.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/filesystemTreeText.ts"><storycode:paste select="#f-cd4772c3dc30-TREE_INDENT"/><storycode:paste select="#f-cd4772c3dc30-FilesystemTextEntry"/><storycode:paste select="#f-cd4772c3dc30-FilesystemTextEdit"/><storycode:paste select="#f-cd4772c3dc30-FilesystemTreeReconciliation"/><storycode:paste select="#f-cd4772c3dc30-matchExistingLines"/><storycode:paste select="#f-cd4772c3dc30-filesystemBufferLines"/><storycode:paste select="#f-cd4772c3dc30-visibleNameSimilarity"/><storycode:paste select="#f-cd4772c3dc30-filesystemTextEntries"/><storycode:paste select="#f-cd4772c3dc30-filesystemTreeText"/><storycode:paste select="#f-cd4772c3dc30-parseFilesystemTreeText"/><storycode:paste select="#f-cd4772c3dc30-DesiredLine"/><storycode:paste select="#f-cd4772c3dc30-parseDesiredLines"/><storycode:paste select="#f-cd4772c3dc30-subsequencePositions"/><storycode:paste select="#f-cd4772c3dc30-reconcileFilesystemTreeText"/><storycode:paste select="#f-cd4772c3dc30-filesystemTreeOperations"/></storycode:file>

<a id="output-d2a3983c1549"></a>
## `apps/web/src/shell/TreeRenameEditor.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The Workspace Tree Is An Editor Buffer](contract.md).

<a id="f-d2a3983c1549-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-d2a3983c1549-expression_statement">
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { EditorView } from "@codemirror/view";
import { afterEach, describe, expect, it, vi } from "vitest";

import { fileRenameError, renameValues, TreeRenameEditor } from "./TreeRenameEditor";

afterEach(cleanup);

</storycode:copy>
<a id="f-d2a3983c1549-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-d2a3983c1549-expression_statement-2">
// Guarantee: docs/guarantees/authoring/the-workspace-tree-edits-like-an-editor.md
describe("the semantic rename column", () => {
  it("keeps one line per selected node and refuses paths", () => {
    expect(renameValues("a\nb", 2)).toEqual({ values: ["a", "b"] });
    expect(renameValues("a", 2).error).toContain("exactly 2 lines");
    expect(renameValues("a\n", 2).error).toContain("Line 2");
    expect(fileRenameError("elsewhere/a.md")).toContain("not a path");
    expect(fileRenameError("..")).toContain("not a file name");
  });

  it("applies every line independently and reports partial failure", async () => {
    const apply = vi.fn(async (_item: { key: string }, value: string) => {
      if (value === "taken.md") throw new Error("taken.md already exists");
    });
    render(
      <TreeRenameEditor
        items={[
          { key: "file:a", context: "src/a.md", value: "a.md" },
          { key: "file:b", context: "src/b.md", value: "b.md" },
        ]}
        apply={apply}
        validate={fileRenameError}
        onClose={() => {}}
      />,
    );
    const editor = EditorView.findFromDOM(document.querySelector(".cm-editor")!);
    if (!editor) throw new Error("rename editor did not mount");
    editor.dispatch({ changes: { from: 0, to: editor.state.doc.length, insert: "renamed.md\ntaken.md" } });
    fireEvent.click(screen.getByRole("button", { name: "Apply 2 renames" }));
    expect(await screen.findByText(/a\.md → renamed\.md/)).toBeTruthy();
    expect(screen.getByText(/b\.md → taken\.md/).closest("li")?.textContent).toContain("already exists");
    expect(apply).toHaveBeenCalledTimes(2);

    editor.dispatch({ changes: { from: 0, to: editor.state.doc.length, insert: "ignored.md\nfixed.md" } });
    fireEvent.click(screen.getByRole("button", { name: "Retry failed renames" }));
    await screen.findByText(/b\.md → fixed\.md/);
    expect(apply).toHaveBeenCalledTimes(3);
    expect(apply.mock.calls.filter(([item]) => item.key === "file:a")).toHaveLength(1);
  });
});
</storycode:copy>

### Assemble `apps/web/src/shell/TreeRenameEditor.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/TreeRenameEditor.test.tsx"><storycode:paste select="#f-d2a3983c1549-expression_statement"/><storycode:paste select="#f-d2a3983c1549-expression_statement-2"/></storycode:file>

<a id="output-baf9b27bb0eb"></a>
## `apps/web/src/shell/treeNavigation.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The Workspace Tree Is An Editor Buffer](contract.md).

<a id="f-baf9b27bb0eb-rows"></a>
### `rows`

<storycode:copy id="f-baf9b27bb0eb-rows">
import { describe, expect, it } from "vitest";

import { treeNavigationIntent, treeSelectionRange, type NavigableTreeRow } from "./treeNavigation";

const rows: NavigableTreeRow[] = [
  { key: "src", parent: null, expandable: true, expanded: true, selectable: true },
  { key: "src/a", parent: "src", selectable: true },
  { key: "term", parent: "src", selectable: false },
  { key: "readme", parent: null, selectable: true },
];

</storycode:copy>
<a id="f-baf9b27bb0eb-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-baf9b27bb0eb-expression_statement">
// Guarantee: docs/guarantees/authoring/the-workspace-tree-edits-like-an-editor.md
describe("workspace-tree navigation", () => {
  it("moves in visible order and to the ends", () => {
    expect(treeNavigationIntent(rows, "src/a", "ArrowDown")).toEqual({ kind: "focus", key: "term" });
    expect(treeNavigationIntent(rows, "src/a", "ArrowUp")).toEqual({ kind: "focus", key: "src" });
    expect(treeNavigationIntent(rows, "src/a", "Home")).toEqual({ kind: "focus", key: "src" });
    expect(treeNavigationIntent(rows, "src/a", "End")).toEqual({ kind: "focus", key: "readme" });
  });

  it("uses left and right structurally", () => {
    expect(treeNavigationIntent(rows, "src", "ArrowLeft")).toEqual({ kind: "toggle", key: "src" });
    expect(treeNavigationIntent(rows, "src", "ArrowRight")).toEqual({ kind: "focus", key: "src/a" });
    expect(treeNavigationIntent(rows, "src/a", "ArrowLeft")).toEqual({ kind: "focus", key: "src" });
    expect(treeNavigationIntent([{ ...rows[0], expanded: false }], "src", "ArrowRight")).toEqual({ kind: "toggle", key: "src" });
  });

  it("activates with Enter", () => {
    expect(treeNavigationIntent(rows, "readme", "Enter")).toEqual({ kind: "activate", key: "readme" });
  });

  it("selects compatible rows across a visible range", () => {
    expect(treeSelectionRange(rows, "src/a", "readme")).toEqual(["src/a", "readme"]);
  });
});
</storycode:copy>

### Assemble `apps/web/src/shell/treeNavigation.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/treeNavigation.test.ts"><storycode:paste select="#f-baf9b27bb0eb-rows"/><storycode:paste select="#f-baf9b27bb0eb-expression_statement"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/editor/multiCursor.ts](../../../mechanisms/apps/web/src/editor/multiCursor.ts.md#output-08df9c7d9da5)
- [apps/web/src/lib/filesystemTreeText.ts](contract.md#output-cd4772c3dc30)
- apps/web/src/shell/FilesystemTreeEditor.tsx
- [apps/web/src/shell/FolderTreePane.tsx](../../../mechanisms/apps/web/src/shell/FolderTreePane.tsx.md#output-9c0af3061e25)

### Tests and fixtures

- apps/web/e2e/files-editor.spec.ts
- [apps/web/src/lib/filesystemTreeText.test.ts](contract.md#output-a743bd990aba)
- apps/web/src/shell/FilesystemTreeEditor.test.tsx
- [apps/web/src/shell/FolderTreePane.test.tsx](../../../mechanisms/apps/web/src/shell/FolderTreePane.tsx.md#output-508e32e299fd)
- [apps/web/src/shell/GithubBodyAndComment.test.tsx](../../../mechanisms/apps/web/src/shell/GithubBodyAndComment.tsx.md#output-e42c6eb9a161)
- [apps/web/src/shell/GithubTreeControls.test.tsx](../../../mechanisms/apps/web/src/shell/GithubTreeControls.tsx.md#output-66d5cea236ab)
- [apps/web/src/shell/GithubTreeNodes.test.tsx](../../integrations/a-review-belongs-to-the-worktree-for-its-head/contract.md#output-7956bbacfb50)
- [apps/web/src/shell/TreeRenameEditor.test.tsx](contract.md#output-d2a3983c1549)
- [apps/web/src/shell/treeNavigation.test.ts](contract.md#output-baf9b27bb0eb)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
