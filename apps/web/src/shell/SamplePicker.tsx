// Picking a sample: the gesture that puts a few lines of generated code under
// the cell that generated them.
//
// The offer appears only when lines are selected, because that is the moment
// it means something — and it says how many, because "sample these 8 lines"
// is a different promise from "sample this file". What lands in the document
// is one element naming a path and a range; the lines themselves are never
// copied in, so nothing here can drift out of date.
//
// See crates/hickory-cli/src/serve/sample.rs, and the `hick:sample` arm in
// crates/hick-literate/src/weave.rs.

import { useEffect, useState } from "react";
import type { EditorState } from "@codemirror/state";

import { api } from "../api/client";

export interface LineRange {
  from: number;
  to: number;
}

/**
 * The 1-based line range the selection covers, or `null` when there is no
 * selection to speak of.
 *
 * A caret is not a sample: a one-line range is offered only when the user
 * actually selected something, so simply clicking around in a generated file
 * never makes a button appear under the cursor.
 */
export function selectedLines(state: EditorState): LineRange | null {
  const sel = state.selection.main;
  if (sel.empty) return null;
  const from = state.doc.lineAt(sel.from).number;
  // A selection that ends at the very start of a line has not selected that
  // line; counting it would show one more line than was highlighted.
  const endPos = sel.to > sel.from && state.doc.lineAt(sel.to).from === sel.to ? sel.to - 1 : sel.to;
  return { from, to: state.doc.lineAt(endPos).number };
}

export function SamplePicker({ path, range }: { path: string; range: LineRange }) {
  const [caption, setCaption] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [landed, setLanded] = useState<string | null>(null);

  // A new selection is a new sample: whatever the last one said about itself
  // must not linger over it.
  useEffect(() => {
    setError(null);
    setLanded(null);
  }, [path, range.from, range.to]);

  const count = range.to - range.from + 1;

  const pick = async () => {
    setBusy(true);
    setError(null);
    try {
      const created = await api.createSample(path, range.from, range.to, caption.trim());
      setLanded(created.doc_path);
      setCaption("");
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="sample-picker" data-testid="sample-picker">
      <span className="sample-picker__range">
        {count === 1 ? "Line" : "Lines"} {range.from}
        {count > 1 && `–${range.to}`}
      </span>
      <input
        className="sample-picker__caption"
        value={caption}
        placeholder="What does this show? (optional)"
        onChange={(e) => setCaption(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter" && !busy) void pick();
        }}
        aria-label="Caption for this sample"
      />
      <button type="button" onClick={() => void pick()} disabled={busy}>
        {busy ? "Adding…" : "Show in document"}
      </button>
      {landed && (
        <span className="sample-picker__landed" role="status">
          Added to {landed}. It appears in the weave, not in the document.
        </span>
      )}
      {error && (
        <span className="sample-picker__error" role="alert">
          {error}
        </span>
      )}
    </div>
  );
}
