// Results of a find-references request (Shift-F12, or Cmd+Shift-click).
//
// References can land in two different worlds: back in the document
// (`hick:///`) or in a generated file the bridge could not map to source
// (`hick-output:///`). Both are listed together and labelled by which one they
// are, because "where does this symbol live" is exactly the question that the
// document/output split makes confusing.

import type { LspLocation } from "../lsp/client";

export interface ReferencesPanelProps {
  locations: LspLocation[];
  /** What was asked about, for the header. */
  query: string;
  onPick: (location: LspLocation) => void;
  onClose: () => void;
}

function label(uri: string): { where: string; kind: "doc" | "output" | "file" } {
  if (uri.startsWith("hick-output:///")) {
    return { where: uri.slice("hick-output:///".length), kind: "output" };
  }
  if (uri.startsWith("hick:///")) {
    const where = uri.slice("hick:///".length);
    // A document, or a plain file the language server also knows about.
    return { where, kind: where.endsWith(".hick") ? "doc" : "file" };
  }
  return { where: uri, kind: "output" };
}

export function ReferencesPanel({ locations, query, onPick, onClose }: ReferencesPanelProps) {
  return (
    <aside className="refs-panel" role="dialog" aria-label="References">
      <header className="refs-head">
        <span>
          {locations.length} reference{locations.length === 1 ? "" : "s"}
          {query ? <> to <code className="mono">{query}</code></> : null}
        </span>
        <button className="btn-link" aria-label="Close references" onClick={onClose}>
          ×
        </button>
      </header>
      {locations.length === 0 ? (
        <p className="muted refs-empty">
          Nothing found here. The language server only sees symbols inside woven
          code — prose and unrun cells have no definitions to point at.
        </p>
      ) : (
        <ul className="refs-list">
          {locations.map((loc, i) => {
            const { where, kind } = label(loc.uri);
            return (
              <li key={i}>
                <button className="refs-item" onClick={() => onPick(loc)}>
                  <span className={`refs-kind refs-kind-${kind}`}>
                    {kind === "doc" ? "document" : kind === "file" ? "file" : "output"}
                  </span>
                  <span className="mono refs-where">{where}</span>
                  <span className="refs-pos">
                    {loc.range.start.line + 1}:{loc.range.start.character + 1}
                  </span>
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </aside>
  );
}
