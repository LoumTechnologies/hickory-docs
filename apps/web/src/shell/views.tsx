// What goes inside a pane.
//
// The shell knows nothing about documents or generated files; these are the
// pieces that do, kept small so that adding a kind of view is adding a file
// rather than editing the shell.
//
// See docs/specs/freeform/shell-layouts.md.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { Extension } from "@codemirror/state";
import { EditorView } from "@codemirror/view";

import { api } from "../api/client";
import { FILES_CHANGED_EVENT } from "./FolderTreePane";
import { DivergedBanner } from "../components/DivergedBanner";
import { MergeView } from "../components/MergeView";
import type { DivergedOutput } from "../api/types";
import type { OutputFile, SourceEdit } from "../api/types";
import { OutputEditorPane, provToChars, type ProvChar } from "../components/OutputEditorPane";
import { createOutputSaver } from "../lib/outputSave";
import { SamplePicker, selectedLines, type LineRange } from "./SamplePicker";

/** One generated file, live, with its provenance carried into the LSP. */
export function GeneratedFileView({
  docId,
  path,
  liveFile,
  makeOutputLsp,
  onSourceEdits,
  onReady,
}: {
  docId: string;
  path: string;
  /**
   * The session's freshest copy of this file, refetched after every run and
   * every up-loop `files_changed` event. When it arrives, the pane's buffer
   * is reconciled to it in place (and the change flashed) — unless the user
   * has edits of their own still in flight, whose round-trip will bring the
   * same text back moments later.
   */
  liveFile?: OutputFile | null;
  makeOutputLsp?: (provenance: ProvChar[]) => Extension[];
  /**
   * A save resolved into the document: the byte spans it rewrote, so the
   * owning document's editor can flash where the edit landed.
   */
  onSourceEdits?: (edits: SourceEdit[]) => void;
  /**
   * This pane's editor and file, once both exist — for the ribbon overlay,
   * which needs to measure the text on both sides of a relationship.
   */
  onReady?: (target: { view: EditorView; file: OutputFile } | null) => void;
}) {
  const [file, setFile] = useState<OutputFile | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);
  const onSourceEditsRef = useRef(onSourceEdits);
  onSourceEditsRef.current = onSourceEdits;

  useEffect(() => {
    let live = true;
    api.outputFile(docId, path).then(
      (loaded) => {
        if (live) {
          // The session's copy may have arrived first (liveFile effect);
          // never step back to an older fetch.
          setFile((current) => current ?? loaded);
          setError(null);
        }
      },
      (e) => live && setError(e instanceof Error ? e.message : String(e)),
    );
    return () => {
      live = false;
    };
  }, [docId, path]);

  // Which lines are selected, for the sample picker. Watched here rather
  // than asked for on click, because the offer has to APPEAR when there is
  // something to sample — a button that is always there, and usually
  // complains, teaches people to ignore it.
  const [range, setRange] = useState<LineRange | null>(null);
  const watchSelection = useMemo(
    () =>
      EditorView.updateListener.of((update) => {
        if (update.selectionSet || update.docChanged) {
          setRange(selectedLines(update.state));
        }
      }),
    [],
  );

  const extensions = useMemo(
    () => [
      ...(makeOutputLsp && file ? makeOutputLsp(provToChars(file)) : []),
      watchSelection,
    ],
    [makeOutputLsp, file, watchSelection],
  );

  // Edits land in the DOCUMENT, which is the whole point of a generated file
  // being editable: the text here is a working surface onto the prose that
  // produced it. Debounced, because a keystroke is not an edit — a pause is.
  //
  // The saver owns the baseline the diffs are computed against, and advances
  // it only on load and on a successful save — never per render. Holding the
  // baseline in this component re-set it to the first-load weave on every
  // re-render, so each save re-sent every earlier edit and the document
  // gained a duplicate of them all (see lib/outputSave.ts and its test).
  const saver = useMemo(
    () =>
      createOutputSaver({
        post: async (edits) => {
          const response = await api.editOutput(docId, path, edits);
          // The response says WHERE in the document the edit landed; the
          // session flashes those spans in the document editor, so the
          // resolution is visible instead of silent.
          onSourceEditsRef.current?.(response.source_edits);
          return response;
        },
        onError: setSaveError,
      }),
    [docId, path],
  );
  useEffect(() => () => saver.dispose(), [saver]);
  useEffect(() => {
    if (file) saver.load(file.content);
  }, [file, saver]);
  const save = useCallback((next: string) => saver.changed(next), [saver]);

  // A re-weave arrived (run, up-loop, an edit in the document pane): adopt
  // the fresh copy, which the pane reconciles into its live buffer with a
  // flash on what changed. NOT while this pane's own edits are still unsent
  // or in flight — the incoming content predates them, and adopting it would
  // wipe text the user just typed; their round-trip re-weave follows shortly
  // and is adopted then.
  useEffect(() => {
    if (!liveFile) return;
    setFile((current) => {
      if (!current) return liveFile;
      if (current === liveFile) return current;
      if (saver.hasPendingEdits()) return current;
      return liveFile;
    });
  }, [liveFile, saver]);

  // Axis 3: whether the disk holds what the document produces, and if not,
  // why and the versions a merge needs. Read on mount and whenever files
  // change, since the state begins and ends with a batch.
  const [diverged, setDiverged] = useState<DivergedOutput | null>(null);
  const [dismissed, setDismissed] = useState<string | null>(null);
  const [merging, setMerging] = useState(false);
  const [wayError, setWayError] = useState<string | null>(null);
  useEffect(() => {
    let live = true;
    const read = () =>
      void api.divergedOutputs().then(
        (answer) => live && setDiverged(answer.diverged[path] ?? null),
        () => {},
      );
    read();
    window.addEventListener(FILES_CHANGED_EVENT, read);
    return () => {
      live = false;
      window.removeEventListener(FILES_CHANGED_EVENT, read);
    };
  }, [path]);

  if (error) return <p className="error">{error}</p>;
  if (!file) return <p className="muted">Loading {path}…</p>;

  return (
    <div className="generated-view">
      {diverged && dismissed !== diverged.reason && !merging && (
        <DivergedBanner
          what={path}
          reason={diverged.reason}
          mine="what is on disk"
          theirs="what the document produces"
          onKeepMine={() => setDismissed(diverged.reason)}
          onTakeTheirs={
            diverged.kind === "held"
              ? () =>
                  void api.regenerateOutput(path).then(
                    () => setWayError(null),
                    (e) => setWayError(e instanceof Error ? e.message : String(e)),
                  )
              : undefined
          }
          takeTheirsHint={
            diverged.kind === "kept" ? "Run the document to have a version to take." : undefined
          }
          onMerge={diverged.kind === "held" ? () => setMerging(true) : undefined}
          error={wayError}
        />
      )}
      {diverged && merging && file && (
        <MergeView
          path={path}
          base={diverged.base}
          ours={file.content}
          theirs={diverged.theirs}
          oursLabel="What is on disk"
          theirsLabel="What the document produces"
          onAccept={(text) => {
            setMerging(false);
            void api.resolveOutput(path, text).then(
              () => setWayError(null),
              (e) => setWayError(e instanceof Error ? e.message : String(e)),
            );
          }}
          onCancel={() => setMerging(false)}
        />
      )}
      <OutputEditorPane
        key={path}
        file={file}
        extensions={extensions}
        onLocalEdit={save}
        onViewReady={(view) => onReady?.(view ? { view, file } : null)}
      />
      {range && <SamplePicker path={path} range={range} />}
      {saveError && (
        <p className="generated-view__error" role="alert">
          That edit could not be resolved into the document: {saveError}
        </p>
      )}
    </div>
  );
}

// The old OutputsTool (a per-document list of generated files) is gone: the
// folder tree pane (FolderTreePane.tsx) shows every file, generated included.
// The cursor-provenance footer is gone too: the ribbons, braces, and line
// tints say where text came from without a line of prose under the pane.
