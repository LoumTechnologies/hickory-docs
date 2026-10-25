<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/documentCommands.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `ce1b9afc06a9e369a6831d8f144bb21d3d5e0a87d4ebe61e3da051b953d241f0`.

<book:entry path="apps/web/src/views/documentCommands.ts">
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
</book:entry>

</book:doc>
