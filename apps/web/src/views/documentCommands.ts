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
