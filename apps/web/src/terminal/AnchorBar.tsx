// What a terminal that is writing into a document says about itself.
//
// `docs/specs/freeform/a-terminal-that-writes-the-document.md` — **never
// anchor silently**: "a terminal that is writing into a document says which
// document and which container, always visibly. The difference between 'this
// disappears' and 'this is being committed' is the most important thing on
// the screen."
//
// So this is not a toast. It is a strip that is present for exactly as long
// as the terminal is anchored, and absent otherwise — a person can look at
// any pane and know which of the three bindings they are typing into without
// remembering what they clicked.

import type { TerminalAnchor } from "../api/types";

export function AnchorBar({
  anchor,
  docName,
  onResume,
  onUnanchor,
}: {
  /** Null when this terminal writes nothing — the ephemeral binding, and the
   * common case. Nothing is drawn at all then: chrome saying "not recording"
   * on every scratch shell would train people to stop reading this strip,
   * which is the one thing it cannot afford. */
  anchor: TerminalAnchor | null;
  /** The document's name as the tree shows it, when the window knows it. */
  docName?: string;
  onResume: () => void;
  onUnanchor: () => void;
}) {
  if (!anchor) return null;
  // A suspension outranks a foreign program: one needs a person to resume it
  // and the other resolves itself, so if both are true the one with a verb
  // attached is the one to show.
  const suspended = anchor.suspended;
  const note = suspended ?? anchor.foreign;
  return (
    <div
      className={`anchor-bar${suspended ? " anchor-bar-suspended" : ""}${
        !suspended && anchor.foreign ? " anchor-bar-foreign" : ""
      }`}
      data-testid="anchor-bar"
      role="status"
    >
      <span className="anchor-bar-what">
        {suspended ? "paused" : anchor.foreign ? "waiting" : "writing"}{" "}
        <strong>{docName ?? anchor.doc}</strong>
        {" · "}
        <code>{anchor.container}</code>
      </span>
      {note ? (
        // The reason, in the words the server chose, wrapped so the
        // second line reads as the explanation it is. Never a diff
        // afterwards: someone who does not know recording stopped will
        // assume the cell holds what they did.
        <span className="anchor-bar-why" data-testid="anchor-bar-why">
          {note.split("\n").map((line, i) => (
            <span key={i} className={i === 0 ? "anchor-bar-why-head" : "anchor-bar-why-tail"}>
              {line.trim()}
            </span>
          ))}
        </span>
      ) : (
        <span className="anchor-bar-count">
          {anchor.recorded} {anchor.recorded === 1 ? "line" : "lines"}
        </span>
      )}
      <span className="anchor-bar-actions">
        {suspended && (
          <button type="button" onClick={onResume} data-tip="starts a new cell">
            Resume
          </button>
        )}
        <button type="button" onClick={onUnanchor}>
          Stop writing
        </button>
      </span>
    </div>
  );
}
