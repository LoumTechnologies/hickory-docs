<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Resolve commands against current workspace state

For maintainers changing workspace services independently of the editor renderer.

Menus and action search need one vocabulary and one availability decision. The same bundled descriptors name existing operations on every simple platform; platform adapters provide callbacks and current context. A missing adapter stays explicitly unavailable. Searching never grants authority: executing an enabled command still enters the existing source, save or change-preview guard. Keep the menu group and stable ID independent of a translated label or renderer. The ordinary eligibility function can be tested without mounting UI.

<ui:copy id="simple-workspace-commands-browser">
import descriptors from "../../../simple-contract/commands.json";
export interface CommandContext {editor:boolean;writable:boolean;dirty:boolean;saving:boolean;back:boolean;forward:boolean;tab:boolean;groups:number}
export interface WorkspaceCommand {id:string;label:string;group:string;reason:string|null;run:()=>void}
export function commandReason(requires:string,context:CommandContext):string|null {
  if(!["always","editor","writable","save","back","forward","tab","multi"].includes(requires))return "Command unavailable. Reinstall the app.";
  if(["editor","writable","save"].includes(requires) && !context.editor)return "Open a source file first.";
  if(["writable","save"].includes(requires) && !context.writable)return "Choose an authored, editable source file.";
  if(["writable","save"].includes(requires) && context.saving)return "Wait for this file to finish saving.";
  if(requires==="save"&&!context.dirty)return "No unsaved changes.";
  if(requires==="back"&&!context.back||requires==="forward"&&!context.forward)return "No navigation history in this direction.";
  if(requires==="tab"&&!context.tab)return "Open a tab first.";
  if(requires==="multi"&&(!context.tab||context.groups<2))return "Open a tab and split the workspace first.";
  return null;
}
export function workspaceCommands(context:()=>CommandContext,handlers:Record<string,(()=>void)|undefined>):WorkspaceCommand[] {
  return descriptors.map(item=>({id:item.id,label:item.label,group:item.group,reason:handlers[item.id]?commandReason(item.requires,context()):"Unavailable on this platform.",run:()=>{if(handlers[item.id] && commandReason(item.requires,context())===null)handlers[item.id]!();}}));
}
</ui:copy>

<ui:file path="apps/web/src/simple/WorkspaceCommands.ts"><ui:paste select="#simple-workspace-commands-browser"/></ui:file>
</ui:doc>
