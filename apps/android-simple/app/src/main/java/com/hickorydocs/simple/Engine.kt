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

data class FileNode(val name: String, val path: String, val dir: Boolean, val children: List<FileNode>, val docID: String?, val generatedBy: String?)
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
    suspend fun files(): Pair<String, List<FileNode>> {
        val response = request("GET", "/api/files")
        fun decode(array: JSONArray): List<FileNode> = array.objects().map {
            FileNode(it.getString("name"), it.getString("path"), it.getBoolean("dir"), decode(it.optJSONArray("children") ?: JSONArray()), it.optional("doc_id"), it.optional("generated_by"))
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
    suspend fun send(prompt: String, parent: String?): String = request("POST", "/api/docs/workspace/agent",
        JSONObject().put("prompt", prompt).put("parent_id", parent ?: JSONObject.NULL).put("backend", "python")).getString("session_id")
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
class NativeDebugger(private val engine: Engine, private val scope: CoroutineScope) {
    var status by mutableStateOf("idle"); private set
    var message by mutableStateOf(""); private set
    var owner by mutableStateOf(""); private set
    var sessionID: String? = null; private set
    var frames by mutableStateOf<List<JSONObject>>(emptyList()); private set
    var variables by mutableStateOf<List<JSONObject>>(emptyList()); private set
    var capabilities by mutableStateOf(JSONObject()); private set
    var breakpoints by mutableStateOf<Map<String, List<JSONObject>>>(emptyMap()); private set
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
    fun sourceMatches(buffer: DocumentBuffer) = !buffer.dirty && buffer.text == (if (buffer.file.path == owner) launchSource else sourceSnapshots[buffer.file.path])
    fun rememberSource(buffer: DocumentBuffer) { if (active && buffer.file.path !in sourceSnapshots) sourceSnapshots = sourceSnapshots + (buffer.file.path to buffer.base) }
    suspend fun prepare(buffer: DocumentBuffer, line: Int = 0, program: String? = null) {
        if (active) { message = "Stop the current debugger before launching another program."; return }
        if (buffer.dirty || buffer.saving) { message = "Save this file before debugging so the adapter runs the displayed text."; return }
        close(); val current = epoch
        owner = buffer.file.path; launchBuffer = buffer; launchSource = buffer.text; sourceSnapshots = mapOf(owner to launchSource); message = "Resolving launch targets…"; targets = emptyList()
        try {
            val literate = buffer.file.docID != null || owner.substringAfterLast('.').lowercase() in listOf("md", "markdown")
            var target = program
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
            "started" -> { sessionID = event.getString("session"); capabilities = event.optJSONObject("capabilities") ?: JSONObject(); status = "running"; bind(event.optJSONArray("breakpoints"), owner); if (stopPending) { stopPending = false; stop() } }
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
