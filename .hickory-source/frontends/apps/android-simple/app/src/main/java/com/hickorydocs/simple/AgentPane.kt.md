<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Match the explicit workspace agent conversation in Compose

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Polling and prompt state are confined to the agent pane. Cancelling its composition cancels polling; it never cancels or retries a dispatched agent turn implicitly. A failed provider leaves the editor operational.


Initial Send and edit refresh carry exact editor bytes for guarded work on the host.
The explicit empty reading projection excludes those bytes from model input. The
composer points to the message as the place to include excerpts.

<ui:copy id="simple-716e739687-0">
package com.hickorydocs.simple

import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.clickable
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.type
import androidx.compose.ui.input.key.isShiftPressed
import androidx.compose.ui.input.key.isCtrlPressed
import androidx.compose.ui.input.key.isAltPressed
import androidx.compose.ui.input.key.isMetaPressed
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.json.JSONArray
import org.json.JSONObject

data class AgentFenceReading(val turn: String, val side: String, val source: String, val current: () -> Boolean) {
    fun endpoint(action: String): String {
        val id = java.net.URLEncoder.encode(turn, "UTF-8").replace("+", "%20")
        return "/api/docs/workspace/agent/turns/$id/$action"
    }
}

data class AgentTool(val name: String, val input: String? = null, val ok: Boolean? = null, val result: String? = null)
data class AgentActivity(val phase: String = "Starting…", val answer: String? = null, val tools: List<AgentTool> = emptyList(), val tokens: Long = 0)

internal fun liveAgentContext(snapshot: JSONObject): JSONObject =
    snapshot.put("live", true).put("projection", JSONObject()
        .put("version", 1).put("documents", JSONArray()).put("observations", JSONArray()))

internal fun formattedToolJSON(source: String): String {
    if (source.length > 262144) return source
    val first = source.trimStart().firstOrNull() ?: return source
    if (first !in "{[\"-0123456789" && source.trim() !in listOf("true", "false", "null")) return source
    try { val tokens = org.json.JSONTokener(source); tokens.nextValue(); if (tokens.nextClean() != '\u0000') return source } catch (_: Exception) { return source }
    val output = StringBuilder(); var depth = 0; var quoted = false; var escaped = false
    fun newline() { output.append('\n').append("  ".repeat(depth.coerceAtLeast(0))) }
    source.forEachIndexed { index, char ->
        if (quoted) { output.append(char); if (escaped) escaped = false else if (char == '\\') escaped = true else if (char == '"') quoted = false }
        else if (!char.isWhitespace()) when (char) {
            '"' -> { quoted = true; output.append(char) }
            '{', '[' -> {
                output.append(char)
                var nextIndex = index + 1; while (nextIndex < source.length && source[nextIndex].isWhitespace()) nextIndex++
                val next = source.getOrNull(nextIndex)
                if (next != if (char == '{') '}' else ']') { depth++; if (depth > 64) return source; newline() }
            }
            '}', ']' -> {
                var previousIndex = index - 1; while (previousIndex >= 0 && source[previousIndex].isWhitespace()) previousIndex--
                val previous = source.getOrNull(previousIndex)
                if (previous != if (char == '}') '{' else '[') { depth--; newline() }
                output.append(char)
            }
            ',' -> { output.append(char); newline() }
            ':' -> output.append(": ")
            else -> output.append(char)
        }
    }
    return output.toString()
}

class AgentSession {
    data class BranchEdit(val turn: String, val user: Boolean, val source: String, val request: String = java.util.UUID.randomUUID().toString().replace("-", ""))
    var branchEdit by mutableStateOf<BranchEdit?>(null)
    var openedTip: String? = null
    var conversationKey: String? = null
    fun sameConversation(id: String): Boolean {
        if (branchEdit != null) return false
        val selected = turns.firstOrNull { it.id == tip }; val requested = turns.firstOrNull { it.id == id }
        return openedTip == id || tip == id || (selected?.session ?: conversationKey) != null && (selected?.session ?: conversationKey) == requested?.session
    }
    fun working(id: String? = tip): Boolean {
        if (id == tip && sending) return true
        val selected = turns.firstOrNull { it.id == id }
        return turns.any { it.status == "running" && (it.id == id || selected?.session != null && it.session == selected.session) && waitingTurn != it.id }
    }
    var waitingTurn by mutableStateOf<String?>(null)
    var totals by mutableStateOf<AgentTotals?>(null)
    val activity = mutableStateMapOf<String, AgentActivity>()
    private val earlyEvents = mutableListOf<JSONObject>()
    var stopPending by mutableStateOf(false)
    val canStop get() = transcript.any { !it.second && it.first.status == "running" }

    fun receive(message: JSONObject) {
        val id = message.optString("run_id")
        if (turns.none { it.id == id && it.status == "running" }) {
            if (sending && earlyEvents.size < 128) earlyEvents.add(message)
            return
        }
        val event = message.optJSONObject("event") ?: return
        val previous = activity[id] ?: AgentActivity()
        val name = event.optString("name")
        val next = when (event.optString("kind")) {
            "thinking" -> previous.copy(phase = "Waiting for the model…")
            "token" -> previous.copy(phase = "Receiving model output…")
            "response_complete" -> previous.copy(answer = event.optString("text"))
            "turn_usage" -> {
                val usage = event.optJSONObject("total_usage") ?: return
                previous.copy(tokens = listOf("input_tokens", "output_tokens", "cache_read_input_tokens", "cache_creation_input_tokens").sumOf { usage.optLong(it) })
            }
            "tool_started" -> previous.copy(phase = name, tools = previous.tools + AgentTool(name, event.optString("data")))
            "tool_finished" -> {
                val index = previous.tools.indexOfLast { it.name == name && it.ok == null }
                val tools = previous.tools.toMutableList()
                if (index >= 0) tools[index] = tools[index].copy(ok = event.optBoolean("ok"), result = event.optString("text").take(262144))
                previous.copy(phase = "Preparing the next step…", tools = tools)
            }
            else -> return
        }
        if (previous != next) activity[id] = next
    }

    internal suspend fun stop(engine: Engine?) {
        if (engine == null || !canStop || stopPending) return
        stopPending = true
        try { engine.stop() }
        catch (cancelled: CancellationException) { throw cancelled }
        catch (failure: Exception) {
            error = "${failure.message} Refresh history and try Stop again."
            stopPending = false
        }
    }
    var turns by mutableStateOf<List<AgentTurn>>(emptyList())
    data class Selection(val id: String?, val userFork: Boolean? = null)
    var selection by mutableStateOf<Selection?>(null)
    val tip: String? get() = selection.let { if (it == null) turns.lastOrNull()?.id else it.id }
    var historyLoaded by mutableStateOf(false)
    var messageForks by mutableStateOf(false)
    var historyError by mutableStateOf("")
    val transcript: List<Pair<AgentTurn, Boolean>> get() {
        val byId = turns.associateBy { it.id }
        val seen = mutableSetOf<String>()
        val rows = mutableListOf<Pair<AgentTurn, Boolean>>()
        var id = tip
        var userOnly = selection?.userFork == true
        while (id != null && seen.add(id)) {
            val turn = byId[id] ?: break
            rows.add(turn to userOnly)
            id = turn.parentID
            userOnly = turn.parentIsUser
        }
        return rows.asReversed()
    }
    fun select(id: String?, fork: Boolean? = null) {
        if (running) return
        if (fork != null && !messageForks) return
        if (prompt.isNotEmpty()) { error = "Send or clear your current draft before switching conversations."; return }
        selection = Selection(id, fork)
        error = ""
    }
    fun resetHistory() {
        turns = emptyList(); selection = null; historyLoaded = false; messageForks = false; historyError = ""; liveTurn = null; sending = false; mutation++; earlyEvents.clear()
    }
    var provider by mutableStateOf("codex-subscription")
    var model by mutableStateOf("")
    var effort by mutableStateOf("")
    val draft = DocumentBuffer(FileNode("Message", "conversation/draft.md", false, emptyList(), null, null), FileText("", ""))
    var prompt: String
        get() = draft.text
        set(value) {
            if (draft.text == value) return
            draft.state.edit { replace(0, length, value) }
            draft.advanceSourceRevision()
        }
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
</ui:copy>

## Admit an explicit Run without giving a late reply the editor

The current card carries its source bytes and exact opener. A changing message,
workspace or original owner retires the reading even while a request is in
flight. Terminal reads only follow the admitted id.

<ui:copy id="simple-android-recorded-run-controller">
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

</ui:copy>

## Clear only the acknowledged composer revision

Send captures the live editor revision before suspending. An edit followed by
Undo can reproduce the same source bytes while representing new unsent work.
The visible editor retains that revision, along with text edited during the
request or an active composition. Only an unchanged, settled draft clears on
acknowledgment. A refusal leaves the draft available for inspection.

<ui:copy id="simple-android-agent-send">
    internal suspend fun send(engine: Engine, current: () -> Boolean = { true }, captureContext: () -> JSONObject) {
        if (!current() || sending) return
        val edit = branchEdit
        if (edit != null && !edit.user) {
            val text = prompt
            sending = true; mutation++; error = ""
            try {
                val turn = decodedAgentTurn(engine.request("POST", "/api/docs/workspace/agent/branch", JSONObject().put("turn", edit.turn).put("source_hash", debugHash(edit.source)).put("answer", text).put("request_id", edit.request)))
                if (!current()) return
                turns = turns.filterNot { it.id == turn.id } + turn; selection = Selection(turn.id); openedTip = turn.id; conversationKey = turn.session
                if (prompt == text) { prompt = ""; branchEdit = null }
                else branchEdit = BranchEdit(turn.id, false, text)
            } catch (cancelled: CancellationException) { throw cancelled }
            catch (failure: Exception) { if (current()) error = "${failure.message} Your edited answer is retained. Refresh history before retrying." }
            finally { sending = false; mutation++ }
            return
        }
        val submitted = prompt
        if (running || submitted.isBlank()) return
        val revision = draft.sourceRevision
        val parent = tip
        val fork = selection?.userFork
        val backend = turns.firstOrNull { it.id == parent }?.backend
        if (backend != null && backend != "python") {
            error = "This conversation uses a different agent. Start a new conversation to send with Hickory Python."
            return
        }
        if (fork != null && !messageForks) {
            error = "This workspace server does not support message branches. Update the server before sending."
            return
        }
        sending = true
        error = ""
        mutation++
        try {
            val id = engine.send(submitted, parent, provider, model, effort, liveAgentContext(captureContext()), fork)
            if (!current()) return
            liveTurn = id
            conversationKey = if (fork == null) conversationKey ?: turns.firstOrNull { it.id == parent }?.session else null
            selection = Selection(id); openedTip = id; branchEdit = null
            turns = turns.filter { it.id != id } + AgentTurn(id, submitted, null, null, "running", parentID = parent, parentIsUser = fork == true)
            earlyEvents.filter { it.optString("run_id") == id }.forEach(::receive)
            if (draft.sourceRevision == revision && prompt == submitted && draft.state.composition == null) prompt = ""
        } catch (cancelled: CancellationException) { throw cancelled }
        catch (failure: Exception) {
            if (current()) error = "${failure.message} The request was not retried. Refresh history before sending again."
        } finally {
            if (current()) {
                earlyEvents.clear()
                sending = false
                mutation++
            }
        }
    }

    var sending by mutableStateOf(false)
    var error by mutableStateOf("")
    var mutation = 0
    var liveTurn by mutableStateOf<String?>(null)
    val running get() = sending || turns.any { it.status == "running" }
}
</ui:copy>

## Retain the exact result of an explicit Run

Run uses the immutable message's side, source hash and fence opener. A receipt
can reach the gutter only while its card, engine and workspace still own that
reading. Read the admitted terminal by id and let its own exit status settle.
Each card retains its latest result. Closing the floating view leaves that
result available when the person reopens it.

<ui:copy id="simple-android-recorded-run">
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
</ui:copy>

## Inspect the captured construction for one declaration

Keep the opener, exact message bytes, server and physical workspace together.
A disclosure that closed or changed cards cannot accept a late receipt. The
compiler supplies local Before and After even when a later operation rewrote
the same output. Absent entries and omitted previews retain their distinct
meanings in the comparison.

<ui:copy id="simple-android-recorded-construction">
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
</ui:copy>

## Reuse the document renderer inside message cards

LanguageEditor accepts the same Markdown buffer for previous messages and the
composer. A read-only history buffer protects inline edits as well as typing.
Both recorded sides can use their code fence's gutter. The session retains one
Agent debugger even when a scrolling message card leaves composition. A
streaming answer and the editable composer have no recorded launch identity.
Under the keyboard, the composer action measures before the scrollable editor.
Send becomes Stop once the engine identifies the running turn. The same editor
retains the draft through viewport changes. The workspace keeps answering
matching capture requests while its turn runs, even if the reader switches back
to the code pane. The activity socket carries progress and reported usage while
history confirms the final source and its fence actions.

Render a depth-indented tree from the same workspace history used by Send.
Expand state is local to the sidebar, never a change to the recorded turns.

<ui:copy id="simple-android-agent-history">
@Composable internal fun AgentHistoryPane(session: AgentSession, open: (String?) -> Unit) {
    val byId = session.turns.associateBy { it.id }
    val children = session.turns.groupBy { turn -> turn.parentID?.takeIf(byId::containsKey) }
    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(5.dp)) {
        OutlinedButton(onClick = { open(null) }, modifier = Modifier.semantics { contentDescription = "New conversation" }) { Text("✎") }
        if (session.historyError.isNotEmpty()) Text(session.historyError, color = MaterialTheme.colorScheme.error)
        if (session.turns.isEmpty() && session.historyError.isEmpty())
            Text(if (session.historyLoaded) "No conversations yet." else "Loading conversations…")
        AgentHistoryRows(children[null].orEmpty(), children, byId, emptySet(), session, open)
    }
}

@Composable private fun AgentHistoryRows(
    turns: List<AgentTurn>, children: Map<String?, List<AgentTurn>>, byId: Map<String, AgentTurn>,
    seen: Set<String>, session: AgentSession, open: (String?) -> Unit
) {
    turns.filterNot { it.id in seen }.forEach { turn ->
        key(turn.id) {
            var expanded by remember { mutableStateOf(false) }
            val descendants = children[turn.id].orEmpty()
            val source = turn.parentID?.let(byId::get)
            val fork = source?.session != null && turn.session != null && source.session != turn.session
            val title = turn.prompt.trim().lineSequence().firstOrNull()?.ifBlank { "Conversation" } ?: "Conversation"
            Row(Modifier.fillMaxWidth()) {
                if (descendants.isNotEmpty()) Text(if (expanded) "▾" else "▸",
                    Modifier.clickable { expanded = !expanded }.padding(4.dp)
                        .semantics { contentDescription = if (expanded) "Hide replies" else "Show replies" })
                Text(if (session.working(turn.id)) "●" else "○", Modifier.semantics { contentDescription = if (session.working(turn.id)) "Working" else "Waiting for input" }.padding(4.dp))
                Text(if (fork) "Branch from ${if (turn.parentIsUser) "user" else "agent"} · $title" else title,
                    Modifier.weight(1f).clickable { open(turn.id) }.padding(4.dp), maxLines = 2)
            }
            if (expanded) Column(Modifier.padding(start = 8.dp)) {
                AgentHistoryRows(descendants, children, byId, seen + turn.id, session, open)
            }
        }
    }
}
</ui:copy>

History and message branches allocate a conversation session per tab. Selection,
draft, settings and recorded editors remain with that tab while independent
observers attach durable history and live activity. The server still admits runs;
a pane's Stop and usage only describe its selected ancestry. Put branch controls
after the message and settings beneath the composer, so split panes stay clear.

Branch copies are composer drafts, never editable recorded cards. An assistant-answer save creates an immutable human-edited branch with a request ID and original-source hash; it performs no inference. Tool inputs/results retain raw payloads and display formatted JSON on expansion. Conversation reuse and activity indicators are independent of file identity.

<ui:copy id="simple-android-agent-pane">
@Composable internal fun ObserveSimpleAgent(workspace: Workspace) {
    val engine = workspace.engine
    workspace.allAgentSessions.forEach { session -> key(session) { ObserveConversation(workspace, engine, session) } }
}

@Composable private fun ObserveConversation(workspace: Workspace, engine: Engine?, session: AgentSession) {
    LaunchedEffect(engine, session) {
        if (engine == null) { session.historyError = "Connect to a workspace server to use the agent."; return@LaunchedEffect }
        launch {
            while (isActive && workspace.engine === engine) {
                engine.agentEvents().collect { event -> if (workspace.engine === engine) session.receive(event) }
                delay(2000)
            }
        }
        while (isActive) {
            try {
                val mutation = session.mutation
                val (latest, totals, messageForks) = engine.agentHistory()
                if (workspace.engine !== engine) return@LaunchedEffect
                if (mutation == session.mutation && !session.sending) {
                    session.turns = latest
                    if (session.branchEdit == null) latest.firstOrNull { it.id == session.tip }?.session?.let { session.conversationKey = it }
                    session.historyLoaded = true
                    session.messageForks = messageForks
                    session.historyError = ""
                    session.totals = totals
                    if (!session.canStop) session.stopPending = false
                    val own = session.liveTurn
                    val active = latest.firstOrNull { it.status == "running" }
                    if (active != null) {
                        val state = engine.request("GET", "/api/docs/workspace/agent/python")
                        val capture = state.optJSONObject("edits")?.opt("captureRequest") as? String
                        val changes = state.optJSONObject("edits")?.optJSONArray("changes")
                        session.waitingTurn = active.id.takeIf { state.opt("turn") == active.id && changes != null && (0 until changes.length()).any { changes.optJSONObject(it)?.optString("status") == "pending" } }
                        if (capture != null && state.opt("turn") == own && isActive && workspace.engine === engine &&
                            mutation == session.mutation && own == session.liveTurn) {
                            engine.request("POST", "/api/docs/workspace/agent/python/review",
                                JSONObject().put("capture_request", capture)
                                    .put("context", liveAgentContext(simpleAgentContext(workspace))))
                        }
                    } else { session.waitingTurn = null; if (own != null) session.liveTurn = null }
                }
            }
            catch (cancelled: CancellationException) { throw cancelled }
            catch (failure: Exception) { if (isActive && workspace.engine === engine) session.historyError = "Cannot read agent history: ${failure.message}" }
            delay(1000)
        }
    }
}

@Composable fun AgentPane(engine: Engine?, session: AgentSession, workspace: Workspace) {
    val scope = workspace.viewModelScope
    val running = if (session.branchEdit?.user == false) session.sending else session.running
    Column(Modifier.fillMaxSize()) {
        LazyColumn(Modifier.weight(1f).fillMaxWidth(), contentPadding = PaddingValues(10.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            if (session.transcript.isEmpty()) item {
                Text("What shall we work on?", style = MaterialTheme.typography.titleLarge)
                Text("Write a literate document below. Code and explanations belong together.")
            }
            items(session.transcript, key = { it.first.id }) { (turn, userOnly) -> Column {
                Text("You", style = MaterialTheme.typography.labelMedium)
                Box(Modifier.fillMaxWidth().padding(start = 24.dp)) {
                    Surface(shape = androidx.compose.foundation.shape.RoundedCornerShape(12.dp), color = MaterialTheme.colorScheme.surfaceContainer) {
                        AgentDocument(session, session.document("${turn.id}-user", turn.prompt), session.fenceReading(turn, false), workspace)
                    }
                }
                if (session.messageForks && (turn.backend == null || turn.backend == "python"))
                    TextButton(onClick = { workspace.openConversation(turn.id, true, session) }) { Text("Branch from user message") }
                if (!userOnly) {
                val activity = session.activity[turn.id]
                Text(turn.model?.let { "Agent · $it" } ?: "Agent", style = MaterialTheme.typography.labelMedium)
                if (activity != null && activity.tools.isNotEmpty()) {
                    var expanded by remember { mutableStateOf(false) }
                    TextButton(onClick = { expanded = !expanded }) { Text("Tool calls (${activity.tools.size})") }
                    if (expanded) activity.tools.forEach { tool ->
                        var detail by remember { mutableStateOf(false) }
                        TextButton(onClick = { detail = !detail }) {
                            Text(tool.name + " · " + when (tool.ok) {
                                true -> "Completed"
                                false -> "Failed"
                                null -> if (turn.status == "running") "Running" else "Outcome unavailable"
                            })
                        }
                        if (detail) {
                            tool.input?.let { Text("Input", fontSize = 11.sp); SelectionContainer { Text(formattedToolJSON(it), fontSize = 12.sp, fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace) } }
                            tool.result?.let { Text("Result", fontSize = 11.sp); SelectionContainer { Text(formattedToolJSON(it), fontSize = 12.sp, fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace) } }
                        }
                    }
                }
                if (turn.status == "running") Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    CircularProgressIndicator(Modifier.size(14.dp), strokeWidth = 2.dp)
                    Text(if (session.stopPending) "Stopping…" else activity?.phase ?: "Starting…", fontSize = 12.sp)
                }
                val answer = turn.answer ?: turn.error ?: activity?.answer
                if (answer != null) {
                    if (turn.status != "running" && listOf("```", "~~~", "xmlns:", "<hick").any(answer::contains)) {
                        AgentDocument(session, session.document("${turn.id}-agent", answer), session.fenceReading(turn, true), workspace)
                    } else AgentProse(answer)
                }
                if (session.messageForks && turn.answer != null && (turn.backend == null || turn.backend == "python"))
                    TextButton(onClick = { workspace.openConversation(turn.id, false, session) }) { Text("Branch from assistant message") }
                }
            } }
        }
        if (session.error.isNotEmpty()) Text(session.error, color = MaterialTheme.colorScheme.error, fontSize = 12.sp, modifier = Modifier.padding(6.dp))
        session.selection?.userFork?.let { Text("New branch from ${if (it) "user" else "assistant"} message. ${if (session.branchEdit?.user == false) "Save the edited answer to create it." else "Send to start it."}", fontSize = 12.sp) }
        HorizontalDivider()
        Column(Modifier.fillMaxWidth().padding(12.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            Column(
                Modifier.weight(1f, fill = false).fillMaxWidth().verticalScroll(rememberScrollState()),
                verticalArrangement = Arrangement.spacedBy(6.dp),
            ) {
                Text(session.branchEdit?.let { "Edit ${if (it.user) "user" else "assistant"} message · The original stays unchanged." } ?: "Your message · Paste any source excerpts you want the model to read.", style = MaterialTheme.typography.labelMedium)
                LanguageEditor(session.draft, null, Modifier.fillMaxWidth().height(140.dp).onPreviewKeyEvent { event ->
                    if (event.key == Key.Enter && event.isShiftPressed && !event.isCtrlPressed && !event.isAltPressed && !event.isMetaPressed && session.draft.state.composition == null) {
                        if (event.type == KeyEventType.KeyDown && engine != null && session.draft.state.composition == null) scope.launch(start = CoroutineStart.UNDISPATCHED) {
                            session.send(engine, current = { workspace.engine === engine }) { simpleAgentContext(workspace) }
                        }
                        true
                    } else false
                })
            }
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
                if (session.canStop) {
                    WithDisabledReason(if (session.stopPending) "A stop request is in progress." else null) { available -> FilledIconButton(enabled = available, onClick = { scope.launch { session.stop(engine) } }) {
                        Text("■", Modifier.semantics { contentDescription = "Stop" })
                    } }
                } else {
                    WithDisabledReason(if (engine == null) "Connect to a workspace server first." else if (running) "Wait for the current agent turn to finish." else if (session.branchEdit?.user != false && !(session.prompt.isNotBlank())) "Enter a message first." else null) { available -> FilledIconButton(enabled = available, onClick = {
                        if (engine != null) scope.launch(start = CoroutineStart.UNDISPATCHED) {
                            session.send(engine, current = { workspace.engine === engine }) { simpleAgentContext(workspace) }
                        }
                    }) { Text("↑", Modifier.semantics { contentDescription = if (session.branchEdit?.user == false) "Save assistant branch" else "Send" }, fontSize = 22.sp) } }
                }
            }
        }
        Row(Modifier.fillMaxWidth().padding(horizontal = 8.dp), horizontalArrangement = Arrangement.SpaceBetween) {
            AgentStatus(session, engine, Modifier.weight(1f))
            TextButton(onClick = { workspace.openConversation() }) { Text("New conversation") }
        }
    }
}

</ui:copy>

## Bound long messages without changing their source

A long message scrolls within its document card; shorter messages retain compact
heights. Provider and effort choices use floating menus, so opening one does not
move the conversation. Changing a selection retains the draft buffer.

<ui:copy id="simple-android-agent-message-controls">
@Composable private fun AgentProse(text: String) {
    val headingPattern = remember { Regex("^(#{1,6})[ \t]+(.+)$") }
    SelectionContainer {
        Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
            text.lineSequence().forEach { row ->
                val match = headingPattern.matchEntire(row)
                if (match == null) Text(row)
                else Text(match.groupValues[2], style = MaterialTheme.typography.titleMedium,
                    modifier = Modifier.semantics { heading() })
            }
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
            recorded = reading?.copy(current = { reading.current() && workspace.engine === engine && workspace.sourceNavigationEpoch == epoch }),
            reviewPatch = if (reading == null) null else { line -> scope.launch { workspace.agentProposals.prepareRecordedPatch(workspace, reading, line) } },
            recordedRun = actions,
            recordedConstruction = construction,
            patchCommentEngine = engine.takeIf { reading != null },
        )
    }
}
@Composable internal fun AgentStatus(session: AgentSession, engine: Engine?, modifier: Modifier = Modifier) {
    var expanded by remember { mutableStateOf(false) }
    var custom by remember { mutableStateOf(false) }
    var codexModels by remember(engine) { mutableStateOf(listOf("gpt-6.1-sol")) }
    LaunchedEffect(expanded, session.provider, engine) {
        if (!expanded || session.provider != "codex-subscription" || engine == null) return@LaunchedEffect
        try {
            val models = engine.request("GET", "/api/agents/codex-subscription/models").getJSONArray("models")
            if (models.length() > 0) {
                codexModels = (0 until models.length()).map(models::getString)
                if (session.model in codexModels) custom = false
            }
        } catch (cancelled: CancellationException) { throw cancelled }
        catch (_: Exception) { /* The built-in choices remain available. */ }
    }
    val (icon, models, login) = when (session.provider) {
        "claude-subscription" -> Triple("✳", listOf("sonnet", "opus", "haiku"), "claude auth login")
        "grok-subscription" -> Triple("◈", listOf("grok-build"), "grok login")
        else -> Triple("⌘", codexModels, "codex login")
    }
    val efforts = when (session.provider) {
        "claude-subscription" -> listOf("low", "medium", "high", "xhigh", "max")
        "grok-subscription" -> emptyList()
        else -> listOf("low", "medium", "high", "xhigh", "max")
    }
    Row(modifier, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        Box(Modifier.weight(1f, fill = false)) {
            Text("$icon ${session.model.ifEmpty { models.first() }} ◴ ${session.effort.ifEmpty { "Default" }}",
                fontSize = 11.sp, maxLines = 1, modifier = Modifier.semantics { contentDescription = "Agent settings" }
                    .clickable(role = androidx.compose.ui.semantics.Role.Button) {
                        custom = session.model.isNotEmpty() && session.model !in models
                        expanded = true
                    })
            DropdownMenu(expanded, { expanded = false }) {
                Column(Modifier.widthIn(max = 280.dp).padding(12.dp)) {
                    AgentChoice("Provider", session.provider, listOf("codex-subscription" to "Codex", "claude-subscription" to "Claude", "grok-subscription" to "Grok"), !session.running) {
                        session.provider = it; session.model = ""; session.effort = ""; custom = false
                    }
                    AgentChoice("Model", if (custom) "custom" else session.model, listOf("" to "Default (${models.first()})") + models.map { it to it } + ("custom" to "Custom model…"), !session.running) {
                        custom = it == "custom"
                        if (!custom) session.model = it
                    }
                    if (custom) WithDisabledReason(if (session.running) "Wait for the current agent turn to finish." else null) { available -> OutlinedTextField(session.model, { session.model = it }, enabled = available, label = { Text("Custom model") }, singleLine = true) }
                    AgentChoice("Effort", session.effort, listOf("" to if (efforts.isEmpty()) "Managed by provider" else "Default") + efforts.map { it to it }, !session.running && efforts.isNotEmpty()) { session.effort = it }
                    Text("Model presets depend on subscription access. Sign in on the workspace server with $login.", fontSize = 11.sp)
                }
            }
        }
        val known = session.transcript.filterNot { it.second }.mapNotNull { (turn, _) ->
            if (turn.status == "running") session.activity[turn.id]?.tokens else turn.tokens
        }
        Text(if (known.isEmpty() && session.transcript.isEmpty() && !session.sending) "0 tokens" else if (known.isEmpty()) "Tokens unavailable" else "${java.text.NumberFormat.getIntegerInstance().format(known.sum())} tokens", fontSize = 11.sp, maxLines = 1)
    }
}

@Composable private fun AgentChoice(label: String, selected: String, choices: List<Pair<String, String>>, enabled: Boolean, change: (String) -> Unit) {
    var expanded by remember { mutableStateOf(false) }
    Box {
        WithDisabledReason(if (!(enabled)) "Wait for the current agent turn to finish." else null) { available -> OutlinedButton(enabled = available, onClick = { expanded = true }) { Text("$label: ${choices.firstOrNull { it.first == selected }?.second ?: selected}") } }
        DropdownMenu(expanded, { expanded = false }) { choices.forEach { (value, title) ->
            DropdownMenuItem(text = { Text(title) }, onClick = { expanded = false; change(value) })
        } }
    }
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

## Review a recorded change from its gutter line

The workspace review store carries comments and replies independently of the
conversation. The quote, full hash, turn and line identify the retained patch
even when the files it describes have changed.

<ui:copy id="android-recorded-patch-comment">
@Composable internal fun RecordedPatchComment(engine: Engine, buffer: DocumentBuffer, reading: AgentFenceReading, line: Int, opening: Int) {
    val scope = rememberCoroutineScope()
    val source = reading.source
    val path = "hick-agent-message:///${reading.turn}-agent.md"
    val quote = remember(source, line) { source.split('\n').getOrNull(line) } ?: return
    var target by remember(source, line) { mutableStateOf<JSONObject?>(null) }
    var history by remember(source, line) { mutableStateOf<JSONObject?>(null) }
    var thread by remember(source, line) { mutableStateOf<String?>(null) }
    var draft by remember(source, line) { mutableStateOf("") }
    var notice by remember(source, line) { mutableStateOf("") }
    var busy by remember(source, line) { mutableStateOf(false) }
    fun current() = reading.current() && buffer.text == source
    suspend fun load() {
        val reply = engine.request("GET", "/api/reviews")
        if (current()) { history = reply; notice = "" }
    }
    LaunchedEffect(engine, buffer, source, line) {
        val hash = withContext(Dispatchers.Default) { debugHash(source) }
        if (!current()) return@LaunchedEffect
        target = JSONObject().put("kind","patch-line").put("path",path).put("quote",quote)
            .put("source_hash",hash).put("lines",org.json.JSONArray(listOf(line+1,line+1)))
            .put("relationship",JSONObject().put("turn",reading.turn).put("side","assistant").put("patch_opening_line",opening+1))
        try { load() }
        catch (error: kotlinx.coroutines.CancellationException) { throw error }
        catch (error: Exception) { notice = error.message.orEmpty() }
    }
    val events = history?.optJSONArray("events")
    val threads = (0 until (events?.length() ?: 0)).mapNotNull { index ->
        val event = events?.optJSONObject(index) ?: return@mapNotNull null
        val root = event.optJSONObject("target") ?: return@mapNotNull null
        event.takeIf {
            root.optString("kind") == "patch-line" && root.optString("path") == path &&
                root.optString("source_hash") == target?.optString("source_hash") &&
                root.optJSONArray("lines")?.optInt(0) == line + 1 && root.optString("quote") == quote
        }
    }
    Column(Modifier.padding(10.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Text("Patch line ${line+1}", style = MaterialTheme.typography.titleSmall)
        Text(quote, fontSize = 12.sp)
        for (root in threads) TextButton(onClick = { thread = root.getString("thread") }) { Text(root.getString("body").take(70)) }
        if (thread != null) {
            TextButton(onClick = { thread = null }) { Text("New thread") }
            for (index in 0 until (events?.length() ?: 0)) {
                val event = events?.optJSONObject(index) ?: continue
                if (event.optString("thread") == thread) Text(event.optString("body"))
            }
        }
        OutlinedTextField(draft, { draft = it }, label = { Text(if (thread == null) "Write a comment" else "Write a reply") }, maxLines = 4)
        Row {
            WithDisabledReason(if (busy) "An operation is already in progress." else if (target == null) "Select a comment target first." else if (history == null) "Wait for the comment history to load." else if (!(draft.isNotBlank())) "Enter a comment first." else null) { available -> TextButton(enabled = available, onClick = {
                if (!current()) return@TextButton
                val savedDraft = draft
                val selected = thread
                val body = JSONObject().put("scope",history?.getString("scope")).put("action","comment").put("body",savedDraft)
                    .put("target",if (selected == null) target else JSONObject.NULL).put("thread",selected ?: JSONObject.NULL)
                busy = true
                scope.launch {
                    try {
                        val saved = engine.request("POST","/api/reviews",body)
                        if (!current()) return@launch
                        if (draft == savedDraft) draft = ""
                        thread = saved.getString("thread")
                        load()
                    } catch (error: kotlinx.coroutines.CancellationException) { throw error }
                    catch (error: Exception) { history = null; notice = error.message.orEmpty() + " Refresh comments before retrying." }
                    finally { busy = false }
                }
            }) { Text(if (thread == null) "Save comment" else "Save reply") } }
            WithDisabledReason(if (busy) "An operation is already in progress." else null) { available -> TextButton(enabled = available, onClick = {
                scope.launch {
                    try { load() }
                    catch (error: kotlinx.coroutines.CancellationException) { throw error }
                    catch (error: Exception) { notice = error.message.orEmpty() }
                }
            }) { Text("Refresh comments") } }
        }
        if (notice.isNotEmpty()) Text(notice, fontSize = 12.sp)
    }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/AgentPane.kt"><ui:paste select="#simple-716e739687-0"/><ui:paste select="#simple-android-recorded-run-controller"/><ui:paste select="#simple-android-agent-send"/><ui:paste select="#simple-android-recorded-run"/><ui:paste select="#simple-android-recorded-construction"/><ui:paste select="#simple-android-agent-history"/><ui:paste select="#simple-android-agent-pane"/><ui:paste select="#simple-android-agent-message-controls"/><ui:paste select="#android-recorded-patch-comment"/></ui:file>

</ui:doc>
