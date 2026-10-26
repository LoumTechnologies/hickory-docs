<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/documentCommands.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-902128d918d1"></a>
## `apps/web/src/views/documentCommands.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-902128d918d1-documentCommands"></a>
### `documentCommands`

<storycode:copy id="f-902128d918d1-documentCommands">
import type { CommandItem } from "../shell/CommandBar";
import type { DocSession } from "./documentSession";

/** Use the same session actions as the document's header menu. */
export function documentCommands(session: DocSession | null | undefined): CommandItem[] {
  if (!session?.doc || session.runningCells.size || session.banner?.kind === "pending") return [];
  return [
    { id: "document-run", label: "Run document", detail: session.doc.path, run: session.runAll },
    { id: "document-test", label: "Test document", detail: session.doc.path, run: session.verify },
  ];
}
</storycode:copy>

### Assemble `apps/web/src/views/documentCommands.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/documentCommands.ts"><storycode:paste select="#f-902128d918d1-documentCommands"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-cff1ed3a5fcb"></a>
## `apps/web/src/views/documentCommands.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-cff1ed3a5fcb-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-cff1ed3a5fcb-expression_statement">
import { expect, it, vi } from "vitest";
import { documentCommands } from "./documentCommands";
import type { DocSession } from "./documentSession";

it("uses the owning session for Run and Test, and names its target", () => {
  const session = { doc: { path: "project.md" }, runAll: vi.fn(), verify: vi.fn(), runningCells: new Set(), banner: null } as unknown as DocSession;
  const commands = documentCommands(session);
  expect(commands.map(item => [item.label, item.detail])).toEqual([["Run document", "project.md"], ["Test document", "project.md"]]);
  commands[0].run(); commands[1].run();
  expect(session.runAll).toHaveBeenCalledOnce(); expect(session.verify).toHaveBeenCalledOnce();
  expect(documentCommands({ ...session, runningCells: new Set(["cell"]) })).toEqual([]);
  expect(documentCommands({ ...session, banner: { kind: "pending", text: "Testing" } })).toEqual([]);
  expect(documentCommands(null)).toEqual([]);
});
</storycode:copy>

### Assemble `apps/web/src/views/documentCommands.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/documentCommands.test.ts"><storycode:paste select="#f-cff1ed3a5fcb-expression_statement"/></storycode:file>


</storycode:doc>
