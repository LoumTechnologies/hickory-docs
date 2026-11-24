package com.hickorydocs.simple
import androidx.compose.runtime.*
import androidx.compose.foundation.*
import androidx.compose.foundation.relocation.BringIntoViewRequester
import androidx.compose.foundation.relocation.bringIntoViewRequester
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.input.key.*
import kotlinx.coroutines.launch
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
@Composable fun SearchEverywhereView(workspace:Workspace,commands:List<WorkspaceCommand>) {
    var input by remember {mutableStateOf(androidx.compose.ui.text.input.TextFieldValue(""))};val query=input.text;var kind by remember {mutableStateOf("all")};var scopes by remember {mutableStateOf(false)};var selected by remember(query,kind) {mutableStateOf(0)}
    val scope=rememberCoroutineScope();val focus=remember {FocusRequester()}
    val targets=everywhereTargets(workspace.rootFiles+workspace.buffers.values.map {it.file},workspace.analysis.knowledge.values.toList(),{workspace.buffers[it]?.text},commands)
    val (items,total)=searchEverywhere(targets,query,kind);val index=selected.coerceIn(0,maxOf(0,items.size-1))
    LaunchedEffect(Unit) {focus.requestFocus()}
    Column(Modifier.widthIn(min=260.dp,max=420.dp).heightIn(max=480.dp).padding(8.dp)) {
        Row {Text("Search Everywhere",fontSize=16.sp);Spacer(Modifier.weight(1f));TextButton(onClick={workspace.showEverywhere=false;workspace.currentBuffer?.focusSource?.invoke()}) {Text("Close")}}
        OutlinedTextField(input,{input=it},label={Text("Search files, types, members and actions")},singleLine=true,modifier=Modifier.focusRequester(focus).onPreviewKeyEvent {event->
            if(event.type!=KeyEventType.KeyDown||input.composition!=null)false
            else if(event.key==Key.DirectionDown){selected=minOf(index+1,maxOf(0,items.size-1));true}
            else if(event.key==Key.DirectionUp){selected=maxOf(0,index-1);true}
            else if(event.key==Key.Enter){items.getOrNull(index)?.takeIf {it.reason==null}?.let {target->scope.launch {workspace.openEverywhere(target,commands)}};true}else false
        })
        Box {TextButton(onClick={scopes=true}) {Text("Search scope: $kind")};DropdownMenu(scopes,{scopes=false}) {for(value in listOf("all","file","type","member","symbol","action"))DropdownMenuItem(text={Text(value)},onClick={kind=value;scopes=false})}}
        Row {Text("${items.size} of $total matches",fontSize=11.sp);WithDisabledReason(if (workspace.analysis.running) "Workspace analysis is in progress." else null) { available -> TextButton(enabled = available,onClick={workspace.startAnalysis()}) {Text(if(workspace.analysis.running)"Reading…" else "Read workspace")} }}
        Text("Symbols reflect source-current provider reads; missing results may need analysis or language support. Files and actions work independently.",fontSize=11.sp)
        if(commands.isEmpty())Text("Command catalog missing. Reinstall the app; source files remain readable.",fontSize=11.sp)
        Column(Modifier.weight(1f,fill=false).heightIn(max=300.dp).verticalScroll(rememberScrollState())) {items.forEachIndexed {at,target->
            val bring=remember(target.id) {BringIntoViewRequester()}
            LaunchedEffect(target.id,index==at) {if(index==at)bring.bringIntoView()}
            WithDisabledReason(target.reason) { available -> TextButton(enabled = available,onClick={scope.launch {workspace.openEverywhere(target,commands)}},modifier=Modifier.fillMaxWidth().bringIntoViewRequester(bring).background(if(at==index)MaterialTheme.colorScheme.secondaryContainer else androidx.compose.ui.graphics.Color.Transparent)) {Column {Text(target.label);Text(target.kind+" · "+target.detail,fontSize=11.sp)}} }
            target.reason?.let {Text(it,fontSize=11.sp)}
        }}
    }
}
