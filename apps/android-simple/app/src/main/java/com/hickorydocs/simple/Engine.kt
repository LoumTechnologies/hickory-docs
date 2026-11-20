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
data class StartupSource(val path: String, val content: String, val rootPath: String, val workspaceID: String?)
data class FileText(val content: String, val hash: String, val saveGuard:String?=null, val saveGuardReason:String?=null)
data class AgentTurn(val id: String, val prompt: String, val answer: String?, val error: String?, val status: String)
private fun JSONObject.optional(key: String) = if (isNull(key)) null else optString(key).takeIf { it.isNotEmpty() }
private fun JSONArray.objects() = (0 until length()).map { getJSONObject(it) }
class Engine(address: String) {
    var snippetClipboardReader:(()->String?)?=null
    private val origin = URI(address).also {
        require(it.scheme in listOf("http", "https") && it.host != null && it.userInfo == null && it.query == null && it.fragment == null && it.path in listOf("", "/")) {
            "Enter the workspace server's http:// or https:// address, without a path or password."
        }
    }
    var sourceRootAddress:String?=null;private set
    var sourceWorkspaceID:String?=null;private set
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
            if(path.startsWith("/api/workspace/recovery"))nativeReviewProtocolJSON(text,128*1024*1024,"Private recovery",setOf("version","recovery_protocol")) else JSONObject(text)
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
        sourceRootAddress=response.optional("root_path")
        sourceWorkspaceID=response.optional("workspace_id")?.takeIf{it.matches(Regex("[a-f0-9]{32}"))}
        return response.getString("root") to decode(response.getJSONArray("tree"))
    }
    suspend fun startupSource(path: String): StartupSource {
        val reply = request("GET", "/api/startup-source?path=" + URLEncoder.encode(path, "UTF-8"))
        val root = reply.getString("root_path")
        val identity = reply.optional("workspace_id")
        check(reply.opt("version") == 1 && reply.getString("path") == path && root.isNotEmpty() &&
            (identity == null || identity.matches(Regex("[a-f0-9]{32}")))) { "The startup source address could not be confirmed." }
        return StartupSource(path, reply.getString("content"), root, identity)
    }
    suspend fun file(file: FileNode): FileText {
        if (file.docID != null) {
            val document = request("GET", "/api/docs/${file.docID}")
            return FileText(document.getString("source"), "",document.optional("save_guard"),document.optional("save_guard_reason"))
        }
        val response = request("GET", "/api/file?path=" + URLEncoder.encode(file.path, "UTF-8"))
        return FileText(response.getString("content"), response.getString("hash"),response.optional("save_guard"),response.optional("save_guard_reason"))
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
    suspend fun send(prompt: String, parent: String?, provider: String, model: String, effort: String, context:JSONObject?=null): String = request("POST", "/api/docs/workspace/agent",
        JSONObject().put("prompt", prompt).put("context",context ?: JSONObject()).put("parent_id", parent ?: JSONObject.NULL).put("backend", "python").put("provider", provider).put("model", model).put("effort", effort.ifEmpty { null })).getString("session_id")
    suspend fun stop() { request("POST", "/api/docs/workspace/agent/stop", JSONObject()) }
    suspend fun openTerminal(): String = request("POST", "/api/terminals", JSONObject().put("title", "Terminal")).getString("id")
    suspend fun terminalInput(id: String, data: String) { request("POST", "/api/terminals/$id/input", JSONObject().put("data", data)) }
    suspend fun terminalResize(id: String, rows: Int, cols: Int) { request("POST", "/api/terminals/$id/resize", JSONObject().put("rows", rows).put("cols", cols)) }
    suspend fun terminalScreen(id: String): String {
        val bytes = java.io.ByteArrayOutputStream(); var offset = 0; var revision = ""
        val maxBytes = 50_000
        while (true) {
            val page = request("GET", "/api/terminals/$id/output?view=screen&offset=$offset&limit=16384&revision=" + URLEncoder.encode(revision, "UTF-8"))
            val part = page.getJSONArray("bytes")
            val nextRevision = page.getString("revision")
            require(part.length() <= 16_384 && (revision.isEmpty() || revision == nextRevision)) {
                "The retained terminal screen changed. Refresh the current screen."
            }
            val size = minOf(part.length(), maxBytes - bytes.size())
            bytes.write(ByteArray(size) { part.getInt(it).toByte() })
            if (page.isNull("next") && size == part.length()) return bytes.toString("UTF-8")
            if (bytes.size() == maxBytes) return bytes.toString("UTF-8") + "\n[Terminal screen truncated.]"
            val next = page.getInt("next")
            require(next > offset && next - offset == part.length()) { "Invalid retained terminal page." }
            offset = next
            revision = nextRevision
        }
    }

}
data class NativeDebugThread(val id:Long,val name:String)
data class NativeThreadReading(val session:String,val stop:String,val selected:Long,val request:String,val threads:List<NativeDebugThread>)
fun nativeThreadReading(event:JSONObject,session:String,stop:String,selected:Long,request:String):NativeThreadReading {
    fun requireReading(value:Boolean){require(value){"Thread observation changed or is unavailable. Read threads again; the paused stack is retained."}}
    requireReading(event.optString("event")=="threads"&&event.optString("session")==session&&debugStop(event.opt("stop"))==stop&&event.optString("request_id")==request&&debugInteger(event.opt("selected"))==selected)
    val rows=event.optJSONArray("threads");requireReading(rows!=null&&rows.length()<=4096)
    val identities=mutableSetOf<Long>()
    val threads=(0 until rows!!.length()).map{index->val row=rows.optJSONObject(index);val id=row?.let{debugInteger(it.opt("id"))};val name=row?.opt("name") as? String;requireReading(id!=null&&identities.add(id)&&name!=null&&name.length<=4096);NativeDebugThread(id!!,name!!)}
    requireReading(selected in identities)
    return NativeThreadReading(session,stop,selected,request,threads)
}
fun debugURI(path: String): String = languageURI(path)
data class NativeDebugTarget(val path: String, val line: Int, val end: Int)
private val debugHTTP = OkHttpClient.Builder().retryOnConnectionFailure(false).build()
data class NativeRunOutput(val category: String, val text: String, val source: String?, val line: Int?)
class NativeDebugger(private val engine: Engine, private val scope: CoroutineScope, private val recordedMessages: Boolean = false) {
    var status by mutableStateOf("idle"); private set
    var message by mutableStateOf(""); private set
    var owner by mutableStateOf(""); private set
    var sessionID: String? = null; private set
    var frames by mutableStateOf<List<JSONObject>>(emptyList()); private set
    var variables by mutableStateOf<List<JSONObject>>(emptyList()); private set
    var threadReading by mutableStateOf<NativeThreadReading?>(null);private set
    var threadNotice by mutableStateOf("");private set
    var threadBusy by mutableStateOf(false);private set
    var frameNotice by mutableStateOf("");private set
    var framesCurrent by mutableStateOf(false);private set
    private var stoppedGeneration:String?=null
    private var stoppedThread:Long?=null
    private var threadRequest:String?=null
    private var selectingThread:Long?=null
    private val pendingValues=mutableMapOf<String,String>()
    private var frameRequest:String?=null
    private data class Movement(val request:String,val stop:String,val thread:Long,val frame:Long?,val current:Boolean,val variables:List<JSONObject>,val reading:NativeThreadReading?)
    private var movement:Movement?=null
    private data class SourceTestLaunch(val body:JSONObject,val plan:TestDebugPlan,val facts:SourceTestScan,val current:()->Boolean,val admitted:(String)->Unit,val rejected:(String)->Unit,val outcome:(String,String,Int?)->Unit)
    private var sourceTestDeadline:kotlinx.coroutines.Job?=null
    private var sourceTestPreparing=false
    private var sourceTestLaunch:SourceTestLaunch?=null
    private var sourceTestOutcome:((String,String,Int?)->Unit)?=null
    private data class RecordedLaunch(val uri: String, val hash: String, val closingLine: Int, val current: () -> Boolean)
    private var recordedLaunch: RecordedLaunch? = null
    private var pendingRecordedContext: String? = null
    private var launchRequest:String?=null
    private var launchDocument:String?=null
    private val retiredLaunches=linkedMapOf<String,String>()
    var capabilities by mutableStateOf(JSONObject()); private set
    var catalogNotices by mutableStateOf<List<String>>(emptyList());private set
    var attachReading by mutableStateOf<String?>(null);private set
    private var attaching=false
    val breakpointModel=NativeBreakpointModel()
    val variableTree=NativeVariableTree()
    val adapterSources=NativeAdapterSourceModel()
    val sourceLineage=NativeSourceLineage()
    private val breakpointBuffers=mutableMapOf<String,java.lang.ref.WeakReference<DocumentBuffer>>()
    private val breakpointPaths=mutableMapOf<String,String>()
    private var launchHash:String?=null
    private var launchTicket=0L
    private var runSourceHashes by mutableStateOf<Map<String,String>>(emptyMap())
    private var launchSourceBindings=emptyMap<String,Pair<NativeSourceSnapshot,String>>()
    val breakpoints get()=breakpointModel.intents.values.associate{intent->val buffer=breakpointBuffers[intent.source.identity]?.get();intent.source.path to (buffer?.let{breakpointRows(it)} ?: if(active)emptyList() else breakpointModel.rows(intent.source,null,null))}
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
    private var selectedFrame by mutableStateOf<Long?>(null)
    private var topVariables: List<JSONObject> = emptyList()
    private var stopPending = false
    private var epoch = UUID.randomUUID()
    private var socket: WebSocket? = null
    var onLocation: ((String, Int) -> Unit)? = null
    var emitForTest: ((JSONObject) -> Unit)? = null
    var observeRequestForTest: ((JSONObject) -> Unit)? = null
    val active get() = status in listOf("starting", "running", "paused", "stopping")
    val hasInspection get()=active||owner.isNotEmpty()&&(status!="idle"||message.isNotBlank()||profiles.isNotEmpty()||offer!=null||output.isNotEmpty())
        ||breakpointModel.intents.isNotEmpty()||adapterSources.reading!=null
    private fun isRecordedSource(buffer: DocumentBuffer): Boolean = recordedMessages && buffer.file.generatedBy == "conversation"
    private fun sourceSnapshot(buffer:DocumentBuffer)=NativeSourceSnapshot(buffer.debugIdentity,buffer.file.path,buffer.text,!buffer.dirty&&!buffer.saving,(!buffer.readOnly||isRecordedSource(buffer))&&!buffer.saving)
    private fun currentBreakpointSource(identity:String)=breakpointBuffers[identity]?.get()?.let{sourceSnapshot(it)}
    fun registerSource(buffer:DocumentBuffer,confirmedAddressChange:Boolean=false){
        breakpointBuffers[buffer.debugIdentity]=java.lang.ref.WeakReference(buffer);breakpointPaths[buffer.file.path]=buffer.debugIdentity
        if(confirmedAddressChange)breakpointModel.readdress(sourceSnapshot(buffer))
    }
    fun breakpointReadingPage(buffer:DocumentBuffer,page:Int)=breakpointModel.readingPage(sourceSnapshot(buffer),page)
    fun breakpointReadings(buffer:DocumentBuffer)=breakpointModel.readings(sourceSnapshot(buffer))
    fun breakpointRows(buffer:DocumentBuffer):List<JSONObject>{val source=sourceSnapshot(buffer);if(active&&!sourceMatches(buffer))return emptyList();return breakpointModel.rows(source,sessionID,sourceLineage.hash(source))}
    fun breakpointNotice(buffer:DocumentBuffer):String{val source=sourceSnapshot(buffer);if(active&&!sourceMatches(buffer))return "Source positions and breakpoint intentions are unbound to this editor identity or resource namespace. The runtime session and previous stack remain available; restart or review restoration before applying coordinates.";return breakpointModel.notice(source,sessionID,sourceLineage.hash(source))}
    fun resourceSourcesChanged(routes:List<NativeSourceRoute>){
        sourceLineage.transition(routes);breakpointModel.retireSession()
        if(active)message="Source namespace changed. Runtime session controls and history remain available, but positions and breakpoint mutations are unbound. Restart from current saved source, or review a guarded restoration of the original editor identity."
    }
    fun discardBreakpointIntentions(buffer:DocumentBuffer){if(active){message="Stop the current session before discarding stale breakpoint intentions.";return};breakpointModel.discard(buffer.debugIdentity)}
    fun line(path: String): Int? {
        if (attaching||status != "paused"||!framesCurrent) return null
        val frame = frames.firstOrNull { it.optLong("id") == selectedFrame } ?: frames.firstOrNull()
        if((frame?.opt("source_reference") as? Number)?.toDouble()?.let{it>0}==true)return null
        if (recordedMessages) {
            val reading = recordedLaunch ?: return null
            return frame?.takeIf { path == owner && reading.current() && it.optBoolean("in_document") &&
                it.optString("source") == reading.uri && it.optString("source_hash") == reading.hash && !it.isNull("line") }?.getInt("line")
        }
        if (path == owner && frame?.optBoolean("in_document") == true) return if (frame.isNull("line")) null else frame.getInt("line")
        if (frame?.optString("source") == path) return if (frame.isNull("source_line")) null else frame.getInt("source_line")
        return if (path == owner && selectedFrame == frames.firstOrNull()?.optLong("id")) pausedLine else null
    }
    fun outputPlacements(buffer: DocumentBuffer): Map<Int, String> {
        if (!sourceMatches(buffer)) return emptyMap()
        if (recordedMessages) return recordedLaunch?.takeIf { it.current() && output.isNotEmpty() }
            ?.let { mapOf(it.closingLine to output.joinToString("") { event -> event.text }) }.orEmpty()
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
    fun sourceEditor(runtime:String):DocumentBuffer?=sourceLineage.sources[runtime]?.identity?.let{breakpointBuffers[it]?.get()}
    fun sourceMatches(buffer: DocumentBuffer) = !attaching&&!buffer.resourceDeleted&&
        (!recordedMessages || recordedLaunch?.current?.invoke() == true) && sourceLineage.matches(sourceSnapshot(buffer))
    val adapterSourceRequired get()=frames.firstOrNull{debugInteger(it.opt("id"))==selectedFrame}?.let{(it.opt("source_reference") as? Number)?.toString()?.toLongOrNull()?.let{reference->reference>0}}==true
    fun positionSourceMatches(buffer:DocumentBuffer)=!adapterSourceRequired&&sourceMatches(buffer)
    suspend fun rememberSource(buffer: DocumentBuffer) {
        if(attaching||buffer.file.path==owner||status!="paused"||!framesCurrent||buffer.dirty)return
        val frame=frames.firstOrNull{debugInteger(it.opt("id"))==selectedFrame} ?: return
        if((frame.opt("source_reference") as? Number)?.toString()?.toLongOrNull()?.let{it>0}==true)return
        if(frame.optional("source")!=buffer.file.path)return
        val expected=frame.optional("source_hash")?.removePrefix("sha256:")?.lowercase()?.takeIf{it.matches(Regex("[0-9a-f]{64}"))} ?: return
        val source=buffer.text;val file=buffer.file;val session=sessionID;val stop=stoppedGeneration;val thread=stoppedThread;val selected=selectedFrame;val namespace=sourceLineage.epoch
        val hash=kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.Default){java.security.MessageDigest.getInstance("SHA-256").digest(source.toByteArray(Charsets.UTF_8)).joinToString(""){"%02x".format(it)}}
        if(status=="paused"&&framesCurrent&&sessionID==session&&stoppedGeneration==stop&&stoppedThread==thread&&selectedFrame==selected&&buffer.file===file&&buffer.text==source&&!buffer.dirty&&!buffer.resourceDeleted&&hash==expected&&sourceLineage.learn(file.path,sourceSnapshot(buffer),expected,namespace)){registerSource(buffer);sourceSnapshots=sourceSnapshots+(file.path to source);runSourceHashes=runSourceHashes+(file.path to expected)}
    }
    fun locationGuard(path:String,line:Int):()->Boolean {
        val session=sessionID;val stop=stoppedGeneration;val thread=stoppedThread;val frame=selectedFrame;val connection=epoch;val namespace=sourceLineage.epoch
        return {sourceLineage.epoch==namespace&&epoch==connection&&sessionID==session&&status=="paused"&&framesCurrent&&stoppedGeneration==stop&&stoppedThread==thread&&selectedFrame==frame&&this.line(path)==line}
    }
    suspend fun prepare(buffer: DocumentBuffer, line: Int = 0, program: String? = null,chooseProfile:Boolean=false) {
        if (recordedMessages) { message = "Choose Debug on a complete recorded code fence."; return }
        if (buffer.startupObservation) { message = "Refresh Files to confirm this source before debugging."; return }
        if (active) { message = "Stop the current debugger before launching another program."; return }
        if (buffer.dirty || buffer.saving) { message = "Save this file before debugging so the adapter runs the displayed text."; return }
        close(); val current = epoch;val namespace=sourceLineage.epoch
        registerSource(buffer)
        owner = buffer.file.path; launchBuffer = buffer; launchSource = buffer.text; sourceSnapshots = mapOf(owner to launchSource); message = "Resolving launch targets…"; targets = emptyList()
        try {
            val literate = buffer.file.docID != null || owner.substringAfterLast('.').lowercase() in listOf("md", "markdown")
            var target = program ?: inlineDebugPath(buffer.text, owner, line, true) ?: inlineDebugPath(buffer.text, owner, line)
            if (literate && target == null) {
                val id = buffer.file.docID ?: run { message = "Refresh files to discover this document before choosing a file block to debug."; return }
                val rendered = engine.request("GET", "/api/docs/$id/render")
                if (epoch != current || sourceLineage.epoch!=namespace || buffer.file.path!=owner || buffer.resourceDeleted || buffer.text != launchSource || buffer.dirty) return
                targets = rendered.getJSONArray("blocks").objects().mapNotNull { block ->
                    val span = block.optJSONArray("span"); val bytes = launchSource.toByteArray(Charsets.UTF_8)
                    if (block.optString("kind") != "file" || block.isNull("path") || span == null || span.length() != 2 || span.getInt(0) < 0 || span.getInt(1) < span.getInt(0) || span.getInt(1) > bytes.size) null
                    else NativeDebugTarget(block.getString("path"), bytes.take(span.getInt(0)).count { it == 10.toByte() }, bytes.take(span.getInt(1)).count { it == 10.toByte() })
                }
                target = targets.firstOrNull { line in it.line..it.end }?.path
                if (target == null&&!chooseProfile) { message = if (targets.isEmpty()) "No file blocks to debug. Add a supported file block and save this document." else "Choose the file block to debug."; return }
            }
            launchProgram = target
            val query = JSONObject().put("uri", debugURI(owner)).put(if (literate) "expected_document" else "expected_source", launchSource)
            if (target != null) query.put("program", target)
            val result = engine.request("POST", "/api/launch-profiles", query)
            if (epoch != current || sourceLineage.epoch!=namespace || buffer.file.path!=owner || buffer.resourceDeleted || buffer.text != launchSource || buffer.dirty) return
            catalogNotices=nativeCatalogNotices(result)
            val protocol=(result.opt("debug_protocol") as? Number)?.toString()?.toIntOrNull()
            if(protocol==null||protocol<3){catalog=null;profiles=emptyList();offer=null;message="Debug unavailable: this engine does not support source-bound debugger protocol 3. Reconnect with the updated engine before launching; the source editor remains unchanged.";return}
            catalog = result; profiles = result.getJSONArray("profiles").objects().filter { it.opt("can_debug")==true }.onEach{nativeAttachProfile(it)}
            val automatic = profiles.firstOrNull { it.optString("program") == (target ?: result.optString("viewed", owner)) } ?: profiles.firstOrNull{it.opt("default")==true&&nativeAttachProfile(it)!=null} ?: profiles.singleOrNull()
            if (!chooseProfile&&automatic != null) { launch(automatic); return }
            message = if (profiles.isEmpty()) "No debug launch targets. Configure a debug profile in this project and try again." else "Choose a debug target."
        } catch (cancelled: kotlinx.coroutines.CancellationException) { throw cancelled }
        catch (failure: Exception) { if (epoch == current) message = "${failure.message} Save or refresh this file and try debugging again." }
    }
    suspend fun launchSourceTest(buffer:DocumentBuffer,body:JSONObject,plan:TestDebugPlan,facts:SourceTestScan,current:()->Boolean,admitted:(String)->Unit,rejected:(String)->Unit,outcome:(String,String,Int?)->Unit) {
        if(active||!current()||!plan.available||buffer.dirty||buffer.saving||buffer.resourceDeleted){rejected("Test Debug context changed or another debugger is active. No Start was sent.");return}
        close();registerSource(buffer);owner=buffer.file.path;launchBuffer=buffer;launchSource=buffer.text
        val profile=JSONObject().put("id","source-test")
        catalog=JSONObject().put("fingerprint","");profiles=emptyList();launchProgram=null
        val pending=SourceTestLaunch(JSONObject(body.toString()),plan,facts,current,admitted,rejected,outcome)
        sourceTestLaunch=pending;catalogNotices=plan.notices
        launch(profile,pending)
    }

    suspend fun prepareRecordedFence(buffer: DocumentBuffer, reading: AgentFenceReading, line: Int) {
        if (active) { message = "Stop the current Agent debugger before launching another fence."; return }
        if (!isRecordedSource(buffer) || !reading.current() || buffer.text != reading.source) return
        val closing = buffer.inlineMarkup.takeIf { it.source == reading.source }?.debug?.recordedClosing?.get(line) ?: return
        close()
        val preparation = epoch
        val workspace = engine.sourceRootAddress
        owner = buffer.file.path
        launchBuffer = buffer
        launchSource = reading.source
        var requested: String? = null
        try {
            check(workspace != null) { "The workspace identity is unconfirmed. Refresh Files and try Debug again." }
            val snapshot = sourceSnapshot(buffer)
            val hash = withContext(Dispatchers.Default) { debugHash(snapshot.text) }
            if (epoch != preparation || !reading.current() || snapshot != sourceSnapshot(buffer)) return
            val proof = engine.request("POST", reading.endpoint("fence-debug"),
                JSONObject().put("side", reading.side).put("source_hash", hash).put("opening_line", line))
            val id = proof.optional("id")?.takeIf { it.matches(Regex("[a-f0-9]{16}")) }
            val uri = proof.optional("uri")
            check(id != null && uri == "hick-project:///$id/message.md" && proof.opt("version") == 1 &&
                proof.opt("started") == false && proof.optString("turn") == reading.turn && proof.optString("side") == reading.side &&
                proof.optString("source_hash") == hash && debugInteger(proof.opt("opening_line")) == line.toLong() &&
                proof.optString("workspace") == workspace) { "Recorded fence did not match this message. Refresh the conversation and try Debug again." }
            requested = id
            if (epoch != preparation || !reading.current() || snapshot != sourceSnapshot(buffer) || engine.sourceRootAddress != workspace) return
            registerSource(buffer)
            val intent = breakpointModel.intents[buffer.debugIdentity]?.takeIf { breakpointModel.matches(it, snapshot) }
            intent?.let { breakpointModel.stamp(buffer.debugIdentity, it.revision, hash) }
            launchHash = hash
            launchSourceBindings = mapOf(buffer.debugIdentity to (snapshot to hash))
            sourceLineage.begin(launchSourceBindings.values.toList())
            recordedLaunch = RecordedLaunch(checkNotNull(uri), hash, closing, reading.current)
            onLocation = { path, pausedLine ->
                val current = locationGuard(path, pausedLine)
                if (path == buffer.file.path && current() && sourceMatches(buffer)) {
                    buffer.debugRevealSource = reading.source
                    buffer.debugRevealCurrent = { current() && sourceMatches(buffer) }
                    buffer.debugReveal = pausedLine to ((buffer.debugReveal?.second ?: 0) + 1)
                }
            }
            pendingRecordedContext = id
            requested = null
            startWire(checkNotNull(uri)) { request ->
                breakpointModel.request(snapshot, "pending-start", request)
                JSONObject().put("op", "start").put("doc", uri).put("request_id", request).put("source_hash", hash)
                    .put("breakpoints", JSONArray(intent?.marks.orEmpty()))
            }
        } catch (cancelled: kotlinx.coroutines.CancellationException) { throw cancelled }
        catch (failure: Exception) {
            if (epoch == preparation) fail("${failure.message} Reopen the recorded fence and choose Debug again.")
        } finally {
            requested?.let { releaseRecordedContext(it) }
        }
    }

    private fun releaseRecordedContext(id: String? = pendingRecordedContext) {
        if (id == null) return
        if (pendingRecordedContext == id) pendingRecordedContext = null
        scope.launch(start = kotlinx.coroutines.CoroutineStart.UNDISPATCHED) {
            withContext(kotlinx.coroutines.NonCancellable) {
                runCatching { engine.request("DELETE", "/api/project-contexts/$id") }
            }
        }
    }

    suspend fun launch(profile:JSONObject)=launch(profile,null)
    private suspend fun launch(profile: JSONObject,test:SourceTestLaunch?) {
        val buffer = launchBuffer; val catalog = catalog
        if (buffer == null || catalog == null || (test==null&&profiles.none { it.optString("id") == profile.optString("id") }) || buffer.text != launchSource || buffer.file.path!=owner || buffer.resourceDeleted || buffer.dirty || buffer.saving || active) {
            message = "The source or launch target changed. Save and choose Debug again."; return
        }
        if(test!=null&&!test.current()){sourceTestLaunch=null;test.rejected("Test Debug context changed before launch. No Start was sent.");return}
        val attach=try{nativeAttachProfile(profile)}catch(e:Exception){message=e.message.orEmpty();return};attaching=attach!=null;attachReading=attach?.notice
        val preparation=epoch;val ticket=++launchTicket;val namespace=sourceLineage.epoch;val primarySource=sourceSnapshot(buffer)
        val candidates=if(attaching)emptyList() else breakpointModel.intents.values.filter{intent->val current=currentBreakpointSource(intent.source.identity);current!=null&&current.saved&&breakpointPaths[current.path]==current.identity&&breakpointModel.matches(intent,current)}
        val hashes=kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.Default){candidates.associate{it.source.identity to debugHash(it.source.text)} to debugHash(primarySource.text)}
        if((test!=null&&!test.current())||sourceLineage.epoch!=namespace||sourceSnapshot(buffer)!=primarySource||epoch!=preparation||ticket!=launchTicket||active||buffer.text!=launchSource||buffer.dirty||buffer.saving||candidates.any{intent->val current=currentBreakpointSource(intent.source.identity);current==null||!current.saved||!breakpointModel.matches(intent,current)||breakpointModel.intents[intent.source.identity]?.revision!=intent.revision}){message="Source or breakpoint intentions changed while preparing launch. Review and choose Debug again.";sourceTestLaunch=null;test?.rejected?.invoke(message);return}
        for(intent in candidates)breakpointModel.stamp(intent.source.identity,intent.revision,hashes.first.getValue(intent.source.identity))
        launchHash=hashes.second
        launchSourceBindings=candidates.associate{it.source.identity to (it.source to hashes.first.getValue(it.source.identity))}+(buffer.debugIdentity to (sourceSnapshot(buffer) to hashes.second))
        if(attaching){launchSourceBindings=emptyMap();sourceLineage.clear()}else sourceLineage.begin(launchSourceBindings.values.toList())
        val address = test?.plan?.sourceURI ?: debugURI(owner)
        startWire(address) { requestID ->
            val request = JSONObject().put("op", "start").put("request_id",requestID).put("source_hash",launchHash).put("doc", address)
                .put("profile", JSONObject().put("id", profile.getString("id")).put("fingerprint", catalog.getString("fingerprint")))
            if(!attaching)launchProgram?.let { request.put("program", it) }
            val sources=JSONObject();val sourceHashes=JSONObject();var primary=JSONArray()
            for(intent in candidates){val source=currentBreakpointSource(intent.source.identity) ?: continue;breakpointModel.request(source,"pending-start",requestID);if(source.path==owner)primary=JSONArray(intent.marks) else {sources.put(debugURI(source.path),JSONArray(intent.marks));sourceHashes.put(debugURI(source.path),hashes.first.getValue(source.identity))}}
            request.put("breakpoints",primary).put("sources",sources).put("source_hashes",sourceHashes)
            if(test!=null){request.put("op","start_test").put("source_test",test.body);request.remove("profile");request.remove("program")}
            if(test!=null)waitForSourceTest(requestID,30_000,preparing=false)
            request
        }
    }

    private fun startWire(address: String, request: (String) -> JSONObject) {
        status = "starting"; message = ""; profiles = emptyList(); values = emptyMap(); variables = emptyList(); frames = emptyList(); pausedLine = null; offer = null
        val requestID = UUID.randomUUID().toString()
        launchRequest = requestID
        launchDocument = address
        val current = epoch
        if (emitForTest == null) socket = debugHTTP.newWebSocket(Request.Builder().url(engine.debugURL).build(), object : WebSocketListener() {
            override fun onMessage(webSocket: WebSocket, bytes: ByteString) {
                val data = bytes.toByteArray()
                if (data.size < 2 || data[0] != 3.toByte()) return
                val event = runCatching { JSONObject(data.copyOfRange(1, data.size).toString(Charsets.UTF_8)) }.getOrNull() ?: return
                scope.launch { if (epoch == current) receive(event) else if(event.optString("event")=="started")cleanupRetiredLaunch(event) }
            }
            override fun onFailure(webSocket: WebSocket, failure: Throwable, response: Response?) {
                scope.launch { if (epoch == current) fail("Debugger disconnected: ${failure.message}. Reconnect explicitly; commands were not retried.") }
            }
            override fun onClosing(webSocket: WebSocket, code: Int, reason: String) { webSocket.close(code, reason) }
            override fun onClosed(webSocket: WebSocket, code: Int, reason: String) {
                scope.launch { if (epoch == current && active) fail("Debugger connection closed. Reconnect explicitly; commands were not retried.") }
            }
        })
        send(request(requestID))
    }
    private fun waitForSourceTest(request:String,milliseconds:Long,preparing:Boolean) {
        sourceTestDeadline?.cancel()
        sourceTestPreparing=preparing
        sourceTestDeadline=scope.launch {
            kotlinx.coroutines.delay(milliseconds)
            expireSourceTestPreparation(request,preparing)
        }
    }
    internal fun expireSourceTestPreparation(request:String,preparing:Boolean=false) {
        if(request!=launchRequest||sourceTestLaunch==null||sessionID!=null||sourceTestPreparing!=preparing)return
        val phase=if(preparing)"Test Debug session was not confirmed within the preparation wait."
                 else "Test Debug admission was not confirmed within 30 seconds."
        fail("$phase No request was retried. A late owned session will be stopped.")
    }
    private fun send(request: JSONObject) {
        observeRequestForTest?.invoke(JSONObject(request.toString()))
        emitForTest?.let { it(request); return }
        if (socket?.send((byteArrayOf(3) + request.toString().toByteArray(Charsets.UTF_8)).toByteString()) != true) fail("Debugger command was not sent. Reconnect explicitly; it was not retried.")
    }
    private fun retireThreads(forgetStop:Boolean=false){adapterSources.retire();variableTree.clear();threadReading=null;threadNotice="";threadBusy=false;threadRequest=null;selectingThread=null;pendingValues.clear();frameRequest=null;frameNotice="";framesCurrent=false;if(forgetStop){stoppedGeneration=null;stoppedThread=null;selectedFrame=null}}
    private fun retireLaunch(){sourceTestPreparing=false;val id=launchRequest;val doc=launchDocument;if(id!=null&&doc!=null){retiredLaunches[id]=doc;while(retiredLaunches.size>64)retiredLaunches.remove(retiredLaunches.keys.first())};launchRequest=null;launchDocument=null}
    private fun cleanupRetiredLaunch(event:JSONObject):Boolean {
        val request=event.optional("request_id") ?: return false
        if(retiredLaunches[request]!=event.optString("doc"))return false
        retiredLaunches.remove(request)
        val session=event.optional("session") ?: return true
        if(session==sessionID)return true
        val stop=JSONObject().put("op","stop").put("session",session);val emit=emitForTest
        if(emit!=null)emit(stop) else if(socket?.send((byteArrayOf(3)+stop.toString().toByteArray(Charsets.UTF_8)).toByteString())!=true)message="Cleanup of retired owned debugger session $session was not confirmed. Inspect it before further debug work; no cleanup command was retried."
        return true
    }
    private fun fail(detail: String) { releaseRecordedContext();sourceTestDeadline?.cancel();sourceTestDeadline=null;val pending=sourceTestLaunch;sourceTestLaunch=null;pending?.rejected?.invoke(detail);sessionID?.let{sourceTestOutcome?.invoke(it,"failed",null)};sourceTestOutcome=null;sourceLineage.clear();movement=null;retireLaunch();retireThreads(true);breakpointModel.retireSession();runSourceHashes=emptyMap();status = "failed"; message = detail; sessionID = null; pausedLine = null; variables = emptyList(); frames = emptyList(); values = emptyMap() }
    fun close() { releaseRecordedContext();recordedLaunch=null;attaching=false;attachReading=null;sourceTestDeadline?.cancel();sourceTestDeadline=null;sourceTestLaunch?.rejected?.invoke("Test Debug preparation closed. No replacement was requested.");sourceTestLaunch=null;sessionID?.let{sourceTestOutcome?.invoke(it,"closed",null)};sourceTestOutcome=null;sourceLineage.clear();catalogNotices=emptyList();movement=null;launchTicket++;retireLaunch();retireThreads(true);breakpointModel.retireSession();runSourceHashes=emptyMap();stopPending = false; epoch = UUID.randomUUID(); socket?.cancel(); socket = null; sessionID = null; status = "idle"; pausedLine = null; frames = emptyList(); variables = emptyList(); values = emptyMap(); profiles = emptyList() }
    fun receive(event: JSONObject) {
        val kind = event.optString("event")
        if(kind=="test_preparing"){
            val request=launchRequest ?: return
            val test=sourceTestLaunch ?: return
            val wait=debugInteger(event.opt("wait_ms")) ?: return
            if(sourceTestPreparing||status !in listOf("starting","stopping")||
               !test.current()||event.optional("request_id")!=request||event.optional("doc")!=launchDocument||
               event.has("session")||wait !in 30_001L..180_000L||
               !test.plan.admits(event.optJSONObject("source_test"),test.facts))return
            waitForSourceTest(request,wait+5_000,preparing=true)
            if(status=="starting")message="Preparing the saved test. Waiting for the debugger session…"
            return
        }
        if(kind=="started"){
            val request=event.optional("request_id");val doc=event.optString("doc")
            if(request!=launchRequest||doc!=launchDocument){
                cleanupRetiredLaunch(event)
                return
            }
            if(request==null||status !in listOf("starting","stopping"))return
        }else if(kind=="build"||(kind=="failed"&&event.isNull("session"))){
            if(event.optional("request_id")==null||event.optional("request_id")!=launchRequest||event.optString("doc")!=launchDocument)return
            if(kind=="build"&&!active||kind=="failed"&&status !in listOf("starting","stopping"))return
        } else if (event.optional("session") == null || event.optional("session") != sessionID) return
        when (kind) {
            "started" -> {
                val test=sourceTestLaunch
                if(test!=null&&(!test.current()||!test.plan.admits(event.optJSONObject("source_test"),test.facts))){fail("Test Debug receipt or source context changed. The owned session was not adopted.");cleanupRetiredLaunch(event);return}
                if (recordedLaunch?.current?.invoke() == false) {
                    fail("The recorded message changed during Debug preparation. The owned session was not adopted.")
                    cleanupRetiredLaunch(event)
                    return
                }
                if((!attaching&&event.optString("source_hash")!=launchHash)||(attaching&&!event.isNull("source_hash")&&event.optString("source_hash")!=launchHash)){fail("Debugger source receipt does not match this launch. Its owned session was not adopted; review source and launch again.");cleanupRetiredLaunch(event);return}
                releaseRecordedContext()
                retireThreads(true);message="";output = emptyList(); sessionID = event.getString("session");capabilities = event.optJSONObject("capabilities") ?: JSONObject();status = "running"
                runSourceHashes=launchSourceBindings.values.associate{it.first.path to it.second};sourceSnapshots=launchSourceBindings.values.associate{it.first.path to it.first.text};breakpointModel.adoptStart(launchRequest!!,sessionID!!);bind(event,owner)
                if(test!=null){sourceTestDeadline?.cancel();sourceTestDeadline=null;sourceTestPreparing=false;sourceTestLaunch=null;sourceTestOutcome=test.outcome;test.admitted(sessionID!!)}
                if(stopPending){stopPending=false;stop()}
            }
            "stopped" -> {
                if(status=="stopping")return
                val stop=debugStop(event.opt("stop"));val thread=debugInteger(event.opt("thread"));val previous=stoppedGeneration
                if(previous!=null&&(stop==null||java.math.BigInteger(stop)<java.math.BigInteger(previous)||(stop==previous&&status!="paused")))return
                val previousAdapterSource=adapterSourceBinding()
                val changed=stop!=previous||thread!=stoppedThread
                val selectedReply=stop!=null&&thread!=null&&selectingThread==thread&&threadRequest!=null&&threadBusy
                if(changed){message="";threadReading=null;values=emptyMap();pendingValues.clear();frameRequest=null;if(!selectedReply){threadRequest=null;selectingThread=null;threadBusy=false}}
                stoppedGeneration=stop;stoppedThread=thread
                if(movement?.let{stop!=it.stop}==true)movement=null
                val stackError=event.optional("stack_error");val variablesError=event.optional("variables_error")
                if(stackError!=null){values=emptyMap();pendingValues.clear();frameRequest=null}
                frameNotice=listOfNotNull(stackError?.let{"Stack unavailable: $it"},variablesError?.let{"Frame locals unavailable: $it"}).joinToString("\n")
                status = "paused";if(stackError==null)frames = event.optJSONArray("frames")?.objects().orEmpty();framesCurrent=stackError==null&&stop!=null&&thread!=null
                variables = if(stackError==null&&variablesError==null)event.optJSONArray("variables")?.objects().orEmpty() else emptyList();topVariables = variables
                pausedLine = if (event.isNull("line")) null else event.getInt("line"); selectedFrame = if (!changed&&frames.any{debugInteger(it.opt("id"))==selectedFrame})selectedFrame else if (!event.isNull("selected_frame")) event.getLong("selected_frame") else frames.firstOrNull()?.let{debugInteger(it.opt("id"))}
                if(!framesCurrent)selectedFrame=null
                variables=if(selectedFrame==frames.firstOrNull()?.let{debugInteger(it.opt("id"))})topVariables else emptyList()
                syncVariableTree()
                if(previousAdapterSource!=adapterSourceBinding())adapterSources.retire()
                val top = frames.firstOrNull().takeIf{framesCurrent}; val path = top?.optional("source"); val line = top?.let { if (it.isNull("source_line")) null else it.getInt("source_line") }
                if(top?.let{(it.opt("source_reference") as? Number)?.toDouble()?.let{reference->reference>0}==true}!=true){
                 if (path != null && line != null && !path.startsWith('/') && ':' !in path) onLocation?.invoke(path, line)
                 else if(framesCurrent)pausedLine?.let { onLocation?.invoke(owner, it) }
                }
                watches.forEach { evaluate(it, true) }
                if(stop==null||thread==null)threadNotice="Thread inspection requires a current engine pause observation. Reconnect with the updated engine; the paused stack remains readable."
                else if(!selectedReply&&changed)readThreads()
            }
            "threads" -> {
                if(status!="paused"||event.optional("request_id")!=threadRequest||threadRequest==null)return
                try{threadReading=nativeThreadReading(event,sessionID!!,stoppedGeneration ?: return,stoppedThread ?: return,threadRequest!!);threadNotice="${threadReading!!.threads.size} reported threads for this pause."}
                catch(error:IllegalArgumentException){threadNotice=error.message.orEmpty()}
                threadRequest=null;selectingThread=null;threadBusy=false
            }
            "frame_variables" -> {
                val request=frameRequest ?: return
                if(status!="paused"||!framesCurrent||event.optional("request_id")!=request||debugStop(event.opt("stop"))!=stoppedGeneration||debugInteger(event.opt("thread"))!=stoppedThread||debugInteger(event.opt("frame"))!=selectedFrame)return
                try{val rows=event.optJSONArray("variables");require(rows!=null&&rows.length()<=4096){"Frame locals exceed the supported reading. Choose another frame or evaluate a named value."};val next=(0 until rows!!.length()).map{rows.optJSONObject(it) ?: error("Malformed frame locals. Read this frame again.")};variables=next;syncVariableTree();frameNotice="${next.size} reported locals for this frame."}
                catch(error:Exception){frameNotice=error.message.orEmpty()};frameRequest=null
            }
            "adapter_source" -> scope.launch{adapterSources.receive(event){adapterSourceBinding().takeIf{capabilities.opt("source_content")==true}}}
            "children" -> variableTree.admit(event)
            "breakpoints" -> bind(event,owner)
            "source_breakpoints" -> event.optional("source")?.let{SemanticLocation(it,0,0).path}?.let{bind(event,it)}
            "run_output" -> output = (output + event.optJSONArray("events")?.objects().orEmpty().map { item -> NativeRunOutput(item.optString("category", "stdout"), item.optString("text"), item.optional("source"), if (item.has("line") && !item.isNull("line")) item.optInt("line") else null) }).takeLast(256)
            "value" -> if (status == "paused") {
                val expression=event.optString("expression")
                if(pendingValues[expression]==null||debugStop(event.opt("stop"))!=stoppedGeneration||debugInteger(event.opt("thread"))!=stoppedThread||debugInteger(event.opt("frame"))!=selectedFrame||event.optional("request_id")!=pendingValues[expression])return
                pendingValues.remove(expression);values = values + (expression to event.getString("value"));variableTree.replace(variableScope(),"eval:$expression",listOf(JSONObject().put("name",expression).put("value",event.getString("value")).put("variables_reference",event.opt("reference"))))
            }
            "finished", "ended" -> { sessionID?.let{sourceTestOutcome?.invoke(it,kind,if(event.isNull("exit_code"))null else event.optInt("exit_code"))};sourceTestOutcome=null;movement=null;retireThreads(true);status = "finished"; sessionID = null; pausedLine = null; frames = emptyList(); variables = emptyList(); values = emptyMap() }
            "failed" -> {
                val request=event.optional("request_id");if(request!=null&&adapterSources.refuse(request,event.optString("message")))return;if(request!=null&&sessionID?.let{breakpointModel.refuse(it,request,event.optString("message"))}==true){message="Breakpoint update refused: "+event.optString("message")+" Review the current source before applying again.";return};if(request!=null&&variableTree.refuse(request,event.optString("message")+" Expand again explicitly; the paused stack is retained."))return;val expression=pendingValues.entries.firstOrNull{it.value==request}?.key
                movement?.takeIf{request==it.request}?.let{old->if(event.optBoolean("scope_retired")){movement=null;retireThreads(true);status="running";variables=emptyList();values=emptyMap();message=event.optString("message")+" Movement resumed execution, but a new pause is not confirmed. Historical frames are readable; positions and evaluation are unavailable. Stop remains available.";return};movement=null;status="paused";stoppedGeneration=old.stop;stoppedThread=old.thread;selectedFrame=old.frame;framesCurrent=old.current;variables=old.variables;threadReading=old.reading;syncVariableTree();message=event.optString("message")+" The move was refused; the prior reported pause is retained. Inspect it before another move.";return}
                if(request!=null&&request!=threadRequest&&request!=frameRequest&&request!=launchRequest&&expression==null)return
                if(request!=null&&request==threadRequest){threadRequest=null;selectingThread=null;threadBusy=false;threadNotice=event.optString("message")+" Read threads again; the paused stack is retained.";return}
                if(request!=null&&request==frameRequest){frameRequest=null;frameNotice=event.optString("message")+" Read locals again or choose another current frame/thread; the paused stack is retained.";return}
                expression?.let{pendingValues.remove(it);message=event.optString("message")+" Choose another current frame/thread or evaluate at a later pause.";return}
                offer = event.optJSONObject("offer_install"); if (sessionID == null) fail(event.optString("message", "Debug launch failed. Check adapter installation and try again.")) else message = event.optString("message") + " Review the paused state before trying again."
            }
            "build" -> message = event.optJSONArray("events")?.objects()?.joinToString("") { it.optString("data") }.orEmpty()
        }
    }
    private fun bind(event:JSONObject,path:String){
        if(attaching)return
        val identity=breakpointPaths[path] ?: return;val source=currentBreakpointSource(identity) ?: return
        val session=sessionID ?: return;val request=event.optional("request_id") ?: return;val hash=event.optional("source_hash") ?: return
        if(runSourceHashes[path]!=hash)return
        breakpointModel.admit(source,event,session,request,hash)
    }
    fun toggle(buffer:DocumentBuffer,line:Int,condition:String?=null){
        registerSource(buffer);val source=sourceSnapshot(buffer)
        val intent=try{breakpointModel.toggle(source,line,condition)}catch(error:Exception){message=error.message.orEmpty();return}
        val connection=epoch;val session=sessionID
        scope.launch{
            val hash=kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.Default){debugHash(source.text)}
            val current=currentBreakpointSource(source.identity) ?: return@launch
            if(epoch!=connection||!breakpointModel.matches(intent,current)||breakpointModel.intents[source.identity]?.revision!=intent.revision)return@launch
            breakpointModel.stamp(source.identity,intent.revision,hash)
            if(session==null||sessionID!=session||!current.saved||runSourceHashes[source.path]!=hash||!sourceMatches(buffer))return@launch
            val id=UUID.randomUUID().toString();val request=breakpointModel.request(current,session,id) ?: return@launch
            send(JSONObject().put("op",if(source.path==owner)"breakpoints" else "source_breakpoints").put("session",session).put("source",debugURI(source.path)).put("source_hash",hash).put("request_id",id).put("breakpoints",JSONArray(breakpointModel.intents.getValue(request.identity).marks)))
        }
    }
    fun readThreads() {
        if(selectingThread!=null){threadNotice="Wait for the selected thread's paused stack before refreshing threads.";return}
        val session=sessionID;val stop=stoppedGeneration;val thread=stoppedThread
        if(status!="paused"||session==null||stop==null||thread==null){threadNotice="Pause with a current engine observation before reading threads.";return}
        val request=UUID.randomUUID().toString();threadRequest=request;selectingThread=null;threadBusy=true;threadNotice="Reading reported threads…"
        send(JSONObject().put("op","threads").put("session",session).put("request_id",request))
    }
    fun selectThread(reading:NativeThreadReading,thread:Long) {
        if(threadBusy||status!="paused"||threadReading!==reading||reading.session!=sessionID||reading.stop!=stoppedGeneration||reading.selected!=stoppedThread||reading.threads.none{it.id==thread}){threadNotice="Thread selection changed. Read the current thread list and choose again.";return}
        if(thread==reading.selected)return
        adapterSources.retire("Thread selection requested; captured adapter source is historical.");variableTree.clear();val request=UUID.randomUUID().toString();threadRequest=request;selectingThread=thread;threadBusy=true;threadNotice="Reading the selected thread's paused stack…"
        send(JSONObject().put("op","select_thread").put("session",reading.session).put("thread",thread).put("stop",java.math.BigInteger(reading.stop)).put("request_id",request))
    }
    fun step(how: String) {
        val id = sessionID ?: return; if (status != "paused") return
        if(selectingThread!=null){message="Wait for the selected thread's paused stack before stepping, or inspect a reported selection failure.";return}
        val stop=stoppedGeneration;val thread=stoppedThread
        if(stop==null||thread==null){message="A current paused thread observation is required before moving.";return}
        val request=UUID.randomUUID().toString();val frame=selectedFrame.takeIf{framesCurrent};movement=Movement(request,stop,thread,frame,framesCurrent,variables,threadReading)
        retireThreads()
        status = "running"; pausedLine = null; variables = emptyList(); values = emptyMap()
        send(JSONObject().put("op", "step").put("session", id).put("how", how).put("stop",java.math.BigInteger(stop)).put("thread",thread).put("request_id",request).apply { frame?.let { put("frame", it) } })
    }
    fun moveTo(buffer:DocumentBuffer,line:Int,jump:Boolean=false){
        val session=sessionID;val stop=stoppedGeneration;val thread=stoppedThread;val hash=runSourceHashes[owner]
        if(status!="paused"||selectingThread!=null||session==null||stop==null||thread==null||hash==null||buffer.file.path!=owner||buffer.dirty||buffer.saving||!positionSourceMatches(buffer)||line<0||languageOffset(buffer.text,JSONObject().put("line",line).put("character",0))==null){message="Move unavailable: choose a line in the saved, captured primary source at this pause. Unbound or edited text supplies no coordinates.";return}
        if(jump&&!capabilities.optBoolean("goto_targets")){message="This adapter does not support moving the instruction pointer.";return}
        val request=UUID.randomUUID().toString();movement=Movement(request,stop,thread,selectedFrame,framesCurrent,variables,threadReading);retireThreads();status="running";variables=emptyList();values=emptyMap()
        send(JSONObject().put("op",if(jump)"jump" else "run_to").put("session",session).put("stop",java.math.BigInteger(stop)).put("thread",thread).put("request_id",request).put("source_hash",hash).put("line",line))
    }
    fun stop() {
        val id = sessionID ?: run { if (status in listOf("starting", "stopping")) { stopPending = true; status = "stopping"; message = "Stopping as soon as the adapter acknowledges launch…" }; return }
        movement=null;retireThreads();status = "stopping"; pausedLine = null; variables = emptyList(); values = emptyMap(); send(JSONObject().put("op", "stop").put("session", id))
    }
    fun evaluate(expression: String, watch: Boolean = false) {
        val id = sessionID ?: return; if (status != "paused" || expression.isBlank()) return
        if(selectingThread!=null){message="Wait for the selected thread's paused stack before evaluating an expression.";return}
        if(stoppedGeneration==null||stoppedThread==null||!framesCurrent||selectedFrame==null){message="A current frame is unavailable. Choose another reported thread or wait for a fresh pause before evaluating.";return}
        if (watch && expression !in watches) watches = watches + expression
        val request=UUID.randomUUID().toString();pendingValues[expression]=request;values=values-expression
        send(JSONObject().put("op", "eval").put("session", id).put("expression", expression).put("request_id",request).put("context", if (watch) "watch" else "repl").apply { selectedFrame?.let { put("frame", it) };stoppedGeneration?.let{put("stop",java.math.BigInteger(it))};stoppedThread?.let{put("thread",it)} })
    }
    fun removeWatch(expression: String) { variableTree.remove("eval:$expression"); watches = watches - expression; values = values - expression;pendingValues.remove(expression) }
    fun select(frame: JSONObject) {
        if(selectingThread!=null){message="Wait for the selected thread's paused stack before choosing a frame.";return}
        if(status!="paused"||!framesCurrent||frames.none{it===frame}){message="Frame observation changed. Select a frame from the current paused stack.";return}
        adapterSources.retire("Frame selection changed; captured adapter source is historical.")
        selectedFrame = debugInteger(frame.opt("id")) ?: return
        variableTree.clear()
        variables = if (selectedFrame == frames.firstOrNull()?.optLong("id")) topVariables else emptyList(); values = emptyMap();pendingValues.clear()
        watches.forEach { evaluate(it, true) }
        readFrameVariables()
        if((frame.opt("source_reference") as? Number)?.toDouble()?.let{it>0}==true)return
        val path = frame.optional("source")
        if (path != null && !path.startsWith('/') && ':' !in path && !frame.isNull("source_line")) onLocation?.invoke(path, frame.getInt("source_line"))
        else if (!frame.isNull("line")) onLocation?.invoke(owner, frame.getInt("line"))
    }
    fun adapterSourceBinding():NativeAdapterSourceBinding? {
        val current=variableScope() ?: return null
        val frame=frames.filter{debugInteger(it.opt("id"))==current.frame}.singleOrNull() ?: return null
        val reference=debugInteger(frame.opt("source_reference"))?.takeIf{it>0} ?: return null
        return NativeAdapterSourceBinding(current,reference)
    }
    fun readAdapterSource(inspection:String){adapterSources.begin(adapterSourceBinding(),capabilities.opt("source_content"),inspection)?.let{send(it)}}
    private fun variableScope():NativeDebugScope? {val session=sessionID ?: return null;val stop=stoppedGeneration ?: return null;val thread=stoppedThread ?: return null;val frame=selectedFrame ?: return null;return if(status=="paused"&&framesCurrent&&selectingThread==null)NativeDebugScope(session,stop,thread,frame) else null}
    private fun syncVariableTree(){variableTree.replace(variableScope(),"locals",variables)}
    fun expandVariable(node:NativeVariableNode,inspection:String){
        if(variableScope()!=node.scope){message="Variable scope changed. Read the current frame before expanding.";return}
        val request=variableTree.expand(node,inspection) ?: return
        send(node.scope.put(JSONObject().put("op","children").put("reference",node.reference).put("request_id",request.request)))
    }
    fun readFrameVariables(){
        val session=sessionID;val stop=stoppedGeneration;val thread=stoppedThread;val frame=selectedFrame
        if(status!="paused"||!framesCurrent||selectingThread!=null||session==null||stop==null||thread==null||frame==null){frameNotice="Choose a current paused frame before reading its locals.";return}
        if(!capabilities.optBoolean("frame_locals")){frameNotice="Frame locals are unavailable from this engine. Reconnect with the updated engine; the paused stack remains readable.";return}
        val request=UUID.randomUUID().toString();frameRequest=request;frameNotice="Reading this frame's locals…"
        send(JSONObject().put("op","state").put("session",session).put("frame",frame).put("stop",java.math.BigInteger(stop)).put("thread",thread).put("request_id",request))
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
data class NativeLanguageCompletion(val label: String, val detail: String, val edit: NativeSourceEdit, val source: String, val selection: androidx.compose.ui.text.TextRange,val original:JSONObject?=null,val sourceRevision:Long?=null,val group:NativeCompletionGroupCapture?=null)
data class NativeCompletionGroupCapture(val ranges:List<NativeSourceEdit>,val revision:Long,val file:FileNode,val epoch:Int,val snippetIntent:Int)

private fun nativeCompletionPrefix(source:String,at:Int):Int {
    var offset=at
    while(offset>0){
        val codePoint=source.codePointBefore(offset)
        if(!Character.isLetterOrDigit(codePoint)&&codePoint!='_'.code)break
        offset-=Character.charCount(codePoint)
    }
    return offset
}

private fun completionOfferIdentity(item:JSONObject):List<Any?> {
    val details=item.optJSONObject("labelDetails")
    fun field(value:Any?)=value.takeUnless { it==JSONObject.NULL }
    return listOf(item.opt("label"),item.opt("kind"),item.opt("detail"),details?.opt("detail"),details?.opt("description"))
        .map(::field)
}

private data class OrdinaryCompletionProof(val token:String,val revision:String)
private fun ordinaryCompletionProof(item:JSONObject,hash:String,revision:String?=null):OrdinaryCompletionProof {
    check((!item.has("insertTextFormat")||item.opt("insertTextFormat")==1)&&item.opt("kind")!=15&&item.isNull("command")){
        "Completion groups require ordinary provider edits."
    }
    val data=item.optJSONObject("data")
    val proof=data?.optJSONObject("hickory_context")
    val token=data?.opt("hickory_completion") as? String
    val currentRevision=proof?.opt("source_revision") as? String
    check(!token.isNullOrEmpty()&&proof?.opt("source_hash")==hash&&currentRevision!=null&&
        currentRevision.matches(Regex("[a-f0-9]{64}"))&&(revision==null||revision==currentRevision)){
        "Completion proof changed across carets. Request the completions again."
    }
    return OrdinaryCompletionProof(token,currentRevision)
}
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
internal fun validateNativeSourcePreparation(raw:JSONObject,source:String,requireParsed:Boolean=false) {
 val processed=raw.opt("processed") as? Boolean;val current=raw.opt("current") as? Boolean;val parsed=raw.opt("parsed") as? Boolean;val modified=raw.opt("source_files_modified") as? Boolean
 val count=(raw.opt("open_files") as? Number)?.toString();val countValid=count!=null&&count.matches(Regex("0|[1-9][0-9]{0,19}"))&&java.math.BigInteger(count)<=java.math.BigInteger("18446744073709551615")
 check(processed==true&&current==true&&parsed!=null&&modified==false&&(!requireParsed||parsed)&&countValid&&raw.opt("source_hash")=="sha256:"+debugHash(source)&&(raw.opt("source_revision") as? String)?.matches(Regex("[a-f0-9]{64}"))==true){"Source preparation is unsupported, superseded or incomplete. Retry language services and prepare again."}
}
class NativeLanguage(private val engine: Engine, private val buffer: DocumentBuffer, private val scope: CoroutineScope, sharedConnection: NativeLanguageConnection? = null) {
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
    private var locationOpening = 0L
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
        check(languageURI(buffer.file.path)==uri&&buffer.text==source) {"Source changed before analysis. Run again."}
        sync();val current=epoch
        val ack=request("hick/captureDocument",JSONObject().put("textDocument",JSONObject().put("uri",uri).put("languageId","hick").put("version",version).put("text",source))) as? JSONObject ?: error("Source preparation acknowledgment is unavailable.")
        check(epoch==current&&languageURI(buffer.file.path)==uri&&buffer.text==source){"Source or provider changed during preparation. Run again."}
        validateNativeSourcePreparation(ack,source,requireParsed=buffer.file.docID!=null)
        val result=request("textDocument/documentSymbol",JSONObject().put("textDocument",doc()))
        check(epoch==current && languageURI(buffer.file.path)==uri && buffer.text==source) {"Source or provider changed during analysis. Run again."}
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
            locationOpening++
        } else {
            val opening = ++locationOpening
            val query = navigation
            val connection = epoch
            val source = locationSource
            val revision = buffer.sourceRevision
            val selection = locationSelection
            val symbols = locationSymbolQuery
            val file = buffer.file
            buffer.openSemanticLocation?.invoke(location) {
                locationOpening == opening && navigation == query && epoch == connection && buffer.file === file &&
                    buffer.sourceRevision == revision && buffer.text == source && buffer.state.selection == selection &&
                    location in locations && (symbols == null || symbols == symbolQuery)
            }
        }
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
        check(opened && !buffer.readOnly && !buffer.resourceDeleted && !buffer.saving && languageURI(buffer.file.path)==uri) { "Choose writable source with a connected language provider." }
        sync(); val source = buffer.text; val selection = buffer.state.selection; val identity = epoch;val revision=buffer.sourceRevision;val owner=buffer.file
        fun current()=epoch==identity&&buffer.sourceRevision==revision&&buffer.file===owner&&languageURI(buffer.file.path)==uri&&buffer.text==source&&buffer.state.selection==selection&&!buffer.readOnly&&!buffer.resourceDeleted&&!buffer.saving&&buffer.state.composition==null
        val params = JSONObject().put("textDocument",doc())
        val method = when(kind) {
            "rename" -> { params.put("position",languagePosition(source,selection.min)).put("newName",name); "hick/rename" }
            "references" -> { params.put("position",languagePosition(source,selection.min)).put("context",JSONObject().put("includeDeclaration",true)); "textDocument/references" }
            "format" -> { params.put("options",JSONObject().put("tabSize",4).put("insertSpaces",source.lines().none { it.startsWith("\t") })); "hick/formatting" }
            "fixAll" -> { params.put("range",JSONObject().put("start",languagePosition(source,0)).put("end",languagePosition(source,source.length))).put("context",JSONObject().put("diagnostics",JSONArray()).put("only",JSONArray().put("source.fixAll"))); "textDocument/codeAction" }
            "sourceActions" -> { params.put("range",JSONObject().put("start",languagePosition(source,0)).put("end",languagePosition(source,source.length))).put("context",JSONObject().put("diagnostics",JSONArray()).put("only",JSONArray().put("source"))); "textDocument/codeAction" }
            else -> { params.put("range",JSONObject().put("start",languagePosition(source,selection.min)).put("end",languagePosition(source,selection.max))).put("context",JSONObject().put("diagnostics",JSONArray())); "textDocument/codeAction" }
        }
        if(kind=="rename"||kind=="format"){
            val ack=request("hick/captureDocument",JSONObject().put("textDocument",JSONObject().put("uri",uri).put("languageId","hick").put("version",version).put("text",source))) as? JSONObject ?: error("Source preparation proof is unavailable. Prepare again.")
            check(current()){"Source, selection or provider changed during preparation. Prepare again."}
            validateNativeSourcePreparation(ack,source,requireParsed=buffer.file.docID!=null)
            params.put("expected",JSONObject().put("source_hash",ack.getString("source_hash")).put("source_revision",ack.getString("source_revision")))
        }
        val result = request(method,params)
        check(current()) { "Source, selection or provider changed. Prepare again." }
        return result
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
    private fun completionRanges(source:String):List<NativeSourceEdit> =
        if(buffer.multiSource==source&&buffer.multiRanges.size>1)buffer.multiRanges else emptyList()

    private suspend fun queryCompletionItems(source:String,at:Int):List<JSONObject>? {
        val result = request("hick/completion",JSONObject().put("textDocument",doc())
            .put("position",languagePosition(source,at))
            .put("transform_program_versions",JSONArray(supportedSnippetTransformPrograms)))
        if(result==JSONObject.NULL)return null
        val items = result as? JSONArray ?: (result as? JSONObject)?.optJSONArray("items") ?: JSONArray()
        return items.objects()
    }

    suspend fun complete() {
        if (!opened || buffer.readOnly || buffer.saving || buffer.state.composition != null || buffer.snippets.inMarkedInput) return
        sync()
        val source=buffer.text;val selection=buffer.state.selection;val identity=epoch;val revision=buffer.sourceRevision
        val id=++completion;val owner=buffer.file
        val ranges=completionRanges(source)
        val group=ranges.takeIf { it.isNotEmpty() }?.let { NativeCompletionGroupCapture(it,buffer.multiSelectionRevision,owner,identity,buffer.snippets.intent) }
        fun current() = epoch==identity&&completion==id&&buffer.sourceRevision==revision&&buffer.file===owner&&
            !buffer.readOnly&&!buffer.saving&&!buffer.resourceDeleted&&languageURI(buffer.file.path)==uri&&
            buffer.text==source&&buffer.state.selection==selection&&completionRanges(source)==ranges&&
            (group==null||buffer.multiSelectionRevision==group.revision&&buffer.snippets.intent==group.snippetIntent)&&buffer.state.composition==null&&!buffer.snippets.inMarkedInput
        try {
            kotlinx.coroutines.delay(350)
            if(!current())return
            if(group!=null&&(ranges.size !in 2..16||buffer.snippets.isActive)){
                suggestions=emptyList()
                completionNotice="Choose 2 to 16 ordinary carets outside template editing for a completion group."
                return
            }
            val at=group?.ranges?.first()?.to ?: selection.min
            val prefix=nativeCompletionPrefix(source,selection.min)
            val typed=if(group!=null&&!selection.collapsed)"" else source.substring(prefix,selection.min).lowercase()
            val items=queryCompletionItems(source,at) ?: return
            if(!current())return
            val duplicates=if(group!=null)items.groupingBy(::completionOfferIdentity).eachCount() else emptyMap()
            suggestions=items.mapNotNull { item ->
                val label=item.opt("label") as? String ?: return@mapNotNull null
                if(!(item.opt("filterText") as? String ?: label).lowercase().startsWith(typed))return@mapNotNull null
                if(group!=null&&duplicates[completionOfferIdentity(item)]!=1)return@mapNotNull null
                NativeLanguageCompletion(label,item.optString("detail"),NativeSourceEdit(prefix,selection.max,item.optString("insertText",label)),source,selection,JSONObject(item.toString()),revision,group)
            }.take(100)
        } catch (cancelled: kotlinx.coroutines.CancellationException) { throw cancelled }
        catch (error: Exception) { if(current())status="Completion unavailable: ${error.message}. Retry language services." }
    }
    var completionNotice by mutableStateOf("")
    private var acceptingCompletion=0
    private var completionPending=false
    fun cancelCompletion(){acceptingCompletion++;if(completionPending)completionNotice="Completion cancelled.";completionPending=false}
    suspend fun resolveCompletion(item:JSONObject):JSONObject {
        if(item.isNull("data"))return item
        check(opened&&!buffer.readOnly&&!buffer.saving&&!buffer.resourceDeleted){"Choose writable source with a connected language provider."}
        sync();val source=buffer.text;val selection=buffer.state.selection;val identity=epoch;val revision=buffer.sourceRevision;val acceptance=acceptingCompletion;val owner=buffer.file
        fun current()=opened&&epoch==identity&&acceptingCompletion==acceptance&&buffer.file===owner&&buffer.sourceRevision==revision&&!buffer.readOnly&&!buffer.saving&&!buffer.resourceDeleted&&languageURI(buffer.file.path)==uri&&buffer.text==source&&buffer.state.selection==selection
        return resolveSnippetCompletion(item,source,selection.min,selection.max,::current,request={item->val result=request("completionItem/resolve",JSONObject(item.toString()).put("textDocument",doc()));check(result is JSONObject&&result.opt("label") is String){"Provider did not resolve this completion. Request again."};result},clipboard=engine.snippetClipboardReader)
    }
    internal fun canAcceptCompletion(item:NativeLanguageCompletion):Boolean {
        val group=item.group
        if(buffer.language!==this||buffer.readOnly||buffer.saving||buffer.resourceDeleted||buffer.state.composition!=null||buffer.snippets.inMarkedInput||
            item.sourceRevision?.let{it!=buffer.sourceRevision}==true||buffer.text!=item.source||buffer.state.selection!=item.selection)return false
        val ranges=completionRanges(item.source)
        if(group==null)return ranges.isEmpty()
        val primary=ranges.firstOrNull() ?: return false
        return ranges.size in 2..16&&group.ranges==ranges&&buffer.multiSelectionRevision==group.revision&&
            buffer.file===group.file&&epoch==group.epoch&&buffer.snippets.intent==group.snippetIntent&&!buffer.snippets.isActive&&
            item.selection.min==primary.from&&item.selection.max==primary.to
    }

    private suspend fun resolveCompletionGroup(
        item:NativeLanguageCompletion,original:JSONObject,valid:()->Boolean,
        query:suspend (Int)->List<JSONObject>,resolve:suspend (JSONObject)->JSONObject,
    ):AndroidCompletionGroup {
        val ranges=checkNotNull(item.group).ranges
        val hash="sha256:"+debugHash(item.source)
        val identity=completionOfferIdentity(original)
        val revision=ordinaryCompletionProof(original,hash).revision
        val tokens=mutableSetOf<String>()
        fun requireCurrent(){check(valid()){"Source, carets or provider changed. Request completions again."}}
        val resolved=ranges.mapIndexed { index,range ->
            requireCurrent()
            sourceChangeOffset(item.source,languagePosition(item.source,range.from))
            sourceChangeOffset(item.source,languagePosition(item.source,range.to))
            val matches=if(index==0)listOf(original) else query(range.to).filter{completionOfferIdentity(it)==identity}
            requireCurrent()
            check(matches.size==1){"A caret has no unique matching completion. Choose a different completion."}
            val offer=matches.single()
            val proof=ordinaryCompletionProof(offer,hash,revision)
            check(tokens.add(proof.token)){"Each caret requires its own provider completion offer."}
            val result=resolve(JSONObject(offer.toString()))
            requireCurrent()
            check(ordinaryCompletionProof(result,hash,revision).token==proof.token&&result.opt("label")==item.label){
                "The provider returned a different completion. Request the completions again."
            }
            CapturedAndroidCompletion(result,range.from,range.to,
                nativeCompletionPrefix(item.source,range.from).takeIf{range.from==range.to})
        }
        return planCompletionGroup(uri,SourceSnapshot(item.source,true),resolved)
    }

    suspend fun accept(item:NativeLanguageCompletion,resolution:(suspend (JSONObject)->JSONObject)?=null):Boolean =
        accept(item,resolution,null)

    internal suspend fun accept(item:NativeLanguageCompletion,resolution:(suspend (JSONObject)->JSONObject)?,
                                query:(suspend (Int)->List<JSONObject>)?):Boolean {
        if(!canAcceptCompletion(item)){completionNotice="Suggestion is out of date. Request completions again.";return false}
        val original=item.original
        if(original==null){
            if(item.group!=null){completionNotice="Each caret requires a provider completion offer.";return false}
            cancelCompletion()
            val applied=if(buffer.snippets.active!=null)buffer.snippets.applyInput(item.source,item.edit.from,item.edit.to,item.edit.text)
                else buffer.applySourceEdits(listOf(item.edit),item.source)
            completionNotice=if(applied)"Completion selected" else "Editor refused the completion. Finish composition or request again."
            return applied
        }
        val requestID=++acceptingCompletion;val identity=epoch;val revision=buffer.sourceRevision
        val snippetIntent=buffer.snippets.intent;val owner=buffer.file
        fun current()=acceptingCompletion==requestID&&epoch==identity&&buffer.sourceRevision==revision&&buffer.file===owner&&
            languageURI(buffer.file.path)==uri&&buffer.snippets.intent==snippetIntent&&canAcceptCompletion(item)
        completionPending=true;completionNotice="Resolving completion…"
        try {
            if(item.group!=null){
                val group=resolveCompletionGroup(item,original,::current,query ?: { queryCompletionItems(item.source,it).orEmpty() },
                    resolution ?: { offer ->
                        val result=request("completionItem/resolve",offer.put("textDocument",doc()))
                        result as? JSONObject ?: error("Provider did not resolve this completion. Request again.")
                    })
                check(current()){"Source, carets or provider changed. Request completions again."}
                check(buffer.snippets.applyOrdinaryCompletionGroup(group,item.source)){
                    "Editor refused the completion group. Finish composition or request again."
                }
                completionNotice=if(group.plan.patches.isEmpty())"Completion selected" else "Completion applied · draft"
                return true
            }
            val resolved=(resolution ?: {resolveCompletion(it)})(original)
            if(acceptingCompletion!=requestID)return false
            check(current()){"Source, cursor or provider changed. Request completions again."}
            val contextNotice=snippetContextNotice(resolved)
            val initialDraft=planCompletion(uri,resolved,SourceSnapshot(item.source,!buffer.readOnly&&!buffer.saving),item.edit.from,item.edit.to,completionExpansion(resolved))
            val draft=buffer.snippets.prepareCompletion(initialDraft)
            val edits=draft.plan.patches.firstOrNull()?.edits.orEmpty().map{NativeSourceEdit(it.from,it.to,it.text)}
            val snippetBefore=buffer.snippets.captureBefore(draft.before,buffer.state.selection,true)!!;val snippetRecord=buffer.snippets.recordSerial
            check(buffer.applySourceEdits(edits,draft.before,androidx.compose.ui.text.TextRange(draft.selection.from,draft.selection.to))){"Editor refused the complete completion. Finish composition or request again."}
            buffer.snippets.attachCompletionContext(snippetBefore,draft.continuing ?: beginSnippet(buffer.text,draft.fields),snippetRecord)
            completionNotice=(if(draft.plan.patches.isEmpty())"Completion selected" else "Completion applied · draft")+(if(contextNotice.isEmpty())"" else " · "+contextNotice)
            return true
        }catch(cancelled:kotlinx.coroutines.CancellationException){if(acceptingCompletion==requestID)completionNotice="Completion cancelled.";throw cancelled}
        catch(error:Exception){if(acceptingCompletion==requestID)completionNotice=error.message.orEmpty();return false}
        finally{if(acceptingCompletion==requestID)completionPending=false}
    }

}
