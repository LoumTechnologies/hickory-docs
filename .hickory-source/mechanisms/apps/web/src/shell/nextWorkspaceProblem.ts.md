<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/nextWorkspaceProblem.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b7e03171fcb6"></a>
## `apps/web/src/shell/nextWorkspaceProblem.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-b7e03171fcb6-nextWorkspaceProblem"></a>
### `nextWorkspaceProblem`

<storycode:copy id="f-b7e03171fcb6-nextWorkspaceProblem">
import { EditorView } from "@codemirror/view";
import type { SessionRegistry } from "../views/documentSession";
import { severityOf } from "../lib/problems";
import { positionToUtf16 } from "../lsp/positions";

export function nextWorkspaceProblem(registry: SessionRegistry, focusedId: string | null) {
    const session = registry.get(focusedId);
    const view = session?.docEditor;
    const diagnostics = session?.lspDiagnostics ?? [];
    if (!view || diagnostics.length === 0) return;
    const ranked = [...diagnostics]
      .filter((d) => severityOf(d) <= 2)
      .map((d) => ({
        d,
        at: positionToUtf16(view.state.doc.toString(), d.range.start),
      }))
      .sort((a, b) => a.at - b.at);
    if (ranked.length === 0) return;
    const head = view.state.selection.main.head;
    // Wraps: pressing it at the last problem takes you back to the first,
    // which is what "next" means in a list you are working through.
    const next = ranked.find((r) => r.at > head) ?? ranked[0];
    view.dispatch({
      selection: { anchor: next.at },
      effects: EditorView.scrollIntoView(next.at, { y: "center" }),
    });
    view.focus();
}
</storycode:copy>

### Assemble `apps/web/src/shell/nextWorkspaceProblem.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/nextWorkspaceProblem.ts"><storycode:paste select="#f-b7e03171fcb6-nextWorkspaceProblem"/></storycode:file>

</storycode:doc>
