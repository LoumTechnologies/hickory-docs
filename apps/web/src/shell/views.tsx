import { GeneratedDebugTools } from "../debug/GeneratedDebugTools";
import type { DebugSession } from "../debug/useDebugger";
import { viewBuffers } from "../lib/viewBuffers";
import { GeneratedBufferRecovery } from "./GeneratedBufferRecovery";
import { SurfaceInfo } from "../components/SurfaceInfo";
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
import { createOutputSaver, type OutputSaver } from "../lib/outputSave";
import { SamplePicker, selectedLines, type LineRange } from "./SamplePicker";

const pendingOutputs = new Map<string, { contents: string; base: string }>();
const pendingWrites = new Map<string, Promise<unknown>>();
const bufferOwners = new Map<string, OutputSaver>();

/** One generated file, live, with its provenance carried into the LSP. */
export function GeneratedFileView({
  docId,
  path,
  bufferId = `${docId}:${path}`,
  liveFile,
  makeOutputLsp,
  onSourceEdits,
  onSource,
  onReady,
  debugger: debuggerContext,
}: {
  docId: string;
  path: string;
  debugger?: {debug:DebugSession;source:string;docPath:string;askText:(title:string,initial:string)=>Promise<string|null>};
  bufferId?: string;
  onSource?: (path: string, source: string) => void;
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
  const outputView = useRef<EditorView | null>(null);
  const [debugView,setDebugView] = useState<EditorView|null>(null);
  const [saveState, setSaveState] = useState<"saving" | "saved" | "failed" | null>(null);
  const pendingKey = `${docId}:${path}`;
  const [saveError, setSaveError] = useState<string | null>(null);
  const changedWhileAway = useRef(false);
  const onSourceRef = useRef(onSource);
  onSourceRef.current = onSource;
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
        if (update.docChanged) onSourceRef.current?.(path, update.state.doc.toString());
        if (update.selectionSet || update.docChanged) {
          setRange(selectedLines(update.state));
        }
      }),
    [path],
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
    () => {
      const saver = createOutputSaver({
        post: async (edits) => {
          // A tab may remount while its previous save is still settling.
          // Never send a second diff based on the same old bytes.
          if (pendingWrites.has(pendingKey)) throw new Error("An earlier save is still in progress. Your edits are held; reload the generated file after it finishes.");
          if (changedWhileAway.current) throw new Error("The generated file changed while this buffer was inactive. Reload it before editing again.");
          const pending = pendingOutputs.get(pendingKey);
          if (pending && pending.base !== saver.baselineContent()) throw new Error("An earlier save changed this buffer’s baseline. Reload the generated file before editing again.");
          const write = api.editOutput(docId, path, edits);
          pendingWrites.set(pendingKey, write);
          const response = await write.finally(() => {
            if (pendingWrites.get(pendingKey) === write) pendingWrites.delete(pendingKey);
          });
          // The response says WHERE in the document the edit landed; the
          // session flashes those spans in the document editor, so the
          // resolution is visible instead of silent.
          onSourceEditsRef.current?.(response.source_edits);
          return response;
        },
        onError: setSaveError,
        onState: state => {
          const pending = pendingOutputs.get(pendingKey);
          if (state === "saved" && pending?.contents === saver.baselineContent()) pendingOutputs.delete(pendingKey);
          else if (state === "saved" && pending) pending.base = saver.baselineContent();
          if (bufferOwners.get(bufferId) !== saver) return;
          setSaveState(state);
          viewBuffers.set(bufferId, { dirty: state !== "saved", revision: () => outputView.current?.state.doc.toString() ?? pendingOutputs.get(pendingKey)?.contents ?? saver.baselineContent(), isDirty: () => saver.hasPendingEdits(), save: () => saver.flushNow(), discard: async () => {
            const expected = outputView.current?.state.doc.toString() ?? pendingOutputs.get(pendingKey)?.contents ?? saver.baselineContent();
            const fresh = changedWhileAway.current ? await api.outputFile(docId, path) : null;
            const text = await saver.discardPending(expected);
            if (outputView.current && outputView.current.state.doc.toString() !== expected) throw Error("Generated source changed during discard; newer editor bytes were retained.");
            pendingOutputs.delete(pendingKey);
            const restored = fresh?.content ?? text;
            changedWhileAway.current = false;
            saver.load(restored);
            const view = outputView.current;
            if (view) view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: restored } });
          } });
        },
      });
      bufferOwners.set(bufferId, saver);
      return saver;
    },
    [docId, path, bufferId, pendingKey],
  );
  useEffect(() => () => saver.dispose(), [saver]);
  useEffect(() => {
    if (file) {
      const pending = pendingOutputs.get(pendingKey);
      changedWhileAway.current = pending !== undefined && (pending.base !== file.content || pendingWrites.has(pendingKey));
      saver.load(pending?.base ?? file.content);
      if (pending !== undefined) saver.changed(pending.contents);
    }
  }, [file, saver]);
  const save = useCallback((next: string) => {
    pendingOutputs.set(pendingKey, { contents: next, base: saver.baselineContent() });
    saver.changed(next);
  }, [saver, pendingKey]);

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
      <SurfaceInfo name={`Generated file · ${path}`} editable destination="Editable regions save automatically into the owning document; generated regions are read-only." />
      {saveState && <span className="save-state" role="status">{saveState === "saving" ? "Saving…" : saveState === "saved" ? "Saved to owning document" : "Unsaved changes"}</span>}
      {diverged && dismissed !== diverged.reason && !merging && (
        <DivergedBanner
          what={path}
          sourceRevision={() => [docId, file?.content, diverged, outputView.current?.state.doc.toString()]}
          reason={diverged.reason}
          mine="what is on disk"
          theirs="what the document produces"
          onKeepMine={() => setDismissed(diverged.reason)}
          keepLabel="Leave disk unchanged" keepEffect="Dismisses this warning; the document and disk still differ."
          takeLabel="Regenerate from document" takeEffect="Replaces disk contents with the document’s output."
          onTakeTheirs={
            diverged.kind === "held" && file
              ? () =>
                  api.regenerateOutput(path, file.content, diverged.theirs).then(
                    reply => { setWayError(null); return { ...reply, scope: "Actual output-file application reply; not source verification" }; },
                    (e) => { setWayError(e instanceof Error ? e.message : String(e)); throw e; },
                  )
              : undefined
          }
          takeTheirsHint={
            diverged.kind === "kept" ? "Run the document to have a version to take." : !file ? "Loading the disk version before regeneration." : undefined
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
          onAccept={async (text) => {
            try { const reply = await api.resolveOutput(path, text, file.content, diverged.theirs); setWayError(null); setMerging(false); return reply; }
            catch (e) { setWayError(e instanceof Error ? e.message : String(e)); throw e; }
          }}
          onCancel={() => setMerging(false)}
        />
      )}
      <OutputEditorPane
        key={path}
        file={file}
        extensions={extensions}
        onLocalEdit={save}
        onViewReady={(view) => {
        outputView.current = view;
        setDebugView(view);
        if (view && pendingOutputs.has(pendingKey)) {
          const pending = pendingOutputs.get(pendingKey)!.contents;
          view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: pending } });
        }
        if (view) onSourceRef.current?.(path, view.state.doc.toString());
        onReady?.(view ? { view, file } : null);
      }}
      />
      {debuggerContext && file && <GeneratedDebugTools view={debugView} file={file} {...debuggerContext} />}
      {range && <SamplePicker path={path} range={range} contents={outputView.current?.state.doc.toString() ?? file?.content ?? ""} />}
      {saveError && <GeneratedBufferRecovery bufferId={bufferId} saver={saver} error={saveError}
        isCurrent={() => bufferOwners.get(bufferId) === saver && !!outputView.current?.dom.isConnected}
        source={() => outputView.current?.state.doc.toString() ?? null} />}
    </div>
  );
}

// The old OutputsTool (a per-document list of generated files) is gone: the
// folder tree pane (FolderTreePane.tsx) shows every file, generated included.
// The cursor-provenance footer is gone too: the ribbons, braces, and line
// tints say where text came from without a line of prose under the pane.
