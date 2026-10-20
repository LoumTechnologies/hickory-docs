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
  saveUntitled(saveAs: boolean, id?: string): Promise<boolean>;
  flush(): void; print(): void; insert(id: string | null): void; editElement(): void;
};

/** Capture the target before awaiting formatting/dialogs; never re-read focus. */
export async function runWorkspaceMenu(action: MenuAction, actor: UiActor, scope: MenuScope, handlers: MenuHandlers): Promise<UiOutcome> {
  scope = { ...scope, target: { ...scope.target }, untitled: [...scope.untitled], untitledNeedingDestination: scope.untitledNeedingDestination ? [...scope.untitledNeedingDestination] : undefined, sessions: scope.sessions.map(s => ({ ...s })), session: scope.session ? { ...scope.session } : undefined, working: scope.working ? { ...scope.working } : undefined };
  if (action === "insert" || insertTarget(action) !== null) { handlers.insert(insertTarget(action)); return { status: "completed", value: { insert_panel_requested: true, source_written: false } }; }
  if (action === "edit-element") { handlers.editElement(); return { status: "completed", value: { element_editor_requested: true, source_written: false } }; }
  if (action === "print") {
    if (actor.kind === "agent") return { status: "requires_input", request: `workspace-print:${scope.target.id ?? "none"}`, reason: "The human opens/completes the actual browser or OS print dialog; no physical output is established" };
    handlers.print(); return { status: "completed", value: { print_requested: true, physical_output: "not observed" } };
  }
  if (action === "save-all") {
    if (actor.kind === "agent" && (scope.untitledNeedingDestination ?? scope.untitled).length) return { status: "requires_input", request: `workspace-save-all:${scope.untitled.join(":")}`, reason: "The human chooses destinations for unnamed buffers in the native Save dialogs; inspect their actual resulting paths" };
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
    if (actor.kind === "agent") return { status: "requires_input", request: `workspace-save:${scope.target.id}`, reason: "The human chooses this unnamed buffer's destination in the actual native Save dialog" };
    await handlers.formatFirst();
    const returned = await handlers.saveUntitled(action === "save-as", scope.target.id ?? undefined);
    return { status: "completed", value: { target: scope.target.id, save_handler_returned: returned, path: "Inspect actual resulting named buffer; empty/cancelled buffers may create no file" } };
  }
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
