// The generated-output editor, shared by the Output view and the right pane of
// Split. It is EDITABLE ON ARRIVAL: you type in woven output and the parent
// resolves the edit backwards through provenance into the source document
// (POST /outputs/edit, via `onLocalEdit`). There are no output rooms — the
// server refuses them, because generated files live on disk rather than in a
// CRDT — so the pane seeds itself from `file.content`.
//
// Because the buffer can diverge from the server's last-woven copy by a local
// edit, provenance offsets are mapped through every change since load
// (`changesRef`), which keeps the lineage highlight under your cursor honest
// instead of drifting a character per keystroke.

import { useCallback, useEffect, useRef, useState } from "react";
import { ChangeDesc, EditorState, StateEffect, StateField, RangeSetBuilder } from "@codemirror/state";
import type { Extension } from "@codemirror/state";
import { Decoration, EditorView, keymap, lineNumbers } from "@codemirror/view";
import type { DecorationSet } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { search, searchKeymap } from "@codemirror/search";
import type { OutputFile, Provenance } from "../api/types";
import { changeFlashField, syncAndFlash } from "../editor/changeFlash";
import { languageExtensions } from "../editor/languages";
import { forgetFocusedEditor, markFocusedEditor } from "../editor/activeEditor";
import { lineHighlightField } from "../editor/lineHighlight";
import { RightRail } from "../editor/RightRail";
import { wrapGutterMarkers } from "../editor/wrapGutter";
import { fencedCodeRanges, isMarkdownPath, markdownStyling } from "../editor/markdownStyling";
import { taskCheckboxes } from "../editor/taskList";
import { renderedMath } from "../editor/mathRender";
import { proseWrap } from "../editor/wrapColumn";
import { editorChrome } from "../editor/chrome";
import { completions } from "../lsp/completion";
import { api } from "../api/client";
import { blameGutter } from "../editor/blameGutter";
import { useBlame } from "../editor/useBlame";
import { claimReveal, onRevealLine } from "../lib/revealLine";
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
  className?: string;
  testId?: string;
  /** Extra CodeMirror extensions (LSP navigation, ribbon highlights, …). */
  extensions?: Extension[];
  /** Live EditorView on mount, null on teardown. */
  onViewReady?: (view: EditorView | null) => void;
  /** Provenance entries under the cursor/pointer, for a lineage readout. */
  onLineage?: (hits: ProvChar[]) => void;
  /**
   * Edits a person typed, reported for whoever knows how to resolve them
   * back into the document (GeneratedFileView's debounced POST
   * /outputs/edit). Absent means the buffer is read-only in effect: edits
   * stay in the pane and go nowhere.
   */
  onLocalEdit?: (next: string) => void;
}

export function OutputEditorPane({
  file,
  className = "output-editor",
  testId = "output-editor",
  extensions,
  onViewReady,
  onLineage,
  onLocalEdit,
}: OutputEditorPaneProps) {
  const hostRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  // The live view AS STATE, for the right rail: a ref never re-renders, and
  // the rail must mount its sync against the view that actually exists.
  const [railView, setRailView] = useState<EditorView | null>(null);
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
      // A changeset only covers positions up to its pre-change length;
      // mapping an offset past that would throw, so skip rather than crash.
      if (changes && (p.charFrom > changes.length || p.charTo > changes.length)) continue;
      const a = changes ? changes.mapPos(p.charFrom, 1) : p.charFrom;
      const b = changes ? changes.mapPos(p.charTo, -1) : p.charTo;
      if (b <= a) continue;
      if (from < b && to >= a) {
        hits.push(p);
        marks.push({
          from: a,
          to: b,
          // Three states, not two: derived-and-uneditable, yours, and
          // somebody else's that you may nonetheless edit. Ingested bytes
          // are editable like literal ones and must not READ like them.
          cls:
            p.origin.kind === "synthetic"
              ? "cm-prov-synthetic"
              : p.origin.kind === "ingested"
                ? "cm-prov-ingested"
                : "cm-prov-active",
        });
      }
    }
    onLineageRef.current?.(hits);
    viewRef.current?.dispatch({ effects: setHighlights.of(marks) });
  }, []);

  // The view is created ONCE per mounted pane (callers key the pane by file
  // path). Later `file` props — a re-weave arriving from the server — are
  // reconciled into the live buffer below instead of rebuilding the editor,
  // which would throw away the cursor and undo history mid-edit.
  const fileRef = useRef(file);
  fileRef.current = file;
  const syncedFileRef = useRef(file);

  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;
    changesRef.current = null;
    const initial = fileRef.current;
    syncedFileRef.current = initial;

    const view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: initial.content,
        extensions: [
          // The same chrome every editor wears; `code` is the look a
          // `hick:file` body has inside a document, so opening the file and
          // reading the block that writes it are not two different programs.
          editorChrome("code"),
          highlightField,
          // The fading mark on text a re-weave just changed.
          changeFlashField,
          // Line numbers everywhere, including generated files. Two reasons:
          // a line number is how a person says WHERE, and the gutter's width
          // is the channel the lineage ribbons are drawn through — without it
          // they are squeezed into the four pixels of the pane divider.
          // Before lineNumbers, which is what puts it to their LEFT:
          // CodeMirror lays gutters out in the order they are declared, and
          // the numbers stay against the text because they are the
          // coordinate everything else in this app refers to.
          blameGutter(),
          lineNumbers(),
          // Wrap marks on soft-wrapped continuation rows, in the number
          // gutter — the number renders once, the rest of the tall cell
          // says "still that line".
          wrapGutterMarkers(),
          // The hovered-ribbon line tint, shared with the right rail.
          lineHighlightField,
          ...languageExtensions(initial.language),
          // A generated .md file gets the document editor's Typora-style
          // markdown look (big headings, styled bold/em/code). Display-only
          // decorations — the buffer's text is untouched, and they compose
          // with the lineage highlights, search, and any `extensions`.
          ...(isMarkdownPath(initial.path) ? [markdownStyling(), taskCheckboxes(), renderedMath()] : []),
          history(),
          // In-buffer find (Mod-F), same shape as the document editor's:
          // panel on top, keymap first, shifted chord left to the shell.
          // Completions from this project's own text. The language server's
          // half is wired per document (see editor/DocumentEditor.tsx); a
          // plain file has no room yet, and one honest source beats none.
          completions({
            project: (prefix, around) =>
              api.complete(prefix, around).then((answer) => answer.suggestions),
          }),
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
          // Which buffer Print means. Not `markActiveEditor` — that one
          // answers "where does an Insert go?", and a hick element written
          // into a file this document generates would land in the woven
          // output, where it means nothing.
          EditorView.focusChangeEffect.of((_state, focusing) => {
            const live = viewRef.current;
            if (focusing && live) markFocusedEditor(live);
            return null;
          }),
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
    setRailView(view);
    onViewReady?.(view);

    const onMove = (ev: MouseEvent) => {
      const pos = view.posAtCoords({ x: ev.clientX, y: ev.clientY });
      if (pos !== null) updateLineage(pos, pos);
    };
    view.dom.addEventListener("mousemove", onMove);

    return () => {
      view.dom.removeEventListener("mousemove", onMove);
      onViewReady?.(null);
      forgetFocusedEditor(view);
      view.destroy();
      viewRef.current = null;
      setRailView(null);
    };
    // Mounted once; `extensions` and the file are captured at mount
    // deliberately — re-creating the editor on a parent render would throw
    // away the user's cursor and undo history mid-edit.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [updateLineage]);

  // A NEW file prop on a live view is the server's copy after a re-weave:
  // reconcile the buffer to it with minimal edits and flash what changed.
  // Content identical to the buffer (the round-trip of an edit typed right
  // here) dispatches nothing and flashes nothing — but the fresh provenance
  // still lands, and the change mapping resets to match it.
  useEffect(() => {
    const view = viewRef.current;
    if (!view || file === syncedFileRef.current) return;
    syncedFileRef.current = file;
    syncAndFlash(view, file.content);
    // Whether or not the text moved, the buffer now equals the server's
    // last-woven copy, which is exactly what `file.provenance` indexes.
    changesRef.current = null;
  }, [file]);


  // Who last touched each line, when the column is on.
  useBlame(railView, file.path);

  // A find hit asked for this file at a line. Claimed on mount as well as on
  // the event, because the request is usually made before this pane exists.
  useEffect(() => {
    const jump = () => {
      const view = viewRef.current;
      if (!view) return;
      const line = claimReveal(file.path);
      if (line === null) return;
      const target = view.state.doc.line(Math.min(line, view.state.doc.lines));
      view.dispatch({
        selection: { anchor: target.from },
        // `center`, not `nearest`: a hit that lands on the last visible row
        // is technically shown and practically missed.
        effects: EditorView.scrollIntoView(target.from, { y: "center" }),
      });
      view.focus();
    };
    jump();
    return onRevealLine((asked) => {
      if (asked === file.path) jump();
    });
  });

  return (
    <div className="output-pane">
      {/* The wrapper is the pane's old bordered box; the editor and its
          right rail sit side by side inside it. The ribbon overlay finds the
          rail through `.with-right-rail` to anchor on its outer edge. */}
      <div className={`${className} with-right-rail`} data-testid={testId}>
        <div ref={hostRef} className="editor-cm-host" />
        <RightRail view={railView} />
      </div>
    </div>
  );
}
