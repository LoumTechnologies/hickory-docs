package com.hickorydocs.simple
import org.json.JSONArray
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.PlainTooltip
import androidx.compose.material3.Text
import androidx.compose.material3.TooltipAnchorPosition
import androidx.compose.material3.TooltipBox
import androidx.compose.material3.TooltipDefaults
import androidx.compose.material3.rememberTooltipState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription

@OptIn(ExperimentalMaterial3Api::class)
@Composable
internal fun WithDisabledReason(reason: String?, content: @Composable (Boolean) -> Unit) {
    val description = if (reason == null) Modifier else Modifier.semantics { stateDescription = reason }
    val state = rememberTooltipState()
    if (reason == null && state.isVisible) {
        LaunchedEffect(state) { state.dismiss() }
    }
    TooltipBox(
        modifier = description,
        positionProvider = TooltipDefaults.rememberTooltipPositionProvider(TooltipAnchorPosition.Above),
        tooltip = { PlainTooltip { Text(reason.orEmpty()) } },
        state = state,
        focusable = false,
        enableUserInput = reason != null,
    ) {
        content(reason == null)
    }
}

data class CommandDescriptor(val id:String,val label:String,val group:String,val requires:String)
data class CommandContext(val editor:Boolean,val writable:Boolean,val dirty:Boolean,val saving:Boolean,val back:Boolean,val forward:Boolean,val tab:Boolean,val groups:Int,val snippet:Boolean=false,val snippetChoices:Boolean=false,val sourceUndo:Boolean=false,val sourceRedo:Boolean=false,val separateFiles:Boolean=false)
data class WorkspaceCommand(val id:String,val label:String,val group:String,val reason:String?,val run:()->Unit)
fun commandReason(requires:String,context:CommandContext):String? {
    if(requires !in listOf("always","editor","writable","save","back","forward","tab","multi","template","templateChoice","sourceUndo","sourceRedo","separateFiles"))return "Command unavailable. Reinstall the app."
    if(requires in listOf("editor","writable","save","template","templateChoice","sourceUndo","sourceRedo")&&!context.editor)return "Open a source file first."
    if(requires in listOf("writable","save","templateChoice","sourceUndo","sourceRedo")&&!context.writable)return "Choose an authored, editable source file."
    if(requires in listOf("writable","save","templateChoice")&&context.saving)return "Wait for this file to finish saving."
    if(requires in listOf("template","templateChoice")&&!context.snippet)return "Start a template or snippet first."
    if(requires=="templateChoice"&&!context.snippetChoices)return "This field has no offered values."
    if(requires=="sourceUndo"&&!context.sourceUndo||requires=="sourceRedo"&&!context.sourceRedo)return "No text history in this direction."
    if(requires=="save"&&!context.dirty)return "No unsaved changes."
    if(requires=="back"&&!context.back||requires=="forward"&&!context.forward)return "No navigation history in this direction."
    if(requires=="tab"&&!context.tab)return "Open a tab first."
    if(requires=="separateFiles"&&!context.separateFiles)return "Open a document beside a separate Files pane."
    if(requires=="multi"&&(!context.tab||context.groups<2))return "Open a tab and split the workspace first."
    return null
}
fun commandDescriptors(text:String):List<CommandDescriptor> {
    val data=JSONArray(text);val result=(0 until data.length()).map {val item=data.getJSONObject(it);CommandDescriptor(item.getString("id"),item.getString("label"),item.getString("group"),item.getString("requires"))}
    check(result.map {it.id}.toSet().size==result.size) {"Command IDs are duplicated. Reinstall the app."};return result
}
fun workspaceCommands(context:()->CommandContext,handlers:Map<String,()->Unit>,descriptors:List<CommandDescriptor>):List<WorkspaceCommand> = descriptors.map {item->WorkspaceCommand(item.id,item.label,item.group,if(item.id !in handlers) "Unavailable on this platform." else commandReason(item.requires,context()),{if(commandReason(item.requires,context())==null)handlers[item.id]?.invoke()})}
