<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/views.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-68a8fba10658"></a>
## `apps/web/src/shell/views.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Sample Shows Generated Lines Without Storing Them](../../../../../guarantees/authoring/a-sample-shows-generated-lines-without-storing-them/contract.md), [Conflict actions name the write](../../../../../guarantees/authoring/conflict-actions-name-the-write/contract.md), [Saved means the displayed bytes reached the file](../../../../../guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file/contract.md), [Close or recover a document without losing the wrong draft](../../../../../guarantees/authoring/unsaved-work-survives-closing-the-app/contract.md), [A failed action keeps its editor](../../../../../guarantees/execution/a-failed-action-keeps-its-editor/contract.md).

<a id="f-68a8fba10658-pendingOutputs"></a>
### `pendingOutputs`

<storycode:copy id="f-68a8fba10658-pendingOutputs">
import { useUiReading } from "../lib/uiReading";
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
</storycode:copy>
<a id="f-68a8fba10658-pendingWrites"></a>
### `pendingWrites`

<storycode:copy id="f-68a8fba10658-pendingWrites">
const pendingWrites = new Map<string, Promise<unknown>>();
</storycode:copy>
<a id="f-68a8fba10658-bufferOwners"></a>
### `bufferOwners`

<storycode:copy id="f-68a8fba10658-bufferOwners">
const bufferOwners = new Map<string, OutputSaver>();

</storycode:copy>
<a id="f-68a8fba10658-GeneratedFileView"></a>
### `GeneratedFileView`

One generated file, live, with its provenance carried into the LSP.

<storycode:copy id="f-68a8fba10658-GeneratedFileView">
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

  useUiReading("generated file load and save status", { doc_id: docId, path, loading: !file && !error, loaded: !!file, error, save_state: saveState, save_error: saveError, recovery_error: wayError, message: error || (!file ? `Loading ${path}…` : null), source_read: false });
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
</storycode:copy>

### Assemble `apps/web/src/shell/views.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/views.tsx"><storycode:paste select="#f-68a8fba10658-pendingOutputs"/><storycode:paste select="#f-68a8fba10658-pendingWrites"/><storycode:paste select="#f-68a8fba10658-bufferOwners"/><storycode:paste select="#f-68a8fba10658-GeneratedFileView"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-003396968e5c"></a>
## `apps/web/src/shell/views.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Saved means the displayed bytes reached the file](../../../../../guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file/contract.md), [The editor keeps its space](../../../../../guarantees/authoring/the-editor-keeps-its-space/contract.md).

<a id="f-003396968e5c-expression_statement"></a>
### `expression_statement`

@vitest-environment jsdom
The generated pane's half of the two-way edit flow:
- a fresh copy arriving from the session (a re-weave) is reconciled into
the live buffer and the changed text flashes;
- content identical to the buffer — the round-trip of what was typed right
here — flashes nothing;
- while the pane's own edits are unsent, an incoming stale copy is NOT
adopted (it predates the keystrokes and would eat them);
- a successful save reports the server's `source_edits`, which is what the
owning document's editor flashes.

<storycode:copy id="f-003396968e5c-expression_statement">
// @vitest-environment jsdom
//
// The generated pane's half of the two-way edit flow:
//  - a fresh copy arriving from the session (a re-weave) is reconciled into
//    the live buffer and the changed text flashes;
//  - content identical to the buffer — the round-trip of what was typed right
//    here — flashes nothing;
//  - while the pane's own edits are unsent, an incoming stale copy is NOT
//    adopted (it predates the keystrokes and would eat them);
//  - a successful save reports the server's `source_edits`, which is what the
//    owning document's editor flashes.

import { afterEach, describe, expect, it } from "vitest";
import { act, cleanup, render, waitFor } from "@testing-library/react";
import type { EditorView } from "@codemirror/view";
import { installMockHandler } from "../api/client";
import type { OutputFile, SourceEdit } from "../api/types";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { GeneratedFileView } from "./views";
import { viewBuffers } from "../lib/viewBuffers";

afterEach(async () => {
  cleanup();
  for (const id of viewBuffers.ids()) {
    await viewBuffers.get(id)?.discard();
    viewBuffers.forget(id);
  }
});

</storycode:copy>
<a id="f-003396968e5c-file"></a>
### `file`

<storycode:copy id="f-003396968e5c-file">
const file = (content: string): OutputFile => ({
  path: "src/hello.py",
  language: "python",
  content,
  provenance: [],
});

</storycode:copy>
<a id="f-003396968e5c-serve"></a>
### `serve`

Mock server: serves `initial`, applies output edits, records them.

<storycode:copy id="f-003396968e5c-serve">
/** Mock server: serves `initial`, applies output edits, records them. */
function serve(initial: string) {
  const state = { content: initial, editBodies: [] as unknown[] };
  installMockHandler(async (method, path, body) => {
    if (method === "GET" && path.includes("/outputs/file")) return file(state.content);
    if (method === "POST" && path.endsWith("/outputs/edit")) {
      state.editBodies.push(body);
      const edits: SourceEdit[] = [
        { doc_path: "hello.md", span: [10, 15], text: "typed" },
      ];
      return { source_edits: edits, applied: true };
    }
    throw new Error(`unexpected ${method} ${path}`);
  });
  return state;
}

</storycode:copy>
<a id="f-003396968e5c-mount"></a>
### `mount`

<storycode:copy id="f-003396968e5c-mount">
function mount(liveFile: OutputFile | null, onSourceEdits?: (e: SourceEdit[]) => void) {
  let view: EditorView | null = null;
  const utils = render(
    <GeneratedFileView
      docId="d1"
      path="src/hello.py"
      liveFile={liveFile}
      onSourceEdits={onSourceEdits}
      onReady={(target) => {
        view = target?.view ?? null;
      }}
    />,
  );
  return { ...utils, view: () => view };
}

</storycode:copy>
<a id="f-003396968e5c-rerenderWith"></a>
### `rerenderWith`

<storycode:copy id="f-003396968e5c-rerenderWith">
const rerenderWith = (
  utils: ReturnType<typeof mount>,
  liveFile: OutputFile | null,
  onSourceEdits?: (e: SourceEdit[]) => void,
) =>
  utils.rerender(
    <GeneratedFileView
      docId="d1"
      path="src/hello.py"
      liveFile={liveFile}
      onSourceEdits={onSourceEdits}
      onReady={() => undefined}
    />,
  );

</storycode:copy>
<a id="f-003396968e5c-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-003396968e5c-expression_statement-2">
describe("GeneratedFileView live updates", () => {
  // Guarantee: docs/guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file.md.
  it("retains unsent text across a tab switch and saves it through the workspace", async () => {
    const server = serve("x = 1\n");
    const first = mount(null);
    await waitFor(() => expect(first.view()).not.toBeNull());
    act(() => first.view()!.dispatch({ changes: { from: 4, to: 5, insert: "2" }, userEvent: "input" }));
    first.unmount();
    const second = mount(null);
    await waitFor(() => expect(second.view()?.state.doc.toString()).toBe("x = 2\n"));
    await act(async () => expect(await viewBuffers.get("d1:src/hello.py")!.save()).toBe(true));
    expect(server.editBodies).toHaveLength(1);
    expect(viewBuffers.get("d1:src/hello.py")?.dirty).toBe(false);
  });

  it("holds retained text when the generated file changed while the tab was inactive", async () => {
    const server = serve("x = 1\n");
    const first = mount(null);
    await waitFor(() => expect(first.view()).not.toBeNull());
    act(() => first.view()!.dispatch({ changes: { from: 4, to: 5, insert: "2" }, userEvent: "input" }));
    first.unmount();
    server.content = "x = 3\n";
    const second = mount(null);
    await waitFor(() => expect(second.view()?.state.doc.toString()).toBe("x = 2\n"));
    await act(async () => expect(await viewBuffers.get("d1:src/hello.py")!.save()).toBe(false));
    expect(server.editBodies).toHaveLength(0);
    expect(second.container.textContent).toContain("changed while this buffer was inactive");
    await act(async () => viewBuffers.get("d1:src/hello.py")!.discard());
    expect(second.view()?.state.doc.toString()).toBe("x = 3\n");
  });

  it("does not duplicate an in-flight reverse edit after remounting", async () => {
    let finish!: () => void;
    let writes = 0;
    let remote = "x = 1\n";
    installMockHandler(async (method, path) => {
      if (method === "GET" && path.includes("/outputs/file")) return file(remote);
      if (method === "POST" && path.endsWith("/outputs/edit")) {
        writes++;
        await new Promise<void>(resolve => { finish = resolve; });
        remote = "x = 2\n";
        return { source_edits: [], applied: true };
      }
      throw new Error(`unexpected ${method} ${path}`);
    });
    const first = mount(null);
    await waitFor(() => expect(first.view()).not.toBeNull());
    act(() => first.view()!.dispatch({ changes: { from: 4, to: 5, insert: "2" }, userEvent: "input" }));
    const saving = viewBuffers.get("d1:src/hello.py")!.save();
    await waitFor(() => expect(writes).toBe(1));
    first.unmount();
    const second = mount(null);
    await waitFor(() => expect(second.view()?.state.doc.toString()).toBe("x = 2\n"));
    await act(async () => expect(await viewBuffers.get("d1:src/hello.py")!.save()).toBe(false));
    await act(async () => { finish(); await saving; });
    expect(writes).toBe(1);
    expect(viewBuffers.get("d1:src/hello.py")?.dirty).toBe(true);
    await act(async () => viewBuffers.get("d1:src/hello.py")!.discard());
    expect(second.view()?.state.doc.toString()).toBe("x = 2\n");
  });

  it("reconciles an incoming re-weave into the buffer and flashes the change", async () => {
    serve("print('one')\n");
    const utils = mount(null);
    await waitFor(() =>
      expect(utils.container.querySelector(".cm-content")?.textContent).toContain("print('one')"),
    );

    rerenderWith(utils, file("print('two')\n"));
    await waitFor(() =>
      expect(utils.container.querySelector(".cm-content")?.textContent).toContain("print('two')"),
    );
    // The arrived text carries the flash.
    const flash = [...utils.container.querySelectorAll(".cm-change-flash")]
      .map((el) => el.textContent)
      .join("");
    expect(flash).toContain("two");
  });

  it("flashes nothing when the incoming content matches the buffer (own-edit round-trip)", async () => {
    serve("print('same')\n");
    const utils = mount(null);
    await waitFor(() =>
      expect(utils.container.querySelector(".cm-content")?.textContent).toContain("print('same')"),
    );

    rerenderWith(utils, file("print('same')\n"));
    expect(utils.container.querySelector(".cm-change-flash")).toBeNull();
  });

  it("keeps unsent keystrokes when a stale copy arrives, then reports the save's source edits", async () => {
    serve("print('base')\n");
    const sourceEdits: SourceEdit[][] = [];
    const onSourceEdits = (e: SourceEdit[]) => sourceEdits.push(e);
    const utils = mount(null, onSourceEdits);
    await waitFor(() => expect(utils.view()).not.toBeNull());
    const view = utils.view()!;

    // The user types; the save is debounced, so for a moment the edit exists
    // only in this buffer.
    view.dispatch({
      changes: { from: 7, to: 11, insert: "typed" },
      userEvent: "input",
    });
    expect(view.state.doc.toString()).toBe("print('typed')\n");

    // A stale re-weave (woven before the keystrokes) must not be adopted.
    rerenderWith(utils, file("print('base')\n"), onSourceEdits);
    expect(view.state.doc.toString()).toBe("print('typed')\n");
    expect(utils.container.querySelector(".cm-change-flash")).toBeNull();

    // The debounced save lands and the server answers with where in the
    // document the edit resolved — the hook the document-editor flash hangs on.
    await waitFor(() => expect(sourceEdits).toHaveLength(1), { timeout: 3000 });
    expect(sourceEdits[0]).toEqual([{ doc_path: "hello.md", span: [10, 15], text: "typed" }]);
  }, 10000);
});

</storycode:copy>
<a id="f-003396968e5c-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-003396968e5c-expression_statement-3">
describe("removed chrome stays removed", () => {
  it("renders no explanatory hint and no provenance footer", async () => {
    serve("x = 1\n");
    const utils = mount(null);
    await waitFor(() =>
      expect(utils.container.querySelector(".cm-content")?.textContent).toContain("x = 1"),
    );
    expect(utils.container.textContent).not.toContain("Edit freely");
    expect(utils.container.querySelector(".generated-view__lineage")).toBeNull();
    expect(utils.container.querySelector(".output-pane-status")).toBeNull();
  });
});

</storycode:copy>
<a id="f-003396968e5c-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-003396968e5c-expression_statement-4">
// Guarantee: docs/guarantees/authoring/the-editor-keeps-its-space.md
it("reads the actual generated-file loading/failure path before its editor mounts", async () => {
  let fail!: (cause: Error) => void;
  installMockHandler(async (method, path) => {
    if (method === "GET" && path.includes("/outputs/file")) return new Promise((_resolve, reject) => { fail = reject; });
    if (method === "GET" && path.endsWith("/diverged")) return { diverged: {} };
    throw new Error(`Unexpected ${method} ${path}`);
  });
  const registry = new UiActions({ workspace: "p", window: "generated-loading" }, () => true, async () => {});
  const mounted = render(<UiActionContext.Provider value={registry}><GeneratedFileView docId="load-owner" path="missing.py" bufferId="loading-status-only" /></UiActionContext.Provider>);
  let id = 0;
  const read = async () => {
    const action = registry.groups().groups.flatMap(group => registry.observe(group, 64).actions).find(action => action.label === "Read generated file load and save status bytes")!;
    let reply: any;
    await act(async () => { reply = await registry.act(registry.scope, { id: "acp:status", kind: "agent" }, { handle: action.handle, arguments: { offset: 0, limit: 8192 }, request_id: String(++id) }); });
    return JSON.parse(new TextDecoder().decode(new Uint8Array(reply.outcome.value.bytes)));
  };
  expect(await read()).toMatchObject({ loading: true, loaded: false, message: "Loading missing.py…", source_read: false });
  await act(async () => fail(new Error("Generated file is unavailable")));
  await waitFor(() => expect(mounted.getByText("Generated file is unavailable")).toBeTruthy());
  expect(await read()).toMatchObject({ loading: false, error: "Generated file is unavailable", source_read: false });
  expect(mounted.container.querySelector(".cm-editor")).toBeNull();
});
</storycode:copy>

### Assemble `apps/web/src/shell/views.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/views.test.tsx"><storycode:paste select="#f-003396968e5c-expression_statement"/><storycode:paste select="#f-003396968e5c-file"/><storycode:paste select="#f-003396968e5c-serve"/><storycode:paste select="#f-003396968e5c-mount"/><storycode:paste select="#f-003396968e5c-rerenderWith"/><storycode:paste select="#f-003396968e5c-expression_statement-2"/><storycode:paste select="#f-003396968e5c-expression_statement-3"/><storycode:paste select="#f-003396968e5c-expression_statement-4"/></storycode:file>


</storycode:doc>
