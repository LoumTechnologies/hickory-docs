// The one question a dired verb asks, in the pane: a name, a destination,
// or "really?". Not a browser dialog — a `prompt()` in a webview is a modal
// nobody styled, and a `confirm()` in the desktop shell blocks the event
// loop the terminals run on.
import { useEffect, useRef, useState } from "react";

export interface TreePromptProps {
  /** What is being asked, e.g. "Rename src/a.txt". */
  label: string;
  /** For a text question: the starting value. Absent means a confirmation. */
  initial?: string;
  /** The word on the button: "Rename", "Move", "Delete". */
  verb: string;
  /** What the person typed, or `null` for a confirmation. */
  onSubmit: (value: string | null) => void;
  onCancel: () => void;
  /** Said under the input while a previous try is refused. */
  error?: string | null;
  /** True while the verb is running. */
  busy?: boolean;
}

export function TreePrompt({ label, initial, verb, onSubmit, onCancel, error, busy }: TreePromptProps) {
  const [value, setValue] = useState(initial ?? "");
  const input = useRef<HTMLInputElement>(null);
  useEffect(() => {
    const el = input.current;
    if (!el) return;
    el.focus();
    // A rename starts with the stem selected, the way every file manager
    // does it: the extension is the part you almost never mean to change.
    const dot = (initial ?? "").lastIndexOf(".");
    el.setSelectionRange(0, dot > 0 ? dot : (initial ?? "").length);
  }, [initial]);
  const confirm = initial === undefined;
  return (
    <form
      className="tree-prompt"
      role="dialog"
      aria-label={label}
      onSubmit={(event) => {
        event.preventDefault();
        onSubmit(confirm ? null : value);
      }}
      onKeyDown={(event) => {
        if (event.key === "Escape") {
          event.preventDefault();
          onCancel();
        }
      }}
    >
      <span className="tree-prompt__label">{label}</span>
      {!confirm && (
        <input
          ref={input}
          className="tree-prompt__input mono"
          value={value}
          onChange={(event) => setValue(event.target.value)}
          disabled={busy}
          aria-label={label}
        />
      )}
      {error && <span className="tree-prompt__error">{error}</span>}
      <span className="tree-prompt__buttons">
        <button type="submit" className="tree-prompt__verb" disabled={busy || (!confirm && !value.trim())} autoFocus={confirm}>
          {busy ? `${verb}…` : verb}
        </button>
        <button type="button" onClick={onCancel} disabled={busy}>
          Cancel
        </button>
      </span>
    </form>
  );
}
