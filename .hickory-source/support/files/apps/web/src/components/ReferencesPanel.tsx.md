<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/ReferencesPanel.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `3a5dae0a019f7efe54088c2a569c6e32da757bab49f1a725dee0f115475489f2`.

<book:entry path="apps/web/src/components/ReferencesPanel.tsx">
// Results of a find-references request (Shift-F12, or Cmd+Shift-click).
//
// References can land in two different worlds: back in the document
// (`hick:///`) or in a generated file the bridge could not map to source
// (`hick-output:///`). Both are listed together and labelled by which one they
// are, because "where does this symbol live" is exactly the question that the
// document/output split makes confusing.

import { useUiListActions } from "../lib/uiListActions";
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
    return { where, kind: where.endsWith(".md") ? "doc" : "file" };
  }
  return { where: uri, kind: "output" };
}

export function ReferencesPanel({ locations, query, onPick, onClose }: ReferencesPanelProps) {
  const actions = useUiListActions("References", locations, { query }, onPick, onClose);
  return (
    <aside className="refs-panel" role="dialog" aria-label="References">
      <header className="refs-head">
        <span>
          {locations.length} reference{locations.length === 1 ? "" : "s"}
          {query ? <> to <code className="mono">{query}</code></> : null}
        </span>
        <button className="btn-link" aria-label="Close references" onClick={() => void actions.close()}>
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
                <button className="refs-item" onClick={() => void actions.pick(i)}>
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
</book:entry>

</book:doc>
