package com.hickorydocs.simple

import androidx.compose.foundation.text.input.TextFieldState
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
    val state = TextFieldState(loaded.content)
    var foldSource by mutableStateOf<String?>(null)
    var foldedLines by mutableStateOf<Set<Int>>(emptySet())
    fun activeFolds(): List<SimpleFold> = if (foldSource == text) simpleFoldRanges(text).filter { it.line in foldedLines } else emptyList()
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
    val hasUnsavedChanges get() = agent.prompt.isNotEmpty() || terminal.input.isNotEmpty() || agent.running || terminal.sending || buffers.values.any { it.dirty || it.saving } || debugger?.active == true
    suspend fun connect(): Boolean {
        if (connecting) return false
        if (hasUnsavedChanges) { status = "Save documents and send or clear draft messages before changing workspace servers."; return false }
        connecting = true
        try {
            val next = Engine(address); val (root, tree) = next.files()
            agent.turns = emptyList(); agent.error = ""; terminal.id = null; terminal.screen = ""; terminal.error = ""
            debugger?.close(); debugger = NativeDebugger(next, viewModelScope)
            debugger?.onLocation = { path, line -> viewModelScope.launch { revealDebug(path, line) } }
            generation = UUID.randomUUID(); engine = next; files = tree; buffers.clear(); opening.clear()
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
            buffers[file.path] = DocumentBuffer(file, text)
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
    override fun onCleared() { debugger?.close(); super.onCleared() }
    suspend fun openTerminal() {
        val server = engine ?: run { status = "Connect to a workspace server before opening Terminal."; return }
        show("terminal"); terminal.open(server)
    }
    fun show(tab: String) {
        val target = layout.groups().firstOrNull { tab in it.tabs }?.id ?: focused
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
        else -> tab.substringAfterLast('/') + if (buffers[tab]?.dirty == true) " •" else ""
    }
    suspend fun save() { engine?.let { currentBuffer?.save(it) } }
}
