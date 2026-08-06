import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { EditorState, StateEffect, StateField, RangeSetBuilder } from "@codemirror/state";
import { Decoration, EditorView } from "@codemirror/view";
import type { DecorationSet } from "@codemirror/view";
import { keymap } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { api, ApiError } from "../api/client";
import type {
  OutputFile,
  OutputFileMeta,
  Provenance,
  SourceEdit,
  SyntheticRangeError,
} from "../api/types";
import { languageExtensions } from "../editor/languages";
import { computeEdits, toByteEdits } from "../lib/diff";
import { byteToChar } from "../lib/offsets";

// ---------------------------------------------------------------------------
// Lineage decorations: [char range in the buffer] → highlight class.
// ---------------------------------------------------------------------------

interface HighlightRange {
  from: number;
  to: number;
  cls: string;
}

const setHighlights = StateEffect.define<HighlightRange[]>();

const highlightField = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(deco, tr) {
    deco = deco.map(tr.changes);
    for (const e of tr.effects) {
      if (e.is(setHighlights)) {
        const builder = new RangeSetBuilder<Decoration>();
        for (const r of [...e.value].sort((a, b) => a.from - b.from)) {
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
interface ProvChar extends Provenance {
  charFrom: number;
  charTo: number;
}

function provToChars(file: OutputFile): ProvChar[] {
  return file.provenance.map((p) => ({
    ...p,
    charFrom: byteToChar(file.content, p.start),
    charTo: byteToChar(file.content, p.end),
  }));
}

function originLabel(p: Provenance): string {
  if (p.origin.kind === "synthetic") return "synthetic (weaver-generated, not editable)";
  return `from ${p.origin.doc_path} bytes ${p.origin.span[0]}..${p.origin.span[1]} (${p.origin.kind})`;
}

export interface OutputViewProps {
  docId: string;
  /** Called after a successful edit-back so the parent refreshes the doc. */
  onSourceEdited: (edits: SourceEdit[]) => void;
  /** Jump to a source span in the Document view (lineage click-through). */
  onSelectSpan: (span: [number, number]) => void;
}

/**
 * The Output view: tabs over the doc's generated files, read-only CodeMirror
 * with language highlighting, hover/selection lineage against provenance, and
 * an edit mode that maps buffer edits back to source-document edits via
 * POST /outputs/edit.
 */
export function OutputView({ docId, onSourceEdited, onSelectSpan }: OutputViewProps) {
  const [files, setFiles] = useState<OutputFileMeta[] | null>(null);
  const [activePath, setActivePath] = useState<string | null>(null);
  const [file, setFile] = useState<OutputFile | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [editing, setEditing] = useState(false);
  const [dirty, setDirty] = useState(false);
  const [saving, setSaving] = useState(false);
  const [activeProv, setActiveProv] = useState<ProvChar[]>([]);
  const [editSummary, setEditSummary] = useState<SourceEdit[] | null>(null);
  const [editError, setEditError] = useState<string | null>(null);

  const hostRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const provRef = useRef<ProvChar[]>([]);
  const editingRef = useRef(false);
  editingRef.current = editing;

  const prov = useMemo(() => (file ? provToChars(file) : []), [file]);
  provRef.current = prov;

  const loadFiles = useCallback(() => {
    api.outputs(docId).then(
      (r) => {
        setFiles(r.files);
        setActivePath((p) =>
          p && r.files.some((f) => f.path === p) ? p : (r.files[0]?.path ?? null),
        );
      },
      (e) => setError(e instanceof Error ? e.message : String(e)),
    );
  }, [docId]);

  useEffect(loadFiles, [loadFiles]);

  const loadFile = useCallback(() => {
    if (!activePath) return;
    api.outputFile(docId, activePath).then(
      (f) => {
        setFile(f);
        setError(null);
      },
      (e) => setError(e instanceof Error ? e.message : String(e)),
    );
  }, [docId, activePath]);

  useEffect(loadFile, [loadFile]);

  // Show the provenance range(s) the cursor/selection (or hover) is inside.
  const updateLineage = useCallback((from: number, to: number) => {
    if (editingRef.current) return;
    const hits = provRef.current.filter((p) => from < p.charTo && to >= p.charFrom && p.charTo > p.charFrom);
    setActiveProv(hits);
    viewRef.current?.dispatch({
      effects: setHighlights.of(
        hits.map((p) => ({
          from: p.charFrom,
          to: p.charTo,
          cls: p.origin.kind === "synthetic" ? "cm-prov-synthetic" : "cm-prov-active",
        })),
      ),
    });
  }, []);

  // (Re)build the CodeMirror instance when the file or mode changes.
  useEffect(() => {
    const host = hostRef.current;
    if (!host || !file) return;
    const view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: file.content,
        extensions: [
          highlightField,
          ...languageExtensions(file.language),
          EditorView.lineWrapping,
          ...(editing
            ? [
                history(),
                keymap.of([...defaultKeymap, ...historyKeymap, indentWithTab]),
                EditorView.updateListener.of((u) => {
                  if (u.docChanged) setDirty(u.state.doc.toString() !== file.content);
                }),
              ]
            : [
                EditorState.readOnly.of(true),
                EditorView.editable.of(false),
                EditorView.updateListener.of((u) => {
                  if (u.selectionSet) {
                    const sel = u.state.selection.main;
                    updateLineage(sel.from, sel.to);
                  }
                }),
              ]),
        ],
      }),
    });
    viewRef.current = view;

    const onMove = (ev: MouseEvent) => {
      if (editingRef.current) return;
      const pos = view.posAtCoords({ x: ev.clientX, y: ev.clientY });
      if (pos !== null) updateLineage(pos, pos);
    };
    view.dom.addEventListener("mousemove", onMove);

    return () => {
      view.dom.removeEventListener("mousemove", onMove);
      view.destroy();
      viewRef.current = null;
    };
  }, [file, editing, updateLineage]);

  const startEdit = () => {
    setEditing(true);
    setDirty(false);
    setEditSummary(null);
    setEditError(null);
    setActiveProv([]);
  };

  const cancelEdit = () => {
    setEditing(false);
    setDirty(false);
    setEditError(null);
  };

  const saveEdit = async () => {
    const view = viewRef.current;
    if (!view || !file || !activePath) return;
    const edited = view.state.doc.toString();
    const charEdits = computeEdits(file.content, edited);
    if (charEdits.length === 0) {
      setEditing(false);
      return;
    }
    setSaving(true);
    setEditError(null);
    try {
      const res = await api.editOutput(docId, activePath, toByteEdits(file.content, charEdits));
      setEditSummary(res.source_edits);
      setEditing(false);
      setDirty(false);
      // Refresh both views: this file (rewoven) and the parent document.
      loadFile();
      onSourceEdited(res.source_edits);
    } catch (e) {
      // Synthetic-range 422: message inline + offending range highlighted.
      if (e instanceof ApiError && e.status === 422) {
        const body = e.body as SyntheticRangeError | undefined;
        setEditError(e.message);
        if (body?.range && viewRef.current) {
          const from = byteToChar(file.content, body.range.start);
          const to = byteToChar(file.content, body.range.end);
          const max = viewRef.current.state.doc.length;
          viewRef.current.dispatch({
            effects: setHighlights.of([
              { from: Math.min(from, max), to: Math.min(to, max), cls: "cm-prov-error" },
            ]),
            selection: { anchor: Math.min(from, max) },
            scrollIntoView: true,
          });
        }
      } else {
        setEditError(e instanceof Error ? e.message : String(e));
      }
    } finally {
      setSaving(false);
    }
  };

  if (error) {
    return (
      <div className="output-view">
        <p className="error">{error}</p>
      </div>
    );
  }
  if (files === null) {
    return (
      <div className="output-view">
        <p className="muted">Loading outputs…</p>
      </div>
    );
  }
  if (files.length === 0) {
    return (
      <div className="output-view">
        <p className="muted">
          No generated outputs yet — this document has no woven files, or it has not
          completed a successful run.
        </p>
      </div>
    );
  }

  return (
    <div className="output-view">
      <div className="output-tabs" role="tablist">
        {files.map((f) => (
          <button
            key={f.path}
            role="tab"
            aria-selected={f.path === activePath}
            className={`output-tab mono${f.path === activePath ? " on" : ""}`}
            onClick={() => {
              setActivePath(f.path);
              setEditing(false);
              setDirty(false);
              setEditSummary(null);
              setEditError(null);
            }}
          >
            {f.path}
          </button>
        ))}
        <span className="output-tab-spacer" />
        {editing ? (
          <>
            <button className="btn" onClick={cancelEdit} disabled={saving}>
              Cancel
            </button>
            <button
              className="btn btn-primary"
              onClick={() => void saveEdit()}
              disabled={!dirty || saving}
            >
              {saving ? "Saving…" : "Save edits"}
            </button>
          </>
        ) : (
          <button className="btn" onClick={startEdit} disabled={!file}>
            Edit output
          </button>
        )}
      </div>

      {editError && (
        <div className="banner banner-fail" role="alert">
          {editError}
        </div>
      )}
      {editSummary && (
        <div className="banner banner-pass" role="status">
          <strong>Applied.</strong>{" "}
          {editSummary.length} source edit{editSummary.length === 1 ? "" : "s"}:{" "}
          {editSummary.map((se, i) => (
            <button
              key={i}
              className="btn btn-link mono"
              onClick={() => onSelectSpan(se.span)}
              title={se.text ? `→ ${se.text.slice(0, 80)}` : "(deletion)"}
            >
              {se.doc_path} {se.span[0]}..{se.span[1]}
            </button>
          ))}
        </div>
      )}

      <div className="output-body">
        <div ref={hostRef} className="output-editor" data-testid="output-editor" />
        {!editing && (
          <aside className="lineage-strip" data-testid="lineage-strip">
            <h3>Lineage</h3>
            {activeProv.length === 0 ? (
              <p className="muted">Hover or select output text to trace its origin.</p>
            ) : (
              <ul>
                {activeProv.map((p, i) => (
                  <li key={i} className={p.origin.kind === "synthetic" ? "prov-synthetic" : ""}>
                    {p.origin.kind === "synthetic" ? (
                      <span>{originLabel(p)}</span>
                    ) : (
                      <button className="btn btn-link" onClick={() => {
                        const o = p.origin;
                        if (o.kind !== "synthetic") onSelectSpan(o.span);
                      }}>
                        {originLabel(p)}
                      </button>
                    )}
                  </li>
                ))}
              </ul>
            )}
          </aside>
        )}
      </div>
    </div>
  );
}
