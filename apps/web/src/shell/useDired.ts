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

const said = (e: unknown) => (e instanceof Error ? e.message : String(e));
const many = (paths: string[]) => (paths.length === 1 ? paths[0] : `${paths.length} items`);

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
