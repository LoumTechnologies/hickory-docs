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

import { useEffect, useId, useRef, useState } from "react";
import type { EditorState } from "@codemirror/state";

import { api } from "../api/client";
import { sha256 } from "../lib/reviews";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiOutcome } from "../lib/uiActions";

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

export function SamplePicker({ path, range, contents }: { path: string; range: LineRange; contents: string }) {
  const parent = `sample:${useId()}`;
  const selection = JSON.stringify([path, range.from, range.to, contents]);
  const current = useRef(selection); current.current = selection;
  const [caption, setCaption] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [landed, setLanded] = useState<string | null>(null);

  // A new selection is a new sample: whatever the last one said about itself
  // must not linger over it.
  useEffect(() => {
    setError(null);
    setLanded(null);
  }, [selection]);

  const count = range.to - range.from + 1;

  const pick = async (): Promise<UiOutcome> => {
    setBusy(true); setError(null);
    try {
      const hash = await sha256(contents);
      if (current.current !== selection) return { status: "unavailable", reason: "Selected output bytes changed before sample dispatch" };
      const created = await api.createSample(path, range.from, range.to, caption.trim(), hash);
      if (current.current === selection) { setLanded(created.doc_path); setCaption(""); }
      return { status: "completed", value: { doc_path: created.doc_path, output_hash: hash, from: range.from, to: range.to, selection_still_current: current.current === selection, execution_certified: false } };
    } catch (e) {
      const reason = e instanceof Error ? e.message : String(e);
      if (current.current === selection) setError(reason);
      return { status: "unavailable", reason };
    } finally { setBusy(false); }
  };
  const revision = useUiRevision(() => [selection, caption, busy, error, landed]);
  useUiReading("generated sample selection", { path, from: range.from, to: range.to, caption, busy, error, added_to: landed });
  useUiCommands([
    { key: `${parent}:caption`, command: { label: "Set caption for this sample", parent, subject: parent, parameters: { caption: { type: "string", maxLength: 8192 } }, prerequisites: [],
      read: () => ({ revision: revision(), enabled: !busy, value: caption }), execute: async args => { setCaption(args.caption as string); return { status: "completed", value: args.caption }; } } },
    { key: `${parent}:pick`, command: { label: "Show selected generated lines in document", parent, subject: parent, parameters: {}, prerequisites: [],
      read: () => ({ revision: revision(), enabled: !busy }), execute: pick } },
  ]);

  return (
    <div className="sample-picker" data-testid="sample-picker">
      <span className="sample-picker__range">
        {count === 1 ? "Line" : "Lines"} {range.from}
        {count > 1 && `–${range.to}`}
      </span>
      <input
        className="sample-picker__caption"
        value={caption}
        disabled={busy}
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
