<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Filesystem Acts Keep Dired's Safety

For: Authors editing executable documents and the files those documents produce.

Given an entry in the editable Files buffer, when a person opens its context
menu, then rename, move, copy, delete, new-folder, and new-file acts are routed
to one filesystem-operation endpoint inside the open folder. A rename never
overwrites, a move or copy names a destination, `.git` and paths outside the
root are refused, and the Git pane reports the resulting working-tree change.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/the-tree-is-a-dired.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-f8ba3159313f"></a>
## `apps/web/src/api/files.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Filesystem Acts Keep Dired's Safety](contract.md).

<a id="f-f8ba3159313f-FileOpRequest"></a>
### `FileOpRequest`

The tree's file verbs, as the server takes them (`POST /api/files/op`).
See docs/guarantees/authoring/the-tree-is-a-dired.md.
One dired verb on the tree.

<storycode:copy id="f-f8ba3159313f-FileOpRequest">
// The tree's file verbs, as the server takes them (`POST /api/files/op`).
// See docs/guarantees/authoring/the-tree-is-a-dired.md.

/** One dired verb on the tree. */
export interface FileOpRequest {
  op: "rename" | "move" | "copy" | "delete" | "mkdir" | "create";
  /** Root-relative path the verb acts on. */
  path: string;
  /** rename: the new name; move/copy: the destination directory. */
  to?: string;
}
</storycode:copy>

### Assemble `apps/web/src/api/files.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/api/files.ts"><storycode:paste select="#f-f8ba3159313f-FileOpRequest"/></storycode:file>

<a id="output-0cb807eba045"></a>
## `apps/web/src/shell/dired.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Filesystem Acts Keep Dired's Safety](contract.md), [Every Shortcut Is A Setting](../../editor-intelligence/every-shortcut-is-a-setting/contract.md).

<a id="f-0cb807eba045-none"></a>
### `none`

<storycode:copy id="f-0cb807eba045-none">
// docs/guarantees/authoring/the-tree-is-a-dired.md
import { describe, expect, it } from "vitest";

import { containing, diredIntent, nextMarks, targets, toggleMark } from "./dired";

const none: ReadonlySet<string> = new Set();
</storycode:copy>
<a id="f-0cb807eba045-marks"></a>
### `marks`

<storycode:copy id="f-0cb807eba045-marks">
const marks: ReadonlySet<string> = new Set(["a.md", "b.md"]);

</storycode:copy>
<a id="f-0cb807eba045-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-0cb807eba045-expression_statement">
describe("the dired keys", () => {
  it("mark, unmark and unmark-all are m, u and U", () => {
    expect(diredIntent({ key: "m" }, "x.md", false, none)).toEqual({ kind: "mark", path: "x.md" });
    expect(diredIntent({ key: "u" }, "x.md", false, marks)).toEqual({ kind: "unmark", path: "x.md" });
    expect(diredIntent({ key: "U", shiftKey: true }, "", true, marks)).toEqual({ kind: "unmark-all" });
  });

  it("D, C and M act on the marks when the row is marked, else on the row", () => {
    expect(diredIntent({ key: "D", shiftKey: true }, "a.md", false, marks)).toEqual({ kind: "delete", paths: ["a.md", "b.md"] });
    expect(diredIntent({ key: "D", shiftKey: true }, "c.md", false, marks)).toEqual({ kind: "delete", paths: ["c.md"] });
    expect(diredIntent({ key: "Delete" }, "c.md", false, none)).toEqual({ kind: "delete", paths: ["c.md"] });
    expect(diredIntent({ key: "C", shiftKey: true }, "a.md", false, marks)).toEqual({ kind: "copy", paths: ["a.md", "b.md"] });
    expect(diredIntent({ key: "M", shiftKey: true }, "c.md", false, none)).toEqual({ kind: "move", paths: ["c.md"] });
  });

  it("R opens the rename column over marks when the row is marked", () => {
    expect(diredIntent({ key: "F2" }, "a.md", false, marks)).toEqual({ kind: "rename", paths: ["a.md", "b.md"] });
    expect(diredIntent({ key: "F2" }, "c.md", false, marks)).toEqual({ kind: "rename", paths: ["c.md"] });
  });

  it("+ and n create in the row's directory, or beside a file", () => {
    expect(diredIntent({ key: "+" }, "src", true, none)).toEqual({ kind: "mkdir", dir: "src" });
    expect(diredIntent({ key: "n" }, "src/a.rs", false, none)).toEqual({ kind: "create", dir: "src" });
    expect(diredIntent({ key: "+" }, "", true, none)).toEqual({ kind: "mkdir", dir: "" });
    expect(containing("a.md", false)).toBe("");
  });

  it("is silent with a modifier held, on other keys, and on the header with nothing marked", () => {
    expect(diredIntent({ key: "m", ctrlKey: true }, "x.md", false, none)).toBeNull();
    expect(diredIntent({ key: "j" }, "x.md", false, none)).toBeNull();
    expect(diredIntent({ key: "D", shiftKey: true }, "", true, none)).toBeNull();
    expect(diredIntent({ key: "m" }, "", true, none)).toBeNull();
  });
});

</storycode:copy>
<a id="f-0cb807eba045-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-0cb807eba045-expression_statement-2">
describe("the marks", () => {
  it("follow mark, unmark and unmark-all, and leave other intents alone", () => {
    const after = nextMarks(none, { kind: "mark", path: "a.md" });
    expect([...after]).toEqual(["a.md"]);
    expect([...nextMarks(after, { kind: "unmark", path: "a.md" })]).toEqual([]);
    expect([...nextMarks(marks, { kind: "unmark-all" })]).toEqual([]);
    expect([...nextMarks(marks, { kind: "delete", paths: ["a.md"] })]).toEqual([...marks]);
  });

  it("toggle with Ctrl+click, and targets pick the marks only when the row is one", () => {
    expect([...toggleMark(none, "a.md")]).toEqual(["a.md"]);
    expect([...toggleMark(marks, "a.md")]).toEqual(["b.md"]);
    expect(targets("a.md", marks)).toEqual(["a.md", "b.md"]);
    expect(targets("c.md", marks)).toEqual(["c.md"]);
  });
});
</storycode:copy>

### Assemble `apps/web/src/shell/dired.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/dired.test.ts"><storycode:paste select="#f-0cb807eba045-none"/><storycode:paste select="#f-0cb807eba045-marks"/><storycode:paste select="#f-0cb807eba045-expression_statement"/><storycode:paste select="#f-0cb807eba045-expression_statement-2"/></storycode:file>

<a id="output-fabcac2ae554"></a>
## `apps/web/src/shell/dired.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Filesystem Acts Keep Dired's Safety](contract.md), [The Workspace Tree Is A Lens](../the-workspace-tree-is-a-lens/contract.md), [Every Shortcut Is A Setting](../../editor-intelligence/every-shortcut-is-a-setting/contract.md).

<a id="f-fabcac2ae554-DiredIntent"></a>
### `DiredIntent`

The tree as dired: marks, and the keys that act on them.
Emacs's dired is a list of files you MARK and then act on — `m` marks,
`u` unmarks, `U` unmarks all, and `D`, `C` delete and copy
the marks (or the row under point when nothing is marked). `+` makes a
folder. Those are the keys here, verbatim, because a person who wants
dired wants those keys and nobody else is hurt by them: they fire only while
a tree row has focus. Rename is F2, the familiar editor gesture, because
this tree is meant to feel like an editor.
Pure: a key and the state in, an intent out. The pane owns the marks and
runs the intents (docs/guarantees/authoring/the-tree-is-a-dired.md).

<storycode:copy id="f-fabcac2ae554-DiredIntent">
// The tree as dired: marks, and the keys that act on them.
//
// Emacs's dired is a list of files you MARK and then act on — `m` marks,
// `u` unmarks, `U` unmarks all, and `D`, `C` delete and copy
// the marks (or the row under point when nothing is marked). `+` makes a
// folder. Those are the keys here, verbatim, because a person who wants
// dired wants those keys and nobody else is hurt by them: they fire only while
// a tree row has focus. Rename is F2, the familiar editor gesture, because
// this tree is meant to feel like an editor.
//
// Pure: a key and the state in, an intent out. The pane owns the marks and
// runs the intents (docs/guarantees/authoring/the-tree-is-a-dired.md).

import { isAction } from "../lib/keymap";

export type DiredIntent =
  | { kind: "mark"; path: string }
  | { kind: "unmark"; path: string }
  | { kind: "unmark-all" }
  | { kind: "delete"; paths: string[] }
  | { kind: "rename"; paths: string[] }
  | { kind: "copy"; paths: string[] }
  | { kind: "move"; paths: string[] }
  | { kind: "mkdir"; dir: string }
  | { kind: "create"; dir: string };

</storycode:copy>
<a id="f-fabcac2ae554-DiredKey"></a>
### `DiredKey`

<storycode:copy id="f-fabcac2ae554-DiredKey">
export type DiredKey = {
  key: string;
  ctrlKey?: boolean;
  metaKey?: boolean;
  altKey?: boolean;
  shiftKey?: boolean;
};

</storycode:copy>
<a id="f-fabcac2ae554-targets"></a>
### `targets`

The rows a verb acts on: the marks when the row is one of them, else
the row alone.

<storycode:copy id="f-fabcac2ae554-targets">
/** The rows a verb acts on: the marks when the row is one of them, else
 * the row alone. */
export function targets(path: string, marked: ReadonlySet<string>): string[] {
  return marked.has(path) ? [...marked] : [path];
}

</storycode:copy>
<a id="f-fabcac2ae554-containing"></a>
### `containing`

The directory a new entry goes in: the row when it is a directory, else
the row's parent (`""` is the root).

<storycode:copy id="f-fabcac2ae554-containing">
/** The directory a new entry goes in: the row when it is a directory, else
 * the row's parent (`""` is the root). */
export function containing(path: string, dir: boolean): string {
  if (dir) return path;
  const i = path.lastIndexOf("/");
  return i < 0 ? "" : path.slice(0, i);
}

</storycode:copy>
<a id="f-fabcac2ae554-diredIntent"></a>
### `diredIntent`

What a key on a focused row means, or null when it is not a dired key.
`path` is the focused row (`""` for the folder header), `dir` whether it
is a directory. The keys are the keymap's `tree.*` actions
(lib/keymap.ts), so a person can move them; Delete always deletes.

<storycode:copy id="f-fabcac2ae554-diredIntent">
/**
 * What a key on a focused row means, or null when it is not a dired key.
 * `path` is the focused row (`""` for the folder header), `dir` whether it
 * is a directory. The keys are the keymap's `tree.*` actions
 * (lib/keymap.ts), so a person can move them; Delete always deletes.
 */
export function diredIntent(
  event: DiredKey,
  path: string,
  dir: boolean,
  marked: ReadonlySet<string>,
): DiredIntent | null {
  const full = {
    key: event.key,
    ctrlKey: !!event.ctrlKey,
    metaKey: !!event.metaKey,
    altKey: !!event.altKey,
    shiftKey: !!event.shiftKey,
  };
  const is = (id: string) => isAction(full, id);
  const onRow = path !== "";
  const something = onRow || marked.size > 0;
  if (is("tree.mark")) return onRow ? { kind: "mark", path } : null;
  if (is("tree.unmark")) return onRow ? { kind: "unmark", path } : null;
  if (is("tree.unmarkAll")) return { kind: "unmark-all" };
  if (is("tree.delete") || (full.key === "Delete" && !full.ctrlKey && !full.metaKey && !full.altKey))
    return something ? { kind: "delete", paths: targets(path, marked) } : null;
  if (is("tree.rename")) return onRow ? { kind: "rename", paths: targets(path, marked) } : null;
  if (is("tree.copy")) return something ? { kind: "copy", paths: targets(path, marked) } : null;
  if (is("tree.move")) return something ? { kind: "move", paths: targets(path, marked) } : null;
  if (is("tree.newFolder")) return { kind: "mkdir", dir: containing(path, dir) };
  if (is("tree.newFile")) return { kind: "create", dir: containing(path, dir) };
  return null;
}

</storycode:copy>
<a id="f-fabcac2ae554-nextMarks"></a>
### `nextMarks`

The marks after a mark/unmark intent; other intents leave them alone.

<storycode:copy id="f-fabcac2ae554-nextMarks">
/** The marks after a mark/unmark intent; other intents leave them alone. */
export function nextMarks(marked: ReadonlySet<string>, intent: DiredIntent): ReadonlySet<string> {
  const next = new Set(marked);
  if (intent.kind === "mark") next.add(intent.path);
  else if (intent.kind === "unmark") next.delete(intent.path);
  else if (intent.kind === "unmark-all") next.clear();
  return next;
}

</storycode:copy>
<a id="f-fabcac2ae554-toggleMark"></a>
### `toggleMark`

<storycode:copy id="f-fabcac2ae554-toggleMark">
/** A mark toggled by Ctrl+click (Cmd on a Mac). */
export function toggleMark(marked: ReadonlySet<string>, path: string): ReadonlySet<string> {
  const next = new Set(marked);
  if (next.has(path)) next.delete(path);
  else next.add(path);
  return next;
}
</storycode:copy>

### Assemble `apps/web/src/shell/dired.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/dired.ts"><storycode:paste select="#f-fabcac2ae554-DiredIntent"/><storycode:paste select="#f-fabcac2ae554-DiredKey"/><storycode:paste select="#f-fabcac2ae554-targets"/><storycode:paste select="#f-fabcac2ae554-containing"/><storycode:paste select="#f-fabcac2ae554-diredIntent"/><storycode:paste select="#f-fabcac2ae554-nextMarks"/><storycode:paste select="#f-fabcac2ae554-toggleMark"/></storycode:file>

<a id="output-2d2304fdc7fc"></a>
## `apps/web/src/shell/fileMenuActions.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Filesystem Acts Keep Dired's Safety](contract.md).

<a id="f-2d2304fdc7fc-runFileMenuItem"></a>
### `runFileMenuItem`

File actions and their prompts are shared by Files rows and file tabs.

<storycode:copy id="f-2d2304fdc7fc-runFileMenuItem">
// File actions and their prompts are shared by Files rows and file tabs.
import { api } from "../api/client";
import { FILES_CHANGED_EVENT } from "./FolderTreePane";
import { TreePrompt } from "./TreePrompt";
import { fileRenameError, TreeRenameEditor } from "./TreeRenameEditor";
import type { Dired } from "./useDired";
import { copyText, type TreeMenuItem } from "./treeMenu";

export function runFileMenuItem(item: TreeMenuItem, dired: Dired, notice: (text: string) => void, openDocument: (id: string) => void): boolean {
  const action = item.action;
  const failed = (e: unknown) => notice(e instanceof Error ? e.message : String(e));
  switch (action.kind) {
    case "copy": void copyText(action.text).catch(failed); return true;
    case "reveal": void api.reveal(action.path).catch(failed); return true;
    case "open": void api.openExternal(action.path).catch(failed); return true;
    case "literate":
    case "ingest":
      void api.adopt(action.path, action.kind === "ingest" ? action.into : undefined).then(outcome => {
        window.dispatchEvent(new Event(FILES_CHANGED_EVENT));
        openDocument(outcome.doc_id);
      }, failed);
      return true;
    case "rename":
    case "move":
    case "delete": dired.run({ kind: action.kind, paths: action.paths }); return true;
    case "copy-to": dired.run({ kind: "copy", paths: action.paths }); return true;
    case "create":
    case "mkdir": dired.run({ kind: action.kind, dir: action.dir }); return true;
    default: return false;
  }
}

</storycode:copy>
<a id="f-2d2304fdc7fc-FileActionPrompts"></a>
### `FileActionPrompts`

<storycode:copy id="f-2d2304fdc7fc-FileActionPrompts">
export function FileActionPrompts({ dired, inlineRename = false }: { dired: Dired; inlineRename?: boolean }) {
  return <>
      {dired.prompt && (
        <TreePrompt
          owner={dired.prompt.id} subject={dired.uiSubject} value={dired.promptDraft} onValueChange={dired.setPromptDraft}
          label={dired.prompt.label}
          verb={dired.prompt.verb}
          initial={dired.prompt.initial}
          onSubmit={dired.answer}
          onCancel={dired.cancel}
          error={dired.promptError}
          busy={dired.busy}
        />
      )}
      {dired.renamePaths && (
        <TreeRenameEditor
          items={dired.renamePaths.map((path) => ({
            key: `filesystem:${encodeURIComponent(path)}`,
            context: path,
            value: path.split("/").pop() ?? path,
          }))}
          validate={fileRenameError}
          inlinePath={inlineRename && dired.renamePaths.length === 1 ? dired.renamePaths[0] : undefined}
          apply={async (item, value) => {
            await api.fileOp({ op: "rename", path: item.context, to: value });
            const slash = item.context.lastIndexOf("/");
            const to = slash < 0 ? value : `${item.context.slice(0, slash + 1)}${value}`;
            window.dispatchEvent(
              new CustomEvent(FILES_CHANGED_EVENT, { detail: { renames: [{ from: item.context, to }] } }),
            );
          }}
          onClose={dired.closeRename}
        />
      )}
  </>;
}
</storycode:copy>

### Assemble `apps/web/src/shell/fileMenuActions.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/fileMenuActions.tsx"><storycode:paste select="#f-2d2304fdc7fc-runFileMenuItem"/><storycode:paste select="#f-2d2304fdc7fc-FileActionPrompts"/></storycode:file>

<a id="output-b7b29d60436f"></a>
## `apps/web/src/shell/useDired.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Filesystem Acts Keep Dired's Safety](contract.md), [The Workspace Tree Is A Lens](../the-workspace-tree-is-a-lens/contract.md).

<a id="f-b7b29d60436f-DiredPrompt"></a>
### `DiredPrompt`

The pane's half of dired: the marks, the prompt a verb asks through, and
running the verb against the server. See shell/dired.ts for what the keys
mean and docs/guarantees/authoring/the-tree-is-a-dired.md.

<storycode:copy id="f-b7b29d60436f-DiredPrompt">
// The pane's half of dired: the marks, the prompt a verb asks through, and
// running the verb against the server. See shell/dired.ts for what the keys
// mean and docs/guarantees/authoring/the-tree-is-a-dired.md.
import { useCallback, useId, useRef, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiField, UiValue } from "../lib/uiActions";
import type { KeyboardEvent } from "react";

import { api } from "../api/client";
import { FILES_CHANGED_EVENT, FILES_DELETED_EVENT } from "./FolderTreePane";
import { diredIntent, nextMarks, toggleMark } from "./dired";
import type { DiredIntent } from "./dired";

export interface DiredPrompt {
  id?: string;
  label: string;
  verb: string;
  /** Absent for a confirmation. */
  initial?: string;
  run: (value: string | null) => Promise<void>;
}

</storycode:copy>
<a id="f-b7b29d60436f-Dired"></a>
### `Dired`

<storycode:copy id="f-b7b29d60436f-Dired">
export interface Dired {
  uiSubject?: string;
  promptDraft?: string;
  setPromptDraft?: (value: string) => void;
  marked: ReadonlySet<string>;
  toggle: (path: string) => void;
  replaceMarks: (paths: readonly string[]) => void;
  addMarks: (paths: readonly string[]) => void;
  clear: () => void;
  /** A key on a focused row. True when it was a dired key. */
  onKey: (event: KeyboardEvent, path: string, dir: boolean) => boolean;
  run: (intent: DiredIntent) => void;
  renamePaths: readonly string[] | null;
  closeRename: (changed?: boolean) => void;
  prompt: DiredPrompt | null;
  promptError: string | null;
  busy: boolean;
  answer: (value: string | null) => Promise<boolean>;
  cancel: () => void;
}

</storycode:copy>
<a id="f-b7b29d60436f-said"></a>
### `said`

<storycode:copy id="f-b7b29d60436f-said">
const said = (e: unknown) => (e instanceof Error ? e.message : String(e));
</storycode:copy>
<a id="f-b7b29d60436f-many"></a>
### `many`

<storycode:copy id="f-b7b29d60436f-many">
const many = (paths: string[]) => (paths.length === 1 ? paths[0] : `${paths.length} items`);

</storycode:copy>
<a id="f-b7b29d60436f-runPaths"></a>
### `runPaths`

<storycode:copy id="f-b7b29d60436f-runPaths">
async function runPaths(paths: readonly string[], run: (path: string) => Promise<unknown>) {
  const completed: string[] = [];
  try { for (const path of paths) { await run(path); completed.push(path); } }
  catch (cause) {
    if (completed.length) {
      window.dispatchEvent(new Event(FILES_CHANGED_EVENT));
      throw Error(`Completed ${completed.length} of ${paths.length} paths before failure: ${JSON.stringify(completed)}. No rollback occurred. ${said(cause)}`);
    }
    throw cause;
  }
}

</storycode:copy>
<a id="f-b7b29d60436f-useDired"></a>
### `useDired`

<storycode:copy id="f-b7b29d60436f-useDired">
export function useDired(onNotice: (text: string | null) => void): Dired {
  const instance = useId();
  const inFlight = useRef(false);
  const [promptDraft, updateDraft] = useState("");
  const draftOwner = useRef("");
  const setPromptDraft = useCallback((value: string) => { if (!inFlight.current) { draftOwner.current = value; updateDraft(value); } }, []);
  const [marked, setMarked] = useState<ReadonlySet<string>>(() => new Set());
  const [prompt, updatePrompt] = useState<DiredPrompt | null>(null);
  const promptOwner = useRef<DiredPrompt | null>(null);
  const setPrompt = useCallback((next: DiredPrompt | null) => { promptOwner.current = next; draftOwner.current = next?.initial ?? ""; updateDraft(draftOwner.current); updatePrompt(next); }, []);
  const [renamePaths, setRenamePaths] = useState<readonly string[] | null>(null);
  const [promptError, setPromptError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const done = useCallback(() => {
    setPrompt(null);
    setRenamePaths(null);
    setPromptError(null);
    setMarked(new Set());
    // Every mounted tree refetches on this — the same event a run fires.
    window.dispatchEvent(new Event(FILES_CHANGED_EVENT));
  }, []);

  const run = useCallback(
    (intent: DiredIntent) => {
      if (inFlight.current) { onNotice("Wait for the current filesystem operation before replacing its prompt"); return; }
      onNotice(null);
      setPromptError(null);
      switch (intent.kind) {
        case "mark":
        case "unmark":
        case "unmark-all":
          setMarked((current) => nextMarks(current, intent));
          return;
        case "rename": {
          setPrompt(null);
          setRenamePaths(intent.paths);
          return;
        }
        case "move":
        case "copy": {
          setRenamePaths(null);
          const verb = intent.kind === "move" ? "Move" : "Copy";
          setPrompt({
            id: crypto.randomUUID(),
            label: `${verb} ${many(intent.paths)} to folder (relative to the open folder; "." is the root)`,
            verb,
            initial: "",
            run: async (value) => {
              await runPaths(intent.paths, path => api.fileOp({ op: intent.kind, path, to: value ?? "." }));
            },
          });
          return;
        }
        case "delete":
          setRenamePaths(null);
          setPrompt({
            id: crypto.randomUUID(),
            label: `Delete ${many(intent.paths)}? There is no trash: this removes ${
              intent.paths.length === 1 ? "it" : "them"
            } from the disk.`,
            verb: "Delete",
            run: async () => {
              await runPaths(intent.paths, async path => {
                await api.fileOp({ op: "delete", path });
                // Announce only after this particular host operation succeeds:
                // a batch may have completed paths before a later failure.
                window.dispatchEvent(new CustomEvent(FILES_DELETED_EVENT, { detail: { paths: [path] } }));
              });
            },
          });
          return;
        case "mkdir":
        case "create": {
          setRenamePaths(null);
          const what = intent.kind === "mkdir" ? "folder" : "file";
          setPrompt({
            id: crypto.randomUUID(),
            label: `New ${what} in ${intent.dir || "the folder"}`,
            verb: "Create",
            initial: "",
            run: async (value) => {
              const path = intent.dir ? `${intent.dir}/${value ?? ""}` : (value ?? "");
              await api.fileOp({ op: intent.kind, path });
            },
          });
          return;
        }
      }
    },
    [onNotice],
  );

  const answer = useCallback(
    async (value: string | null) => {
      if (!prompt || busy || inFlight.current || promptOwner.current !== prompt) return false;
      if (value !== null) setPromptDraft(value);
      inFlight.current = true; setBusy(true);
      try { await prompt.run(value); done(); return true; }
      catch (e) { setPromptError(said(e)); return false; }
      finally { inFlight.current = false; setBusy(false); }
    },
    [prompt, done, busy],
  );

  const onKey = useCallback(
    (event: KeyboardEvent, path: string, dir: boolean) => {
      const intent = diredIntent(event, path, dir, marked);
      if (!intent) return false;
      event.preventDefault();
      event.stopPropagation();
      run(intent);
      return true;
    },
    [marked, run],
  );

  const controller: Dired = {
    uiSubject: `file-operations:${instance}`, promptDraft, setPromptDraft,
    marked,
    toggle: (path) => setMarked((current) => toggleMark(current, path)),
    replaceMarks: (paths) => setMarked(new Set(paths)),
    addMarks: (paths) =>
      setMarked((current) => {
        const next = new Set(current);
        for (const path of paths) next.add(path);
        return next;
      }),
    clear: () => setMarked(new Set()),
    onKey,
    run,
    renamePaths,
    closeRename: (changed = false) => {
      setRenamePaths(null);
      setMarked(new Set());
      if (changed) window.dispatchEvent(new Event(FILES_CHANGED_EVENT));
    },
    prompt,
    promptError,
    busy,
    answer,
    cancel: () => {
      if (inFlight.current) return;
      setPrompt(null);
      setPromptError(null);
    },
  };
  const parent = `file-operations:${instance}`;
  const revision = useUiRevision(() => [[...marked], prompt?.id, promptOwner.current?.id, draftOwner.current, prompt?.label, prompt?.initial, renamePaths, promptError, busy]);
  const commands: { key: string; command: UiCommand }[] = [];
  const text: UiField = { type: "string", maxLength: 8192 };
  const paths: UiField = { type: "array", maxItems: 128, items: text };
  function add(key: string, label: string, parameters: Record<string, UiField>, execute: UiCommand["execute"], enabled = !busy) {
    commands.push({ key: `${parent}/${key}`, command: { label, parent, subject: parent, parameters, prerequisites: ["Paths are relative to this workspace; existing host filesystem checks apply"], read: () => ({ revision: revision(), enabled: enabled && (key === "read" || !inFlight.current), reason: enabled && (key === "read" || !inFlight.current) ? undefined : "Wait for the current filesystem operation" }), execute } });
  }
  const doneUi = (value: UiValue = { result: "Observe current file-operation state" }) => ({ status: "completed" as const, value });
  add("read", "Read file-operation marks and prompt", {}, async () => doneUi({ marked: [...marked], rename_paths: renamePaths ? [...renamePaths] : null, prompt: prompt ? { label: prompt.label, verb: prompt.verb, initial: prompt.initial ?? null, draft: prompt.initial === undefined ? null : draftOwner.current, confirmation: prompt.initial === undefined } : null, error: promptError, busy }), true);
  add("marks", "Set marked files", { paths }, async args => { controller.replaceMarks(args.paths as string[]); return doneUi(); });
  for (const kind of ["copy", "move", "delete", "rename"] as const) {
    add(kind, `Prepare file ${kind}`, { paths }, async args => {
      const targets = args.paths as string[];
      if (!targets.length || targets.some(path => !path.trim())) return { status: "unavailable", reason: "Choose at least one workspace-relative file path" };
      run({ kind, paths: targets }); return doneUi({ dispatch: "prepared", operation: kind, paths: targets, result: "Read the actual prompt or rename editor before applying" });
    });
  }
  for (const kind of ["create", "mkdir"] as const) add(kind, `Prepare file ${kind}`, { directory: text }, async args => { run({ kind, dir: String(args.directory) }); return doneUi(); });
  if (prompt) {
    add("answer", `Submit file prompt: ${prompt.verb}`, prompt.initial === undefined ? { confirm: { type: "boolean" } } : { value: text }, async args => {
      if (prompt.initial === undefined && args.confirm !== true) return { status: "unavailable", reason: "Confirmation was not supplied; use Cancel to dismiss" };
      if (prompt.initial !== undefined && !String(args.value).trim()) return { status: "unavailable", reason: "Enter a nonempty name or destination" };
      return await answer(prompt.initial === undefined ? null : String(args.value)) ? doneUi({ operation: prompt.verb, result: "Host filesystem operation completed" }) : { status: "unavailable", reason: "Operation failed or was busy; earlier effects may have occurred. Read the retained prompt error before retrying" };
    });
    add("cancel", "Cancel file prompt", {}, async () => { controller.cancel(); return doneUi(); });
  }
  useUiCommands(commands);
  return controller;
}
</storycode:copy>

### Assemble `apps/web/src/shell/useDired.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/useDired.ts"><storycode:paste select="#f-b7b29d60436f-DiredPrompt"/><storycode:paste select="#f-b7b29d60436f-Dired"/><storycode:paste select="#f-b7b29d60436f-said"/><storycode:paste select="#f-b7b29d60436f-many"/><storycode:paste select="#f-b7b29d60436f-runPaths"/><storycode:paste select="#f-b7b29d60436f-useDired"/></storycode:file>

<a id="output-f95b6e0cdbfa"></a>
## `apps/web/src/views/useFileTabMenu.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Filesystem Acts Keep Dired's Safety](contract.md).

<a id="f-f95b6e0cdbfa-findFile"></a>
### `findFile`

<storycode:copy id="f-f95b6e0cdbfa-findFile">
import { useEffect, useId, useMemo, useState, type MouseEvent } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { FileNode } from "../api/types";
import { docPathOf, isLikelyBinaryPath, type FolderTree } from "../shell/FolderTreePane";
import { FileActionPrompts, runFileMenuItem } from "../shell/fileMenuActions";
import { TreeContextMenu } from "../shell/TreeContextMenu";
import { treeMenuItems } from "../shell/treeMenu";
import { useDired } from "../shell/useDired";
import type { Tab } from "../shell/layout";

function findFile(nodes: readonly FileNode[], path: string): FileNode | undefined {
  for (const node of nodes) {
    if (node.path === path) return node;
    const found = node.children && findFile(node.children, path);
    if (found) return found;
  }
}

</storycode:copy>
<a id="f-f95b6e0cdbfa-useFileTabMenu"></a>
### `useFileTabMenu`

<storycode:copy id="f-f95b6e0cdbfa-useFileTabMenu">
export function useFileTabMenu(roots: readonly FolderTree[], activeDocId: string | null | undefined, openable: ReadonlyMap<string, string>, openDocument: (id: string) => void, tabs: readonly Tab[] = []) {
  const ownerSignature = JSON.stringify([roots, activeDocId, [...openable], tabs]);
  const owner = useMemo(() => crypto.randomUUID(), [ownerSignature]);
  const [menu, setMenu] = useState<{ owner: string; x: number; y: number; path: string; folder: FolderTree; plainText: boolean } | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const dired = useDired(setNotice);
  useEffect(() => { if (menu && menu.owner !== owner) setMenu(null); }, [menu, owner]);
  const open = (tab: Tab, position: { x: number; y: number }) => {
    if (!["document", "generated", "file"].includes(tab.kind) || (tabs.length > 0 && !tabs.some(current => current.id === tab.id && current.kind === tab.kind && current.target === tab.target && current.docId === tab.docId))) return false;
    for (const folder of roots) {
      const path = tab.kind === "document" && tab.docId ? docPathOf(folder.tree, tab.docId) ?? tab.target : tab.target;
      const node = findFile(folder.tree, path);
      // Tabs can outlive a truncated listing. Their kind still identifies
      // document and generated files, which must never offer adoption.
      if (!node && roots.length !== 1) continue;
      setNotice(null);
      setMenu({ owner, ...position, path, folder,
        plainText: tab.kind === "file" && !node?.doc_id && !node?.generated_by && !openable.has(path) && !isLikelyBinaryPath(path) });
      return true;
    }
    return false;
  };
  const onTabContextMenu = (event: MouseEvent, tab: Tab) => {
    if (open(tab, { x: event.clientX, y: event.clientY })) { event.preventDefault(); event.stopPropagation(); }
  };
  const parent = `file-tab-menus:${useId()}`;
  const targets = tabs.filter(tab => ["document", "generated", "file"].includes(tab.kind));
  const revision = useUiRevision(() => [roots, activeDocId, [...openable], targets, menu]);
  useUiCommands([{ key: `${parent}:read`, command: { parent, subject: `${parent}:reading`, label: "Read file tab menu targets", parameters: { first: { type: "number", integer: true, minimum: 0, maximum: 100000000 }, count: { type: "number", integer: true, minimum: 1, maximum: 64 } }, prerequisites: [],
    read: () => ({ revision: revision(), enabled: true }), execute: async args => {
      const first = Number(args.first), count = Number(args.count);
      if (first > targets.length) return { status: "unavailable", reason: "Index exceeds current file tabs" };
      const value = { tabs: targets.slice(first, first + count).map(tab => ({ id: tab.id, kind: tab.kind, path: tab.target })), total: targets.length, next: first + count < targets.length ? first + count : null };
      if (new TextEncoder().encode(JSON.stringify(value)).length > 8192) return { status: "unavailable", reason: "Tab page exceeds 8192 UTF-8 bytes; request fewer tabs" };
      return { status: "completed", value };
    } } }, { key: `${parent}:open`, command: { parent, subject: parent, label: "Open file tab context menu", parameters: { tab: { type: "string", maxLength: 4096 } }, prerequisites: ["Current document/generated/plain-file tab in its owning folder"],
      read: () => ({ revision: revision(), enabled: targets.length > 0 }), execute: async args => {
        const matches = targets.filter(tab => tab.id === args.tab);
        if (matches.length !== 1 || !open(matches[0], { x: 16, y: 16 })) return { status: "unavailable", reason: "Tab closed, ambiguous or its owning folder is unavailable" };
        return { status: "completed", value: { menu_open_requested: true, source_changed: false } };
      } } }]);
  const activePath = menu && activeDocId ? docPathOf(menu.folder.tree, activeDocId) : null;
  return {
    onTabContextMenu,
    fileTabMenu: <>
      {menu && menu.owner === owner && <TreeContextMenu x={menu.x} y={menu.y} subject={menu.path}
        items={treeMenuItems({ rootPath: menu.folder.root_path, separator: menu.folder.separator, fileManager: menu.folder.file_manager }, menu.path, false, {
          plainText: menu.plainText,
          activeDoc: activePath ? { path: activePath, name: activePath.split("/").pop() ?? activePath } : undefined,
        })}
        onClose={() => setMenu(null)} onPick={item => {
          setMenu(null);
          runFileMenuItem(item, dired, setNotice, openDocument);
        }} />}
      {notice && <p role="alert">{notice}</p>}
      <FileActionPrompts dired={dired} />
    </>,
  };
}
</storycode:copy>

### Assemble `apps/web/src/views/useFileTabMenu.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/useFileTabMenu.tsx"><storycode:paste select="#f-f95b6e0cdbfa-findFile"/><storycode:paste select="#f-f95b6e0cdbfa-useFileTabMenu"/></storycode:file>

<a id="output-4808c613eac1"></a>
## `crates/hickory-cli/src/serve/files_ops.rs`

Related contracts: [Filesystem Acts Keep Dired's Safety](contract.md).

<a id="f-4808c613eac1-FileOp"></a>
### `FileOp`

The file operations a tree has in dired: rename, move, copy, delete, a
new file, a new folder. `POST /api/files/op`.
Each verb is one filesystem call on a path inside the open folder, with
the refusals said plainly: nothing outside the folder, nothing under
`.git`, never the folder itself, never over something that exists. There
is no trash — a delete is a delete, which is why the pane asks twice —
and no `git mv`: the Git pane shows what changed and stages it as itself.
See docs/guarantees/authoring/the-tree-is-a-dired.md.

<storycode:copy id="f-4808c613eac1-FileOp">
//! The file operations a tree has in dired: rename, move, copy, delete, a
//! new file, a new folder. `POST /api/files/op`.
//!
//! Each verb is one filesystem call on a path inside the open folder, with
//! the refusals said plainly: nothing outside the folder, nothing under
//! `.git`, never the folder itself, never over something that exists. There
//! is no trash — a delete is a delete, which is why the pane asks twice —
//! and no `git mv`: the Git pane shows what changed and stages it as itself.
//!
//! See docs/guarantees/authoring/the-tree-is-a-dired.md.

use std::path::{Component, Path, PathBuf};

use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use serde_json::{Value, json};

use super::LocalState;
use super::api::{ApiError, ApiResult};

#[derive(Deserialize)]
pub struct FileOp {
    pub op: String,
    /// Root-relative path the verb acts on.
    pub path: String,
    /// `rename`: the new name (or root-relative path); `move`/`copy`: the
    /// destination directory.
    #[serde(default)]
    pub to: Option<String>,
}

</storycode:copy>
<a id="f-4808c613eac1-resolve"></a>
### `resolve`

The absolute path `rel` names inside `root`, or why this surface will
not touch it.

<storycode:copy id="f-4808c613eac1-resolve">
/// The absolute path `rel` names inside `root`, or why this surface will
/// not touch it.
fn resolve(root: &Path, rel: &str) -> Result<PathBuf, ApiError> {
    let rel = rel.trim().trim_end_matches('/');
    let as_path = Path::new(rel);
    if rel.is_empty() {
        return Err(ApiError::bad_request(
            "path must name something inside this folder — the folder itself is not a target",
        ));
    }
    if as_path.is_absolute()
        || as_path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(ApiError::bad_request(format!(
            "path must be relative to the open folder, with no `..` — got {rel:?}"
        )));
    }
    if as_path.components().next() == Some(Component::Normal(".git".as_ref())) {
        return Err(ApiError::bad_request(format!(
            "{rel} is inside .git, which this app never edits by hand — the Git pane is the way in"
        )));
    }
    Ok(root.join(rel))
}

</storycode:copy>
<a id="f-4808c613eac1-vacant"></a>
### `vacant`

A destination that must not already exist.

<storycode:copy id="f-4808c613eac1-vacant">
/// A destination that must not already exist.
fn vacant(target: &Path, rel: &str) -> Result<(), ApiError> {
    if target.exists() {
        return Err(ApiError::conflict(format!(
            "{rel} already exists — nothing here overwrites; choose another name or delete it first"
        )));
    }
    Ok(())
}

</storycode:copy>
<a id="f-4808c613eac1-rel_of"></a>
### `rel_of`

<storycode:copy id="f-4808c613eac1-rel_of">
fn rel_of(root: &Path, abs: &Path) -> String {
    abs.strip_prefix(root)
        .unwrap_or(abs)
        .to_string_lossy()
        .replace('\\', "/")
}

</storycode:copy>
<a id="f-4808c613eac1-copy_tree"></a>
### `copy_tree`

<storycode:copy id="f-4808c613eac1-copy_tree">
fn copy_tree(from: &Path, to: &Path) -> std::io::Result<()> {
    if from.is_dir() {
        std::fs::create_dir_all(to)?;
        for entry in std::fs::read_dir(from)? {
            let entry = entry?;
            copy_tree(&entry.path(), &to.join(entry.file_name()))?;
        }
        Ok(())
    } else {
        std::fs::copy(from, to).map(|_| ())
    }
}

</storycode:copy>
<a id="f-4808c613eac1-file_op"></a>
### `file_op`

<storycode:copy id="f-4808c613eac1-file_op">
/// `POST /api/files/op` — one dired verb.
pub async fn file_op(
    State(state): State<LocalState>,
    Json(body): Json<FileOp>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let source = resolve(&root, &body.path)?;
    let op = body.op.as_str();
    let needs_source = matches!(op, "rename" | "move" | "copy" | "delete");
    if needs_source && !source.exists() {
        return Err(ApiError::not_found(format!(
            "{} is not in this folder any more — the tree may be behind; it refreshes on its own",
            body.path
        )));
    }
    let to = body.to.as_deref().map(str::trim).filter(|t| !t.is_empty());
    let outcome = match op {
        "rename" => {
            let name =
                to.ok_or_else(|| ApiError::bad_request("rename needs `to`: the new name"))?;
            // A bare name stays in the same directory; a path moves as well.
            let target_rel = if name.contains('/') {
                name.to_string()
            } else {
                let dir = body
                    .path
                    .trim_end_matches('/')
                    .rsplit_once('/')
                    .map(|(d, _)| d);
                match dir {
                    Some(dir) => format!("{dir}/{name}"),
                    None => name.to_string(),
                }
            };
            let target = resolve(&root, &target_rel)?;
            vacant(&target, &target_rel)?;
            std::fs::rename(&source, &target)
                .map_err(|e| ApiError::unprocessable(format!("rename failed: {e}")))?;
            json!({ "op": "rename", "from": body.path, "to": rel_of(&root, &target) })
        }
        "move" | "copy" => {
            let dir_rel = to.ok_or_else(|| {
                ApiError::bad_request(format!("{op} needs `to`: the destination directory"))
            })?;
            let dir = if dir_rel == "." {
                root.clone()
            } else {
                resolve(&root, dir_rel)?
            };
            if !dir.is_dir() {
                return Err(ApiError::bad_request(format!(
                    "{dir_rel} is not a directory in this folder"
                )));
            }
            let name = source
                .file_name()
                .ok_or_else(|| ApiError::bad_request("no file name"))?;
            let target = dir.join(name);
            let target_rel = rel_of(&root, &target);
            if source.is_dir() && target.starts_with(&source) {
                return Err(ApiError::bad_request(format!(
                    "{} cannot be {}d into itself",
                    body.path, op
                )));
            }
            vacant(&target, &target_rel)?;
            if op == "move" {
                std::fs::rename(&source, &target)
            } else {
                copy_tree(&source, &target)
            }
            .map_err(|e| ApiError::unprocessable(format!("{op} failed: {e}")))?;
            json!({ "op": op, "from": body.path, "to": target_rel })
        }
        "delete" => {
            if source.is_dir() {
                std::fs::remove_dir_all(&source)
            } else {
                std::fs::remove_file(&source)
            }
            .map_err(|e| ApiError::unprocessable(format!("delete failed: {e}")))?;
            json!({ "op": "delete", "path": body.path })
        }
        "mkdir" => {
            vacant(&source, &body.path)?;
            std::fs::create_dir_all(&source).map_err(|e| {
                ApiError::unprocessable(format!("could not create the folder: {e}"))
            })?;
            json!({ "op": "mkdir", "path": body.path })
        }
        "create" => {
            if body.path.ends_with(".md") {
                return Err(ApiError::bad_request(
                    "a `.md` file is a document — File → New Document makes one, with a room to edit it in",
                ));
            }
            vacant(&source, &body.path)?;
            if let Some(parent) = source.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    ApiError::unprocessable(format!("could not create the folder: {e}"))
                })?;
            }
            std::fs::write(&source, b"")
                .map_err(|e| ApiError::unprocessable(format!("could not create the file: {e}")))?;
            json!({ "op": "create", "path": body.path })
        }
        other => {
            return Err(ApiError::bad_request(format!(
                "unknown file operation {other:?} — one of rename, move, copy, delete, mkdir, create"
            )));
        }
    };
    Ok(Json(outcome))
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/files_ops.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/files_ops.rs"><storycode:paste select="#f-4808c613eac1-FileOp"/><storycode:paste select="#f-4808c613eac1-resolve"/><storycode:paste select="#f-4808c613eac1-vacant"/><storycode:paste select="#f-4808c613eac1-rel_of"/><storycode:paste select="#f-4808c613eac1-copy_tree"/><storycode:paste select="#f-4808c613eac1-file_op"/></storycode:file>

<a id="output-a981752cc448"></a>
## `crates/hickory-cli/tests/tree_file_ops.rs`

Related contracts: [Filesystem Acts Keep Dired's Safety](contract.md).

<a id="f-a981752cc448-ENV_LOCK"></a>
### `ENV_LOCK`

The tree's dired verbs over real HTTP: `POST /api/files/op`.
Protects docs/guarantees/authoring/the-tree-is-a-dired.md.

<storycode:copy id="f-a981752cc448-ENV_LOCK">
//! The tree's dired verbs over real HTTP: `POST /api/files/op`.
//!
//! Protects docs/guarantees/authoring/the-tree-is-a-dired.md.

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Arc;

static ENV_LOCK: std::sync::LazyLock<Arc<tokio::sync::Mutex<()>>> =
    std::sync::LazyLock::new(|| Arc::new(tokio::sync::Mutex::new(())));

</storycode:copy>
<a id="f-a981752cc448-Session"></a>
### `Session`

<storycode:copy id="f-a981752cc448-Session">
struct Session {
    base: String,
    root: PathBuf,
    _dir: tempfile::TempDir,
    _state: tempfile::TempDir,
    _env: tokio::sync::OwnedMutexGuard<()>,
}

</storycode:copy>
<a id="f-a981752cc448-start"></a>
### `start`

<storycode:copy id="f-a981752cc448-start">
async fn start() -> Session {
    let env = ENV_LOCK.clone().lock_owned().await;
    let dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("note.md"), "# note\n").unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/a.txt"), "a\n").unwrap();
    std::fs::write(root.join("src/b.txt"), "b\n").unwrap();
    std::fs::create_dir_all(root.join(".git")).unwrap();
    std::fs::write(root.join(".git/HEAD"), "ref: refs/heads/master\n").unwrap();
    unsafe { std::env::set_var("HICKORY_STATE_DIR", state_dir.path()) };
    let prepared = prepare(ServeOptions {
        target: root.clone(),
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .expect("session prepares");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    Session {
        base: format!("http://127.0.0.1:{port}"),
        root,
        _dir: dir,
        _state: state_dir,
        _env: env,
    }
}

</storycode:copy>
<a id="f-a981752cc448-op"></a>
### `op`

<storycode:copy id="f-a981752cc448-op">
async fn op(s: &Session, body: Value) -> (u16, Value) {
    let res = reqwest::Client::new()
        .post(format!("{}/api/files/op", s.base))
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = res.status().as_u16();
    (status, res.json().await.unwrap_or(Value::Null))
}

</storycode:copy>
<a id="f-a981752cc448-every_verb_does_one_thing_on_disk"></a>
### `every_verb_does_one_thing_on_disk`

<storycode:copy id="f-a981752cc448-every_verb_does_one_thing_on_disk">
#[tokio::test]
async fn every_verb_does_one_thing_on_disk() {
    let s = start().await;
    let (status, body) = op(
        &s,
        json!({"op": "rename", "path": "src/a.txt", "to": "c.txt"}),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["to"], "src/c.txt");
    assert!(s.root.join("src/c.txt").exists() && !s.root.join("src/a.txt").exists());

    let (status, _) = op(&s, json!({"op": "mkdir", "path": "docs"})).await;
    assert_eq!(status, 200);
    assert!(s.root.join("docs").is_dir());

    let (status, body) = op(&s, json!({"op": "move", "path": "src/c.txt", "to": "docs"})).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["to"], "docs/c.txt");
    assert!(s.root.join("docs/c.txt").exists());

    let (status, body) = op(&s, json!({"op": "copy", "path": "src", "to": "docs"})).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        std::fs::read_to_string(s.root.join("docs/src/b.txt")).unwrap(),
        "b\n"
    );
    assert!(
        s.root.join("src/b.txt").exists(),
        "a copy leaves the source"
    );

    let (status, body) = op(&s, json!({"op": "create", "path": "docs/new.txt"})).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        std::fs::read_to_string(s.root.join("docs/new.txt")).unwrap(),
        ""
    );

    let (status, _) = op(&s, json!({"op": "delete", "path": "docs/src"})).await;
    assert_eq!(status, 200);
    assert!(!s.root.join("docs/src").exists());
    let (status, _) = op(&s, json!({"op": "delete", "path": "docs/new.txt"})).await;
    assert_eq!(status, 200);
    assert!(!s.root.join("docs/new.txt").exists());
}

</storycode:copy>
<a id="f-a981752cc448-a_rename_to_an_indented_tree_destination_moves_a_root_file_on_disk"></a>
### `a_rename_to_an_indented_tree_destination_moves_a_root_file_on_disk`

<storycode:copy id="f-a981752cc448-a_rename_to_an_indented_tree_destination_moves_a_root_file_on_disk">
#[tokio::test]
async fn a_rename_to_an_indented_tree_destination_moves_a_root_file_on_disk() {
    let s = start().await;
    std::fs::write(s.root.join("test53.py"), "print('test')\n").unwrap();
    let (status, body) = op(&s, json!({"op": "mkdir", "path": "test 1"})).await;
    assert_eq!(status, 200, "{body}");

    // This is the operation the Files buffer derives from:
    // `test 1/` followed by `  test53.py`.
    let (status, body) = op(
        &s,
        json!({"op": "rename", "path": "test53.py", "to": "test 1/test53.py"}),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["to"], "test 1/test53.py");
    assert!(s.root.join("test 1/test53.py").is_file());
    assert!(!s.root.join("test53.py").exists());
}

</storycode:copy>
<a id="f-a981752cc448-the_refusals_are_said_plainly"></a>
### `the_refusals_are_said_plainly`

<storycode:copy id="f-a981752cc448-the_refusals_are_said_plainly">
#[tokio::test]
async fn the_refusals_are_said_plainly() {
    let s = start().await;
    let cases = [
        (json!({"op": "delete", "path": "../etc"}), 400, "no `..`"),
        (
            json!({"op": "delete", "path": ".git/HEAD"}),
            400,
            "inside .git",
        ),
        (
            json!({"op": "delete", "path": ""}),
            400,
            "the folder itself",
        ),
        (
            json!({"op": "rename", "path": "src/a.txt", "to": "b.txt"}),
            409,
            "already exists",
        ),
        (
            json!({"op": "move", "path": "src", "to": "src"}),
            400,
            "into itself",
        ),
        (json!({"op": "create", "path": "x.md"}), 400, "New Document"),
        (
            json!({"op": "delete", "path": "gone.txt"}),
            404,
            "not in this folder",
        ),
        (
            json!({"op": "shred", "path": "src/a.txt"}),
            400,
            "unknown file operation",
        ),
    ];
    for (body, want, says) in cases {
        let (status, answer) = op(&s, body.clone()).await;
        assert_eq!(status, want, "{body}: {answer}");
        assert!(
            answer["error"].as_str().unwrap_or("").contains(says),
            "{body}: expected {says:?} in {answer}"
        );
    }
    // Nothing above touched the disk.
    assert!(s.root.join("src/a.txt").exists() && s.root.join("src/b.txt").exists());
    assert!(s.root.join(".git/HEAD").exists());
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/tree_file_ops.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/tree_file_ops.rs"><storycode:paste select="#f-a981752cc448-ENV_LOCK"/><storycode:paste select="#f-a981752cc448-Session"/><storycode:paste select="#f-a981752cc448-start"/><storycode:paste select="#f-a981752cc448-op"/><storycode:paste select="#f-a981752cc448-every_verb_does_one_thing_on_disk"/><storycode:paste select="#f-a981752cc448-a_rename_to_an_indented_tree_destination_moves_a_root_file_on_disk"/><storycode:paste select="#f-a981752cc448-the_refusals_are_said_plainly"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/api/files.ts](contract.md#output-f8ba3159313f)
- [apps/web/src/shell/FilesystemTreeEditor.tsx](../../../mechanisms/apps/web/src/shell/FilesystemTreeEditor.tsx.md#output-d30e9ea77e15)
- [apps/web/src/shell/TreePrompt.tsx](../../../mechanisms/apps/web/src/shell/TreePrompt.tsx.md#output-5ba0915f937c)
- [apps/web/src/shell/dired.ts](contract.md#output-fabcac2ae554)
- [apps/web/src/shell/fileMenuActions.tsx](contract.md#output-2d2304fdc7fc)
- [apps/web/src/shell/treeMenu.ts](../../../mechanisms/apps/web/src/shell/treeMenu.ts.md#output-19605cc8cff0)
- [apps/web/src/shell/useDired.ts](contract.md#output-b7b29d60436f)
- [apps/web/src/views/useFileTabMenu.tsx](contract.md#output-f95b6e0cdbfa)
- [crates/hickory-cli/src/serve/files_ops.rs](contract.md#output-4808c613eac1)

### Tests and fixtures

- [apps/web/src/lib/filesystemTreeText.test.ts](../the-workspace-tree-edits-like-an-editor/contract.md#output-a743bd990aba)
- [apps/web/src/shell/FilesystemTreeEditor.test.tsx](../the-workspace-tree-edits-like-an-editor/contract.md#output-720e9d0bf0b4)
- [apps/web/src/shell/FolderTreePane.test.tsx](../../../mechanisms/apps/web/src/shell/FolderTreePane.tsx.md#output-508e32e299fd)
- [apps/web/src/shell/dired.test.ts](contract.md#output-0cb807eba045)
- [apps/web/src/shell/treeMenu.test.ts](../../../mechanisms/apps/web/src/shell/treeMenu.ts.md#output-f48c486ac3c1)
- [crates/hickory-cli/tests/tree_file_ops.rs](contract.md#output-a981752cc448)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
