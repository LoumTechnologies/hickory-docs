<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Launch a native Compose workspace

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

The activity retains a ViewModel across recreation. A development launch supplies
`hickory_api_origin` and can include `hickory_open_file`, a workspace-relative
path. Give both to the ViewModel before composing the UI. It owns the first
connection and open so activity recreation cannot cancel the text read or replay
the launch over a live editor. A failed connection leaves manual Connect
available. System back refuses to discard unsaved buffers.

See the [workspace connection and open](Workspace.kt.md) and the
[loopback startup witness](../../../../../test/java/com/hickorydocs/simple/ParityTest.kt.md).

File commands act on the connected workspace: path entry opens a file or directory, New Tab keeps an unnamed buffer, and Save As exclusively creates a new destination before a hash-checked write. Existing destinations are never overwritten by Save As. Save All visits each named buffer; unnamed drafts ask for a path and failed saves retain their original edits. Refresh observes the tree without replacing open buffers.

Opening Move releases the workspace editor's text focus so its separate
dialog can take keyboard input. Android can otherwise leave Gboard targeting
the previous editor after the dialog displays a caret. Releasing focus
preserves the editor's buffer and layout.

Android Back or outside dismissal of Construction reading closes the dialog
through the same visibility flag as its Close button. This leaves the existing
editor's text focus alone.

The Edit menu offers selection commands only while the active editor holds a
current group. Before dispatch, check that the same editor still owns that
source and selection revision. The actions can therefore live in the menu
without covering the first source lines.

Android applies appearance changes to the existing Activity. Update the system
bar icon contrast with the Compose palette so the status and navigation icons
remain readable when that Activity stays mounted.

Android opens against an existing host. Offer the connection action on the
welcome screen and ask for the address only after that action. An intent with
a known host still connects immediately, and any readable startup preview
uses the same editor pane as a confirmed workspace.

<ui:copy id="simple-d8c991adb6-0">
package com.hickorydocs.simple

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.BackHandler
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.viewModels
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.clickable
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.key.*
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.view.WindowCompat
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.launch

class MainActivity : ComponentActivity() {
    private val workspace: Workspace by viewModels()

    override fun onResume() {
        super.onResume()
        lifecycleScope.launch { workspace.refreshCurrentSource() }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        workspace.connectFromLaunch(
            intent.getStringExtra("hickory_api_origin"),
            intent.getStringExtra("hickory_open_file"),
        )
        setContent {
            val dark = isSystemInDarkTheme()
            SideEffect {
                val insets = WindowCompat.getInsetsController(window, window.decorView)
                insets.isAppearanceLightStatusBars = !dark
                insets.isAppearanceLightNavigationBars = !dark
            }
            MaterialTheme(colorScheme = if (dark) darkColorScheme() else lightColorScheme()) {
                Surface { SimpleWorkspace(workspace) }
            }
        }
    }
}
@OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
@Composable fun SimpleWorkspace(workspace: Workspace) {
    ObserveSimpleAgent(workspace)
    var connection by remember { mutableStateOf(false) }
    var connectionError by remember { mutableStateOf("") }
    val showServerConnection = { connectionError = ""; connection = true }
    val scope = rememberCoroutineScope()
    val workspaceFocus = LocalFocusManager.current
    val moveOpen = workspace.changes.mode == "entryMove"
    LaunchedEffect(moveOpen) {
        if (moveOpen) workspaceFocus.clearFocus(force = true)
    }
    LaunchedEffect(workspace){workspace.editorRecovery.captureLoop(workspace)}
    val context = androidx.compose.ui.platform.LocalContext.current
    val descriptors=remember {runCatching {context.assets.open("commands.json").bufferedReader().use {commandDescriptors(it.readText())}}.getOrDefault(emptyList())}
    val commands=workspace.commands(descriptors,mapOf("file.connect" to showServerConnection,"file.window" to {context.startActivity(android.content.Intent(context,MainActivity::class.java).putExtra("hickory_api_origin",workspace.address).addFlags(android.content.Intent.FLAG_ACTIVITY_NEW_DOCUMENT or android.content.Intent.FLAG_ACTIVITY_MULTIPLE_TASK));Unit}))
    LaunchedEffect(workspace.engine) {
        if (workspace.engine != null) connection = false
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
            CommandMenu("Edit",commands.filter {it.group=="Edit"} + multipleSelectionCommands(workspace))
            CommandMenu("View",commands.filter {it.group=="View"})
        }
        HorizontalDivider()
        Box(Modifier.weight(1f).fillMaxWidth(), contentAlignment = androidx.compose.ui.Alignment.Center) {
            if (workspace.engine != null || workspace.currentBuffer != null) {
                Pane(workspace, workspace.layout.rootID)
            } else {
                Column(Modifier.padding(24.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
                    if (workspace.connecting) {
                        Text("Opening workspace…")
                    } else {
                        Text("Welcome to Hickory Docs", style = MaterialTheme.typography.headlineMedium,
                            modifier = Modifier.semantics { heading() })
                        Button(onClick = showServerConnection) { Text("Connect to a Workspace Server…") }
                        if (workspace.status.isNotEmpty()) Text(workspace.status, fontSize = 12.sp)
                    }
                }
            }
        }
        HorizontalDivider()
        Row(Modifier.fillMaxWidth().height(24.dp).padding(horizontal = 8.dp)) {
            if (BuildConfig.DEV_BUILD_STAMP.isNotEmpty()) {
                DevBuildStatus(BuildConfig.DEV_BUILD_STAMP, workspace.engine)
            }
            if (workspace.active == "agent") AgentStatus(workspace.agent, workspace.engine, Modifier.weight(1f))
            else Text(workspace.currentBuffer?.language?.completionNotice?.takeIf{it.isNotEmpty()} ?: workspace.currentBuffer?.snippets?.notice?.takeIf{it.isNotEmpty()} ?: workspace.status, fontSize = 11.sp, maxLines = 1, modifier = Modifier.weight(1f))
            Box {
                Text(if(workspace.analysis.running) "Reading ${workspace.analysis.completed}/${workspace.analysis.scope.files.size}" else "Analysis",fontSize=11.sp,maxLines=1,modifier=Modifier.widthIn(max = 112.dp).clickable(role=androidx.compose.ui.semantics.Role.Button) {workspace.showAnalysis=!workspace.showAnalysis}.padding(horizontal=4.dp))
                DropdownMenu(expanded=workspace.showAnalysis,onDismissRequest={workspace.showAnalysis=false}) {WorkspaceAnalysisView(workspace)}
            }
            Box {
                Text("Problems ${workspace.problems.size}", fontSize = 11.sp, maxLines = 1, modifier = Modifier.widthIn(max = 144.dp).clickable(role = androidx.compose.ui.semantics.Role.Button) { workspace.showProblems = !workspace.showProblems }.padding(horizontal = 4.dp))
                DropdownMenu(expanded = workspace.showProblems, onDismissRequest = { workspace.showProblems = false }) { WorkspaceProblemsView(workspace) }
            }
            workspace.currentBuffer?.let { buffer ->
                val position = languagePosition(buffer.text, buffer.state.selection.end)
                Text("Ln ${position.getInt("line") + 1}, Col ${position.getInt("character") + 1} · ", fontSize = 11.sp)
                Text(if (buffer.readOnly) "Read-only" else if (buffer.saving) "Saving…" else if (buffer.dirty) "Modified" else "Saved", fontSize = 11.sp) }
        }
    }
    if (workspace.fileCommand != null) AlertDialog(onDismissRequest = { workspace.fileCommand = null; workspace.savingAll = false }, title = { Text(workspace.fileCommand ?: "File") }, text = {
        Column { Text("Enter a path in the connected workspace. Save As requires an unused path; an empty directory path opens the root.", fontSize = 12.sp)
            OutlinedTextField(workspace.filePath, { workspace.filePath = it }, label = { Text("Workspace path") }, singleLine = true) }
    }, confirmButton = { TextButton(onClick = { scope.launch { workspace.performFileCommand() } }) { Text("OK") } }, dismissButton = { TextButton(onClick = { workspace.fileCommand = null; workspace.savingAll = false }) { Text("Cancel") } })
    if(workspace.showCoverage) androidx.compose.ui.window.Dialog(onDismissRequest={workspace.showCoverage=false;workspace.currentBuffer?.focusSource?.invoke()}) {Surface {SimpleCoverageView(workspace)}}
    if(workspace.editorRecovery.showing) androidx.compose.ui.window.Dialog(onDismissRequest={workspace.editorRecovery.close();workspace.currentBuffer?.focusSource?.invoke()}) {Surface {EditorRecoveryView(workspace)}}
    if(workspace.showRecovery) androidx.compose.ui.window.Dialog(onDismissRequest={workspace.showRecovery=false;workspace.currentBuffer?.focusSource?.invoke()}) {Surface {SourceRecoveryView(workspace)}}
    if(workspace.agentProposals.showing) androidx.compose.ui.window.Dialog(onDismissRequest={workspace.agentProposals.close();workspace.currentBuffer?.focusSource?.invoke()}) {Surface {AgentProposalView(workspace)}}
    if(workspace.showConstruction) androidx.compose.ui.window.Dialog(onDismissRequest={workspace.showConstruction=false}) {Surface {ConstructionReadingView(workspace)}}
    if(workspace.showAttribution) androidx.compose.ui.window.Dialog(onDismissRequest={workspace.showAttribution=false;workspace.currentBuffer?.focusSource?.invoke()}) {Surface {GitAttributionView(workspace,workspace.gitAttribution)}}
    if(workspace.showHistory) androidx.compose.ui.window.Dialog(onDismissRequest={workspace.showHistory=false;workspace.currentBuffer?.focusSource?.invoke()}) {Surface {GitHistoryView(workspace)}}
    if(workspace.showReview) androidx.compose.ui.window.Dialog(onDismissRequest={workspace.showReview=false;workspace.currentBuffer?.focusSource?.invoke()}) {Surface {GitReviewView(workspace)}}
    if(workspace.sourceHistory.showing) androidx.compose.ui.window.Dialog(onDismissRequest={workspace.sourceHistory.close();workspace.currentBuffer?.focusSource?.invoke()}) {Surface {SourceHistoryView(workspace)}}
    if(workspace.showTemplates) androidx.compose.ui.window.Dialog(onDismissRequest={workspace.showTemplates=false;workspace.currentBuffer?.focusSource?.invoke()}) {Surface {SourceTemplatesView(workspace)}}
    if(workspace.showTests) androidx.compose.ui.window.Dialog(onDismissRequest={workspace.showTests=false;workspace.currentBuffer?.focusSource?.invoke()}) {Surface {TestSourcesView(workspace)}}
    if(workspace.showEverywhere) androidx.compose.ui.window.Dialog(onDismissRequest={workspace.showEverywhere=false;workspace.currentBuffer?.focusSource?.invoke()}) {Surface {SearchEverywhereView(workspace,commands)}}
    if (workspace.changes.mode != null) androidx.compose.ui.window.Dialog(onDismissRequest={workspace.changes.close(workspace)}) { Surface { WorkspaceChangesView(workspace) } }
    if (workspace.showWorkspaceSearch) androidx.compose.ui.window.Dialog(onDismissRequest = { workspace.showWorkspaceSearch = false; workspace.search.revision = java.util.UUID.randomUUID(); workspace.currentBuffer?.focusSource?.invoke() }) { Surface { WorkspaceSearchView(workspace) } }
    if (connection) {
        val pendingReason = if (workspace.connecting) "A connection attempt is in progress." else null
        val connectReason = pendingReason ?: if (workspace.address.isBlank()) "Enter the workspace server address." else null
        AlertDialog(
            onDismissRequest = { connection = false },
            title = { Text("Connect to a Workspace Server") },
            text = {
                Column {
                    WithDisabledReason(pendingReason) { available ->
                        OutlinedTextField(
                            workspace.address, { workspace.address = it },
                            label = { Text("Workspace server address") },
                            placeholder = { Text("https://server:port") },
                            enabled = available, singleLine = true,
                        )
                    }
                    if (connectionError.isNotEmpty()) Text(connectionError, fontSize = 12.sp)
                }
            },
            confirmButton = {
                WithDisabledReason(connectReason) { available ->
                    TextButton(enabled = available, onClick = {
                        connectionError = ""
                        workspace.address = workspace.address.trim()
                        scope.launch {
                            if (workspace.connect()) connection = false
                            else connectionError = workspace.status
                        }
                    }) { Text("Connect") }
                }
            },
            dismissButton = { TextButton(onClick = { connection = false }) { Text("Cancel") } },
        )
    }
}
@Composable private fun CommandMenu(label: String, commands: List<WorkspaceCommand>) {
    var expanded by remember { mutableStateOf(false) }
    Box {
        TextButton(onClick = { expanded = true }, contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp)) { Text(label, fontSize = 13.sp) }
        DropdownMenu(expanded, { expanded = false }) {
            commands.forEach { command ->
                WithDisabledReason(command.reason) { available ->
                    DropdownMenuItem(
                        text = { Text(command.label) },
                        enabled = available,
                        onClick = { expanded = false; command.run() },
                    )
                }
            }
        }
    }
}

private fun multipleSelectionCommands(workspace: Workspace): List<WorkspaceCommand> {
    val buffer = workspace.currentBuffer ?: return emptyList()
    val source = buffer.text
    if (buffer.multiSource != source || buffer.multiRanges.size < 2) return emptyList()
    val revision = buffer.multiSelectionRevision
    fun current() = workspace.currentBuffer === buffer && buffer.text == source &&
        buffer.multiSource == source && buffer.multiSelectionRevision == revision
    return listOf(
        WorkspaceCommand("editor.editSelections", "Edit selections", "Edit", null) {
            if (current()) buffer.showMulti = true
        },
        WorkspaceCommand("editor.clearSelections", "Clear selections", "Edit", null) {
            if (current()) buffer.clearMulti()
        },
    )
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
            WithDisabledReason(if (!(problem.current)) "Refresh workspace problems before opening this stale diagnostic." else null) { available -> TextButton(enabled = available, onClick = { scope.launch { workspace.openProblem(problem) } }) { Text(problem.path + " · " + problem.message, fontSize = 12.sp) } }
            if (!problem.current) {
                Text("Source location stale or unversioned.", fontSize = 11.sp)
                TextButton(onClick = { scope.launch { workspace.openProblem(problem, precise = false) } }) { Text("Open file") }
            }
        }
        if (last > 0) Row {
            WithDisabledReason(if (selected <= 0) "You’re on the first page." else null) { available -> TextButton(enabled = available, onClick = { page = selected - 1 }) { Text("Previous results") } }
            Text("${selected + 1}/${last + 1}")
            WithDisabledReason(if (selected >= last) "You’re on the last page." else null) { available -> TextButton(enabled = available, onClick = { page = selected + 1 }) { Text("Next results") } }
        }
    }
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

Observe build identity in a coroutine keyed to the engine connection. Compose
discards that observation when the engine changes. The dropdown expands over
the editor and gives the full identities with explicit dirty-state labels.

<ui:copy id="dev-android-status-view">

@Composable private fun DevBuildStatus(ui: String, engine: Engine?) {
    var reading by remember(engine) { mutableStateOf<org.json.JSONObject?>(null) }
    var verdict by remember(engine) { mutableStateOf("Checking") }
    var showing by remember { mutableStateOf(false) }
    LaunchedEffect(engine, ui) {
        if (engine == null) {
            verdict = "Backend unknown"
            return@LaunchedEffect
        }
        val query = ui.replace('+', '~')
        while (true) {
            val result = try {
                engine.request("GET", "/api/dev-build?ui=$query")
            } catch (cancelled: kotlinx.coroutines.CancellationException) {
                throw cancelled
            } catch (_: Exception) {
                null
            }
            reading = result
            verdict = result?.optString("verdict", "Backend unknown") ?: "Backend unknown"
            kotlinx.coroutines.delay(5000)
        }
    }
    val short = if (ui == "unknown") "?" else ui.take(8) + if (ui.endsWith("+")) "+" else ""
    fun describe(stamp: String?): String = when {
        stamp.isNullOrEmpty() || stamp == "unknown" -> "unknown"
        stamp.endsWith("+") -> stamp.dropLast(1) + " · uncommitted changes"
        else -> stamp + " · clean"
    }
    Box {
        Text("$verdict · $short", fontSize = 11.sp, maxLines = 1, modifier = Modifier
            .clickable(role = androidx.compose.ui.semantics.Role.Button) { showing = !showing }
            .padding(end = 4.dp))
        DropdownMenu(expanded = showing, onDismissRequest = { showing = false }) {
            Column(Modifier.widthIn(max = 300.dp).padding(horizontal = 12.dp)) {
                Text("UI · ${describe(ui)}")
                Text("Backend · ${describe(reading?.optString("backend"))}")
                Text("Engine checkout · ${describe(reading?.optString("checkout"))}")
            }
        }
    }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/MainActivity.kt"><ui:paste select="#simple-d8c991adb6-0"/><ui:paste select="#dev-android-status-view"/></ui:file>

</ui:doc>
