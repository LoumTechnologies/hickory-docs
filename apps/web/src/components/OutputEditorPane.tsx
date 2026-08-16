// The generated-output editor, shared by the Output view and the right pane of
// Split. It is EDITABLE ON ARRIVAL and LIVE COLLABORATIVE — a Yjs CRDT room
// per (doc, output path), symmetric to the Document editor's own room
// (editor/DocumentEditor.tsx). You type in woven output and the server
// resolves the edit backwards through provenance into the source document on
// its own debounce (apps/server/src/output_rooms.rs) — there is no save
// button and no REST call from here; applying is as invisible to this
// component as the Document room's persist is to DocumentEditor.
//
// Because the buffer can diverge from the server's last-woven copy — by a
// local edit OR a remote collaborator's — provenance offsets are mapped
// through every change since load (`changesRef`), which keeps the lineage
// highlight under your cursor honest instead of drifting a character per
// keystroke, from either side.

import { useCallback, useEffect, useRef } from "react";
import { ChangeDesc, EditorState, StateEffect, StateField, RangeSetBuilder } from "@codemirror/state";
import type { Extension } from "@codemirror/state";
import { Decoration, EditorView, keymap, lineNumbers } from "@codemirror/view";
import type { DecorationSet } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import * as Y from "yjs";
import { Awareness } from "y-protocols/awareness";
import { yCollab } from "y-codemirror.next";
import type { Realtime } from "../api/realtime";
import type { OutputFile, Provenance } from "../api/types";
import { languageExtensions } from "../editor/languages";
import { byteToChar } from "../lib/offsets";

export interface HighlightRange {
  from: number;
  to: number;
  cls: string;
}

export const setHighlights = StateEffect.define<HighlightRange[]>();

export const highlightField = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(deco, tr) {
    deco = deco.map(tr.changes);
    for (const e of tr.effects) {
      if (e.is(setHighlights)) {
        const builder = new RangeSetBuilder<Decoration>();
        for (const r of [...e.value].sort((a, b) => a.from - b.from || a.to - b.to)) {
          if (r.to > r.from) builder.add(r.from, r.to, Decoration.mark({ class: r.cls }));
        }
        deco = builder.finish();
      }
    }
    return deco;
  },
  provide: (f) => EditorView.decorations.from(f),
});

/** Provenance entry with char (UTF-16) offsets alongside the wire bytes. */
export interface ProvChar extends Provenance {
  charFrom: number;
  charTo: number;
}

export function provToChars(file: OutputFile): ProvChar[] {
  return file.provenance.map((p) => ({
    ...p,
    charFrom: byteToChar(file.content, p.start),
    charTo: byteToChar(file.content, p.end),
  }));
}

export interface OutputEditorPaneProps {
  file: OutputFile;
  /** This file's own live room connection — a fresh channel per (doc,
   * path), scoped and owned by the parent (SplitView/OutputView) the same
   * way DocumentView owns the Document room's `realtime`. */
  realtime: Realtime;
  className?: string;
  testId?: string;
  /** Extra CodeMirror extensions (LSP navigation, ribbon highlights, …). */
  extensions?: Extension[];
  /** Live EditorView on mount, null on teardown. */
  onViewReady?: (view: EditorView | null) => void;
  /** Provenance entries under the cursor/pointer, for a lineage readout. */
  onLineage?: (hits: ProvChar[]) => void;
  /**
   * Edit this buffer without a live room.
   *
   * A local session has no output rooms — the server refuses them, because
   * generated files are on disk rather than in a CRDT — so the pane seeds
   * itself from `file.content` and reports edits here, for whoever knows how
   * to resolve them back into the document. Absent means the room is the
   * writer, which is the hosted arrangement.
   */
  onLocalEdit?: (next: string) => void;
}

export function OutputEditorPane({
  file,
  realtime,
  className = "output-editor",
  testId = "output-editor",
  extensions,
  onViewReady,
  onLineage,
  onLocalEdit,
}: OutputEditorPaneProps) {
  const hostRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const provRef = useRef<ProvChar[]>([]);
  // Every change since this buffer was loaded — local OR a remote
  // collaborator's, now that this is a live room — so provenance offsets
  // (which index the server's last-woven copy) can be mapped onto the
  // buffer as it changes.
  const changesRef = useRef<ChangeDesc | null>(null);
  const onLineageRef = useRef(onLineage);
  onLineageRef.current = onLineage;
  const onLocalEditRef = useRef(onLocalEdit);
  onLocalEditRef.current = onLocalEdit;

  provRef.current = provToChars(file);

  const updateLineage = useCallback((from: number, to: number) => {
    const changes = changesRef.current;
    const hits: ProvChar[] = [];
    const marks: HighlightRange[] = [];
    for (const p of provRef.current) {
      // A changeset only covers positions up to its pre-change length —
      // provenance offsets are indexed against `file.content`, but the very
      // first change this pane sees in mock mode is the programmatic seed
      // insert (empty -> file.content), whose changeset has pre-change
      // length 0. Mapping through it would throw; skip rather than crash.
      if (changes && (p.charFrom > changes.length || p.charTo > changes.length)) continue;
      const a = changes ? changes.mapPos(p.charFrom, 1) : p.charFrom;
      const b = changes ? changes.mapPos(p.charTo, -1) : p.charTo;
      if (b <= a) continue;
      if (from < b && to >= a) {
        hits.push(p);
        marks.push({
          from: a,
          to: b,
          cls: p.origin.kind === "synthetic" ? "cm-prov-synthetic" : "cm-prov-active",
        });
      }
    }
    onLineageRef.current?.(hits);
    viewRef.current?.dispatch({ effects: setHighlights.of(marks) });
  }, []);

  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;
    changesRef.current = null;

    const ydoc = new Y.Doc();
    const ytext = ydoc.getText("source");
    const awareness = new Awareness(ydoc);
    awareness.setLocalStateField("user", {
      name: localStorage.getItem("hickory.name") ?? "anonymous",
      color: "#8f6f3f",
    });
    realtime.bindDoc(ydoc, awareness);

    // Same reasoning as DocumentEditor: only a realtime with no server
    // behind it (mock mode) may seed — the server builds a fresh room from
    // the last-woven `run_outputs` row, and seeding on top of that doubles
    // the content on every connect.
    let cancelled = false;
    // Without a room, the file IS the content: seed it directly. This is the
    // local case and it is the common one — an empty editor over a file with
    // 187 bytes in it was what "the outputs are always empty" turned out to
    // be.
    if (onLocalEdit) {
      changesRef.current = null;
    } else if (!realtime.serverAuthoritative) {
      void realtime.whenSynced().then(() => {
        if (!cancelled && ytext.length === 0 && file.content.length > 0) {
          ytext.insert(0, file.content);
          // The seed insert reaches the editor through the same
          // updateListener as any other change (yCollab observes the
          // Y.Text and dispatches a CM transaction) and would otherwise be
          // folded into `changesRef` as if it were an edit made SINCE
          // `file.content` loaded — but it's what *produces* that exact
          // state, so provenance (already indexed against `file.content`)
          // needs to map through nothing here, not through an
          // empty-to-seeded changeset.
          changesRef.current = null;
        }
      });
    }

    const view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: onLocalEdit ? file.content : ytext.toString(),
        extensions: [
          highlightField,
          // Line numbers everywhere, including generated files. Two reasons:
          // a line number is how a person says WHERE, and the gutter's width
          // is the channel the lineage ribbons are drawn through — without it
          // they are squeezed into the four pixels of the pane divider.
          lineNumbers(),
          ...languageExtensions(file.language),
          history(),
          keymap.of([...defaultKeymap, ...historyKeymap, indentWithTab]),
          EditorView.lineWrapping,
          // The room is the writer only when there is one.
          ...(onLocalEdit ? [] : [yCollab(ytext, awareness)]),
          EditorView.updateListener.of((u) => {
            if (u.docChanged) {
              changesRef.current = changesRef.current
                ? changesRef.current.composeDesc(u.changes.desc)
                : u.changes.desc;
              // Only edits a person made: a programmatic reload is this
              // pane catching up with the document, and sending it back
              // would be an edit nobody typed.
              if (onLocalEditRef.current && u.transactions.some((t) => t.isUserEvent("input") || t.isUserEvent("delete") || t.isUserEvent("move") || t.isUserEvent("undo") || t.isUserEvent("redo"))) {
                onLocalEditRef.current(u.state.doc.toString());
              }
            }
            if (u.selectionSet || u.docChanged) {
              const sel = u.state.selection.main;
              updateLineage(sel.from, sel.to);
            }
          }),
          ...(extensions ?? []),
        ],
      }),
    });
    viewRef.current = view;
    onViewReady?.(view);

    const onMove = (ev: MouseEvent) => {
      const pos = view.posAtCoords({ x: ev.clientX, y: ev.clientY });
      if (pos !== null) updateLineage(pos, pos);
    };
    view.dom.addEventListener("mousemove", onMove);

    return () => {
      cancelled = true;
      view.dom.removeEventListener("mousemove", onMove);
      onViewReady?.(null);
      view.destroy();
      viewRef.current = null;
      awareness.destroy();
      ydoc.destroy();
      // `realtime` is owned by the parent (a fresh one per file, but the
      // parent decides its lifecycle — same split of responsibility as
      // DocumentEditor never closing the `realtime` prop it's handed).
    };
    // The buffer is rebuilt per file; `extensions` is captured once per file
    // deliberately — re-creating the editor on every parent render would throw
    // away the user's cursor, undo history, and live room mid-edit.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [file, realtime, updateLineage]);

  return (
    <div className="output-pane">
      <div className="output-pane-status" role="status">
        <span className="pane-hint">
          Edit freely — changes resolve back into the document automatically.
        </span>
      </div>
      <div ref={hostRef} className={className} data-testid={testId} />
    </div>
  );
}
