<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/nextWorkspaceProblem.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `48c395030dc6f122702937dbb67fd608bb8ecfad318cbf7469ac3d19bf655d98`.

<book:entry path="apps/web/src/shell/nextWorkspaceProblem.ts">
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
</book:entry>

</book:doc>
