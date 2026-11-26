package com.hickorydocs.simple

import android.content.Intent
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
        workspace.exclusiveWindow = true
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
    val commands = workspace.commands(descriptors, mapOf(
        "file.connect" to showServerConnection,
        "file.window" to {
            context.startActivity(Intent(context, MainActivity::class.java)
                .addFlags(Intent.FLAG_ACTIVITY_NEW_DOCUMENT or Intent.FLAG_ACTIVITY_MULTIPLE_TASK))
        },
    ))
    LaunchedEffect(workspace.engine) {
        if (workspace.engine != null) connection = false
    }
    BackHandler(workspace.hasUnsavedChanges) { workspace.status = "Save your documents before leaving this workspace." }
    Column(Modifier.fillMaxSize().safeDrawingPadding().imePadding().onPreviewKeyEvent {
        if (workspace.windowConnectionLost) false
        else if(it.type==KeyEventType.KeyDown&&it.key==Key.P&&(it.isCtrlPressed||it.isMetaPressed)) {workspace.showEverywhere=true;true}
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
            CommandMenu("File", commands.filter {
                it.group == "File" && (!workspace.windowConnectionLost || it.id == "file.connect" || it.id == "file.window")
            })
            if (!workspace.windowConnectionLost) {
                val editCommands = commands.filter { it.group == "Edit" }.map { command ->
                    when {
                        command.id == "workspace.problems" -> command.copy(label = "Problems (${workspace.problems.size})")
                        command.id == "workspace.analysis" && workspace.analysis.running ->
                            command.copy(label = "Workspace analysis · ${workspace.analysis.completed}/${workspace.analysis.scope.files.size}")
                        else -> command
                    }
                }
                CommandMenu("Edit", editCommands + multipleSelectionCommands(workspace), overlays = {
                    DropdownMenu(expanded = workspace.showAnalysis, onDismissRequest = { workspace.showAnalysis = false }) {
                        WorkspaceAnalysisView(workspace)
                    }
                    DropdownMenu(expanded = workspace.showProblems, onDismissRequest = { workspace.showProblems = false }) {
                        WorkspaceProblemsView(workspace)
                    }
                })
                CommandMenu("View", commands.filter { it.group == "View" }, extraItems = {
                    HorizontalDivider()
                    Column(Modifier.widthIn(max = 300.dp).padding(horizontal = 12.dp, vertical = 6.dp)) {
                        if (workspace.status.isNotEmpty()) Text(workspace.status)
                        workspace.currentBuffer?.let { buffer ->
                            val position = languagePosition(buffer.text, buffer.state.selection.end)
                            val state = if (buffer.readOnly) "Read-only" else if (buffer.saving) "Saving…" else if (buffer.dirty) "Modified" else "Saved"
                            Text("${buffer.file.name} · $state")
                            Text("Ln ${position.getInt("line") + 1}, Col ${position.getInt("character") + 1}")
                            val notice = buffer.language?.completionNotice.orEmpty().ifEmpty { buffer.snippets.notice }
                            if (notice.isNotEmpty()) Text(notice)
                        }
                        if (BuildConfig.DEV_BUILD_STAMP.isNotEmpty()) {
                            DevBuildStatus(BuildConfig.DEV_BUILD_STAMP, workspace.engine)
                        }
                    }
                })
            }
        }
        HorizontalDivider()
        Box(Modifier.weight(1f).fillMaxWidth(), contentAlignment = androidx.compose.ui.Alignment.Center) {
            if (workspace.windowConnectionLost) {
                Column(Modifier.padding(24.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
                    Text("Workspace connection ended. Reconnect to continue.")
                    Button(onClick = showServerConnection) { Text("Connect to a Workspace Server…") }
                }
            } else if (workspace.engine != null || workspace.currentBuffer != null) {
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
@Composable private fun CommandMenu(
    label: String,
    commands: List<WorkspaceCommand>,
    extraItems: @Composable () -> Unit = {},
    overlays: @Composable () -> Unit = {},
) {
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
            extraItems()
        }
        overlays()
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
    val received = workspace.problems
    val reports = received.filter { (it.path + " " + it.message).contains(query, ignoreCase = true) }
    val last = maxOf(0, (reports.size + 99) / 100 - 1); val selected = minOf(page, last)
    Column(Modifier.widthIn(min = 260.dp, max = 380.dp).heightIn(max = 420.dp).verticalScroll(rememberScrollState()).padding(8.dp)) {
        Text("Problems", fontSize = 16.sp)
        Text("Language-provider reports · ${received.count { it.current }} current locations", fontSize = 11.sp)
        val buffer = workspace.currentBuffer
        if (buffer == null) {
            Text("Open a source file to see its diagnostic status.", fontSize = 12.sp)
        } else {
            Text("${buffer.file.name} · ${buffer.file.language ?: "Unclassified source"}", fontSize = 12.sp)
            val language = buffer.language
            Text(when {
                language == null -> "No language session is attached to this file."
                language.status.startsWith("Language services unavailable") -> language.status
                received.any { it.path == buffer.file.path && it.current } -> "Provider locations match the current file text."
                language.versionedEmptyDiagnostics == buffer.text -> "The provider reported zero problems for the current file text."
                else -> "No source-bound diagnostic report for the current file text."
            }, fontSize = 12.sp)
        }
        OutlinedTextField(query, { query = it; page = 0 }, label = { Text("Filter problems") }, singleLine = true)
        if (reports.isEmpty()) Text(if (query.isEmpty()) "No workspace problems have been reported." else "No received reports match the filter.", fontSize = 12.sp)
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

@Composable private fun DevBuildStatus(ui: String, engine: Engine?) {
    var reading by remember(engine) { mutableStateOf<org.json.JSONObject?>(null) }
    var verdict by remember(engine) { mutableStateOf("Checking") }
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
    Text("Development build · $verdict · $short")
    Text("UI · ${describe(ui)}")
    Text("Backend · ${describe(reading?.optString("backend"))}")
    Text("Engine checkout · ${describe(reading?.optString("checkout"))}")
}
