package com.hickorydocs.simple

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.json.JSONObject

data class AgentFenceReading(val turn: String, val side: String, val source: String, val current: () -> Boolean) {
    fun endpoint(action: String): String {
        val id = java.net.URLEncoder.encode(turn, "UTF-8").replace("+", "%20")
        return "/api/docs/workspace/agent/turns/$id/$action"
    }
}

class AgentSession {
    var turns by mutableStateOf<List<AgentTurn>>(emptyList())
    var provider by mutableStateOf("codex-subscription")
    var model by mutableStateOf("")
    var effort by mutableStateOf("")
    val draft = DocumentBuffer(FileNode("Message", "conversation/draft.md", false, emptyList(), null, null), FileText("", ""))
    var prompt: String
        get() = draft.text
        set(value) { draft.state.edit { replace(0, length, value) } }
    private val documents = mutableMapOf<String, DocumentBuffer>()
    var debugger by mutableStateOf<NativeDebugger?>(null)
        internal set
    fun document(id: String, text: String): DocumentBuffer {
        val held = documents[id]
        if (held != null && held.text == text) return held
        val path = "conversation/$id.md"
        return DocumentBuffer(FileNode(id, path, false, emptyList(), null, "conversation"), FileText(text, "")).also { documents[id] = it }
    }
    fun fenceReading(turn: AgentTurn, assistant: Boolean): AgentFenceReading? {
        fun source(record: AgentTurn): String? = if (assistant) record.answer.takeUnless { record.status == "running" } else record.prompt
        val text = source(turn) ?: return null
        return AgentFenceReading(turn.id, if (assistant) "assistant" else "user", text) {
            turns.firstOrNull { it.id == turn.id }?.let(::source) == text
        }
    }
    private val recordedRuns = mutableStateMapOf<DocumentBuffer, RecordedFenceRun>()
    internal fun recordedRun(buffer: DocumentBuffer): RecordedFenceRun? =
        recordedRuns[buffer]?.takeIf { it.current() }

    internal suspend fun runRecordedFence(engine: Engine, buffer: DocumentBuffer, reading: AgentFenceReading, line: Int, ownerCurrent: () -> Boolean) {
        val root = engine.sourceRootAddress
        val workspace = engine.sourceWorkspaceID
        fun current() = ownerCurrent() && reading.current() && documents[buffer.file.name] === buffer &&
            buffer.text == reading.source && engine.sourceRootAddress == root && engine.sourceWorkspaceID == workspace
        if (!current() || !buffer.readOnly || buffer.inlineMarkup.source != buffer.text ||
            line !in buffer.inlineMarkup.debug.recordedClosing) return
        if (recordedRun(buffer)?.let { it.line == line && it.busy } == true) return
        recordedRuns.entries.filter { !it.value.current() }.forEach { recordedRuns.remove(it.key) }
        val run = RecordedFenceRun(buffer, line, ::current)
        recordedRuns[buffer] = run
        try {
            check(root != null) { "Confirm the workspace in Files before running this fence." }
            val hash = withContext(Dispatchers.Default) { debugHash(reading.source) }
            if (!current() || recordedRuns[buffer] !== run) return
            val reply = engine.request("POST", reading.endpoint("fence-run"),
                JSONObject().put("side", reading.side).put("source_hash", hash).put("opening_line", line))
            if (!current() || recordedRuns[buffer] !== run) return
            check(debugInteger(reply.opt("version")) == 1L && reply.opt("turn") == reading.turn &&
                reply.opt("side") == reading.side && reply.opt("source_hash") == hash &&
                debugInteger(reply.opt("opening_line")) == line.toLong() && reply.opt("workspace") == root) {
                "Run receipt belongs to a different recorded fence."
            }
            val receipt = recordedTerminal(reply.getJSONObject("terminal"))
            run.terminal = receipt
            refreshRecordedRun(engine, run)
        } catch (cancelled: CancellationException) { throw cancelled }
        catch (failure: Exception) {
            if (current() && recordedRuns[buffer] === run) run.notice = "${failure.message} Check the retained session before running again."
        } finally { run.busy = false }
    }

    internal suspend fun refreshRecordedRun(engine: Engine, run: RecordedFenceRun) {
        val id = run.terminal?.id ?: return
        fun current() = recordedRuns[run.buffer] === run && run.current()
        if (!current() || run.reading) return
        run.reading = true
        run.notice = ""
        try {
            while (current()) {
                val sessions = engine.request("GET", "/api/terminals").getJSONArray("sessions")
                val row = (0 until sessions.length()).asSequence().map { sessions.getJSONObject(it) }
                    .firstOrNull { it.opt("id") == id } ?: error("The retained terminal session is unavailable.")
                val summary = recordedTerminal(row)
                if (!current()) return
                val screen = engine.terminalScreen(id)
                if (!current()) return
                run.terminal = summary
                run.output = screen
                if (summary.done) break
                delay(350)
            }
        } catch (cancelled: CancellationException) { throw cancelled }
        catch (failure: Exception) { if (current()) run.notice = failure.message.orEmpty() }
        finally { run.reading = false }
    }

    var sending by mutableStateOf(false)
    var error by mutableStateOf("")
    var mutation = 0
    val running get() = sending || turns.any { it.status == "running" }
}
internal data class RecordedRunTerminal(val id: String, val state: String, val exit: Int?) {
    val done get() = exit != null || state == "finished" || state == "failed"
    val status get() = exit?.let { "Process exit $it" } ?: state
}

private fun recordedTerminal(row: JSONObject): RecordedRunTerminal {
    val id = row.opt("id") as? String
    val state = row.opt("state") as? String
    val exit = debugInteger(row.opt("exit_code"))?.toInt()
    require(id != null && id.matches(Regex("[A-Za-z0-9_-]{1,128}")) &&
        state in setOf("needs-you", "working", "idle", "finished", "failed") &&
        row.has("exit_code") && (row.isNull("exit_code") || exit != null)) {
        "The terminal result does not match the recorded Run protocol."
    }
    return RecordedRunTerminal(id, checkNotNull(state), exit)
}

internal class RecordedFenceRun(val buffer: DocumentBuffer, val line: Int, val current: () -> Boolean) {
    var terminal by mutableStateOf<RecordedRunTerminal?>(null)
    var output by mutableStateOf("")
    var notice by mutableStateOf("")
    var busy by mutableStateOf(true)
    var reading by mutableStateOf(false)
}

internal data class RecordedRunActions(
    val result: RecordedFenceRun?,
    val start: (Int) -> Unit,
    val refresh: (RecordedFenceRun) -> Unit,
)
internal data class RecordedConstructionValue(val kind: String, val executable: Boolean, val bytes: Long, val text: String?, val status: String)
internal data class RecordedConstructionChange(val path: String, val before: RecordedConstructionValue?, val after: RecordedConstructionValue?)
internal data class RecordedConstructionReceipt(val document: String, val operation: String, val changes: List<RecordedConstructionChange>)

internal class RecordedConstructionReader(
    private val engine: Engine, private val buffer: DocumentBuffer, private val reading: AgentFenceReading,
    private val ownerCurrent: () -> Boolean,
) {
    private val root = engine.sourceRootAddress
    private val workspace = engine.sourceWorkspaceID

    fun current(line: Int): Boolean = ownerCurrent() && reading.current() && buffer.readOnly &&
        buffer.text == reading.source && buffer.inlineMarkup.source == reading.source &&
        line in buffer.inlineMarkup.debug.declaredPatches &&
        engine.sourceRootAddress == root && engine.sourceWorkspaceID == workspace

    suspend fun read(line: Int, disclosureCurrent: () -> Boolean): RecordedConstructionReceipt? {
        fun active() = disclosureCurrent() && current(line)
        if (!active()) return null
        check(root != null && workspace != null) { "Refresh Files before reviewing the captured construction." }
        val hash = withContext(Dispatchers.Default) { debugHash(reading.source) }
        if (!active()) return null
        val reply = engine.request("POST", reading.endpoint("fence-construction"),
            JSONObject().put("side", reading.side).put("source_hash", hash).put("opening_line", line))
        if (!active()) return null
        val receipt = withContext(Dispatchers.Default) {
            check(debugInteger(reply.opt("version")) == 1L && reply.opt("read_only") == true &&
                reply.opt("turn") == reading.turn && reply.opt("side") == reading.side &&
                reply.opt("source_hash") == hash && debugInteger(reply.opt("opening_line")) == line.toLong() &&
                reply.opt("workspace") == root && reply.opt("workspace_id") == workspace) {
                "The engine returned a reading for a different recorded patch or workspace."
            }
            recordedConstructionReceipt(reply)
        }
        return receipt.takeIf { active() }
    }
}

private fun recordedConstructionReceipt(reply: JSONObject): RecordedConstructionReceipt {
    val hex = Regex("[a-f0-9]{64}")
    val invalid = "The captured construction comparison is incomplete."
    fun identity(row: JSONObject, key: String): String {
        val value = row.opt(key) as? String
        require(value != null && value.length in 1..1024) { invalid }
        return value
    }
    val document = identity(reply, "document")
    val operation = identity(reply, "operation_id")
    identity(reply, "entry")
    require(debugInteger(reply.opt("opening_byte"))?.let { it >= 0 } == true &&
        (reply.opt("completed_fingerprint") as? String)?.matches(hex) == true) { invalid }
    val rows = reply.getJSONArray("changes")
    require(rows.length() in 1..128) { invalid }
    var remaining = 64 * 1024
    fun value(raw: Any): RecordedConstructionValue? {
        if (raw === JSONObject.NULL) return null
        require(raw is JSONObject) { invalid }
        val kind = raw.opt("kind") as? String
        val executable = raw.opt("executable") as? Boolean
        val bytes = debugInteger(raw.opt("bytes"))
        val checksum = raw.opt("sha256") as? String
        val status = raw.opt("text_status") as? String
        val text = raw.opt("text") as? String
        require(kind != null && kind in setOf("file", "symlink") && executable != null &&
            bytes != null && bytes >= 0 && checksum?.matches(hex) == true && status != null) { invalid }
        if (status == "exact") {
            require(text != null && bytes <= 16 * 1024 && bytes <= remaining &&
                text.toByteArray(Charsets.UTF_8).size.toLong() == bytes && debugHash(text) == checksum) { invalid }
            remaining -= bytes.toInt()
        } else {
            require(status in setOf("over_limit", "non_utf8") && raw.has("text") && raw.isNull("text")) { invalid }
        }
        return RecordedConstructionValue(kind, executable, bytes, text, status)
    }
    val seen = mutableSetOf<String>()
    val changes = List(rows.length()) { index ->
        val row = rows.getJSONObject(index)
        val path = identity(row, "path")
        val before = value(row.get("before"))
        val after = value(row.get("after"))
        require(seen.add(path) && (before != null || after != null)) { invalid }
        RecordedConstructionChange(path, before, after)
    }
    return RecordedConstructionReceipt(document, operation, changes)
}
@OptIn(ExperimentalLayoutApi::class)
@Composable fun AgentPane(engine: Engine?, session: AgentSession, workspace: Workspace) {
    val scope = workspace.viewModelScope
    val running = session.running
    LaunchedEffect(engine) {
        if (engine == null) { session.error = "Connect to a workspace server to use the agent."; return@LaunchedEffect }
        while (isActive) {
            try { val mutation = session.mutation; val latest = engine.turns(); if (mutation == session.mutation && !session.sending) session.turns = latest }
            catch (cancelled: CancellationException) { throw cancelled }
            catch (failure: Exception) { session.error = "${failure.message} Check subscription sign-in on the workspace server." }
            delay(1000)
        }
    }
    Column(Modifier.fillMaxSize()) {
        Text("Workspace agent · Hickory Python only", style = MaterialTheme.typography.titleSmall, modifier = Modifier.padding(12.dp))
        LazyColumn(Modifier.weight(1f).fillMaxWidth(), contentPadding = PaddingValues(10.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            if (session.turns.isEmpty()) item {
                Text("What shall we work on?", style = MaterialTheme.typography.titleLarge)
                Text("Write a literate document below. Code and explanations belong together.")
            }
            items(session.turns, key = { it.id }) { turn -> Column {
                Text("You", style = MaterialTheme.typography.labelMedium)
                Box(Modifier.fillMaxWidth().padding(start = 24.dp)) {
                    Surface(shape = androidx.compose.foundation.shape.RoundedCornerShape(12.dp), color = MaterialTheme.colorScheme.surfaceContainer) {
                        AgentDocument(session, session.document("${turn.id}-user", turn.prompt), session.fenceReading(turn, false), workspace)
                    }
                }
                Text("Agent · ${turn.status}", style = MaterialTheme.typography.labelMedium)
                AgentDocument(session, session.document("${turn.id}-agent", turn.answer ?: turn.error ?: if (turn.status == "running") "Working…" else ""), session.fenceReading(turn, true), workspace)
            } }
        }
        if (session.error.isNotEmpty()) Text(session.error, color = MaterialTheme.colorScheme.error, fontSize = 12.sp, modifier = Modifier.padding(6.dp))
        HorizontalDivider()
        Column(Modifier.fillMaxWidth().padding(12.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            Text("Your message · literate document", style = MaterialTheme.typography.labelMedium)
            LanguageEditor(session.draft, null, Modifier.fillMaxWidth().height(140.dp))
            FlowRow(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                AgentChoice("Provider", session.provider, listOf("codex-subscription" to "ChatGPT subscription", "claude-subscription" to "Claude Code subscription", "grok-subscription" to "Grok subscription"), !running) {
                    session.provider = it; session.model = ""; session.effort = ""
                }
                AgentChoice("Effort", session.effort, listOf("" to "Default") + listOf("low", "medium", "high", "xhigh", "max").map { it to it }, !running && session.provider != "grok-subscription") { session.effort = it }
            }
            OutlinedTextField(session.model, { session.model = it }, enabled = !running, label = { Text("Model · provider default") }, singleLine = true, modifier = Modifier.fillMaxWidth())
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
            TextButton(enabled = engine != null && !running && session.prompt.isNotBlank(), onClick = {
                val submitted = session.prompt
                session.sending = true; session.error = ""; session.mutation++
                scope.launch {
                    try {
                        val id = engine!!.send(submitted, session.turns.lastOrNull()?.id, session.provider, session.model, session.effort,simpleAgentContext(workspace))
                        session.turns = session.turns.filter { it.id != id } + AgentTurn(id, submitted, null, null, "running")
                        if (session.prompt == submitted) session.prompt = ""
                    } catch (cancelled: CancellationException) { throw cancelled }
                    catch (failure: Exception) { session.error = "${failure.message} The request was not retried. Refresh history before sending again." }
                    finally { session.sending = false; session.mutation++ }
                }
            }) { Text("Send") }
            TextButton(enabled = running, onClick = { scope.launch {
                try { engine?.stop() } catch (cancelled: CancellationException) { throw cancelled }
                catch (failure: Exception) { session.error = "${failure.message} Refresh history and try Stop again." }
            } }) { Text("Stop") }
            }
            Text("Sign in on the workspace server with " + when (session.provider) { "claude-subscription" -> "claude auth login"; "grok-subscription" -> "grok login"; else -> "codex login" } + ". Uses your subscription; no API fallback.", style = MaterialTheme.typography.labelSmall)
        }
    }
}

@Composable private fun AgentDocument(session: AgentSession, buffer: DocumentBuffer, reading: AgentFenceReading?, workspace: Workspace) {
    key(buffer) {
        val engine = workspace.engine
        val epoch = workspace.sourceNavigationEpoch
        val scope = workspace.viewModelScope
        val construction = if (reading != null && engine != null) {
            remember(buffer, engine, epoch, engine.sourceRootAddress, engine.sourceWorkspaceID, reading.turn, reading.side, reading.source) {
                RecordedConstructionReader(engine, buffer, reading) {
                    workspace.engine === engine && workspace.sourceNavigationEpoch == epoch
                }
            }
        } else null
        val actions = if (reading != null && engine != null) RecordedRunActions(
            session.recordedRun(buffer),
            start = { line -> scope.launch {
                session.runRecordedFence(engine, buffer, reading, line) {
                    workspace.engine === engine && workspace.sourceNavigationEpoch == epoch
                }
            } },
            refresh = { run -> scope.launch { session.refreshRecordedRun(engine, run) } },
        ) else null
        LanguageEditor(
            buffer, session.debugger.takeIf { reading != null },
            Modifier.fillMaxWidth().height((buffer.text.lines().size * 22 + 36).coerceIn(100, 420).dp),
            recorded = reading,
            reviewPatch = if (reading == null) null else { line -> scope.launch { workspace.agentProposals.prepareRecordedPatch(workspace, reading, line) } },
            recordedRun = actions,
            recordedConstruction = construction,
        )
    }
}
@Composable private fun AgentChoice(label: String, selected: String, choices: List<Pair<String, String>>, enabled: Boolean, change: (String) -> Unit) {
    var expanded by remember { mutableStateOf(false) }
    Box {
        OutlinedButton(enabled = enabled, onClick = { expanded = true }) { Text("$label: ${choices.firstOrNull { it.first == selected }?.second ?: selected}") }
        DropdownMenu(expanded, { expanded = false }) { choices.forEach { (value, title) ->
            DropdownMenuItem(text = { Text(title) }, onClick = { expanded = false; change(value) })
        } }
    }
}
