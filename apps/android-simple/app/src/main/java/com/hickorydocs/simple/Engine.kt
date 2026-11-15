package com.hickorydocs.simple

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.json.JSONArray
import org.json.JSONObject
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.collect
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.Response
import okhttp3.WebSocket
import okhttp3.WebSocketListener
import okio.ByteString
import okio.ByteString.Companion.toByteString
import java.util.UUID
import java.net.HttpURLConnection
import java.net.URI
import java.net.URLEncoder

data class FileNode(val name: String, val path: String, val dir: Boolean, val children: List<FileNode>, val docID: String?, val generatedBy: String?, val language: String? = null)
data class FileText(val content: String, val hash: String)
data class AgentTurn(val id: String, val prompt: String, val answer: String?, val error: String?, val status: String)
private fun JSONObject.optional(key: String) = if (isNull(key)) null else optString(key).takeIf { it.isNotEmpty() }
private fun JSONArray.objects() = (0 until length()).map { getJSONObject(it) }
class Engine(address: String) {
    private val origin = URI(address).also {
        require(it.scheme in listOf("http", "https") && it.host != null && it.userInfo == null && it.query == null && it.fragment == null && it.path in listOf("", "/")) {
            "Enter the workspace server's http:// or https:// address, without a path or password."
        }
    }
    val debugURL get() = origin.resolve("/api/ws?doc=workspace").toString().replaceFirst("http", "ws")
    suspend fun request(method: String, path: String, body: JSONObject? = null): JSONObject = withContext(Dispatchers.IO) {
        val connection = origin.resolve(path).toURL().openConnection() as HttpURLConnection
        try {
            connection.requestMethod = method; connection.connectTimeout = 30000; connection.readTimeout = 30000
            if (body != null) {
                connection.doOutput = true; connection.setRequestProperty("Content-Type", "application/json")
                connection.outputStream.use { it.write(body.toString().toByteArray(Charsets.UTF_8)) }
            }
            val status = connection.responseCode
            val stream = if (status in 200..299) connection.inputStream else connection.errorStream
            val text = stream?.bufferedReader(Charsets.UTF_8)?.use { it.readText() }.orEmpty()
            if (status !in 200..299) {
                val detail = runCatching { JSONObject(text).optString("error") }.getOrDefault("").ifEmpty { text.take(600) }
                error("$detail (HTTP $status). Check the server and review its current state before retrying.")
            }
            JSONObject(text)
        } finally { connection.disconnect() }
    }
    suspend fun find(query: String, regex: Boolean, matchCase: Boolean, wholeWord: Boolean): WorkspaceSearchResult {
        val result = request("GET", "/api/find?q=" + URLEncoder.encode(query, "UTF-8") + "&regex=$regex&case=$matchCase&whole_word=$wholeWord")
        val hits = result.getJSONArray("files").objects().flatMap { file -> file.getJSONArray("matches").objects().flatMap { row -> row.getJSONArray("at").objects().map { at ->
            WorkspaceSearchHit(file.getString("path"), row.getInt("line"), row.getString("text"), at.getInt("column"), file.optional("source_hash"), file.optional("generated_by"))
        } } }
        return WorkspaceSearchResult(hits, result.getBoolean("truncated"))
    }
    suspend fun files(): Pair<String, List<FileNode>> {
        val response = request("GET", "/api/files")
        fun decode(array: JSONArray): List<FileNode> = array.objects().map {
            FileNode(it.getString("name"), it.getString("path"), it.getBoolean("dir"), decode(it.optJSONArray("children") ?: JSONArray()), it.optional("doc_id"), it.optional("generated_by"), it.optional("language"))
        }
        return response.getString("root") to decode(response.getJSONArray("tree"))
    }
    suspend fun file(file: FileNode): FileText {
        if (file.docID != null) {
            val document = request("GET", "/api/docs/${file.docID}")
            return FileText(document.getString("source"), "")
        }
        val response = request("GET", "/api/file?path=" + URLEncoder.encode(file.path, "UTF-8"))
        return FileText(response.getString("content"), response.getString("hash"))
    }
    suspend fun save(file: FileNode, text: String, base: String, hash: String): String {
        if (file.docID != null) {
            request("PUT", "/api/docs/${file.docID}", JSONObject().put("source", text).put("expected", JSONObject().put("source", base).put("disk", base)))
            return hash
        }
        return request("PUT", "/api/file", JSONObject().put("path", file.path).put("content", text).put("base_hash", hash)).getString("hash")
    }
    suspend fun turns(): List<AgentTurn> = request("GET", "/api/docs/workspace/agent/turns").getJSONArray("turns").objects().map {
        AgentTurn(it.getString("id"), it.getString("prompt"), it.optional("answer"), it.optional("error"), it.getString("status"))
    }
    suspend fun send(prompt: String, parent: String?, provider: String, model: String, effort: String): String = request("POST", "/api/docs/workspace/agent",
        JSONObject().put("prompt", prompt).put("parent_id", parent ?: JSONObject.NULL).put("backend", "python").put("provider", provider).put("model", model).put("effort", effort.ifEmpty { null })).getString("session_id")
    suspend fun stop() { request("POST", "/api/docs/workspace/agent/stop", JSONObject()) }
    suspend fun openTerminal(): String = request("POST", "/api/terminals", JSONObject().put("title", "Terminal")).getString("id")
    suspend fun terminalInput(id: String, data: String) { request("POST", "/api/terminals/$id/input", JSONObject().put("data", data)) }
    suspend fun terminalResize(id: String, rows: Int, cols: Int) { request("POST", "/api/terminals/$id/resize", JSONObject().put("rows", rows).put("cols", cols)) }
    suspend fun terminalScreen(id: String): String {
        val bytes = java.io.ByteArrayOutputStream(); var offset = 0; var revision = ""
        while (true) {
            val page = request("GET", "/api/terminals/$id/output?view=screen&offset=$offset&limit=16384&revision=" + URLEncoder.encode(revision, "UTF-8"))
            val part = page.getJSONArray("bytes")
            bytes.write(ByteArray(part.length()) { part.getInt(it).toByte() }); revision = page.getString("revision")
            if (page.isNull("next")) return bytes.toString("UTF-8")
            offset = page.getInt("next")
        }
    }

}
fun debugURI(path: String): String = "hick:///" + path.trimStart('/')
data class NativeDebugTarget(val path: String, val line: Int, val end: Int)
private val debugHTTP = OkHttpClient.Builder().retryOnConnectionFailure(false).build()
data class NativeRunOutput(val category: String, val text: String, val source: String?, val line: Int?)
class NativeDebugger(private val engine: Engine, private val scope: CoroutineScope) {
    var status by mutableStateOf("idle"); private set
    var message by mutableStateOf(""); private set
    var owner by mutableStateOf(""); private set
    var sessionID: String? = null; private set
    var frames by mutableStateOf<List<JSONObject>>(emptyList()); private set
    var variables by mutableStateOf<List<JSONObject>>(emptyList()); private set
    var capabilities by mutableStateOf(JSONObject()); private set
    var breakpoints by mutableStateOf<Map<String, List<JSONObject>>>(emptyMap()); private set
    var output by mutableStateOf<List<NativeRunOutput>>(emptyList()); private set
    var values by mutableStateOf<Map<String, String>>(emptyMap()); private set
    var watches by mutableStateOf<List<String>>(emptyList()); private set
    var profiles by mutableStateOf<List<JSONObject>>(emptyList()); private set
    var offer by mutableStateOf<JSONObject?>(null); private set
    var launchSource = ""; private set
    private var sourceSnapshots by mutableStateOf<Map<String, String>>(emptyMap())
    var targets by mutableStateOf<List<NativeDebugTarget>>(emptyList()); private set
    private var launchProgram: String? = null
    private var catalog: JSONObject? = null
    private var launchBuffer: DocumentBuffer? = null
    private var pausedLine: Int? = null
    private var selectedFrame: Int? = null
    private var topVariables: List<JSONObject> = emptyList()
    private var stopPending = false
    private var epoch = UUID.randomUUID()
    private var socket: WebSocket? = null
    var onLocation: ((String, Int) -> Unit)? = null
    var emitForTest: ((JSONObject) -> Unit)? = null
    val active get() = status in listOf("starting", "running", "paused", "stopping")
    fun line(path: String): Int? {
        if (status != "paused") return null
        val frame = frames.firstOrNull { it.optInt("id") == selectedFrame } ?: frames.firstOrNull()
        if (path == owner && frame?.optBoolean("in_document") == true) return if (frame.isNull("line")) null else frame.getInt("line")
        if (frame?.optString("source") == path) return if (frame.isNull("source_line")) null else frame.getInt("source_line")
        return if (path == owner && selectedFrame == frames.firstOrNull()?.optInt("id")) pausedLine else null
    }
    fun outputPlacements(buffer: DocumentBuffer): Map<Int, String> {
        if (!sourceMatches(buffer)) return emptyMap()
        val context = inlineDebugContext(buffer.text, buffer.file.path)
        val last = (buffer.text.count { it == '\n' } - if (buffer.text.endsWith('\n')) 1 else 0).coerceAtLeast(0)
        val rows = buffer.text.split('\n')
        fun blockEnd(line: Int): Int {
            val header = context.launch.keys.filter { it <= line }.maxOrNull() ?: return line.coerceAtMost(last)
            val declaration = rows.getOrNull(header)?.trim().orEmpty()
            val fence = if (declaration.startsWith("```")) '`' else if (declaration.startsWith("~~~")) '~' else null
            for (candidate in line + 1..last) {
                val row = rows[candidate].trim()
                if (fence != null && row.length >= 3 && row.all { it == fence }) return candidate
                if (fence == null && row.startsWith("</") && row.contains(":file>")) return candidate
            }
            return last
        }
        val result = mutableMapOf<Int, String>()
        for (chunk in output) {
            var anchor = last
            if (chunk.source == buffer.file.path && chunk.line != null) {
                anchor = chunk.line
                if (context.code[anchor] != null) anchor = blockEnd(anchor)
            } else context.code.filterValues { it == launchProgram }.keys.maxOrNull()?.let { anchor = blockEnd(it) }
            result[anchor] = result[anchor].orEmpty() + chunk.text
        }
        return result
    }
    fun sourceMatches(buffer: DocumentBuffer) = !buffer.dirty && buffer.text == (if (buffer.file.path == owner) launchSource else sourceSnapshots[buffer.file.path])
    fun rememberSource(buffer: DocumentBuffer) { if (active && buffer.file.path !in sourceSnapshots) sourceSnapshots = sourceSnapshots + (buffer.file.path to buffer.base) }
    suspend fun prepare(buffer: DocumentBuffer, line: Int = 0, program: String? = null) {
        if (active) { message = "Stop the current debugger before launching another program."; return }
        if (buffer.dirty || buffer.saving) { message = "Save this file before debugging so the adapter runs the displayed text."; return }
        close(); val current = epoch
        owner = buffer.file.path; launchBuffer = buffer; launchSource = buffer.text; sourceSnapshots = mapOf(owner to launchSource); message = "Resolving launch targets…"; targets = emptyList()
        try {
            val literate = buffer.file.docID != null || owner.substringAfterLast('.').lowercase() in listOf("md", "markdown")
            var target = program ?: inlineDebugPath(buffer.text, owner, line, true) ?: inlineDebugPath(buffer.text, owner, line)
            if (literate && target == null) {
                val id = buffer.file.docID ?: run { message = "Refresh files to discover this document before choosing a file block to debug."; return }
                val rendered = engine.request("GET", "/api/docs/$id/render")
                if (epoch != current || buffer.text != launchSource || buffer.dirty) return
                targets = rendered.getJSONArray("blocks").objects().mapNotNull { block ->
                    val span = block.optJSONArray("span"); val bytes = launchSource.toByteArray(Charsets.UTF_8)
                    if (block.optString("kind") != "file" || block.isNull("path") || span == null || span.length() != 2 || span.getInt(0) < 0 || span.getInt(1) < span.getInt(0) || span.getInt(1) > bytes.size) null
                    else NativeDebugTarget(block.getString("path"), bytes.take(span.getInt(0)).count { it == 10.toByte() }, bytes.take(span.getInt(1)).count { it == 10.toByte() })
                }
                target = targets.firstOrNull { line in it.line..it.end }?.path
                if (target == null) { message = if (targets.isEmpty()) "No file blocks to debug. Add a supported file block and save this document." else "Choose the file block to debug."; return }
            }
            launchProgram = target
            val query = JSONObject().put("uri", debugURI(owner)).put(if (literate) "expected_document" else "expected_source", launchSource)
            if (target != null) query.put("program", target)
            val result = engine.request("POST", "/api/launch-profiles", query)
            if (epoch != current || buffer.text != launchSource || buffer.dirty) return
            catalog = result; profiles = result.getJSONArray("profiles").objects().filter { it.optBoolean("can_debug") }
            val automatic = profiles.firstOrNull { it.optString("program") == (target ?: result.optString("viewed", owner)) } ?: profiles.singleOrNull()
            if (automatic != null) { launch(automatic); return }
            message = if (profiles.isEmpty()) "No debug launch targets. Configure a debug profile in this project and try again." else "Choose a debug target."
        } catch (cancelled: kotlinx.coroutines.CancellationException) { throw cancelled }
        catch (failure: Exception) { if (epoch == current) message = "${failure.message} Save or refresh this file and try debugging again." }
    }
    fun launch(profile: JSONObject) {
        val buffer = launchBuffer; val catalog = catalog
        if (buffer == null || catalog == null || profiles.none { it.optString("id") == profile.optString("id") } || buffer.text != launchSource || buffer.dirty || buffer.saving || active) {
            message = "The source or launch target changed. Save and choose Debug again."; return
        }
        status = "starting"; message = ""; profiles = emptyList(); values = emptyMap(); variables = emptyList(); frames = emptyList(); pausedLine = null; offer = null
        val current = epoch
        if (emitForTest == null) socket = debugHTTP.newWebSocket(Request.Builder().url(engine.debugURL).build(), object : WebSocketListener() {
            override fun onMessage(webSocket: WebSocket, bytes: ByteString) {
                val data = bytes.toByteArray()
                if (data.size < 2 || data[0] != 3.toByte()) return
                val event = runCatching { JSONObject(data.copyOfRange(1, data.size).toString(Charsets.UTF_8)) }.getOrNull() ?: return
                scope.launch { if (epoch == current) receive(event) }
            }
            override fun onFailure(webSocket: WebSocket, failure: Throwable, response: Response?) {
                scope.launch { if (epoch == current) fail("Debugger disconnected: ${failure.message}. Reconnect explicitly; commands were not retried.") }
            }
            override fun onClosing(webSocket: WebSocket, code: Int, reason: String) { webSocket.close(code, reason) }
            override fun onClosed(webSocket: WebSocket, code: Int, reason: String) {
                scope.launch { if (epoch == current && active) fail("Debugger connection closed. Reconnect explicitly; commands were not retried.") }
            }
        })
        val request = JSONObject().put("op", "start").put("doc", debugURI(owner)).put("breakpoints", JSONArray(breakpoints[owner].orEmpty()))
            .put("profile", JSONObject().put("id", profile.getString("id")).put("fingerprint", catalog.getString("fingerprint")))
        launchProgram?.let { request.put("program", it) }
        val sources = JSONObject(); breakpoints.filterKeys { it != owner }.forEach { (path, held) -> sources.put(debugURI(path), JSONArray(held)) }; request.put("sources", sources)
        send(request)
    }
    private fun send(request: JSONObject) {
        emitForTest?.let { it(request); return }
        if (socket?.send((byteArrayOf(3) + request.toString().toByteArray(Charsets.UTF_8)).toByteString()) != true) fail("Debugger command was not sent. Reconnect explicitly; it was not retried.")
    }
    private fun fail(detail: String) { status = "failed"; message = detail; sessionID = null; pausedLine = null; variables = emptyList(); frames = emptyList(); values = emptyMap() }
    fun close() { stopPending = false; epoch = UUID.randomUUID(); socket?.cancel(); socket = null; sessionID = null; status = "idle"; pausedLine = null; frames = emptyList(); variables = emptyList(); values = emptyMap(); profiles = emptyList() }
    fun receive(event: JSONObject) {
        val kind = event.optString("event")
        if (kind in listOf("started", "build") || (kind == "failed" && event.isNull("session"))) {
            if (event.optString("doc") != debugURI(owner) || status !in listOf("starting", "stopping")) return
        } else if (event.optional("session") == null || event.optional("session") != sessionID) return
        when (kind) {
            "started" -> { output = emptyList(); sessionID = event.getString("session"); capabilities = event.optJSONObject("capabilities") ?: JSONObject(); status = "running"; bind(event.optJSONArray("breakpoints"), owner); if (stopPending) { stopPending = false; stop() } }
            "stopped" -> {
                status = "paused"; frames = event.optJSONArray("frames")?.objects().orEmpty(); variables = event.optJSONArray("variables")?.objects().orEmpty(); topVariables = variables
                pausedLine = if (event.isNull("line")) null else event.getInt("line"); selectedFrame = if (!event.isNull("selected_frame")) event.getInt("selected_frame") else frames.firstOrNull()?.optInt("id")
                val top = frames.firstOrNull(); val path = top?.optional("source"); val line = top?.let { if (it.isNull("source_line")) null else it.getInt("source_line") }
                if (path != null && line != null && !path.startsWith('/') && ':' !in path) onLocation?.invoke(path, line)
                else pausedLine?.let { onLocation?.invoke(owner, it) }
                watches.forEach { evaluate(it, true) }
            }
            "breakpoints" -> bind(event.optJSONArray("breakpoints"), owner)
            "source_breakpoints" -> event.optional("source")?.takeIf { it.startsWith("hick:///") }?.removePrefix("hick:///")?.let { bind(event.optJSONArray("breakpoints"), it) }
            "run_output" -> output = (output + event.optJSONArray("events")?.objects().orEmpty().map { item -> NativeRunOutput(item.optString("category", "stdout"), item.optString("text"), item.optional("source"), if (item.has("line") && !item.isNull("line")) item.optInt("line") else null) }).takeLast(256)
            "value" -> if (status == "paused") values = values + (event.getString("expression") to event.getString("value"))
            "finished", "ended" -> { status = "finished"; sessionID = null; pausedLine = null; frames = emptyList(); variables = emptyList(); values = emptyMap() }
            "failed" -> { offer = event.optJSONObject("offer_install"); if (sessionID == null) fail(event.optString("message", "Debug launch failed. Check adapter installation and try again.")) else message = event.optString("message") + " Review the paused state before trying again." }
            "build" -> message = event.optJSONArray("events")?.objects()?.joinToString("") { it.optString("data") }.orEmpty()
        }
    }
    private fun bind(statuses: JSONArray?, path: String) {
        val held = breakpoints[path].orEmpty()
        breakpoints = breakpoints + (path to statuses?.objects().orEmpty().map { state ->
            val mark = JSONObject(held.firstOrNull { it.optInt("line", -1) == state.optInt("line", -2) }?.toString() ?: "{}")
            state.keys().forEach { key -> mark.put(key, state.get(key)) }; if (!state.isNull("moved_to")) mark.put("line", state.getInt("moved_to")); mark
        })
    }
    fun toggle(path: String, line: Int, condition: String? = null) {
        if (line < 0) return
        val held = breakpoints[path].orEmpty().toMutableList(); val index = held.indexOfFirst { it.optInt("line") == line }
        if (index >= 0) { if (condition == null) held.removeAt(index) else held[index] = JSONObject(held[index].toString()).put("condition", condition) }
        else held.add(JSONObject().put("line", line).apply { if (condition != null) put("condition", condition) })
        breakpoints = breakpoints + (path to held)
        sessionID?.let { id -> send(JSONObject().put("op", if (path == owner) "breakpoints" else "source_breakpoints").put("session", id).put("source", debugURI(path)).put("breakpoints", JSONArray(held))) }
    }
    fun step(how: String) {
        val id = sessionID ?: return; if (status != "paused") return
        status = "running"; pausedLine = null; variables = emptyList(); values = emptyMap()
        send(JSONObject().put("op", "step").put("session", id).put("how", how).apply { selectedFrame?.let { put("frame", it) } })
    }
    fun stop() {
        val id = sessionID ?: run { if (status in listOf("starting", "stopping")) { stopPending = true; status = "stopping"; message = "Stopping as soon as the adapter acknowledges launch…" }; return }
        status = "stopping"; pausedLine = null; variables = emptyList(); values = emptyMap(); send(JSONObject().put("op", "stop").put("session", id))
    }
    fun evaluate(expression: String, watch: Boolean = false) {
        val id = sessionID ?: return; if (status != "paused" || expression.isBlank()) return
        if (watch && expression !in watches) watches = watches + expression
        send(JSONObject().put("op", "eval").put("session", id).put("expression", expression).put("context", if (watch) "watch" else "repl").apply { selectedFrame?.let { put("frame", it) } })
    }
    fun removeWatch(expression: String) { watches = watches - expression; values = values - expression }
    fun select(frame: JSONObject) {
        selectedFrame = frame.optInt("id")
        variables = if (selectedFrame == frames.firstOrNull()?.optInt("id")) topVariables else emptyList(); values = emptyMap()
        watches.forEach { evaluate(it, true) }
        val path = frame.optional("source")
        if (path != null && !path.startsWith('/') && ':' !in path && !frame.isNull("source_line")) onLocation?.invoke(path, frame.getInt("source_line"))
        else if (!frame.isNull("line")) onLocation?.invoke(owner, frame.getInt("line"))
    }
    suspend fun install() {
        val offer = offer ?: return; val current = epoch
        try { engine.request("POST", "/api/install", JSONObject().put("kind", offer.getString("kind")).put("language", offer.getString("language")))
            if (epoch == current) { this.offer = null; message = "Debugger installed. Choose Debug again to launch the current saved source." }
        } catch (cancelled: kotlinx.coroutines.CancellationException) { throw cancelled }
        catch (failure: Exception) { if (epoch == current) message = "${failure.message} Check adapter installation and retry." }
    }
}
data class NativeLanguageSpan(val from: Int, val to: Int, val kind: String)
data class NativeLanguageDiagnostic(val message: String, val from: Int?, val to: Int?, val severity: Int = 1, val location: SemanticLocation? = null)
data class NativeLanguageCompletion(val label: String, val detail: String, val edit: NativeSourceEdit, val source: String, val selection: androidx.compose.ui.text.TextRange,val original:JSONObject?=null)
fun languageURI(path: String): String = java.net.URI("hick", "", "/" + path.trimStart('/'), null).toASCIIString()
fun languageOffset(source: String, position: JSONObject): Int? {
    val line = position.optInt("line", -1); val column = position.optInt("character", -1)
    val rows = source.split('\n')
    if (line !in rows.indices || column < 0 || column > rows[line].length) return null
    return rows.take(line).sumOf { it.length + 1 } + column
}
fun languagePosition(source: String, offset: Int): JSONObject {
    val prefix = source.take(offset.coerceIn(0, source.length))
    return JSONObject().put("line", prefix.count { it == '\n' }).put("character", prefix.length - prefix.lastIndexOf('\n') - 1)
}
fun nativeMarkupText(value: Any?): String = when (value) {
    is String -> value
    is JSONArray -> (0 until value.length()).map { nativeMarkupText(value.opt(it)) }.filter { it.isNotEmpty() }.joinToString("\n\n")
    is JSONObject -> value.optString("value", value.optString("label", ""))
    else -> ""
}
fun decodeNativeDiagnostics(params: JSONObject, source: String, expectedVersion: Int): List<NativeLanguageDiagnostic> {
    val items = (params.optJSONArray("diagnostics") ?: JSONArray()).objects()
    val hasEvidence = items.any { it.optJSONObject("data")?.has("hickory") == true }
    val digest = if (hasEvidence) "sha256:" + java.security.MessageDigest.getInstance("SHA-256").digest(source.toByteArray(Charsets.UTF_8)).joinToString("") { "%02x".format(it) } else ""
    return items.map { item ->
        val data = item.optJSONObject("data")
        val located = if (data?.has("hickory") == true) data.optJSONObject("hickory")?.optString("source_hash")?.lowercase() == digest else params.has("version") && params.optInt("version", -1) == expectedVersion
        val range = item.optJSONObject("range")
        val from = if (located) range?.optJSONObject("start")?.let { languageOffset(source, it) } else null
        val to = if (located) range?.optJSONObject("end")?.let { languageOffset(source, it) } else null
        val position = range?.optJSONObject("start")
        val uri = params.optString("uri")
        val location = if (located && from != null && to != null && from <= to && position != null && uri.isNotEmpty()) SemanticLocation(uri, position.getInt("line"), position.getInt("character")) else null
        NativeLanguageDiagnostic(item.optString("message", "Language diagnostic"), from?.takeIf { to != null && it <= to }, to?.takeIf { from != null && from <= it }, item.optInt("severity", 1), location)
    }
}
// UTF-16 display cache, never an authority for completion edits or navigation.
fun rebaseNativeTokens(tokens: List<NativeLanguageSpan>, old: String, new: String): List<NativeLanguageSpan> {
    var prefix = 0; var suffix = 0
    while (prefix < minOf(old.length, new.length) && old[prefix] == new[prefix]) prefix++
    while (suffix < minOf(old.length, new.length) - prefix && old[old.length - suffix - 1] == new[new.length - suffix - 1]) suffix++
    val end = old.length - suffix; val shift = new.length - old.length
    return tokens.mapNotNull { token ->
        when {
            token.from < 0 || token.to <= token.from || token.to > old.length -> null
            token.to <= prefix -> token
            token.from <= prefix && token.to >= end && token.to - token.from + shift > 0 &&
                !new.substring(prefix, new.length - suffix).contains('\n') -> token.copy(to = token.to + shift)
            token.from >= end -> token.copy(from = token.from + shift, to = token.to + shift)
            else -> null
        }
    }
}
class NativeLanguage(engine: Engine, private val buffer: DocumentBuffer, private val scope: CoroutineScope, sharedConnection: NativeLanguageConnection? = null) {
    val connection = sharedConnection ?: NativeLanguageConnection(engine,scope)
    private val ownsConnection = sharedConnection == null
    private var subscription: UUID? = null
    var symbolQuery by mutableStateOf("")
    var locations by mutableStateOf<List<SemanticLocation>>(emptyList())
    var locationStatus by mutableStateOf("")
    private var locationSymbolQuery: String? = null
    var locationSource by mutableStateOf("")
    var locationSelection by mutableStateOf(androidx.compose.ui.text.TextRange.Zero)
    private var navigation = 0
    var status by mutableStateOf("Language server connecting…")
    var tokens by mutableStateOf<List<NativeLanguageSpan>>(emptyList())
    var tokenSource by mutableStateOf("")
    var diagnostics by mutableStateOf<List<NativeLanguageDiagnostic>>(emptyList())
    var diagnosticSource by mutableStateOf("")
    var suggestions by mutableStateOf<List<NativeLanguageCompletion>>(emptyList())
    var hover by mutableStateOf("")
    var signature by mutableStateOf("")
    var inlays by mutableStateOf<List<String>>(emptyList())
    var inspectionSource by mutableStateOf("")
    var inspectionSelection by mutableStateOf(androidx.compose.ui.text.TextRange.Zero)
    private val uri = languageURI(buffer.file.path)
    private var epoch = 0
    private var inspection = 0
    private var completion = 0
    private var version = 1
    private var sent = ""
    private var legend = emptyList<String>()
    private val opened get() = subscription != null && connection.active
    private var diagnosticTicket = 0
    private var refresh: kotlinx.coroutines.Job? = null
    private var watcher: kotlinx.coroutines.Job? = null
    fun start() {
        close();sent=buffer.text;version=1;status="Language server connecting…"
        subscription=connection.attach(uri,sent,version,{receive(it)},{failure ->
            epoch++;inspection++;completion++;navigation++;diagnosticTicket++
            status="Language services unavailable: ${failure.message}. Retry language services."
            if(locations.isNotEmpty()) locationStatus="Provider connection changed. Run the query again."
        })
        changed()
        watcher=scope.launch {androidx.compose.runtime.snapshotFlow {buffer.text}.collect {changed()}}
    }
    fun close() {
        cancelCompletion();completionNotice=""
        navigation++;locations=emptyList();locationStatus=""
        epoch++;inspection++;completion++;refresh?.cancel();watcher?.cancel()
        val token=subscription;subscription=null;if(token!=null) connection.detach(token)
        if(ownsConnection) connection.close()
        tokens=emptyList();diagnostics=emptyList();suggestions=emptyList();hover="";signature="";inlays=emptyList()
    }
    private suspend fun request(method:String,params:JSONObject):Any {
        val owner=subscription ?: error("Language source is closed. Retry language services.")
        return connection.request(method,params,owner)
    }
    private fun doc() = JSONObject().put("uri", uri)
    private fun sync() {
        if (sent == buffer.text) return
        sent = buffer.text; version++
        subscription?.let {connection.change(it,sent,version)}
    }
    private fun receive(message: JSONObject) {
        if (message.optString("method") == "hick/serverCapabilities") {
            val raw = message.optJSONObject("params")?.optJSONObject("capabilities")?.optJSONObject("semanticTokensProvider")?.optJSONObject("legend")?.optJSONArray("tokenTypes")
            legend = if (raw == null) emptyList() else (0 until raw.length()).map { raw.getString(it) }
            status = "LSP connected"; changed()
        } else if (message.optString("method") == "textDocument/publishDiagnostics") {
            val params = message.optJSONObject("params") ?: return
            val evidence = (params.optJSONArray("diagnostics") ?: JSONArray()).objects().any { it.optJSONObject("data")?.has("hickory") == true }
            if (params.optString("uri") != uri || !evidence && (sent != buffer.text || params.has("version") && params.optInt("version", -1) != version)) return
            val source = buffer.text; val expectedVersion = version; val identity = epoch; val request = ++diagnosticTicket
            diagnosticSource = ""
            diagnostics = (params.optJSONArray("diagnostics") ?: JSONArray()).objects().map { NativeLanguageDiagnostic(it.optString("message","Language diagnostic"),null,null,it.optInt("severity",1)) }
            scope.launch {
                val decoded = withContext(Dispatchers.Default) { decodeNativeDiagnostics(params,source,expectedVersion) }
                if (epoch == identity && diagnosticTicket == request && buffer.text == source && (evidence || sent == source)) { diagnostics = decoded; diagnosticSource = source }
            }
        }
    }
    fun changed() {
        refresh?.cancel(); inspection++; completion++; navigation++
        if (locationStatus == "Searching…") locationStatus = "Source or provider changed. Run the query again."
        tokens = rebaseNativeTokens(tokens, tokenSource, buffer.text); tokenSource = buffer.text
        if (!opened) return
        val current = epoch
        refresh = scope.launch {
            kotlinx.coroutines.delay(300); sync(); val source = buffer.text
            if (inspectionSource.isNotEmpty() || suggestions.isNotEmpty()) {
                scope.launch { inspect() }; scope.launch { complete() }
            }
            try {
                for (attempt in 0 until 5) {
                val result = request("textDocument/semanticTokens/full", JSONObject().put("textDocument", doc())) as? JSONObject
                if (epoch != current || buffer.text != source) return@launch
                val data = result?.optJSONArray("data") ?: JSONArray(); var line = 0; var column = 0
                val spans = mutableListOf<NativeLanguageSpan>()
                if (data.length() % 5 == 0) for (index in 0 until data.length() step 5) {
                    val delta = data.optInt(index, -1); val start = data.optInt(index + 1, -1); val length = data.optInt(index + 2, -1); val type = data.optInt(index + 3, -1)
                    if (delta < 0 || start < 0 || delta > source.length || start > source.length) { spans.clear(); break }
                    line += delta; column = if (delta == 0) column + start else start
                    if (length <= 0 || length > source.length || type !in legend.indices) continue
                    val from = languageOffset(source, JSONObject().put("line", line).put("character", column)) ?: continue
                    val to = languageOffset(source, JSONObject().put("line", line).put("character", column + length)) ?: continue
                    spans.add(NativeLanguageSpan(from, to, legend[type]))
                }
                if (spans.isNotEmpty() || (attempt == 4 && result?.optJSONArray("data")?.length() == 0)) {
                    tokens = spans; tokenSource = source
                }
                status = "LSP · ${tokens.size} semantic spans"
                if (spans.isNotEmpty() || attempt == 4) break
                kotlinx.coroutines.delay(700)
                }
            } catch (cancelled: kotlinx.coroutines.CancellationException) { throw cancelled }
            catch (error: Exception) { if (epoch == current) status = "Language colors unavailable: ${error.message}. Retry language services." }
        }
    }
    // A read-only background query must not open UI or depend on the caret.
    suspend fun readOutline(source:String):List<SemanticLocation> {
        check(buffer.text==source) {"Source changed before analysis. Run again."}
        sync();val current=epoch
        val result=request("textDocument/documentSymbol",JSONObject().put("textDocument",doc()))
        check(epoch==current && buffer.text==source) {"Source or provider changed during analysis. Run again."}
        return semanticLocations(result,uri).filter {it.uri==uri}
    }
    suspend fun navigate(query: SemanticQuery) {
        val source = buffer.text; val selection = buffer.state.selection; val current = epoch; val id = ++navigation
        locationSource = source; locationSelection = selection; locations = emptyList(); locationStatus = "Searching…"
        locationSymbolQuery = if (query == SemanticQuery.Symbols) symbolQuery else null
        try {
            sync()
            var params = JSONObject().put("textDocument", doc()).put("position", languagePosition(source, selection.min))
            if (query == SemanticQuery.References) params.put("context", JSONObject().put("includeDeclaration", true))
            if (query == SemanticQuery.Symbols) params = JSONObject().put("query", locationSymbolQuery)
            if (query == SemanticQuery.Outline) params = JSONObject().put("textDocument", doc())
            var result = request(query.method, params)
            val followup=query.hierarchyFollowup
            if (followup != null) {
                val items = result as? JSONArray ?: JSONArray()
                check(items.length() <= 32) { "Too many call targets. Choose a more specific symbol." }
                val answers = JSONArray()
                for (index in 0 until items.length()) {
                    if (navigation != id || epoch != current || buffer.text != source || buffer.state.selection != selection) {
                        if(navigation==id&&epoch==current)locationStatus="Source or cursor changed. Run the query again."
                        return
                    }
                    answers.put(request(followup, JSONObject().put("item", items.get(index))))
                }
                result = answers
            }
            if (navigation != id || epoch != current || buffer.text != source || buffer.state.selection != selection) {
                if (navigation == id && epoch == current) locationStatus = "Source or cursor changed. Run the query again."
                return
            }
            locations = semanticLocations(result, if (query == SemanticQuery.Outline) uri else null)
            locationStatus = if (locations.isEmpty()) "No results. The provider may still be indexing or may not support this query. Retry or choose another symbol." else "${locations.size} locations" + if (locations.size >= 10000) "; narrow the query for more" else ""
        } catch (cancelled: kotlinx.coroutines.CancellationException) { throw cancelled }
        catch (failure: Exception) {
            if (navigation == id && epoch == current && buffer.text == source && buffer.state.selection == selection)
                locationStatus = "${query.label} unavailable: ${failure.message}. Retry language services."
        }
    }
    fun open(location: SemanticLocation): Boolean {
        if (buffer.text != locationSource || buffer.state.selection != locationSelection || location !in locations || location.path == null || locationSymbolQuery != null && locationSymbolQuery != symbolQuery) {
            locationStatus = "Location is stale or outside authored workspace files. Run the query again."; return false
        }
        if (location.path == buffer.file.path) {
            val offset = languageOffset(buffer.text, JSONObject().put("line", location.line).put("character", location.character)) ?: return false
            if (!buffer.selectSourceRange(offset, offset, buffer.text)) return false
            val position = languagePosition(locationSource, locationSelection.min)
            buffer.rememberNavigation?.invoke(SemanticLocation(uri, position.getInt("line"), position.getInt("character")), location)
            buffer.focusSource?.invoke()
        } else buffer.openSemanticLocation?.invoke(location)
        return true
    }
    suspend fun inspect() {
        if (!opened) return
        sync(); val source = buffer.text; val selection = buffer.state.selection; val current = epoch; val id = ++inspection
        inspectionSource = source; inspectionSelection = selection
        kotlinx.coroutines.coroutineScope {
            for (method in listOf("textDocument/hover", "textDocument/signatureHelp", "textDocument/inlayHint")) launch {
                val params = JSONObject().put("textDocument", doc()).put("position", languagePosition(source, selection.min))
                if (method == "textDocument/inlayHint") params.put("range", JSONObject().put("start", languagePosition(source, 0)).put("end", languagePosition(source, source.length)))
                val result = try { request(method, params) } catch (cancelled: kotlinx.coroutines.CancellationException) { throw cancelled } catch (_: Exception) { return@launch }
                if (result == JSONObject.NULL || epoch != current || inspection != id || buffer.text != source || buffer.state.selection != selection) return@launch
                when (method) {
                    "textDocument/hover" -> { hover = nativeMarkupText((result as? JSONObject)?.opt("contents")) }
                    "textDocument/signatureHelp" -> { signature = ((result as? JSONObject)?.optJSONArray("signatures") ?: JSONArray()).objects().joinToString("\n") { it.optString("label") } }
                    else -> { inlays = ((result as? JSONArray) ?: JSONArray()).objects().take(200).mapNotNull { hint ->
                        val pos = hint.optJSONObject("position") ?: return@mapNotNull null
                        if (languageOffset(source, pos) == null) return@mapNotNull null
                        val label = hint.opt("label"); val text = if (label is JSONArray) label.objects().joinToString("") { it.optString("value") } else label?.toString().orEmpty()
                        "Line ${pos.optInt("line") + 1}: $text"
                    } }
                }
            }
        }
    }
    val proposalIdentity: Int get() = epoch
    suspend fun propose(kind: String, name: String): Any {
        check(opened && !buffer.readOnly) { "Choose writable source with a connected language provider." }
        sync(); val source = buffer.text; val selection = buffer.state.selection; val identity = epoch
        val params = JSONObject().put("textDocument",doc())
        val method = when(kind) {
            "rename" -> { params.put("position",languagePosition(source,selection.min)).put("newName",name); "textDocument/rename" }
            "references" -> { params.put("position",languagePosition(source,selection.min)).put("context",JSONObject().put("includeDeclaration",true)); "textDocument/references" }
            "format" -> { params.put("options",JSONObject().put("tabSize",4).put("insertSpaces",source.lines().none { it.startsWith("\t") })); "textDocument/formatting" }
            else -> { params.put("range",JSONObject().put("start",languagePosition(source,selection.min)).put("end",languagePosition(source,selection.max))).put("context",JSONObject().put("diagnostics",JSONArray())); "textDocument/codeAction" }
        }
        val result = request(method,params)
        check(epoch == identity && buffer.text == source && buffer.state.selection == selection) { "Source, selection or provider changed. Prepare again." }
        return if (kind == "format") JSONObject().put("changes",JSONObject().put(uri,result)) else result
    }
    suspend fun resolveProposal(action:JSONObject):JSONObject {
        val held=action.optJSONObject("data")?.opt("hickory_action") is String
        if(!held&&(!action.isNull("edit")||action.isNull("data")))return action
        check(opened&&!buffer.readOnly){"Choose writable source with a connected language provider."}
        sync();val source=buffer.text;val selection=buffer.state.selection;val identity=epoch
        val params=JSONObject(action.toString()).put("textDocument",doc());val result=request("codeAction/resolve",params)
        check(epoch==identity&&buffer.text==source&&buffer.state.selection==selection){"Source, selection or provider changed. Prepare actions again."}
        check(result is JSONObject&&result.opt("title") is String){"Provider did not resolve this action. Prepare actions again."};return result
    }
    suspend fun complete() {
        if (!opened || buffer.readOnly) return
        sync(); val source = buffer.text; val selection = buffer.state.selection; val current = epoch; val id = ++completion
        try {
            kotlinx.coroutines.delay(350)
            if(epoch!=current||completion!=id||buffer.text!=source||buffer.state.selection!=selection)return
            val result = request("textDocument/completion", JSONObject().put("textDocument", doc()).put("position", languagePosition(source, selection.min)))
            if (epoch != current || completion != id || buffer.text != source || buffer.state.selection != selection) return
            if (result == JSONObject.NULL) return
            val items = result as? JSONArray ?: (result as? JSONObject)?.optJSONArray("items") ?: JSONArray()
            var prefix = selection.min
            while (prefix > 0 && (source[prefix - 1].isLetterOrDigit() || source[prefix - 1] == '_')) prefix--
            val typed = source.substring(prefix, selection.min).lowercase()
            suggestions = items.objects().mapNotNull { item ->
                val label=item.opt("label") as? String ?: return@mapNotNull null
                if(!(item.opt("filterText") as? String ?: label).lowercase().startsWith(typed))return@mapNotNull null
                NativeLanguageCompletion(label,item.optString("detail"),NativeSourceEdit(prefix,selection.max,item.optString("insertText",label)),source,selection,JSONObject(item.toString()))
            }.take(100)
        } catch (cancelled: kotlinx.coroutines.CancellationException) { throw cancelled }
        catch (error: Exception) { if (epoch == current&&completion==id&&buffer.text==source&&buffer.state.selection==selection) status = "Completion unavailable: ${error.message}. Retry language services." }
    }
    var completionNotice by mutableStateOf("")
    private var acceptingCompletion=0
    private var completionPending=false
    fun cancelCompletion(){acceptingCompletion++;if(completionPending)completionNotice="Completion cancelled.";completionPending=false}
    suspend fun resolveCompletion(item:JSONObject):JSONObject {
        if(item.isNull("data"))return item
        check(opened&&!buffer.readOnly){"Choose writable source with a connected language provider."}
        sync();val source=buffer.text;val selection=buffer.state.selection;val identity=epoch
        val result=request("completionItem/resolve",JSONObject(item.toString()).put("textDocument",doc()))
        check(epoch==identity&&buffer.text==source&&buffer.state.selection==selection){"Source, cursor or provider changed. Request completions again."}
        check(result is JSONObject&&result.opt("label") is String){"Provider did not resolve this completion. Request again."};return result
    }
    suspend fun accept(item:NativeLanguageCompletion,resolution:(suspend (JSONObject)->JSONObject)?=null):Boolean {
        if(buffer.readOnly||buffer.saving||buffer.text!=item.source||buffer.state.selection!=item.selection){completionNotice="Suggestion is out of date. Request completions again.";return false}
        if(buffer.multiSource==buffer.text&&buffer.multiRanges.size>1){completionNotice="Choose one caret before accepting this source completion.";return false}
        val original=item.original ?: return buffer.applySourceEdits(listOf(item.edit),item.source)
        val request=++acceptingCompletion;val identity=epoch;completionPending=true;completionNotice="Resolving completion…"
        try {
            val resolved=(resolution ?: {resolveCompletion(it)})(original)
            if(acceptingCompletion!=request)return false
            check(epoch==identity&&buffer.language===this&&!buffer.readOnly&&!buffer.saving&&buffer.text==item.source&&buffer.state.selection==item.selection&&!(buffer.multiSource==buffer.text&&buffer.multiRanges.size>1)){"Source, cursor or provider changed. Request completions again."}
            val draft=planCompletion(uri,resolved,SourceSnapshot(item.source,!buffer.readOnly&&!buffer.saving),item.edit.from,item.edit.to,completionExpansion(resolved))
            val edits=draft.plan.patches.firstOrNull()?.edits.orEmpty().map{NativeSourceEdit(it.from,it.to,it.text)}
            check(buffer.applySourceEdits(edits,draft.before,androidx.compose.ui.text.TextRange(draft.selection.from,draft.selection.to))){"Editor refused the complete completion. Finish composition or request again."}
            completionPending=false;completionNotice=if(draft.plan.patches.isEmpty())"Completion selected" else "Completion applied · draft";return true
        }catch(cancelled:kotlinx.coroutines.CancellationException){if(acceptingCompletion==request)completionNotice="Completion cancelled.";throw cancelled}
        catch(error:Exception){if(acceptingCompletion==request)completionNotice=error.message.orEmpty();return false}
        finally{if(acceptingCompletion==request)completionPending=false}
    }

}
