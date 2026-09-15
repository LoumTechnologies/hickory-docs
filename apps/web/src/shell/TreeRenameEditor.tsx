// A semantic rename column for workspace-tree nodes.
//
// One line belongs to one selected node. The editor is CodeMirror rather than
// a stack of inputs so the same rectangular selection and multiple-cursor
// gestures used in a `.hick` editor work here. Applying still calls one
// capability per node: a filesystem and a future provider share no transaction,
// so every result is reported separately.

import { useEffect, useLayoutEffect, useRef, useState, type CSSProperties } from "react";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { EditorState } from "@codemirror/state";
import { EditorView, keymap, lineNumbers } from "@codemirror/view";

import { editorChrome } from "../editor/chrome";
import { multipleCursors } from "../editor/multiCursor";

export interface SemanticRenameItem {
  key: string;
  /** Context that does not change: a path today, a provider key later. */
  context: string;
  value: string;
}

export interface SemanticRenameOutcome {
  key: string;
  before: string;
  after: string;
  ok: boolean;
  skipped?: boolean;
  error?: string;
}

export function renameValues(text: string, count: number): { values?: string[]; error?: string } {
  const values = text.split("\n");
  if (values.length !== count) {
    return { error: `Keep exactly ${count} ${count === 1 ? "line" : "lines"} — one for each selected node.` };
  }
  const empty = values.findIndex((value) => value.trim() === "");
  if (empty !== -1) return { error: `Line ${empty + 1} needs a name.` };
  return { values };
}

export function fileRenameError(value: string): string | null {
  if (value.includes("/") || value.includes("\\")) {
    return "A rename is one name, not a path. Use Move to change folders.";
  }
  if (value === "." || value === "..") return `${value} is not a file name.`;
  return null;
}

const said = (error: unknown) => (error instanceof Error ? error.message : String(error));

export function TreeRenameEditor({
  items,
  apply,
  validate,
  onApplied,
  onClose,
  inlinePath,
}: {
  items: readonly SemanticRenameItem[];
  apply: (item: SemanticRenameItem, value: string) => Promise<void>;
  validate?: (value: string, item: SemanticRenameItem) => string | null;
  onApplied?: (changed: boolean) => void;
  onClose: (changed: boolean) => void;
  /** A single filesystem rename occupies the row it edits. */
  inlinePath?: string;
}) {
  const shell = useRef<HTMLElement>(null);
  const host = useRef<HTMLDivElement>(null);
  const submitRef = useRef<() => void>(() => undefined);
  const text = useRef(items.map((item) => item.value).join("\n"));
  const completed = useRef(new Map<string, SemanticRenameOutcome>());
  const [error, setError] = useState<string | null>(null);
  const [outcomes, setOutcomes] = useState<SemanticRenameOutcome[]>([]);
  const [busy, setBusy] = useState(false);
  const [changed, setChanged] = useState(false);
  const [inlineStyle, setInlineStyle] = useState<CSSProperties>();
  const compact = inlinePath !== undefined && items.length === 1;

  useLayoutEffect(() => {
    if (!compact || !shell.current) return;
    const root = shell.current.closest(".folder-tree__root");
    const place = () => {
      const row = [...(root?.querySelectorAll<HTMLElement>("[data-tree-path]") ?? [])]
        .find((candidate) => candidate.dataset.treePath === inlinePath);
      if (!row) return;
      const rect = row.getBoundingClientRect();
      setInlineStyle({ position: "fixed", left: rect.left, top: rect.top, width: rect.width });
    };
    place();
    window.addEventListener("resize", place);
    window.addEventListener("scroll", place, true);
    return () => {
      window.removeEventListener("resize", place);
      window.removeEventListener("scroll", place, true);
    };
  }, [compact, inlinePath]);

  useEffect(() => {
    if (!host.current) return;
    const view = new EditorView({
      parent: host.current,
      state: EditorState.create({
        doc: text.current,
        extensions: [
          editorChrome("code"),
          ...(compact ? [] : [lineNumbers()]),
          history(),
          multipleCursors(),
          keymap.of([
            ...(compact ? [{ key: "Enter", run: () => { submitRef.current(); return true; } }] : []),
            ...defaultKeymap,
            ...historyKeymap,
          ]),
          EditorView.updateListener.of((update) => {
            if (update.docChanged) text.current = update.state.doc.toString();
          }),
        ],
        selection: compact ? { anchor: 0, head: text.current.length } : undefined,
      }),
    });
    view.focus();
    return () => view.destroy();
  }, [compact]);

  const submit = async () => {
    setError(null);
    setOutcomes([]);
    const parsed = renameValues(text.current, items.length);
    if (!parsed.values) {
      setError(parsed.error ?? "The rename column is invalid.");
      return;
    }
    for (let index = 0; index < items.length; index += 1) {
      if (completed.current.has(items[index].key)) continue;
      const refusal = validate?.(parsed.values[index], items[index]);
      if (refusal) {
        setError(`Line ${index + 1}: ${refusal}`);
        return;
      }
    }

    setBusy(true);
    const completedBefore = new Set(completed.current.keys());
    const results = await Promise.all(
      items.map(async (item, index): Promise<SemanticRenameOutcome> => {
        const prior = completed.current.get(item.key);
        // A provider may have already changed this node's name. Retrying a
        // partial batch must never send its stale identity a second time.
        if (prior) return prior;
        const after = parsed.values![index];
        if (after === item.value) {
          return { key: item.key, before: item.value, after, ok: true, skipped: true };
        }
        try {
          await apply(item, after);
          return { key: item.key, before: item.value, after, ok: true };
        } catch (cause) {
          return { key: item.key, before: item.value, after, ok: false, error: said(cause) };
        }
      }),
    );
    for (const result of results) {
      if (result.ok && !result.skipped) completed.current.set(result.key, result);
    }
    setBusy(false);
    setOutcomes(results);
    const didChange = results.some(
      (result) => result.ok && !result.skipped && !completedBefore.has(result.key),
    );
    if (didChange) {
      setChanged(true);
      onApplied?.(true);
    }
    if (compact && results.every((result) => result.ok)) onClose(didChange || changed);
  };
  submitRef.current = () => { if (!busy) void submit(); };

  return (
    <section
      ref={shell}
      className={`tree-rename${compact ? " tree-rename--inline" : ""}`}
      style={inlineStyle}
      role="dialog"
      aria-label={`Rename ${items.length === 1 ? items[0].context : `${items.length} selected nodes`}`}
      onKeyDown={(event) => {
        if (event.key === "Escape" && !busy) {
          event.preventDefault();
          onClose(changed);
        }
      }}
    >
      {!compact && <p className="tree-rename__instruction">
        One line per selected node. Alt-drag selects a column; multiple cursors edit together.
        {completed.current.size > 0 && " Successful lines will not be sent twice."}
      </p>}
      {!compact && <ol className="tree-rename__contexts mono" aria-label="Selected nodes">
        {items.map((item) => <li key={item.key}>{item.context}</li>)}
      </ol>}
      <div ref={host} className="tree-rename__editor" aria-label="New names" />
      {error && <p className="tree-rename__error">{error}</p>}
      {outcomes.length > 0 && (
        <ul className="tree-rename__outcomes" aria-label="Rename results">
          {outcomes.map((outcome) => (
            <li key={outcome.key} className={outcome.ok ? "ok" : "failed"}>
              <span className="mono">{outcome.before} → {outcome.after}</span>
              {outcome.skipped ? " — unchanged" : outcome.ok ? " — renamed" : ` — ${outcome.error}`}
            </li>
          ))}
        </ul>
      )}
      <span className="tree-prompt__buttons">
        <button type="button" className="tree-prompt__verb" disabled={busy} onClick={() => void submit()}>
          {busy
            ? "Renaming…"
            : outcomes.some((outcome) => !outcome.ok)
              ? "Retry failed renames"
              : items.length === 1 ? "Rename" : `Apply ${items.length} renames`}
        </button>
        <button type="button" disabled={busy} onClick={() => onClose(changed)}>
          {outcomes.length > 0 ? "Done" : "Cancel"}
        </button>
      </span>
    </section>
  );
}
