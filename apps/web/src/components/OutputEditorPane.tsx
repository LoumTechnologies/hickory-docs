// The generated-output editor, shared by the Output view and the right pane of
// Split. It is EDITABLE ON ARRIVAL — there is no mode to enter. You type in
// woven output and the edit is resolved backwards through provenance into the
// source document (POST /outputs/edit), so the next run reproduces exactly what
// you typed.
//
// Because the buffer can now diverge from the server's copy, provenance offsets
// are mapped through every change since load (`changesRef`), which keeps the
// lineage highlight under your cursor honest while you edit instead of drifting
// a character per keystroke.

import { useCallback, useEffect, useRef, useState } from "react";
import { ChangeDesc, EditorState, StateEffect, StateField, RangeSetBuilder } from "@codemirror/state";
import type { Extension } from "@codemirror/state";
import { Decoration, EditorView, keymap } from "@codemirror/view";
import type { DecorationSet } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { api, ApiError } from "../api/client";
import type { OutputFile, Provenance, SourceEdit, SyntheticRangeError } from "../api/types";
import { languageExtensions } from "../editor/languages";
import { computeEdits, toByteEdits } from "../lib/diff";
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
  docId: string;
  file: OutputFile;
  className?: string;
  testId?: string;
  /** Extra CodeMirror extensions (LSP navigation, ribbon highlights, …). */
  extensions?: Extension[];
  /** Live EditorView on mount, null on teardown. */
  onViewReady?: (view: EditorView | null) => void;
  /** Provenance entries under the cursor/pointer, for a lineage readout. */
  onLineage?: (hits: ProvChar[]) => void;
  /** Applied source edits, after a successful save. */
  onSaved: (edits: SourceEdit[]) => void;
  /** Dirty-state changes, so a parent toolbar can show its own affordance. */
  onDirty?: (dirty: boolean) => void;
}

export function OutputEditorPane({
  docId,
  file,
  className = "output-editor",
  testId = "output-editor",
  extensions,
  onViewReady,
  onLineage,
  onSaved,
  onDirty,
}: OutputEditorPaneProps) {
  const [dirty, setDirty] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [summary, setSummary] = useState<SourceEdit[] | null>(null);

  const hostRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const provRef = useRef<ProvChar[]>([]);
  // Every change since this buffer was loaded, so provenance offsets (which
  // index the server's copy) can be mapped onto the buffer as it is edited.
  const changesRef = useRef<ChangeDesc | null>(null);
  const saveRef = useRef<() => void>(() => undefined);
  const onLineageRef = useRef(onLineage);
  onLineageRef.current = onLineage;

  provRef.current = provToChars(file);

  const updateLineage = useCallback((from: number, to: number) => {
    const changes = changesRef.current;
    const hits: ProvChar[] = [];
    const marks: HighlightRange[] = [];
    for (const p of provRef.current) {
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

  const save = useCallback(async () => {
    const view = viewRef.current;
    if (!view || saving) return;
    const edited = view.state.doc.toString();
    const charEdits = computeEdits(file.content, edited);
    if (charEdits.length === 0) {
      setDirty(false);
      onDirty?.(false);
      return;
    }
    setSaving(true);
    setError(null);
    try {
      const res = await api.editOutput(docId, file.path, toByteEdits(file.content, charEdits));
      setSummary(res.source_edits);
      setDirty(false);
      onDirty?.(false);
      onSaved(res.source_edits);
    } catch (e) {
      if (e instanceof ApiError && e.status === 422) {
        const body = e.body as SyntheticRangeError | undefined;
        setError(e.message);
        const v = viewRef.current;
        if (body?.range && v) {
          const changes = changesRef.current;
          const raw = [
            byteToChar(file.content, body.range.start),
            byteToChar(file.content, body.range.end),
          ] as const;
          const max = v.state.doc.length;
          const from = Math.min(changes ? changes.mapPos(raw[0], 1) : raw[0], max);
          const to = Math.min(changes ? changes.mapPos(raw[1], -1) : raw[1], max);
          v.dispatch({
            effects: setHighlights.of([{ from, to, cls: "cm-prov-error" }]),
            selection: { anchor: from },
            scrollIntoView: true,
          });
        }
      } else {
        setError(e instanceof Error ? e.message : String(e));
      }
    } finally {
      setSaving(false);
    }
  }, [docId, file, saving, onSaved, onDirty]);
  saveRef.current = () => void save();

  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;
    changesRef.current = null;
    const view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: file.content,
        extensions: [
          highlightField,
          ...languageExtensions(file.language),
          history(),
          keymap.of([
            {
              key: "Mod-s",
              preventDefault: true,
              run: () => {
                saveRef.current();
                return true;
              },
            },
            ...defaultKeymap,
            ...historyKeymap,
            indentWithTab,
          ]),
          EditorView.lineWrapping,
          EditorView.updateListener.of((u) => {
            if (u.docChanged) {
              changesRef.current = changesRef.current
                ? changesRef.current.composeDesc(u.changes.desc)
                : u.changes.desc;
              const isDirty = u.state.doc.toString() !== file.content;
              setDirty(isDirty);
              onDirty?.(isDirty);
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
      view.dom.removeEventListener("mousemove", onMove);
      onViewReady?.(null);
      view.destroy();
      viewRef.current = null;
    };
    // The buffer is rebuilt per file; `extensions` is captured once per file
    // deliberately — re-creating the editor on every parent render would throw
    // away the user's cursor and undo history mid-edit.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [file, updateLineage]);

  // Reset transient banners when a DIFFERENT file is opened — not when this
  // same file reloads. Saving reloads it, so keying on the object identity
  // wiped the "applied N source edits" confirmation the instant it appeared,
  // leaving no sign the edit had landed.
  useEffect(() => {
    setError(null);
    setSummary(null);
    setDirty(false);
  }, [file.path]);

  return (
    <div className="output-pane">
      <div className="output-pane-status" role="status">
        {error ? (
          <span className="pane-error">{error}</span>
        ) : dirty ? (
          <>
            <span className="pane-dirty">Edited — will be resolved back through provenance</span>
            <button className="btn btn-primary btn-sm" disabled={saving} onClick={() => void save()}>
              {saving ? "Applying…" : "Apply to document (⌘S)"}
            </button>
          </>
        ) : summary ? (
          <span className="pane-ok">
            Applied {summary.length} source edit{summary.length === 1 ? "" : "s"} to{" "}
            {[...new Set(summary.map((s) => s.doc_path))].join(", ")}
          </span>
        ) : (
          <span className="pane-hint">Edit freely — changes resolve back into the document.</span>
        )}
      </div>
      <div ref={hostRef} className={className} data-testid={testId} />
    </div>
  );
}
