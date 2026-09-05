// The pane's half of dired: the marks, the prompt a verb asks through, and
// running the verb against the server. See shell/dired.ts for what the keys
// mean and docs/guarantees/authoring/the-tree-is-a-dired.md.
import { useCallback, useState } from "react";
import type { KeyboardEvent } from "react";

import { api } from "../api/client";
import { FILES_CHANGED_EVENT } from "./FolderTreePane";
import { diredIntent, nextMarks, toggleMark } from "./dired";
import type { DiredIntent } from "./dired";

export interface DiredPrompt {
  label: string;
  verb: string;
  /** Absent for a confirmation. */
  initial?: string;
  run: (value: string | null) => Promise<void>;
}

export interface Dired {
  marked: ReadonlySet<string>;
  toggle: (path: string) => void;
  clear: () => void;
  /** A key on a focused row. True when it was a dired key. */
  onKey: (event: KeyboardEvent, path: string, dir: boolean) => boolean;
  run: (intent: DiredIntent) => void;
  prompt: DiredPrompt | null;
  promptError: string | null;
  busy: boolean;
  answer: (value: string | null) => void;
  cancel: () => void;
}

const said = (e: unknown) => (e instanceof Error ? e.message : String(e));
const many = (paths: string[]) => (paths.length === 1 ? paths[0] : `${paths.length} items`);

export function useDired(onNotice: (text: string | null) => void): Dired {
  const [marked, setMarked] = useState<ReadonlySet<string>>(() => new Set());
  const [prompt, setPrompt] = useState<DiredPrompt | null>(null);
  const [promptError, setPromptError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const done = useCallback(() => {
    setPrompt(null);
    setPromptError(null);
    setMarked(new Set());
    // Every mounted tree refetches on this — the same event a run fires.
    window.dispatchEvent(new Event(FILES_CHANGED_EVENT));
  }, []);

  const run = useCallback(
    (intent: DiredIntent) => {
      onNotice(null);
      setPromptError(null);
      switch (intent.kind) {
        case "mark":
        case "unmark":
        case "unmark-all":
          setMarked((current) => nextMarks(current, intent));
          return;
        case "rename": {
          const name = intent.path.split("/").pop() ?? intent.path;
          setPrompt({
            label: `Rename ${intent.path}`,
            verb: "Rename",
            initial: name,
            run: async (value) => {
              await api.fileOp({ op: "rename", path: intent.path, to: value ?? "" });
            },
          });
          return;
        }
        case "move":
        case "copy": {
          const verb = intent.kind === "move" ? "Move" : "Copy";
          setPrompt({
            label: `${verb} ${many(intent.paths)} to folder (relative to the open folder; "." is the root)`,
            verb,
            initial: "",
            run: async (value) => {
              for (const path of intent.paths) {
                await api.fileOp({ op: intent.kind, path, to: value ?? "." });
              }
            },
          });
          return;
        }
        case "delete":
          setPrompt({
            label: `Delete ${many(intent.paths)}? There is no trash: this removes ${
              intent.paths.length === 1 ? "it" : "them"
            } from the disk.`,
            verb: "Delete",
            run: async () => {
              for (const path of intent.paths) await api.fileOp({ op: "delete", path });
            },
          });
          return;
        case "mkdir":
        case "create": {
          const what = intent.kind === "mkdir" ? "folder" : "file";
          setPrompt({
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
    (value: string | null) => {
      if (!prompt) return;
      setBusy(true);
      prompt.run(value).then(
        () => {
          setBusy(false);
          done();
        },
        (e: unknown) => {
          setBusy(false);
          setPromptError(said(e));
        },
      );
    },
    [prompt, done],
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

  return {
    marked,
    toggle: (path) => setMarked((current) => toggleMark(current, path)),
    clear: () => setMarked(new Set()),
    onKey,
    run,
    prompt,
    promptError,
    busy,
    answer,
    cancel: () => {
      setPrompt(null);
      setPromptError(null);
    },
  };
}
