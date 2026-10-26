<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/ReferencesPanel.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-76fb7cf602aa"></a>
## `apps/web/src/components/ReferencesPanel.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-76fb7cf602aa-ReferencesPanelProps"></a>
### `ReferencesPanelProps`

Results of a find-references request (Shift-F12, or Cmd+Shift-click).
References can land in two different worlds: back in the document
(`hick:///`) or in a generated file the bridge could not map to source
(`hick-output:///`). Both are listed together and labelled by which one they
are, because "where does this symbol live" is exactly the question that the
document/output split makes confusing.

<storycode:copy id="f-76fb7cf602aa-ReferencesPanelProps">
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

</storycode:copy>
<a id="f-76fb7cf602aa-label"></a>
### `label`

<storycode:copy id="f-76fb7cf602aa-label">
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

</storycode:copy>
<a id="f-76fb7cf602aa-ReferencesPanel"></a>
### `ReferencesPanel`

<storycode:copy id="f-76fb7cf602aa-ReferencesPanel">
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
</storycode:copy>

### Assemble `apps/web/src/components/ReferencesPanel.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/ReferencesPanel.tsx"><storycode:paste select="#f-76fb7cf602aa-ReferencesPanelProps"/><storycode:paste select="#f-76fb7cf602aa-label"/><storycode:paste select="#f-76fb7cf602aa-ReferencesPanel"/></storycode:file>

</storycode:doc>
