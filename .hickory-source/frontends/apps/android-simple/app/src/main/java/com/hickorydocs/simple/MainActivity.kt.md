<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Launch a native Compose workspace

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

The activity retains a ViewModel across recreation. A development launch supplies `hickory_api_origin` in its Intent; use it only for an untouched, disconnected workspace, then retain the session across recomposition. Failed startup connections leave the manual Connect dialog available. System back refuses to discard unsaved buffers; missing server or agent configuration remains independent of editing.

File commands act on the connected workspace: path entry opens a file or directory, New Tab keeps an unnamed buffer, and Save As exclusively creates a new destination before a hash-checked write. Existing destinations are never overwritten by Save As. Save All visits each named buffer; unnamed drafts ask for a path and failed saves retain their original edits. Refresh observes the tree without replacing open buffers.

<ui:copy id="simple-d8c991adb6-0">
package com.hickorydocs.simple

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.BackHandler
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.clickable
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.key.*
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import kotlinx.coroutines.launch

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState); enableEdgeToEdge()
        setContent { MaterialTheme { Surface { SimpleWorkspace(viewModel(), intent.getStringExtra("hickory_api_origin")) } } }
    }
}
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
@Composable fun SimpleWorkspace(workspace: Workspace, initialAddress: String? = null) {
    var connection by remember { mutableStateOf(workspace.engine == null) }
    val scope = rememberCoroutineScope()
    val context = androidx.compose.ui.platform.LocalContext.current
    val descriptors=remember {runCatching {context.assets.open("commands.json").bufferedReader().use {commandDescriptors(it.readText())}}.getOrDefault(emptyList())}
    val commands=workspace.commands(descriptors,mapOf("file.connect" to {connection=true},"file.window" to {context.startActivity(android.content.Intent(context,MainActivity::class.java).putExtra("hickory_api_origin",workspace.address).addFlags(android.content.Intent.FLAG_ACTIVITY_NEW_DOCUMENT or android.content.Intent.FLAG_ACTIVITY_MULTIPLE_TASK));Unit}))
    LaunchedEffect(initialAddress) {
        if (workspace.engine == null && workspace.address.isEmpty() && !initialAddress.isNullOrEmpty()) {
            workspace.address = initialAddress
            connection = !workspace.connect()
        }
    }
    BackHandler(workspace.hasUnsavedChanges) { workspace.status = "Save your documents before leaving this workspace." }
    Column(Modifier.fillMaxSize().safeDrawingPadding().imePadding().onPreviewKeyEvent {
        if(it.type==KeyEventType.KeyDown&&it.key==Key.P&&(it.isCtrlPressed||it.isMetaPressed)) {workspace.showEverywhere=true;true}
        else if (workspace.active == "terminal") false
        else if (it.type == KeyEventType.KeyDown && it.key == Key.S && (it.isCtrlPressed || it.isMetaPressed)) {
            scope.launch { workspace.save() }; true
        } else if (it.type == KeyEventType.KeyDown && it.key == Key.F && (it.isCtrlPressed || it.isMetaPressed)) {
            workspace.currentBuffer?.showFind = true; workspace.currentBuffer != null
        } else if (it.type == KeyEventType.KeyDown && it.key == Key.Escape && workspace.currentBuffer?.showFind == true) {
            workspace.currentBuffer?.closeFind(); true
        } else false
    }) {
        Row(Modifier.fillMaxWidth().height(32.dp)) {
            CommandMenu("File",commands.filter {it.group=="File"})
            CommandMenu("Edit",commands.filter {it.group=="Edit"})
            CommandMenu("View",commands.filter {it.group=="View"})
        }
        HorizontalDivider()
        Box(Modifier.weight(1f)) { Pane(workspace, workspace.layout.rootID) }
        HorizontalDivider()
        Row(Modifier.fillMaxWidth().height(24.dp).padding(horizontal = 8.dp)) {
            Text(workspace.status, fontSize = 11.sp, maxLines = 1, modifier = Modifier.weight(1f))
            Box {
                Text(if(workspace.analysis.running) "Reading ${workspace.analysis.completed}/${workspace.analysis.scope.files.size}" else "Analysis",fontSize=11.sp,maxLines=1,modifier=Modifier.width(112.dp).clickable(role=androidx.compose.ui.semantics.Role.Button) {workspace.showAnalysis=!workspace.showAnalysis}.padding(horizontal=4.dp))
                DropdownMenu(expanded=workspace.showAnalysis,onDismissRequest={workspace.showAnalysis=false}) {WorkspaceAnalysisView(workspace)}
            }
            Box {
                Text("Problems ${workspace.problems.size}", fontSize = 11.sp, maxLines = 1, modifier = Modifier.widthIn(min = 88.dp, max = 144.dp).clickable(role = androidx.compose.ui.semantics.Role.Button) { workspace.showProblems = !workspace.showProblems }.padding(horizontal = 4.dp))
                DropdownMenu(expanded = workspace.showProblems, onDismissRequest = { workspace.showProblems = false }) { WorkspaceProblemsView(workspace) }
            }
            workspace.currentBuffer?.let { buffer -> Text(if (buffer.readOnly) "Read-only" else if (buffer.saving) "Saving…" else if (buffer.dirty) "Modified" else "Saved", fontSize = 11.sp) }
        }
    }
    if (workspace.fileCommand != null) AlertDialog(onDismissRequest = { workspace.fileCommand = null; workspace.savingAll = false }, title = { Text(workspace.fileCommand ?: "File") }, text = {
        Column { Text("Enter a path in the connected workspace. Save As requires an unused path; an empty directory path opens the root.", fontSize = 12.sp)
            OutlinedTextField(workspace.filePath, { workspace.filePath = it }, label = { Text("Workspace path") }, singleLine = true) }
    }, confirmButton = { TextButton(onClick = { scope.launch { workspace.performFileCommand() } }) { Text("OK") } }, dismissButton = { TextButton(onClick = { workspace.fileCommand = null; workspace.savingAll = false }) { Text("Cancel") } })
    if(workspace.showTests) androidx.compose.ui.window.Dialog(onDismissRequest={workspace.showTests=false;workspace.currentBuffer?.focusSource?.invoke()}) {Surface {TestSourcesView(workspace)}}
    if(workspace.showEverywhere) androidx.compose.ui.window.Dialog(onDismissRequest={workspace.showEverywhere=false;workspace.currentBuffer?.focusSource?.invoke()}) {Surface {SearchEverywhereView(workspace,commands)}}
    if (workspace.changes.mode != null) androidx.compose.ui.window.Dialog(onDismissRequest={workspace.changes.close(workspace)}) { Surface { WorkspaceChangesView(workspace) } }
    if (workspace.showWorkspaceSearch) androidx.compose.ui.window.Dialog(onDismissRequest = { workspace.showWorkspaceSearch = false; workspace.search.revision = java.util.UUID.randomUUID(); workspace.currentBuffer?.focusSource?.invoke() }) { Surface { WorkspaceSearchView(workspace) } }
    if (connection) AlertDialog(onDismissRequest = { connection = false }, title = { Text("Workspace server") }, text = {
        Column {
            OutlinedTextField(workspace.address, { workspace.address = it }, label = { Text("Workspace server") }, singleLine = true)
            Text("Use a host reachable from this device. For an emulator, adb reverse can forward the backend port.", fontSize = 12.sp)
            Text(workspace.status, fontSize = 12.sp)
        }
    }, confirmButton = { TextButton(enabled = !workspace.connecting, onClick = { scope.launch { if (workspace.connect()) connection = false } }) { Text("Connect") } },
        dismissButton = { TextButton(onClick = { connection = false }) { Text("Cancel") } })
}
@Composable private fun CommandMenu(label: String, commands: List<WorkspaceCommand>) {
    var expanded by remember { mutableStateOf(false) }
    Box {
        TextButton(onClick = { expanded = true }, contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp)) { Text(label, fontSize = 13.sp) }
        DropdownMenu(expanded, { expanded = false }) {
            commands.forEach { command -> DropdownMenuItem(text = { Text(command.label) },enabled=command.reason==null,onClick = { expanded = false; command.run() }) }
        }
    }
}

@Composable private fun WorkspaceProblemsView(workspace: Workspace) {
    val scope = rememberCoroutineScope()
    var query by remember { mutableStateOf("") }; var page by remember { mutableStateOf(0) }
    val reports = workspace.problems.filter { (it.path + " " + it.message).contains(query, ignoreCase = true) }
    val last = maxOf(0, (reports.size + 99) / 100 - 1); val selected = minOf(page, last)
    Column(Modifier.widthIn(min = 260.dp, max = 380.dp).heightIn(max = 420.dp).verticalScroll(rememberScrollState()).padding(8.dp)) {
        Text("Problems", fontSize = 16.sp)
        Text("Language-provider reports · ${workspace.problems.count { it.current }} current locations", fontSize = 11.sp)
        OutlinedTextField(query, { query = it; page = 0 }, label = { Text("Filter problems") }, singleLine = true)
        if (reports.isEmpty()) Text("No reported problems match. This is provider information, not a verification result.", fontSize = 12.sp)
        for (problem in reports.drop(selected * 100).take(100)) {
            TextButton(enabled = problem.current, onClick = { scope.launch { workspace.openProblem(problem) } }) { Text(problem.path + " · " + problem.message, fontSize = 12.sp) }
            if (!problem.current) {
                Text("Source location stale or unversioned.", fontSize = 11.sp)
                TextButton(onClick = { scope.launch { workspace.openProblem(problem, precise = false) } }) { Text("Open file") }
            }
        }
        if (last > 0) Row {
            TextButton(enabled = selected > 0, onClick = { page = selected - 1 }) { Text("Previous results") }
            Text("${selected + 1}/${last + 1}")
            TextButton(enabled = selected < last, onClick = { page = selected + 1 }) { Text("Next results") }
        }
    }
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/MainActivity.kt"><ui:paste select="#simple-d8c991adb6-0"/></ui:file>

</ui:doc>
