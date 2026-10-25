<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/workspaceMenuActions.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [New Document Creates Another Unnamed Buffer](../../../../../../guarantees/authoring/new-document-is-an-act/contract.md)
- [Shared print requests do not attest physical output](../../../../../../guarantees/authoring/print-requests-do-not-attest-physical-output/contract.md)

Captured SHA-256: `9d514e8bdb542cee510c9e2e4844fb792c1642755333c61813ac2747dc40aa9b`.

<book:entry path="apps/web/src/views/workspaceMenuActions.ts">
import type { MenuAction } from "../lib/menuBridge";
import { insertTarget } from "../lib/menuBridge";
import type { UiActor, UiOutcome } from "../lib/uiActions";

type Session = { id: string | null; save(): Promise<boolean>; saveAs?(): void };
export type MenuScope = {
  target: { id: string | null; kind: string | null };
  untitled: string[]; untitledNeedingDestination?: string[]; unnamedEmpty?: boolean; sessions: Session[]; session?: Session; working?: { save(): Promise<boolean> };
};
export type MenuHandlers = {
  formatFirst(): Promise<void>;
  saveUntitled(saveAs: boolean, id?: string, destination?: string): Promise<boolean>;
  flush(): void; print(): boolean | void; insert(id: string | null): void; editElement(): void;
};

/** Capture the target before awaiting formatting/dialogs; never re-read focus. */
export async function runWorkspaceMenu(action: MenuAction, actor: UiActor, scope: MenuScope, handlers: MenuHandlers, destination?: string): Promise<UiOutcome> {
  scope = { ...scope, target: { ...scope.target }, untitled: [...scope.untitled], untitledNeedingDestination: scope.untitledNeedingDestination ? [...scope.untitledNeedingDestination] : undefined, sessions: scope.sessions.map(s => ({ ...s })), session: scope.session ? { ...scope.session } : undefined, working: scope.working ? { ...scope.working } : undefined };
  if (action === "insert" || insertTarget(action) !== null) { handlers.insert(insertTarget(action)); return { status: "completed", value: { insert_panel_requested: true, source_written: false } }; }
  if (action === "edit-element") { handlers.editElement(); return { status: "completed", value: { element_editor_requested: true, source_written: false } }; }
  if (action === "print") {
    if (handlers.print() === false) return { status: "unavailable", reason: "Open a document or file to print; no print callback was submitted" };
    return { status: "completed", value: { print_callback_returned: true, physical_output: "not observed", dialog_outcome: "This handler cannot distinguish printed, cancelled or an ignored browser request" } };
  }
  if (action === "save-all") {
    if (actor.kind === "agent" && (scope.untitledNeedingDestination ?? scope.untitled).length) return { status: "requires_input", request: `workspace-save-all:${scope.untitled.join(":")}`, reason: "Name each buffer with Save unnamed buffer to specified path, or use native Save dialogs; no partial Save all was submitted" };
    await handlers.formatFirst();
    for (const id of scope.untitled) if (!await handlers.saveUntitled(false, id)) return { status: "completed", value: { unnamed_save_completed: false, target: id, scope: "Cancelled, unavailable or failed; inspect actual UI result" } };
    const documents = [];
    for (const session of scope.sessions) documents.push({ id: session.id, unchanged_after_save: await session.save() });
    handlers.flush();
    return { status: "completed", value: { documents, self_saving_panes: "Flush requested; completion not established here", verification: "not requested" } };
  }
  if (action !== "save" && action !== "save-as") return { status: "unavailable", reason: "No scoped document handler for this menu action" };
  if (scope.target.kind === "untitled") {
    if (actor.kind === "agent" && scope.unnamedEmpty) return { status: "completed", value: { target: scope.target.id, empty_buffer_no_write: true } };
    if (actor.kind === "agent" && destination === undefined) return { status: "requires_input", request: `workspace-save:${scope.target.id}`, reason: "Choose a destination with Save unnamed buffer to specified path, or use the actual native Save dialog" };
    await handlers.formatFirst();
    const returned = await handlers.saveUntitled(action === "save-as", scope.target.id ?? undefined, destination);
    return { status: "completed", value: { target: scope.target.id, save_handler_returned: returned, path: "Inspect actual resulting named buffer; empty/cancelled buffers may create no file" } };
  }
  if (destination !== undefined) return { status: "unavailable", reason: "An explicit destination belongs to an unnamed buffer. Observe the current focused buffer again." };
  if (action === "save" && scope.working) {
    await handlers.formatFirst();
    return { status: "completed", value: { target: scope.target.id, unchanged_after_save: await scope.working.save(), verification: "not requested" } };
  }
  if (!scope.session) return { status: "unavailable", reason: "Open a saveable document or file" };
  if (action === "save") {
    await handlers.formatFirst();
    const unchanged = await scope.session.save(); handlers.flush();
    return { status: "completed", value: { document: scope.session.id, unchanged_after_save: unchanged, other_self_saving_panes: "Flush requested; unconfirmed", verification: "not requested" } };
  }
  if (!scope.session.saveAs) return { status: "unavailable", reason: "This buffer does not expose Save a copy" };
  scope.session.saveAs();
  return { status: "completed", value: { document: scope.session.id, copy_prompt_requested: true, file_created: "not established; complete and inspect the actual copy operation" } };
}
</book:entry>

</book:doc>
