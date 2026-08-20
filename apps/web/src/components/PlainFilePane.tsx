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
import { fencedCodeRanges, isMarkdownPath, markdownStyling } from "../editor/markdownStyling";
import { taskCheckboxes } from "../editor/taskList";
import { renderedMath } from "../editor/mathRender";
import { proseWrap } from "../editor/wrapColumn";
import { wrapGutterMarkers } from "../editor/wrapGutter";
import { FILES_CHANGED_EVENT } from "../shell/FolderTreePane";
import { createPlainSaver, type PlainSaveState } from "../lib/plainFileSave";
import { draftDisposition, useDraftKeeper } from "../lib/drafts";
import { MergeView } from "./MergeView";

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
  // A draft to put back into the buffer once the view exists. Held as state
  // rather than applied immediately because the load lands before the editor
  // is built.
  const [restored, setRestored] = useState<string | null>(null);
  // Two versions of this file that both have changes worth keeping. Set when
  // a save conflicts, and when a restored draft finds the file has moved on.
  const [merge, setMerge] = useState<{
    base: string;
    ours: string;
    theirs: string;
    oursLabel: string;
    theirsLabel: string;
  } | null>(null);
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
        // Was this buffer holding unsaved work when the app last closed?
        //
        // Three answers, and only one of them interrupts anybody. The file is
        // as we left it: put the text back, still unsaved, silently — that is
        // the common case by a wide margin, and a dialog here would train
        // people to dismiss dialogs. The file already says the same thing:
        // the draft is stale, drop it. The file moved on and so did we: that
        // is a merge, and it is worth someone's attention.
        void api.drafts().then(
          ({ drafts }) => {
            if (!live) return;
            const draft = drafts.find((d) => d.path === path);
            if (!draft) return;
            const next = draftDisposition(draft, loaded.content);
            if (next.kind === "clean") {
              void api.discardDraft(path).catch(() => {});
              return;
            }
            if (next.kind === "restore") {
              setRestored(next.contents);
              return;
            }
            setMerge({
              base: next.base,
              ours: next.ours,
              theirs: next.theirs,
              oursLabel: "Your unsaved changes",
              theirsLabel: "The file on disk",
            });
          },
          () => {
            // No draft store on this machine: the file opens as it is on
            // disk, which is what would have happened anyway.
          },
        );
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
          // Prose wraps at the measure; a fenced code block keeps its lines
          // and takes the whole pane. In a non-markdown file EVERY line is
          // code, which is exactly what `fencedCodeRanges` returning the whole
          // buffer expresses.
          proseWrap((state) =>
            isMarkdownPath(initial.path)
              ? fencedCodeRanges(state.doc.toString())
              : [[0, state.doc.length] as [number, number]],
          ),
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

  /** Put text into the buffer and treat it as an unsaved edit — which is
   * exactly what it is: restored work that the file does not have yet. */
  const putInBuffer = useCallback(
    (text: string) => {
      const view = viewRef.current;
      if (!view) return;
      if (view.state.doc.toString() !== text) syncAndFlash(view, text);
      saver.changed(text);
    },
    [saver],
  );

  // The restored draft goes in once the view exists — the load that found it
  // lands before the editor is built.
  useEffect(() => {
    if (restored === null || !loaded) return;
    putInBuffer(restored);
    setRestored(null);
  }, [restored, loaded, putInBuffer]);

  // Write the buffer down while it differs from the file. Read through a
  // callback so this costs nothing on the typing path — see lib/drafts.ts.
  useDraftKeeper({
    path,
    enabled: loaded,
    read: () => ({
      contents: viewRef.current?.state.doc.toString() ?? "",
      base: saver.baseContent(),
    }),
  });

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

  /** The third answer to a conflict, and the one that does not throw work
   * away: three-way merge the buffer against the disk, over the bytes this
   * session started from. */
  const openMerge = useCallback(() => {
    const view = viewRef.current;
    if (!view) return;
    const ours = view.state.doc.toString();
    const base = saver.baseContent();
    void api.file(path).then(
      (fresh) =>
        setMerge({
          base,
          ours,
          theirs: fresh.content,
          oursLabel: "Your unsaved changes",
          theirsLabel: "The file on disk",
        }),
      (e) => setSaveState({ kind: "error", message: e instanceof Error ? e.message : String(e) }),
    );
  }, [saver, path]);

  /** The merged text becomes the buffer, and the file is reloaded first so
   * the save that follows rides on the hash the disk actually has. */
  const acceptMerge = useCallback(
    (text: string) => {
      setMerge(null);
      saver.resolve("reload");
      void api.file(path).then(
        (fresh) => {
          adoptDiskCopy(fresh);
          putInBuffer(text);
        },
        (e) => setSaveState({ kind: "error", message: e instanceof Error ? e.message : String(e) }),
      );
    },
    [saver, path, adoptDiskCopy, putInBuffer],
  );
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
          </button>{" "}
          {/* The answer that throws nothing away. Reload loses this buffer;
              overwrite loses whatever the other program wrote. */}
          <button type="button" className="btn btn-primary" onClick={openMerge}>
            Merge…
          </button>
        </div>
      )}
      {saveState.kind === "error" && (
        <div className="banner banner-fail" role="alert">
          Could not save {path}: {saveState.message}
        </div>
      )}
      {!file && <p className="muted">Loading {path}…</p>}
      {/* The merge REPLACES the editor rather than floating over it: the two
          sides plus their context need the whole pane to be readable, and a
          modal over the buffer would hide the very text being merged. The
          buffer is untouched underneath until the merge is accepted. */}
      {merge ? (
        <MergeView
          path={path}
          base={merge.base}
          ours={merge.ours}
          theirs={merge.theirs}
          oursLabel={merge.oursLabel}
          theirsLabel={merge.theirsLabel}
          onAccept={acceptMerge}
          onCancel={() => setMerge(null)}
        />
      ) : null}
      <div className="output-editor" hidden={merge !== null}>
        <div ref={hostRef} className="editor-cm-host" />
      </div>
    </div>
  );
}
