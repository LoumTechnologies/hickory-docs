// A plain file, in a pane: any text file in the folder that is neither a
// document nor a woven output. CodeMirror over `GET /api/file`, saved whole
// as you type through lib/plainFileSave.ts — debounced, serialized, and
// guarded by the content hash the load carried, so a file rewritten on disk
// underneath the buffer becomes a visible conflict instead of a silent
// overwrite.
//
// External edits (git, a formatter, an agent) are picked up when the window
// regains focus or the tree announces a change — the same cheap signals the
// folder pane refreshes on — and reconciled into the live buffer with a
// flash, never a rebuild, unless this pane's own edits are still unsent.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { EditorState } from "@codemirror/state";
import { EditorView, keymap, lineNumbers } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { search, searchKeymap } from "@codemirror/search";

import { api } from "../api/client";
import type { AdoptResponse, PlainFile } from "../api/types";
import { changeFlashField, syncAndFlash } from "../editor/changeFlash";
import { languageExtensions } from "../editor/languages";
import { isMarkdownPath, markdownStyling } from "../editor/markdownStyling";
import { taskCheckboxes } from "../editor/taskList";
import { renderedMath } from "../editor/mathRender";
import { wrapGutterMarkers } from "../editor/wrapGutter";
import { FILES_CHANGED_EVENT } from "../shell/FolderTreePane";
import { createPlainSaver, type PlainSaveState } from "../lib/plainFileSave";

export function PlainFilePane({
  path,
  onAdopted,
}: {
  path: string;
  /** Adoption succeeded: the file now has an owning document. The workspace
   * converts this very tab into a generated tab and opens the document. */
  onAdopted?: (adopted: AdoptResponse) => void;
}) {
  const hostRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const [file, setFile] = useState<PlainFile | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [saveState, setSaveState] = useState<PlainSaveState>({ kind: "idle" });
  const [adopting, setAdopting] = useState(false);
  const [adoptError, setAdoptError] = useState<string | null>(null);
  const onAdoptedRef = useRef(onAdopted);
  onAdoptedRef.current = onAdopted;

  const saver = useMemo(
    () =>
      createPlainSaver({
        put: (content, baseHash, force) => api.saveFile(path, content, baseHash, force),
        onState: setSaveState,
      }),
    [path],
  );
  useEffect(() => () => saver.dispose(), [saver]);

  // The initial read. The pane renders its refusals — binary, too large,
  // missing — as text where the editor would be: the tab is still an honest
  // place, it just has nothing to edit.
  useEffect(() => {
    let live = true;
    api.file(path).then(
      (loaded) => {
        if (!live) return;
        setFile(loaded);
        setLoadError(null);
        saver.load(loaded.content, loaded.hash);
      },
      (e) => live && setLoadError(e instanceof Error ? e.message : String(e)),
    );
    return () => {
      live = false;
    };
  }, [path, saver]);

  // The view is created ONCE, when the first load lands. Later content
  // arrives through syncAndFlash below — rebuilding the editor would throw
  // away the cursor and undo history mid-edit.
  const loaded = file !== null;
  const fileRef = useRef(file);
  fileRef.current = file;
  useEffect(() => {
    const host = hostRef.current;
    const initial = fileRef.current;
    if (!host || !initial) return;

    const view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: initial.content,
        extensions: [
          changeFlashField,
          lineNumbers(),
          wrapGutterMarkers(),
          ...languageExtensions(initial.language),
          ...(isMarkdownPath(initial.path) ? [markdownStyling(), taskCheckboxes(), renderedMath()] : []),
          history(),
          search({ top: true }),
          keymap.of([...searchKeymap, ...defaultKeymap, ...historyKeymap, indentWithTab]),
          EditorView.lineWrapping,
          EditorView.updateListener.of((u) => {
            // Only edits a person made: a programmatic reload is this pane
            // catching up with the disk, and saving it back would write
            // bytes nobody typed.
            if (
              u.docChanged &&
              u.transactions.some(
                (t) =>
                  t.isUserEvent("input") ||
                  t.isUserEvent("delete") ||
                  t.isUserEvent("move") ||
                  t.isUserEvent("undo") ||
                  t.isUserEvent("redo"),
              )
            ) {
              saver.changed(u.state.doc.toString());
            }
          }),
        ],
      }),
    });
    viewRef.current = view;
    return () => {
      view.destroy();
      viewRef.current = null;
    };
    // Mounted once per load; the saver is stable per path.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [loaded, saver]);

  // Take the disk copy into the live buffer, flashing what changed.
  const adoptDiskCopy = useCallback(
    (fresh: PlainFile) => {
      setFile(fresh);
      saver.load(fresh.content, fresh.hash);
      const view = viewRef.current;
      if (view && view.state.doc.toString() !== fresh.content) {
        syncAndFlash(view, fresh.content);
      }
    },
    [saver],
  );

  // External changes: refetch on window focus and on the files-changed
  // announcement — unless this pane's own edits are still unsent, in which
  // case the disk copy predates them and adopting it would eat keystrokes.
  // (Their save will 409 against the moved disk, which is the conflict
  // banner's moment, not this one.)
  useEffect(() => {
    const refresh = () => {
      if (!viewRef.current || saver.hasPendingEdits()) return;
      api.file(path).then(
        (fresh) => {
          if (fresh.hash !== fileRef.current?.hash) adoptDiskCopy(fresh);
        },
        () => {}, // deleted or unreadable now; the buffer stays as evidence
      );
    };
    window.addEventListener("focus", refresh);
    window.addEventListener(FILES_CHANGED_EVENT, refresh);
    return () => {
      window.removeEventListener("focus", refresh);
      window.removeEventListener(FILES_CHANGED_EVENT, refresh);
    };
  }, [path, saver, adoptDiskCopy]);

  // Adopt this file into a literate document. The server proves the new
  // document weaves these exact bytes before writing anything, so a success
  // changes ownership and not one byte of content; a refusal (binary,
  // hick:-looking text, a taken document name) lands in the banner.
  const adopt = useCallback(() => {
    setAdopting(true);
    setAdoptError(null);
    api.adopt(path).then(
      (adopted) => {
        setAdopting(false);
        onAdoptedRef.current?.(adopted);
      },
      (e) => {
        setAdopting(false);
        setAdoptError(e instanceof Error ? e.message : String(e));
      },
    );
  }, [path]);

  const overwrite = useCallback(() => saver.resolve("overwrite"), [saver]);
  const reload = useCallback(() => {
    saver.resolve("reload");
    api.file(path).then(adoptDiskCopy, (e) =>
      setSaveState({ kind: "error", message: e instanceof Error ? e.message : String(e) }),
    );
  }, [saver, path, adoptDiskCopy]);

  if (loadError) {
    return (
      <div className="plain-file-pane">
        <p className="error">{loadError}</p>
      </div>
    );
  }

  return (
    <div className="plain-file-pane">
      <div className="doc-tab-toolbar" role="toolbar" aria-label={`Actions for ${path}`}>
        <button
          className="btn"
          // Not while a save is in flight or parked on a conflict: the
          // server adopts the DISK bytes, and those must be this buffer's.
          disabled={adopting || !file || saveState.kind === "saving" || saveState.kind === "conflict"}
          onClick={adopt}
          data-tip="Wrap this file in a new literate document, byte-exactly — the file itself does not change, its ownership does"
        >
          {adopting ? "Adopting…" : "Make literate"}
        </button>
        {(saveState.kind === "saving" || saveState.kind === "saved") && (
          <span
            className={`save-state save-state-${saveState.kind === "saving" ? "editing" : "saved"}`}
            role="status"
          >
            {saveState.kind === "saving" ? "Saving…" : "Saved"}
          </span>
        )}
      </div>
      {adoptError && (
        <div className="banner banner-fail" role="alert">
          Could not adopt {path}: {adoptError}
        </div>
      )}
      {saveState.kind === "conflict" && (
        <div className="banner banner-fail" role="alert">
          This file changed on disk while you were editing — another program
          wrote it. Your text is still in this buffer, unsaved.{" "}
          <button type="button" className="btn" onClick={reload}>
            Reload from disk
          </button>{" "}
          <button type="button" className="btn" onClick={overwrite}>
            Overwrite with my version
          </button>
        </div>
      )}
      {saveState.kind === "error" && (
        <div className="banner banner-fail" role="alert">
          Could not save {path}: {saveState.message}
        </div>
      )}
      {!file && <p className="muted">Loading {path}…</p>}
      <div className="output-editor">
        <div ref={hostRef} className="editor-cm-host" />
      </div>
    </div>
  );
}
