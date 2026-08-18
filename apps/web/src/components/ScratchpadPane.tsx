import { useState } from "react";

import { api } from "../api/client";

/** Where the note landed, or why it did not. */
type Result =
  | { kind: "saved"; path: string }
  | { kind: "refused"; message: string }
  | null;

/**
 * The third way material gets into a notes folder, beside downloading a file
 * into the inbox and copying one there: type it here.
 *
 * This is not a file-ingest surface and deliberately does not look like one.
 * Text typed here is prose a named human wrote, in this app, right now — the
 * strongest material there is on the attributable axis of
 * `docs/specs/freeform/provenance-and-standing.md`. It is never wrapped as a
 * transcript, and nothing about it is summarized, because there is no
 * machine step between the person and the words.
 *
 * Saving is deliberately explicit rather than automatic. A scratchpad that
 * committed every keystroke to a note would turn a place to think into a place
 * that keeps a record, and people stop thinking in those.
 */
export function ScratchpadPane() {
  const [text, setText] = useState("");
  const [saving, setSaving] = useState(false);
  const [result, setResult] = useState<Result>(null);

  const empty = text.trim().length === 0;

  async function save() {
    if (empty || saving) return;
    setSaving(true);
    setResult(null);
    try {
      const saved = await api.saveScratchpad(text);
      setResult({ kind: "saved", path: saved.path });
      setText("");
    } catch (e) {
      setResult({
        kind: "refused",
        message: e instanceof Error ? e.message : String(e),
      });
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="scratchpad-pane">
      <label className="scratchpad-label" htmlFor="scratchpad-text">
        Scratchpad
      </label>
      <p className="scratchpad-hint">
        Type a note. Saving writes it into this folder as a document — the first
        line becomes its title.
      </p>
      <textarea
        id="scratchpad-text"
        className="scratchpad-text"
        value={text}
        placeholder={"# Standup\n\nWhat happened, and what it means."}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={(e) => {
          // Save without leaving the keyboard, the way every other editor does.
          if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
            e.preventDefault();
            void save();
          }
        }}
      />
      <div className="scratchpad-actions">
        <button
          type="button"
          className="scratchpad-save"
          disabled={empty || saving}
          onClick={() => void save()}
        >
          {saving ? "Saving…" : "Save as note"}
        </button>
        {result?.kind === "saved" && (
          <span className="scratchpad-saved">Saved to {result.path}</span>
        )}
      </div>
      {result?.kind === "refused" && (
        // The refusals this can hit are all about hick markup in prose, and
        // they carry a next step — so show the message rather than a generic
        // failure that leaves the text stranded.
        <p className="scratchpad-refused">{result.message}</p>
      )}
    </div>
  );
}
