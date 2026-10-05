import { useEffect, useMemo, useRef } from "react";
import { Annotation, EditorState } from "@codemirror/state";
import type { Extension } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import type * as Y from "yjs";
import type { Awareness } from "y-protocols/awareness";
import { yCollab } from "y-codemirror.next";
import { foldEffect } from "@codemirror/language";
import { EnvRegistry, wysiwyg } from "../editor/wysiwyg";
import { rangeHighlightField } from "../editor/rangeHighlight";
import { foldRangesOf, hickoryFolding } from "../editor/folding";
import { languageExtensions } from "../editor/languages";

export interface EditingSurfaceProps {
  /** Buffer contents. Ignored when `collab` is set — the CRDT owns the text. */
  value: string;
  onChange?: (next: string) => void;
  onSelection?: (selection: { anchor: number; head: number }) => void;
  /** Render `.md` source with the product's document decorations. */
  hick?: boolean;
  /**
   * Hick block names to collapse on arrival, e.g. `["file"]`.
   *
   * A demo pane is a fraction of the height of the real workspace, and every
   * fragment scrolled out of it turns its lineage ribbon into a faded stub
   * clamped to the pane edge. Folding the bodies the visitor does not need to
   * read — the generated file's scaffolding, which they can see rendered on
   * the right — is what puts the whole flow on screen at once. It is the
   * product's own fold, not a demo-only trick: the chevrons work.
   */
  foldBlocks?: string[];
  /** Highlight a generated file's language instead. */
  language?: string;
  readOnly?: boolean;
  /** Live EditorView on mount, null on teardown (ribbon measurement). */
  onViewReady?: (view: EditorView | null) => void;
  /** Bind this pane to a CRDT room shared with another pane. */
  collab?: { ytext: Y.Text; awareness: Awareness };
  /**
   * Bump to force the buffer back to `value` when `value` itself did not
   * change. A rejected edit (one that touched weaver-generated text) leaves
   * the buffer holding text the document never accepted; without this the
   * pane would keep showing it, because the prop it is meant to mirror is
   * byte-for-byte what it was before the keystroke.
   */
  syncToken?: number | string;
  /**
   * Extra CodeMirror extensions, e.g. the language-server bindings.
   *
   * Passed in rather than built here so the demo that wants intelligence gets
   * it and the demos that do not are unaffected — and so the extensions it
   * gets are the product's own, not a demo-shaped imitation of them.
   */
  extraExtensions?: Extension[];
  className?: string;
  ariaLabel: string;
  testId?: string;
}

export function EditingSurface({
  value,
  onChange,
  onSelection,
  hick,
  foldBlocks,
  language,
  readOnly,
  onViewReady,
  collab,
  syncToken,
  extraExtensions,
  className,
  ariaLabel,
  testId,
}: EditingSurfaceProps) {
  const hostRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  // Held in a ref so a changing callback never tears down the editor.
  const onChangeRef = useRef(onChange);
  onChangeRef.current = onChange;
  const selectionRef = useRef(onSelection);
  selectionRef.current = onSelection;
  const envRegistry = useMemo(() => new EnvRegistry(), []);

  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;

    const extensions: Extension[] = [
      history(),
      keymap.of([...defaultKeymap, ...historyKeymap]),
      rangeHighlightField,
      EditorView.lineWrapping,
      EditorView.editable.of(!readOnly),
      EditorState.readOnly.of(!!readOnly),
      EditorView.updateListener.of((u) => {
        if (u.docChanged && !u.transactions.some((tr) => tr.annotation(hostUpdate))) onChangeRef.current?.(u.state.doc.toString());
        if (u.selectionSet) selectionRef.current?.(u.state.selection.main);
      }),
    ];
    if (hick) extensions.push(wysiwyg(envRegistry), hickoryFolding());
    else if (language) extensions.push(...languageExtensions(language));
    if (collab) extensions.push(yCollab(collab.ytext, collab.awareness));
    if (extraExtensions?.length) extensions.push(...extraExtensions);

    const view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: collab ? collab.ytext.toString() : value,
        extensions,
      }),
    });
    viewRef.current = view;
    onViewReady?.(view);
    return () => {
      onViewReady?.(null);
      view.destroy();
      viewRef.current = null;
    };
    // The buffer's identity — not its contents — decides when to rebuild.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [hick, language, readOnly, collab, envRegistry, extraExtensions]);

  // Push an externally-driven value (a walkthrough step, or an edit mapped
  // back from the output pane) into the buffer. Under `collab` the CRDT is the
  // only writer; replacing the doc here would fight it and lose text.
  useEffect(() => {
    const view = viewRef.current;
    if (!view || collab) return;
    const current = view.state.doc.toString();
    if (current === value) return;
    const head = Math.min(view.state.selection.main.head, value.length);
    view.dispatch({
      annotations: hostUpdate.of(true),
      changes: { from: 0, to: current.length, insert: value },
      selection: { anchor: head },
    });
  }, [value, syncToken, collab]);

  // Fold the named blocks as they appear. Runs after the value sync above, so
  // a step that adds three file blocks folds all three.
  //
  // Each range is folded at most ONCE. Without that, unfolding a block to look
  // inside it would last exactly until the next keystroke re-folded it, and
  // the editor would be arguing with the person using it.
  const foldedOnceRef = useRef(new Set<string>());
  useEffect(() => {
    const view = viewRef.current;
    if (!view || !foldBlocks || foldBlocks.length === 0) return;
    const seen = foldedOnceRef.current;
    const effects = foldRangesOf(view.state)
      .filter((r) => r.kind === "block" && r.name && foldBlocks.includes(r.name))
      .filter((r) => !seen.has(`${r.from}:${r.to}`))
      .map((r) => {
        seen.add(`${r.from}:${r.to}`);
        return foldEffect.of({ from: r.from, to: r.to });
      });
    if (effects.length > 0) view.dispatch({ effects });
  }, [value, foldBlocks]);

  return (
    <div
      ref={hostRef}
      className={`demo-editor${hick ? " demo-editor-hick" : ""}${className ? ` ${className}` : ""}`}
      role="group"
      aria-label={ariaLabel}
      data-testid={testId}
    />
  );
}

const hostUpdate = Annotation.define<boolean>();
