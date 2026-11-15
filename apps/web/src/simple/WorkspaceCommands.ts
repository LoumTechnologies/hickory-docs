import descriptors from "../../../simple-contract/commands.json";
export interface CommandContext {editor:boolean;writable:boolean;dirty:boolean;saving:boolean;back:boolean;forward:boolean;tab:boolean;groups:number;snippet?:boolean;snippetChoices?:boolean;sourceUndo?:boolean;sourceRedo?:boolean}
export interface WorkspaceCommand {id:string;label:string;group:string;reason:string|null;run:()=>void}
export function commandReason(requires:string,context:CommandContext):string|null {
  if(!["always","editor","writable","save","back","forward","tab","multi","template","templateChoice","sourceUndo","sourceRedo"].includes(requires))return "Command unavailable. Reinstall the app.";
  if(["editor","writable","save","template","templateChoice","sourceUndo","sourceRedo"].includes(requires) && !context.editor)return "Open a source file first.";
  if(["writable","save","templateChoice","sourceUndo","sourceRedo"].includes(requires) && !context.writable)return "Choose an authored, editable source file.";
  if(["writable","save","templateChoice"].includes(requires) && context.saving)return "Wait for this file to finish saving.";
  if(["template","templateChoice"].includes(requires)&&!context.snippet)return "Start a template or snippet first.";
  if(requires==="templateChoice"&&!context.snippetChoices)return "This field has no offered values.";
  if(requires==='sourceUndo'&&!context.sourceUndo||requires==='sourceRedo'&&!context.sourceRedo)return "No text history in this direction.";
  if(requires==="save"&&!context.dirty)return "No unsaved changes.";
  if(requires==="back"&&!context.back||requires==="forward"&&!context.forward)return "No navigation history in this direction.";
  if(requires==="tab"&&!context.tab)return "Open a tab first.";
  if(requires==="multi"&&(!context.tab||context.groups<2))return "Open a tab and split the workspace first.";
  return null;
}
export function workspaceCommands(context:()=>CommandContext,handlers:Record<string,(()=>void)|undefined>):WorkspaceCommand[] {
  return descriptors.map(item=>({id:item.id,label:item.label,group:item.group,reason:handlers[item.id]?commandReason(item.requires,context()):"Unavailable on this platform.",run:()=>{if(handlers[item.id] && commandReason(item.requires,context())===null)handlers[item.id]!();}}));
}
