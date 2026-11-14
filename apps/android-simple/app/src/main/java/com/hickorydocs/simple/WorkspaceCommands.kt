package com.hickorydocs.simple
import org.json.JSONArray
data class CommandDescriptor(val id:String,val label:String,val group:String,val requires:String)
data class CommandContext(val editor:Boolean,val writable:Boolean,val dirty:Boolean,val saving:Boolean,val back:Boolean,val forward:Boolean,val tab:Boolean,val groups:Int)
data class WorkspaceCommand(val id:String,val label:String,val group:String,val reason:String?,val run:()->Unit)
fun commandReason(requires:String,context:CommandContext):String? {
    if(requires !in listOf("always","editor","writable","save","back","forward","tab","multi"))return "Command unavailable. Reinstall the app."
    if(requires in listOf("editor","writable","save")&&!context.editor)return "Open a source file first."
    if(requires in listOf("writable","save")&&!context.writable)return "Choose an authored, editable source file."
    if(requires in listOf("writable","save")&&context.saving)return "Wait for this file to finish saving."
    if(requires=="save"&&!context.dirty)return "No unsaved changes."
    if(requires=="back"&&!context.back||requires=="forward"&&!context.forward)return "No navigation history in this direction."
    if(requires=="tab"&&!context.tab)return "Open a tab first."
    if(requires=="multi"&&(!context.tab||context.groups<2))return "Open a tab and split the workspace first."
    return null
}
fun commandDescriptors(text:String):List<CommandDescriptor> {
    val data=JSONArray(text);val result=(0 until data.length()).map {val item=data.getJSONObject(it);CommandDescriptor(item.getString("id"),item.getString("label"),item.getString("group"),item.getString("requires"))}
    check(result.map {it.id}.toSet().size==result.size) {"Command IDs are duplicated. Reinstall the app."};return result
}
fun workspaceCommands(context:()->CommandContext,handlers:Map<String,()->Unit>,descriptors:List<CommandDescriptor>):List<WorkspaceCommand> = descriptors.map {item->WorkspaceCommand(item.id,item.label,item.group,if(item.id !in handlers) "Unavailable on this platform." else commandReason(item.requires,context()),{if(commandReason(item.requires,context())==null)handlers[item.id]?.invoke()})}
