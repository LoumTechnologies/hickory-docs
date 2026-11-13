package com.hickorydocs.simple

import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.ui.text.TextRange
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import java.util.UUID

class DocumentBuffer(val file: FileNode, loaded: FileText) {
    var receiptEngine: Engine? = null
    var receiptSource by mutableStateOf("")
    var receipts by mutableStateOf<List<org.json.JSONObject>>(emptyList())
    var receiptNotice by mutableStateOf("")
    suspend fun refreshReceipts() {
        val engine = receiptEngine ?: return; val id = file.docID ?: return
        if (dirty) { receiptNotice = "Save current source to view matching recorded output."; return }
        val source = text; receiptNotice = "Loading recorded output…"
        try {
            val page = engine.request("GET", "/api/docs/$id/render")
            val expected = java.security.MessageDigest.getInstance("SHA-256").digest(source.toByteArray(Charsets.UTF_8)).joinToString("") { "%02x".format(it) }
            if (text != source || dirty || page.optString("document") != id || page.optString("source_hash").removePrefix("sha256:") != expected) {
                receiptNotice = "Recorded output belongs to another source revision. Save or refresh before retrying."; return
            }
            val blocks = page.optJSONArray("blocks") ?: org.json.JSONArray()
            receipts = (0 until blocks.length()).map { blocks.getJSONObject(it) }.filter { it.optString("kind") == "exec" }; receiptSource = source; receiptNotice = ""
        } catch (cancelled: kotlinx.coroutines.CancellationException) { throw cancelled }
        catch (_: Exception) { if (text == source) receiptNotice = "Recorded output unavailable. Retry after checking the server." }
    }
    fun recordedOutputPlacements(): Map<Int, String> {
        if (receiptSource != text || dirty) return emptyMap()
        val bytes = text.toByteArray(Charsets.UTF_8)
        val result = mutableMapOf<Int, String>()
        for (receipt in receipts) {
            val span = receipt.optJSONArray("span") ?: continue
            if (span.length() != 2 || span.optInt(0, -1) < 0 || span.optInt(1, -1) !in (span.optInt(0) + 1)..bytes.size) continue
            val events = receipt.optJSONArray("transcript")
            val chunks = (0 until (events?.length() ?: 0)).mapNotNull { events?.optJSONObject(it) }.filter { it.optString("kind") in listOf("out", "err", "stdout", "stderr") }.joinToString("") { it.optString("data") }
            val output = chunks.ifEmpty { if (receipt.isNull("recorded_output")) "" else receipt.optString("recorded_output") }
            if (output.isNotEmpty()) {
                val line = bytes.take(span.getInt(1) - 1).count { it == 10.toByte() }
                result[line] = result[line].orEmpty() + output
            }
        }
        return result
    }
    fun receiptsOn(line: Int): List<org.json.JSONObject> {
        if (receiptSource != text || dirty) return emptyList()
        val bytes = text.toByteArray(Charsets.UTF_8)
        return receipts.filter { block -> val span = block.optJSONArray("span"); span != null && span.length() == 2 && span.optInt(0, -1) in 0..bytes.size && span.optInt(1, -1) in span.getInt(0)..bytes.size && bytes.take(span.getInt(0)).count { it == 10.toByte() } == line }
    }
    var sourceTable by mutableStateOf<Int?>(null)
    val tableHeights = mutableStateMapOf<Int, Int>()
    var tableSourceVisible by mutableStateOf(false)
    var inlineMarkup by mutableStateOf(InlineSnapshot(""))
    var language: NativeLanguage? = null
    val state = TextFieldState(loaded.content)
    var focusSource: (() -> Unit)? = null
    fun closeFind() { showFind = false; focusSource?.invoke() }
    var multiSource by mutableStateOf("")
    var multiRanges by mutableStateOf<List<NativeSourceEdit>>(emptyList())
    var multiText by mutableStateOf("")
    var showMulti by mutableStateOf(false)
    var applyingMulti = false
    fun selectRectangle() {
        val start = languagePosition(text, state.selection.min); val end = languagePosition(text, state.selection.max)
        if (end.getInt("line") - start.getInt("line") >= 256) return
        val rows = text.split('\n'); val left = minOf(start.getInt("character"), end.getInt("character")); val right = maxOf(start.getInt("character"), end.getInt("character"))
        val ranges = (start.getInt("line")..end.getInt("line")).map { line ->
            NativeSourceEdit(languageOffset(text, org.json.JSONObject().put("line", line).put("character", minOf(left, rows[line].length)))!!, languageOffset(text, org.json.JSONObject().put("line", line).put("character", minOf(right, rows[line].length)))!!, "")
        }
        selectMultiple(ranges, text)
    }
    fun addCursor(direction: Int) {
        val active = if (multiSource == text) multiRanges else emptyList()
        val anchor = (if (direction > 0) active.lastOrNull() else active.firstOrNull())?.from ?: state.selection.min
        val pos = languagePosition(text, anchor); val rows = text.split('\n'); val line = pos.getInt("line") + direction
        if (line !in rows.indices) return
        val at = languageOffset(text, org.json.JSONObject().put("line", line).put("character", minOf(pos.getInt("character"), rows[line].length)))!!
        selectMultiple((if (multiSource == text && multiRanges.isNotEmpty()) multiRanges else listOf(NativeSourceEdit(state.selection.min, state.selection.max, ""))) + NativeSourceEdit(at, at, ""), text)
    }
    fun clearMulti() { multiSource = ""; multiRanges = emptyList() }
    fun selectMultiple(ranges: List<NativeSourceEdit>, source: String): Boolean {
        val sorted = ranges.map { it.copy(text = "") }.distinct().sortedBy { it.from }
        if (source != text || sorted.size !in 2..256 || !validSourceEdits(sorted, source.length)) return false
        applyingMulti = true
        try { if (!selectSourceRange(sorted.first().from, sorted.first().to, source)) return false; multiSource = source; multiRanges = sorted; return true }
        finally { applyingMulti = false }
    }
    fun replaceMultiple(replacement: String): Boolean {
        if (readOnly || multiSource != text || multiRanges.size < 2) return false
        val ranges = multiRanges; val source = text
        applyingMulti = true
        try {
            if (!applySourceEdits(ranges.map { it.copy(text = replacement) }, source)) return false
            var delta = 0
            val carets = ranges.map { range -> val at = range.from + delta + replacement.length; delta += replacement.length - (range.to - range.from); NativeSourceEdit(at, at, "") }
            multiSource = text; multiRanges = carets
            selectSourceRange(carets.first().from, carets.first().to, text); return true
        } finally { applyingMulti = false }
    }
    var showFind by mutableStateOf(false)
    var findQuery by mutableStateOf("")
    var findReplacement by mutableStateOf("")
    var findCase by mutableStateOf(false)
    var findRegex by mutableStateOf(false)
    fun selectSourceRange(from: Int, to: Int, source: String): Boolean {
        if (text != source || state.composition != null || !validSourceEdits(listOf(NativeSourceEdit(from, to, "")), source.length)) return false
        if (!applyingMulti) clearMulti()
        clearFolds(); state.edit { selection = TextRange(from, to) }
        debugReveal = source.take(from).count { it == '\n' } to ((debugReveal?.second ?: 0) + 1)
        return true
    }
    fun applySourceEdits(edits: List<NativeSourceEdit>, source: String): Boolean {
        if (readOnly || text != source || state.composition != null || !validSourceEdits(edits, source.length)) return false
        if (!applyingMulti) clearMulti()
        clearFolds()
        state.edit { edits.sortedByDescending { it.from }.forEach { replace(it.from, it.to, it.text) } }
        return true
    }
    var foldSource by mutableStateOf<String?>(null)
    var foldedLines by mutableStateOf<Set<Int>>(emptySet())
    fun activeFolds(): List<SimpleFold> = if (foldSource == text) simpleFoldRanges(text, file.path.substringAfterLast('.').lowercase() in listOf("md", "markdown")).filter { it.line in foldedLines } else emptyList()
    fun clearFolds() { foldSource = null; foldedLines = emptySet() }
    fun toggleFold(line: Int) {
        if (foldSource != text) clearFolds()
        foldSource = text
        foldedLines = if (line in foldedLines) foldedLines - line else foldedLines + line
    }
    var base by mutableStateOf(loaded.content)
    var hash = loaded.hash
    var debugReveal by mutableStateOf<Pair<Int, Int>?>(null)
    var saving by mutableStateOf(false)
    var error by mutableStateOf("")
    val text get() = state.text.toString()
    val dirty by derivedStateOf { text != base }
    val readOnly get() = file.generatedBy != null
    suspend fun save(engine: Engine) {
        if (!dirty || saving || readOnly) return
        val submitted = text; val previous = base; val previousHash = hash
        saving = true; error = ""
        try { hash = engine.save(file, submitted, previous, previousHash); base = submitted }
        catch (cancelled: CancellationException) { throw cancelled }
        catch (failure: Exception) { error = "${failure.message} Your edits are retained. Copy them before closing; compare with disk before retrying." }
        finally { saving = false }
    }
}
class Workspace : ViewModel() {
    var layout by mutableStateOf(PaneLayout())
    var focused by mutableStateOf("document-group")
    var files by mutableStateOf<List<FileNode>>(emptyList())
    val buffers = mutableStateMapOf<String, DocumentBuffer>()
    var status by mutableStateOf("Connect to a Hickory workspace server")
    var debugger by mutableStateOf<NativeDebugger?>(null)
    var engine by mutableStateOf<Engine?>(null)
    var address by mutableStateOf("")
    var connecting by mutableStateOf(false)
    private var generation = UUID.randomUUID()
    private val opening = mutableSetOf<String>()
    private val newestOpen = mutableMapOf<String, String>()
    val agent = AgentSession()
    val terminal = TerminalSession()
    val active get() = layout.nodes[focused]?.active
    val currentBuffer get() = buffers[active]
    val hasUnsavedChanges get() = agent.prompt.isNotEmpty() || agent.running || buffers.values.any { it.dirty || it.saving } || debugger?.active == true
    suspend fun connect(): Boolean {
        if (connecting) return false
        if (hasUnsavedChanges) { status = "Save documents and send or clear draft messages before changing workspace servers."; return false }
        connecting = true
        try {
            val next = Engine(address); val (root, tree) = next.files()
            agent.turns = emptyList(); agent.error = ""; terminal.id = null; terminal.screen = ""; terminal.error = ""
            debugger?.close(); debugger = NativeDebugger(next, viewModelScope)
            debugger?.onLocation = { path, line -> viewModelScope.launch { revealDebug(path, line) } }
            buffers.values.forEach { it.language?.close() }; generation = UUID.randomUUID(); engine = next; files = tree; buffers.clear(); opening.clear()
            layout = PaneLayout(); focused = "document-group"; status = root
            return true
        } catch (cancelled: CancellationException) { throw cancelled }
        catch (failure: Exception) { status = "${failure.message} Check the workspace address and reconnect."; return false }
        finally { connecting = false }
    }
    suspend fun refresh() {
        val current = generation; val server = engine ?: return
        try { val (root, tree) = server.files(); if (generation == current) { files = tree; status = root } }
        catch (cancelled: CancellationException) { throw cancelled }
        catch (failure: Exception) { if (generation == current) status = "${failure.message} Check the server and refresh again." }
    }
    suspend fun open(file: FileNode) {
        if (buffers.containsKey(file.path)) { show(file.path); return }
        val server = engine ?: return
        if (!opening.add(file.path)) return
        val current = generation
        val target = layout.groups().firstOrNull { it.id == focused && "files" !in it.tabs }?.id
            ?: layout.groups().firstOrNull { "files" !in it.tabs }?.id ?: focused
        newestOpen[target] = file.path; focused = target
        try {
            val text = server.file(file)
            if (generation != current) return
            val buffer = DocumentBuffer(file, text)
            buffers[file.path] = buffer
            buffer.receiptEngine = server
            buffer.language = NativeLanguage(server, buffer, viewModelScope); buffer.language?.start()
            if (newestOpen[target] == file.path) {
                // The original pane may have closed while the request was pending.
                val live = layout.groups().firstOrNull { it.id == target }?.id ?: layout.groups().first().id
                layout = layout.activate(file.path, live); focused = live; status = file.path
            }
        } catch (cancelled: CancellationException) { throw cancelled }
        catch (failure: Exception) { if (generation == current) status = "${failure.message} Refresh files and try opening ${file.path} again." }
        finally { if (generation == current) opening.remove(file.path) }
    }
    suspend fun revealDebug(path: String, line: Int) {
        fun find(nodes: List<FileNode>): FileNode? { nodes.forEach { if (it.path == path) return it; find(it.children)?.let { child -> return child } }; return null }
        val file = find(files) ?: return
        open(file); buffers[path]?.let { buffer -> debugger?.rememberSource(buffer); buffer.debugReveal = line to ((buffer.debugReveal?.second ?: 0) + 1) }
    }
    override fun onCleared() { buffers.values.forEach { it.language?.close() }; debugger?.close(); super.onCleared() }
    suspend fun openTerminal() {
        val server = engine ?: run { status = "Connect to a workspace server before opening Terminal."; return }
        show("terminal"); terminal.open(server)
    }
    fun show(tab: String) {
        val target = layout.groups().firstOrNull { tab in it.tabs }?.id ?: (if (tab != "files") layout.groups().firstOrNull { it.id == focused && "files" !in it.tabs }?.id ?: layout.groups().firstOrNull { "files" !in it.tabs }?.id else null) ?: focused
        layout = layout.activate(tab, target); focused = target
    }
    fun close(tab: String) {
        val buffer = buffers[tab]
        if (buffer != null && (buffer.dirty || buffer.saving)) { status = "Unsaved edits retained. Save this document before closing its tab."; return }
        layout = layout.remove(tab)
        if (layout.groups().none { it.id == focused }) focused = layout.groups().first().id
    }
    fun split(axis: String) {
        val id = UUID.randomUUID().toString(); layout = layout.split(focused, axis, id); focused = "retained-$id"
    }
    fun moveNext() {
        val groups = layout.groups(); val tab = active ?: return
        val index = groups.indexOfFirst { it.id == focused }; if (index < 0) return
        val target = groups[(index + 1) % groups.size].id
        layout = layout.move(tab, target); focused = target
    }
    fun label(tab: String) = when (tab) {
        "files" -> "Files"; "agent" -> "Agent"; "terminal" -> "Terminal"
        else -> (buffers[tab]?.file?.name ?: tab.substringAfterLast('/')) + if (buffers[tab]?.dirty == true) " •" else ""
    }
    var savingAll = false
    var fileCommand by mutableStateOf<String?>(null)
    var filePath by mutableStateOf("")
    fun newTab() {
        val path = "Untitled-" + UUID.randomUUID()
        buffers[path] = DocumentBuffer(FileNode("Untitled ${buffers.keys.count { it.startsWith("Untitled-") } + 1}", path, false, emptyList(), null, null), FileText("", "")); show(path)
    }
    fun chooseFileCommand(command: String) { filePath = ""; fileCommand = command }
    suspend fun performFileCommand() {
        val command = fileCommand ?: return
        val server = engine ?: run { status = "Connect to a workspace server first."; return }
        val path = filePath; val source = currentBuffer; fileCommand = null
        fun find(nodes: List<FileNode>): FileNode? {
            for (node in nodes) { if (node.path == path) return node; find(node.children)?.let { return it } }; return null
        }
        if (command == "Save As") {
            if (source == null || source.saving || path.isEmpty()) return
            if (path == source.file.path) { source.save(server); return }
            val submitted = source.text
            source.saving = true
            fun finish() {
                if (source.text == submitted) { layout = layout.remove(source.file.path); buffers.remove(source.file.path) }
            }
            try {
                if (path.endsWith(".md", ignoreCase = true)) {
                    server.request("POST", "/api/projects/local/docs", org.json.JSONObject().put("path", path).put("source", submitted))
                    refresh(); find(files)?.let { open(it); finish() }
                    source.saving = false
                    if (savingAll) saveAll(); return
                }
                server.request("POST", "/api/files/op", org.json.JSONObject().put("op", "create").put("path", path))
                val file = FileNode(path.substringAfterLast('/'), path, false, emptyList(), null, null)
                val empty = server.file(file)
                if (empty.content.isNotEmpty()) { status = "Destination changed after creation. Original edits retained."; return }
                val copy = DocumentBuffer(file, empty); copy.state.edit { replace(0, length, submitted) }
                copy.save(server)
                if (copy.error.isNotEmpty()) { status = copy.error; return }
                open(file); finish(); refresh()
                source.saving = false
                if (savingAll) saveAll()
            } catch (cancelled: CancellationException) { throw cancelled }
            catch (failure: Exception) { status = "${failure.message} Original edits are retained. Choose an unused path and retry Save As." }
            finally { source.saving = false }
        } else if (command == "Open Directory" && path.isEmpty()) { refresh(); show("files") }
        else {
            val file = find(files)
            if (file == null || file.dir != (command == "Open Directory")) { status = "Path not found. Refresh and choose a path in this workspace."; return }
            if (file.dir) { files = file.children; show("files") } else open(file)
        }
    }
    suspend fun saveAll() {
        val server = engine ?: return
        savingAll = true
        for (buffer in buffers.values.toList().filter { !it.file.path.startsWith("Untitled-") }) buffer.save(server)
        val draft = buffers.values.firstOrNull { it.file.path.startsWith("Untitled-") && it.dirty }
        if (draft != null) { show(draft.file.path); chooseFileCommand("Save As") } else savingAll = false
    }
    suspend fun save() {
        if (currentBuffer?.file?.path?.startsWith("Untitled-") == true) { chooseFileCommand("Save As"); return }
        engine?.let { currentBuffer?.save(it) }
    }
}
