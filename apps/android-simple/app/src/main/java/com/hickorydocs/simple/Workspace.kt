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
import kotlinx.coroutines.async
import java.util.UUID

class DocumentBuffer(file: FileNode, loaded: FileText, untitledDraft:Boolean=false,initialScroll:Int=0, startupObservation: Boolean = false) {
    // A literal filename never determines whether a source has an address.
    var isUntitledDraft by mutableStateOf(untitledDraft)
        internal set
    var startupObservation by mutableStateOf(startupObservation)
        internal set
    val debugIdentity=UUID.randomUUID().toString()
    // This intent token advances synchronously in owned edit/selection/undo
    // paths; source-equal undo cannot revive a reviewed template request.
    private var sourceRevisionValue=0L
    private var sourceRevisionPublished by mutableStateOf(0L)
    val sourceRevision:Long get(){sourceRevisionPublished;return sourceRevisionValue}
    fun advanceSourceRevision(publish:Boolean=true){sourceRevisionValue++;if(publish)publishSourceRevision()}
    fun publishSourceRevision(){sourceRevisionPublished=sourceRevisionValue}
    var file by mutableStateOf(file)
    var resourceDeleted by mutableStateOf(false)
    var unmaterializedHistoryDraft by mutableStateOf(false)
    val sourceScroll = androidx.compose.foundation.ScrollState(initialScroll)
    var recoveryBaseline:EditorRecoveryBaseline?=null
    var terminalOutputRevision by mutableStateOf<Long?>(null)
    var terminalOutput by mutableStateOf<Map<Int, String>>(emptyMap())
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
    var exposedEquation by mutableStateOf<Int?>(null)
    var gitComparison by mutableStateOf<EditorGitComparison?>(null)
    val tableHeights = mutableStateMapOf<Int, Int>()
    var tableSourceVisible by mutableStateOf(false)
    var inlineMarkup by mutableStateOf(InlineSnapshot(""))
    var rememberNavigation: ((SemanticLocation, SemanticLocation) -> Unit)? = null
    var openSemanticLocation: ((SemanticLocation, () -> Boolean) -> Unit)? = null
    var followWorkspaceLink: ((String) -> Unit)? = null
    var coverageSource by mutableStateOf<SimpleCoverageState?>(null)
    var inspectCoverage:((Int)->Unit)?=null
    var testIndex by mutableStateOf<SourceTestIndex?>(null)
    var inspectTests:((Int)->Unit)?=null
    var language: NativeLanguage? = null
    val state = TextFieldState(loaded.content)
    val snippets=SourceSnippetController(this)
    var showSnippetTools by mutableStateOf(false)
    var focusSource: (() -> Unit)? = null
    fun closeFind() { showFind = false; focusSource?.invoke() }
    var multiSource by mutableStateOf("")
    private var logicalSelections by mutableStateOf<List<NativeSourceEdit>>(emptyList())
    var multiSelectionRevision = 0L
        private set
    var multiRanges: List<NativeSourceEdit>
        get() = logicalSelections
        set(ranges) {
            if (logicalSelections == ranges) return
            logicalSelections = ranges.toList()
            multiSelectionRevision++
        }
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
        val previousSelection=state.selection
        clearFolds(); state.edit { selection = TextRange(from, to) }
        if(state.selection!=previousSelection)advanceSourceRevision()
        val selected=state.selection;val scroll=sourceScroll.value
        debugRevealSource=source;debugRevealCurrent={text==source&&state.selection==selected&&sourceScroll.value==scroll}
        debugReveal = source.take(from).count { it == '\n' } to ((debugReveal?.second ?: 0) + 1)
        return true
    }
    fun applySourceEdits(edits: List<NativeSourceEdit>, source: String,resultSelection:TextRange?=null): Boolean {
        if (readOnly || saving || text != source || state.composition != null || !(edits.isEmpty()&&resultSelection!=null||validSourceEdits(edits, source.length))) return false
        val previousSelection=state.selection
        val snippetBefore=snippets.captureBefore(source,state.selection)
        val resultLength=source.length+edits.sumOf {it.text.length-(it.to-it.from)}
        if(resultSelection!=null&&(resultSelection.min<0||resultSelection.max>resultLength))return false
        if (!applyingMulti) clearMulti()
        clearFolds()
        state.edit { edits.sortedByDescending { it.from }.forEach { replace(it.from, it.to, it.text) };if(resultSelection!=null)selection=resultSelection }
        if(text!=source||state.selection!=previousSelection)advanceSourceRevision()
        snippets.finishTextChange(snippetBefore,text,state.selection);if(snippetBefore!=null)snippets.publish()
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
    var saveGuard=loaded.saveGuard
    var saveGuardReason=loaded.saveGuardReason
    private var saveHistoryAttempt:SourceSaveAttempt?=null
    internal var creationAttempt:SourceCreationAttempt?=null
    var debugReveal by mutableStateOf<Pair<Int, Int>?>(null)
    var debugRevealSource:String?=null
    var debugRevealCurrent:()->Boolean={true}
    var saving by mutableStateOf(false)
    var error by mutableStateOf("")
    val text get() = state.text.toString()
    val dirty by derivedStateOf { text != base }
    val readOnly get() = startupObservation || (resourceDeleted && !unmaterializedHistoryDraft) || file.generatedBy != null
    suspend fun save(engine: Engine) {
        if (resourceDeleted || !dirty || saving || readOnly) return
        val submitted = text; val previous = base; val previousHash = hash
        val guard=saveGuard ?: run{error=saveGuardReason ?: "This source has no physical save authority. Reopen or compare it before saving; the draft is retained.";return}
        val attempt=saveHistoryAttempt?.takeIf{it.matches(engine,file.path,submitted,previous,previousHash,guard)} ?: SourceSaveAttempt(engine,file.path,submitted,previous,previousHash,guard).also{saveHistoryAttempt=it}
        saving = true; error = ""
        try { val result=engine.saveSourceWithHistory(file,attempt);check(file.path==attempt.path&&saveGuard==guard&&base==previous&&hash==previousHash){"Source address or saved baseline changed during Save. Inspect history before adoption."};hash=result.hash;base=submitted;saveGuard=result.guard;saveGuardReason=null;saveHistoryAttempt=null;error=result.notice }
        catch (cancelled: CancellationException) { throw cancelled }
        catch (failure: Exception) { error = "${failure.message} Your edits are retained. Copy them before closing; compare with disk before retrying." }
        finally { saving = false }
    }
}
class Workspace(private val backgroundScope: kotlinx.coroutines.CoroutineScope? = null) : ViewModel() {
    private val workspaceScope get() = backgroundScope ?: viewModelScope
    private var languageConnection: NativeLanguageConnection? = null
    val changes = WorkspaceChanges()
    var showWorkspaceSearch by mutableStateOf(false)
    val search = WorkspaceSearch()
    suspend fun openSearchHit(hit: WorkspaceSearchHit) {
        val server = engine ?: return; val current = generation; val request = search.revision
        fun find(nodes: List<FileNode>): FileNode? { nodes.forEach { if (it.path == hit.path) return it; find(it.children)?.let { child -> return child } }; return null }
        val file = (buffers[hit.path]?.file ?: find(rootFiles))?.takeUnless { it.dir } ?: run { search.notice = "File no longer listed. Refresh files and search again."; return }
        try {
            val source = buffers[hit.path]?.text ?: server.file(file).content
            if (generation != current || search.revision != request) return
            val hash = kotlinx.coroutines.withContext(kotlinx.coroutines.Dispatchers.Default) { java.security.MessageDigest.getInstance("SHA-256").digest(source.toByteArray(Charsets.UTF_8)).joinToString("") { "%02x".format(it) } }
            if (generation != current || search.revision != request) return
            val character = hit.character
            if (hit.sourceHash == null || hash != hit.sourceHash.removePrefix("sha256:").lowercase() || character == null || hit.line < 1 || source.split("\n").getOrNull(hit.line - 1)?.trimEnd('\r') != hit.text.trimEnd('\r')) {
                search.notice = "Search source changed or its revision is unknown. Save edits and search again."; return
            }
            openSemanticLocation(SemanticLocation(languageURI(hit.path), hit.line - 1, character), allowDirty = true, expectedSource = source)
            if (generation == current && search.revision == request) showWorkspaceSearch = false
        } catch (cancelled: CancellationException) { throw cancelled }
        catch (failure: Exception) { if (generation == current && search.revision == request) search.notice = failure.message.orEmpty() }
    }
    var analysis by mutableStateOf(WorkspaceAnalysis()); private set
    var sourceTestIndex:SourceTestIndex?=null
    var sourceTemplates by mutableStateOf(SourceTemplatesModel())
    private var templatesVisible by mutableStateOf(false)
    var showTemplates:Boolean
        get()=templatesVisible
        set(value){if(!value&&templatesVisible)sourceTemplates.close();templatesVisible=value}
    var testSources by mutableStateOf(TestSourcesModel())
    var coverage by mutableStateOf(SimpleCoverageState())
    private var coverageVisible by mutableStateOf(false)
    var showCoverage: Boolean
        get() = coverageVisible
        set(value) {
            if (!value && coverageVisible) coverage.retireSourceLink()
            coverageVisible = value
        }
    private var testsVisible by mutableStateOf(false)
    // Every dismissal retires admission, including Escape/outside-click paths.
    // Reopening a floating reading cannot resurrect its earlier launch choice.
    var showTests:Boolean
        get()=testsVisible
        set(value){
            if(!value&&testsVisible){testSources.closeDebugPreparation();testSources.retirePointNavigation()}
            testsVisible=value
        }
    var showReview by mutableStateOf(false)
    var gitReview by mutableStateOf(GitReviewModel())
    var showHistory by mutableStateOf(false)
    var showRecovery by mutableStateOf(false)
    var showConstruction by mutableStateOf(false)
    var showAttribution by mutableStateOf(false)
    var gitAttribution by mutableStateOf(GitAttributionModel(""))
    var gitHistory by mutableStateOf(GitHistoryModel())
    var showEverywhere by mutableStateOf(false)
    fun runCommand(action:suspend ()->Unit) {workspaceScope.launch {action()}}
    var showAnalysis by mutableStateOf(false)
    fun startAnalysis() {
        val model=analysis;val current=generation;val files=rootFiles
        workspaceScope.launch {model.run(files,{file->
            check(generation==current) {"Workspace changed. Run analysis again."};loadBuffer(file)
        },{file,source->
            check(generation==current) {"Workspace changed. Run analysis again."}
            val service=buffers[file.path]?.language ?: error("Language service unavailable. Retry analysis.")
            service.readOutline(source)
        })}
    }
    var showProblems by mutableStateOf(false)
    val problems: List<WorkspaceProblem> get() = buffers.values.sortedBy { it.file.path }.flatMap { buffer ->
        val language = buffer.language ?: return@flatMap emptyList()
        val matchesSource = language.diagnosticSource == buffer.text
        language.diagnostics.mapIndexed { index, diagnostic ->
            val source = if (diagnostic.location != null) language.diagnosticSource else null
            val current = source != null && matchesSource
            val location = if (current) diagnostic.location else null
            WorkspaceProblem("${buffer.file.path}:$index", buffer.file.path, diagnostic.message, diagnostic.severity, location, source, current)
        }
    }
    suspend fun openProblem(problem: WorkspaceProblem, precise: Boolean = true) {
        if (!precise) { buffers[problem.path]?.let { open(it.file) }; return }
        val source = problem.source
        if (!problem.current || problem.location == null || source == null || buffers[problem.path]?.text != source) {
            status = "Problem source changed or its location is unversioned. Refresh diagnostics."; showProblems = true; return
        }
        openSemanticLocation(problem.location, allowDirty = true, expectedSource = source); showProblems = false
    }
    suspend fun navigateProblem(direction: Int) {
        val problem = nextWorkspaceProblem(problems, currentSemanticLocation(), direction)
        if (problem == null) { status = "No current source locations. Open Problems to read provider messages."; showProblems = true; return }
        openProblem(problem)
    }
    var navigationTrail by mutableStateOf(NavigationTrail())
    var layout by mutableStateOf(PaneLayout())
    private var hiddenFilesPane by mutableStateOf<String?>(null)
    private val visibleDocumentPane: PaneNode?
        get() = layout.groups().firstOrNull { group -> bufferForTab(group.active) != null }
    val separateFilesPane: String?
        get() = if (visibleDocumentPane == null) null else layout.groups().firstOrNull { it.tabs == listOf("files") }?.id
    val collapsedFilesPane: String?
        get() = hiddenFilesPane?.takeIf { it == separateFilesPane }

    fun toggleFilesPane() {
        val files = separateFilesPane ?: return
        hiddenFilesPane = if (collapsedFilesPane == null) files else null
        if (hiddenFilesPane != null && focused == files) {
            visibleDocumentPane?.let { focused = it.id }
        }
    }
    private var foregroundNavigation = 0L
    private var focusedGroup by mutableStateOf("document-group")
    var focused: String
        get() = focusedGroup
        set(value) {
            if (focusedGroup != value) foregroundNavigation++
            focusedGroup = value
        }
    internal fun captureForegroundNavigation(): () -> Boolean {
        val intent = foregroundNavigation
        return { foregroundNavigation == intent }
    }
    var rootFiles by mutableStateOf<List<FileNode>>(emptyList())
    var files by mutableStateOf<List<FileNode>>(emptyList())
    var directoryScope by mutableStateOf("")
    val buffers = mutableStateMapOf<String, DocumentBuffer>()
    val retainedResourceEditors=mutableMapOf<String,DocumentBuffer>()
    // Deleted tabs use opaque addresses: reusing a filesystem path must not
    // replace the old editor or silently redirect its unsaved text.
    val retainedResourceTabs=mutableStateMapOf<String,DocumentBuffer>()
    val resourceChanges=ResourceChangeReview()
    val gitResolution=GitResolutionModel()
    val sourceHistory=SourceHistoryModel()
    val editorRecovery=EditorRecoveryModel()
    val behaviorReading=BehaviorReadingModel()
    fun reviewResourceHistory(record:String,workspace:String,current:()->Boolean,published:(String,String)->Unit){
        showRecovery=true
        workspaceScope.launch{resourceChanges.openHistory(this@Workspace,record,workspace,current,published)}
    }
    fun bufferForTab(tab:String?):DocumentBuffer? {if(tab==null)return null;val identity=tabIdentity(tab);return if(identity is SourceTabIdentity.Source)buffers[identity.path] ?: retainedResourceTabs[tab] else null}
    private fun findFileNode(path: String, tree: List<FileNode>): FileNode? =
        tree.firstNotNullOfOrNull { if (it.path == path) it else findFileNode(path, it.children) }

    private fun scopedFiles(tree:List<FileNode>):List<FileNode>{
        if(directoryScope.isEmpty())return tree
        return findFileNode(directoryScope, tree)?.takeIf { it.dir }?.children ?: emptyList()
    }
    private fun resourceTree(patches:List<ResourceEditorPatch>,published:Collection<ResourcePublishedFile<FileNode>>):List<FileNode>{
        val paths=patches.map{it.path}.toSet()
        fun prune(nodes:List<FileNode>):List<FileNode> = nodes.mapNotNull{node->if(!node.dir&&node.path in paths)null else node.copy(children=prune(node.children))}
        fun insert(nodes:List<FileNode>,file:FileNode,parts:List<String>,depth:Int):List<FileNode>{
            if(depth==parts.lastIndex){check(nodes.none{it.path==file.path}){"A directory occupies a published source address. Refresh files and inspect recovery."};return nodes+file}
            val path=parts.take(depth+1).joinToString("/");val previous=nodes.find{it.path==path}
            check(previous==null||previous.dir){"A parent changed in the file index. Refresh files and inspect recovery."}
            val parent=previous ?: FileNode(parts[depth],path,true,emptyList(),null,null)
            val next=parent.copy(children=insert(parent.children,file,parts,depth+1))
            return if(previous==null)nodes+next else nodes.map{if(it===previous)next else it}
        }
        var tree=prune(rootFiles)
        for(entry in published)tree=insert(tree,entry.file,entry.file.path.split('/'),0)
        return tree
    }
    internal fun sourceAcquisitionLease(protectSelection:Boolean=false,external:()->Boolean={true}):SourceAcquisitionLease {
        val epoch=generation;val server=engine;val addresses=buffers.toMutableMap();val retained=retainedResourceEditors.toMap();val tabs=retainedResourceTabs.toMap()
        data class Captured(val buffer:DocumentBuffer,val revision:Long,val source:String,val base:String,val hash:String,val guard:String?,val file:FileNode,val deleted:Boolean,val untitled:Boolean,val selection:TextRange,val readOnly:Boolean)
        fun capture(b:DocumentBuffer)=Captured(b,b.sourceRevision,b.text,b.base,b.hash,b.saveGuard,b.file,b.resourceDeleted,b.isUntitledDraft,b.state.selection,b.readOnly)
        val snapshots=(addresses.values+retained.values+tabs.values).distinct().map(::capture).toMutableList()
        fun same(current:Map<String,DocumentBuffer>,before:Map<String,DocumentBuffer>)=current.size==before.size&&before.all{(path,b)->current[path]===b}
        fun current(allowing:String?)=external()&&generation==epoch&&engine===server&&addresses.all{(path,b)->buffers[path]===b}&&buffers.all{(path,b)->addresses[path]===b||path==allowing}&&same(retainedResourceEditors,retained)&&same(retainedResourceTabs,tabs)&&snapshots.all{s->val b=s.buffer;b.sourceRevision==s.revision&&b.text==s.source&&b.base==s.base&&b.hash==s.hash&&b.saveGuard==s.guard&&b.file===s.file&&b.resourceDeleted==s.deleted&&b.isUntitledDraft==s.untitled&&b.readOnly==s.readOnly&&(!protectSelection||b.state.selection==s.selection)&&!b.saving&&b.state.composition==null}
        return SourceAcquisitionLease(::current){path,b->check(current(path)&&buffers[path]===b){"Original source owners changed during acquisition."};if(addresses[path]!==b){addresses[path]=b;snapshots.add(capture(b))}}
    }
    internal fun captureRetainedSourceNamespace():()->Boolean {
        val lease=sourceAcquisitionLease(protectSelection=true);return {lease.current()}
    }
    fun captureResourceEditors(patches:List<ResourceEditorPatch>,restoring:Map<String,DocumentBuffer> = emptyMap()):()->Boolean {
        val paths=patches.associate{it.path to buffers[it.path]}
        data class Capture(val buffer:DocumentBuffer,val text:String,val base:String,val hash:String,val file:FileNode)
        val captures=(paths.values.filterNotNull()+restoring.values).distinct().map{Capture(it,it.text,it.base,it.hash,it.file)}
        val session=generation
        return {generation==session&&paths.all{(path,buffer)->buffers[path]===buffer}&&captures.all{it.buffer.text==it.text&&it.buffer.base==it.base&&it.buffer.hash==it.hash&&it.buffer.file===it.file&&!it.buffer.saving}}
    }
    fun adoptResourceReceipt(patches:List<ResourceEditorPatch>,receipt:ResourcePublication<FileNode>,request:String,workspace:String,context:String,expected:Map<String,ResourceSaved>,current:()->Boolean,restoring:Map<String,DocumentBuffer> = emptyMap(),absentOriginal:String?=null):List<ResourceEditorRoute<DocumentBuffer>> {
        val server=engine ?: error("Reconnect and inspect Source recovery before changing editors.")
        check(current()) {"Editor drafts or baselines changed after review. Preserve them and inspect Source recovery."}
        val files=confirmedResourceFiles(patches,receipt,request,workspace,context,expected){it.path to (!it.dir&&it.generatedBy==null)}
        val nextTree=resourceTree(patches,files.values)
        val live=patches.mapNotNull{patch->patch.before?.let{before->val buffer=buffers[patch.path] ?: error("Source editor unavailable. Inspect recovery.");check(!buffer.saving&&!buffer.readOnly){"Source editor is saving or read-only. Inspect recovery."};before.identity to ResourceEditorCapture(buffer,buffer.text)}}.toMap()
        check(restoring.values.all{buffer->buffer.resourceDeleted&&retainedResourceEditors.values.any{it===buffer}&&buffers.values.none{it===buffer}}){"Restoration must select a retained deleted editor. Inspect Source recovery."}
        val editorPatches=patches.filter{it.path!=absentOriginal||it.before!=null||it.after!=null||buffers.containsKey(it.path)}
        val routes=routeResourceEditors(editorPatches,live,restoring.mapValues{ResourceEditorCapture(it.value,it.value.text)})
        val targets=routes.map{route->val entry=route.to?.let{files.getValue(it)};route to (route.editor ?: DocumentBuffer(entry!!.file,FileText(route.after!!,entry.hash,entry.saveGuard)))}
        for((route,buffer) in targets){
            check(buffer.state.composition==null){"Finish composition before adopting source editors. Inspect recovery."}
            check(route.to==null||patches.any{it.path==route.to&&it.before!=null}||!buffers.containsKey(route.to)){"A destination editor appeared after review. Inspect recovery."}
        }
        val addresses=linkedMapOf<String,String>()
        for((route,buffer) in targets){
            val destination=route.to?.let(::sourceTab) ?: ("deleted-resource:"+UUID.randomUUID())
            route.from?.let{addresses[sourceTab(it)]=destination}
            retainedResourceTabs.filterValues{it===buffer}.keys.forEach{addresses[it]=destination}
        }
        val nextLayout=layout.copy(nodes=layout.nodes.mapValues{(_,node)->node.copy(tabs=node.tabs.map{addresses[it] ?: it},active=node.active?.let{addresses[it] ?: it})})
        debugger?.resourceSourcesChanged(targets.map{(route,buffer)->NativeSourceRoute(buffer.debugIdentity,route.from,route.to,route.after,route.to?.let{path->files.getValue(path).saved.text==route.after}==true)})
        sourceTemplates.close();sourceHistory.close()
        targets.forEach{(_,buffer)->buffer.advanceSourceRevision()}
        generation=UUID.randomUUID();loading.values.forEach{it.cancel()};loading.clear();newestOpen.clear()
        // Retire the whole URI namespace before attaching destinations, including swaps.
        for((_,buffer) in targets){buffer.language?.close();buffer.language=null}
        for((route,_) in targets)route.from?.let{buffers.remove(it)}
        for((route,buffer) in targets){
            val path=route.to
            if(path==null){buffer.resourceDeleted=true;retainedResourceEditors[route.identity]=buffer;route.from?.let{retainedResourceTabs[addresses.getValue(sourceTab(it))]=buffer};continue}
            val entry=files.getValue(path)
            reviewedSourceReplacement(buffer.text, route.after!!)?.let { edit ->
                val selected = edit.reviewedSelection(buffer.state.selection)
                buffer.state.edit {
                    replace(edit.from, edit.to, edit.text)
                    selection = selected
                }
            }
            buffer.file=entry.file;buffer.base=entry.saved.text!!;buffer.hash=entry.hash;buffer.saveGuard=entry.saveGuard;buffer.saveGuardReason=if(entry.saveGuard==null)"Resource receipt lacks physical save authority. Reopen or compare before ordinary Save." else null;buffer.resourceDeleted=false;buffer.unmaterializedHistoryDraft=false;buffer.error=""
            buffers[path]=buffer;retainedResourceEditors.entries.removeAll{it.value===buffer}
            retainedResourceTabs.entries.removeAll{it.value===buffer}
            retainBuffer(entry.file,FileText(route.after!!,entry.hash,entry.saveGuard),server,buffer)
        }
        layout=nextLayout
        rootFiles=nextTree;this.files=scopedFiles(nextTree)
        return routes
    }

    var status by mutableStateOf("")
    var debugger by mutableStateOf<NativeDebugger?>(null)
    var engine by mutableStateOf<Engine?>(null)
    var address by mutableStateOf("")
    var connecting by mutableStateOf(false)
    private var launchFile: Pair<String, String>? = null
    private data class StartupPreview(val engine: Engine, val source: StartupSource, val editor: DocumentBuffer)
    private var startupPreview: StartupPreview? = null
    private var generation = UUID.randomUUID()
    val sourceNavigationEpoch:UUID get()=generation
    private val loading = mutableMapOf<String,kotlinx.coroutines.Deferred<DocumentBuffer>>()
    private val newestOpen = mutableMapOf<String, Any>()
    val agent = AgentSession()
    val agentProposals = AgentProposalModel()
    val terminal = TerminalSession()
    val active get() = layout.nodes[focused]?.active
    val currentBuffer get() = bufferForTab(active)
    val hasUnsavedChanges get() = agent.prompt.isNotEmpty() || agent.running || resourceChanges.busy || resourceChanges.hasPending || gitResolution.hasPending || (buffers.values+retainedResourceEditors.values).any { it.dirty || it.saving } || debugger?.active == true || agent.debugger?.active == true
    fun connectFromLaunch(origin: String?, path: String?) {
        if (engine != null || address.isNotEmpty() || origin.isNullOrEmpty()) return
        address = origin
        launchFile = path?.takeIf { it.isNotEmpty() }?.let { origin to it }
        workspaceScope.launch { connect() }
    }

    private fun retainStartupPreview(file: FileNode, server: Engine, source: StartupSource, text: FileText): DocumentBuffer {
        val editor = DocumentBuffer(file, text, startupObservation = true)
        startupPreview = StartupPreview(server, source, editor)
        buffers[file.path] = editor
        return editor
    }

    private fun startupPermissionNotice(text: FileText) =
        text.saveGuardReason ?: "Source permission is unconfirmed. Refresh Files before editing."

    private suspend fun confirmStartupPreview(file: FileNode, server: Engine, response: FileText? = null) {
        val held = startupPreview?.takeIf { it.engine === server && it.source.path == file.path } ?: return
        val identity = server.sourceWorkspaceID
        val root = server.sourceRootAddress
        val current = generation
        fun stillOwnsPreview() = generation == current && engine === server && startupPreview === held &&
            buffers[file.path] === held.editor && server.sourceWorkspaceID == identity && server.sourceRootAddress == root &&
            findFileNode(file.path, rootFiles) == file
        try {
            check(identity != null && root != null) { "Workspace identity is unconfirmed. Refresh Files and try again." }
            check(root == held.source.rootPath && (held.source.workspaceID == null || held.source.workspaceID == identity)) {
                "The workspace changed while opening ${file.path}. Reopen it from Files to read the current source."
            }
            val loaded = response ?: server.file(file)
            if (!stillOwnsPreview()) return
            val editor = held.editor
            if (editor.dirty) return
            adoptSavedText(editor, loaded.content)
            editor.saveGuardReason = loaded.saveGuardReason
            check(file.generatedBy != null || !loaded.saveGuard.isNullOrEmpty()) {
                startupPermissionNotice(loaded)
            }
            editor.file = file
            editor.hash = loaded.hash
            editor.saveGuard = loaded.saveGuard
            editor.saveGuardReason = loaded.saveGuardReason
            retainBuffer(file, loaded, server, editor)
            editor.startupObservation = false
            startupPreview = null
            if (currentBuffer === editor) status = file.path
        } catch (cancelled: CancellationException) { throw cancelled }
        catch (failure: Exception) {
            if (stillOwnsPreview() && currentBuffer === held.editor) status = failure.message ?: "Refresh Files to check the current source."
        }
    }

    suspend fun connect(): Boolean {
        if (connecting) return false
        if (hasUnsavedChanges) { status = "Save documents and send or clear draft messages before changing workspace servers."; return false }
        connecting = true
        var reading: kotlinx.coroutines.Job? = null
        try {
            val origin = address
            val next = Engine(origin)
            val requested = launchFile?.takeIf { it.first == origin }?.second
            var requestedGeneration = generation
            var requestedNavigation = foregroundNavigation
            val startupIsCurrent = { generation == requestedGeneration && foregroundNavigation == requestedNavigation }
            if (requested != null && engine == null) reading = workspaceScope.launch {
                val observed = try { next.startupSource(requested) }
                catch (cancelled: CancellationException) { throw cancelled }
                catch (_: Exception) { return@launch }
                val sameHost = engine == null || engine === next && next.sourceRootAddress == observed.rootPath &&
                    (observed.workspaceID == null || observed.workspaceID == next.sourceWorkspaceID)
                if (sameHost && startupIsCurrent() && connecting && address == origin && currentBuffer == null && requested !in buffers) {
                    val file = FileNode(requested.substringAfterLast('/'), requested, false, emptyList(), null, null)
                    retainStartupPreview(file, next, observed, FileText(observed.content, ""))
                    show(sourceTab(requested))
                    status = "Reading ${file.name}. Edit access will follow source confirmation."
                }
            }
            var listingUnavailable = false
            val (root, tree) = try { next.files() }
            catch (cancelled: CancellationException) { throw cancelled }
            catch (failure: Exception) {
                reading?.join()
                val observed = startupPreview?.takeIf { it.engine === next }?.source ?: throw failure
                listingUnavailable = true
                observed.rootPath to emptyList()
            }
            val preview = startupPreview?.takeIf { it.engine === next }
            val requestedTabIsCurrent = startupIsCurrent()
            startupPreview = preview
            launchFile = null
            directoryScope=""
            resourceChanges.reset();gitResolution.reset();sourceHistory.close();behaviorReading.close();retainedResourceTabs.clear()
            agent.turns = emptyList(); agent.totals = null; agent.activity.clear(); agent.stopPending = false; agent.error = ""; terminal.id = null; terminal.error = ""
            agent.debugger?.close()
            agent.debugger = NativeDebugger(next, workspaceScope, recordedMessages = true)
            debugger?.close(); debugger = NativeDebugger(next, workspaceScope)
            debugger?.onLocation = { path, line -> workspaceScope.launch { revealDebug(path, line) } }
            buffers.values.forEach { it.language?.close() }; languageConnection?.close();languageConnection=null; analysis.stop();analysis=WorkspaceAnalysis();showAnalysis=false;showEverywhere=false;showTests=false;showTemplates=false;sourceTemplates.close();sourceTemplates=SourceTemplatesModel();showReview=false;showHistory=false;showAttribution=false;showConstruction=false;showRecovery=false;gitAttribution.close();gitAttribution=GitAttributionModel("");gitHistory.close();gitHistory=GitHistoryModel();gitReview.close();gitReview=GitReviewModel();showCoverage=false;coverage.stop();coverage=SimpleCoverageState();testSources=TestSourcesModel();sourceTestIndex?.stop();sourceTestIndex=null;generation = UUID.randomUUID(); changes.reset(); search.invalidate(); showWorkspaceSearch = false; engine = next; files = tree; rootFiles = tree; navigationTrail = NavigationTrail(); retainedResourceEditors.clear(); buffers.clear(); loading.values.forEach {it.cancel()}; loading.clear()
            if (preview == null) { layout = PaneLayout(); focused = "document-group" }
            else buffers[preview.source.path] = preview.editor
            if (requestedTabIsCurrent) { requestedGeneration = generation; requestedNavigation = foregroundNavigation }
            status = root
            if (requested != null) {
                directoryScope = requested.substringBeforeLast('/', "")
                files = scopedFiles(tree)
                val file = findFileNode(requested, tree)?.takeUnless { it.dir }
                if (listingUnavailable) status = "Read-only source preview. Refresh Files to confirm the workspace and permission."
                else if (file == null) {
                    status = "Requested file not found: $requested. Refresh files and open it from the tree."
                } else {
                    if (preview != null) confirmStartupPreview(file, next)
                    else if (startupIsCurrent()) {
                        val opened = activateFile(file, recordNavigation = false, isCurrent = startupIsCurrent, forStartup = true)
                        if (!opened && startupIsCurrent()) reading?.join()
                    }
                }
            }
            return true
        } catch (cancelled: CancellationException) { throw cancelled }
        catch (failure: Exception) { status = "${failure.message} Check the workspace address and reconnect."; return false }
        finally { reading?.cancel(); connecting = false }
    }
    private fun adoptSavedText(buffer: DocumentBuffer, source: String) {
        val edit = reviewedSourceReplacement(buffer.text, source) ?: return
        val selected = edit.reviewedSelection(buffer.state.selection)
        buffer.state.edit {
            replace(edit.from, edit.to, edit.text)
            selection = selected
        }
        buffer.base = source
        buffer.advanceSourceRevision()
    }

    private var savedSourceRead: Any? = null
    suspend fun refreshCurrentSource() {
        val server = engine ?: return
        val root = server.sourceRootAddress ?: return
        val workspace = server.sourceWorkspaceID ?: return
        val buffer = currentBuffer ?: return
        if (buffer.startupObservation || buffer.saving || buffer.isUntitledDraft || buffer.resourceDeleted || buffer.gitComparison != null) return
        val file = findFileNode(buffer.file.path, rootFiles)?.takeUnless { it.dir } ?: return
        if (file.docID != buffer.file.docID || file.generatedBy != buffer.file.generatedBy) return
        val scope = generation
        val request = Any()
        savedSourceRead = request
        val revision = buffer.sourceRevision
        val base = buffer.base
        val savedGuard = buffer.saveGuard
        fun current() = savedSourceRead === request && generation == scope && engine === server &&
            server.sourceRootAddress == root && server.sourceWorkspaceID == workspace && currentBuffer === buffer &&
            buffers[file.path] === buffer && buffer.sourceRevision == revision && buffer.base == base &&
            buffer.saveGuard == savedGuard && !buffer.saving && !buffer.resourceDeleted
        try {
            val saved = server.file(file)
            if (!current()) return
            if (buffer.dirty) {
                if (saved.content != base || saved.saveGuard != savedGuard) {
                    status = "${file.path} changed on disk. Your unsaved draft is still open."
                }
                return
            }
            if (buffer.state.composition != null) return
            if (saved.content != base) {
                adoptSavedText(buffer, saved.content)
                status = "Reloaded ${file.path} from disk."
            }
            buffer.hash = saved.hash
            buffer.saveGuard = saved.saveGuard
            buffer.saveGuardReason = saved.saveGuardReason
        } catch (cancelled: CancellationException) { throw cancelled }
        catch (failure: Exception) { if (current()) status = "Could not check ${file.path} on disk. ${failure.message}" }
    }
    suspend fun refresh() {
        val current = generation; val server = engine ?: return
        try {
            val (root, tree) = server.files()
            if (generation == current) {
                rootFiles = tree; files = scopedFiles(tree); status = root
                val preview = startupPreview?.source?.path?.let { findFileNode(it, tree) }?.takeUnless { it.dir }
                if (preview != null) confirmStartupPreview(preview, server) else refreshCurrentSource()
            }
        }
        catch (cancelled: CancellationException) { throw cancelled }
        catch (failure: Exception) { if (generation == current) status = "${failure.message} Check the server and refresh again." }
    }
    suspend fun loadBuffer(file: FileNode, forStartup: Boolean = false): DocumentBuffer {
        buffers[file.path]?.let { return it }
        loading[file.path]?.let { return it.await() }
        val server = engine ?: error("Connect to the workspace and choose a file.")
        require(!file.dir) { "Choose a file, not a directory." }
        val current = generation
        val root = server.sourceRootAddress
        val workspace = server.sourceWorkspaceID
        val pending = workspaceScope.async(start=kotlinx.coroutines.CoroutineStart.LAZY) {
            val text = server.file(file)
            if(generation != current) throw CancellationException("Workspace changed")
            if (forStartup && (server.sourceRootAddress != root || server.sourceWorkspaceID != workspace || findFileNode(file.path, rootFiles) != file)) {
                throw CancellationException("Requested source authority changed")
            }
            buffers[file.path]?.let { available ->
                if (available.startupObservation) confirmStartupPreview(file, server, response = text)
                return@async buffers[file.path] ?: available
            }
            if (forStartup && file.generatedBy == null && text.saveGuard.isNullOrEmpty()) {
                val source = StartupSource(file.path, text.content, root.orEmpty(), workspace)
                retainStartupPreview(file, server, source, text.copy(saveGuard = null, saveGuardReason = startupPermissionNotice(text)))
            } else retainBuffer(file, text, server)
        }
        loading[file.path] = pending; pending.start()
        try { return pending.await() }
        finally { if(generation == current) loading.remove(file.path) }
    }
    internal fun adoptCreatedSource(source:DocumentBuffer,file:FileNode,text:FileText,server:Engine):DocumentBuffer {
        check(engine===server&&buffers[file.path]==null&&source.text==text.content){"Creation namespace or source changed before adoption."}
        val old=source.file.path
        val addresses=(buffers.filterValues{it===source}.keys.map(::sourceTab)+retainedResourceTabs.filterValues{it===source}.keys).toSet()
        check(addresses.isNotEmpty()){"The source editor no longer owns its captured address."}
        val adopted=layout.copy(nodes=layout.nodes.mapValues{(_,node)->node.copy(tabs=node.tabs.map{if(it in addresses)sourceTab(file.path) else it}.distinct(),active=node.active?.let{if(it in addresses)sourceTab(file.path) else it})})
        sourceTemplates.close();sourceHistory.close();behaviorReading.close()
        generation=UUID.randomUUID();loading.values.forEach{it.cancel()};loading.clear();newestOpen.clear()
        debugger?.resourceSourcesChanged(listOf(NativeSourceRoute(source.debugIdentity,if(source.resourceDeleted)null else old,file.path,text.content,true)))
        source.language?.close();source.language=null
        buffers.entries.filter{it.value===source}.map{it.key}.forEach{buffers.remove(it)}
        retainedResourceTabs.entries.removeAll{it.value===source};retainedResourceEditors.entries.removeAll{it.value===source}
        source.isUntitledDraft=false;source.file=file;source.base=text.content;source.hash=text.hash;source.saveGuard=text.saveGuard;source.saveGuardReason=null;source.resourceDeleted=false;source.unmaterializedHistoryDraft=false;source.creationAttempt=null;source.error="";source.advanceSourceRevision()
        retainBuffer(file,text,server,source);layout=adopted
        return source
    }
    internal fun retainBuffer(file: FileNode, text: FileText, server: Engine, existing:DocumentBuffer?=null): DocumentBuffer {
        val buffer = existing ?: DocumentBuffer(file, text)
        buffers[file.path] = buffer
        buffer.rememberNavigation = { from, to -> navigationTrail = navigationTrail.record(from, to) }
        buffer.followWorkspaceLink = follow@{ href ->
            if (buffers[buffer.file.path] !== buffer) return@follow
            val target = workspaceLinkTarget(href, buffer.file.path) ?: run {
                status = "This link must point to a file in the workspace."
                return@follow
            }
            val scope = generation
            val revision = buffer.sourceRevision
            workspaceScope.launch {
                openSemanticLocation(SemanticLocation(languageURI(target.path), target.line ?: 0, 0), allowDirty = true,
                    isCurrent = { generation == scope && buffers[buffer.file.path] === buffer && buffer.sourceRevision == revision })
            }
        }
        buffer.openSemanticLocation = { location, current -> workspaceScope.launch {
            openSemanticLocation(location, isCurrent = { buffers[buffer.file.path] === buffer && current() })
        } }
        if(sourceTestIndex==null)sourceTestIndex=SourceTestIndex(workspaceScope){path,source->SourceTestScan.decode(server.request("POST","/api/tests/source-candidates",org.json.JSONObject().put("path",path).put("source",source)))}
        if(coverage.digests==null)coverage.digests=SourceDigests(workspaceScope)
        buffer.coverageSource=coverage
        buffer.inspectCoverage={line->if(buffer.coverageRows().any{it.line==line})workspaceScope.launch {open(buffer.file);showCoverage=true}}
        buffer.testIndex=sourceTestIndex
        buffer.inspectTests={line->workspaceScope.launch {inspectTestSource(buffer,line)}}
        buffer.receiptEngine = server
        debugger?.registerSource(buffer,confirmedAddressChange=existing!=null)
        if(languageConnection==null) languageConnection=NativeLanguageConnection(server,workspaceScope)
        buffer.language = NativeLanguage(server,buffer,workspaceScope,languageConnection); buffer.language?.start()
        return buffer
    }
    suspend fun openGitComparison(comparison: EditorGitComparison, current: () -> Boolean): Boolean {
        val file = findFileNode(comparison.path, files) ?: return false
        if (file.dir || !activateFile(file, recordNavigation = true, isCurrent = current) || !current()) return false
        val owner = currentBuffer?.takeIf { it.file.path == comparison.path } ?: return false
        owner.gitComparison = comparison
        return true
    }

    suspend fun open(file: FileNode, recordNavigation: Boolean = true) {
        activateFile(file, recordNavigation)
    }
    private suspend fun activateFile(file: FileNode, recordNavigation: Boolean, isCurrent: (() -> Boolean)? = null,
                                     forStartup: Boolean = false): Boolean {
        if (isCurrent?.invoke() == false) return false
        val departed = currentSemanticLocation()
        if (file.path in buffers) {
            show(sourceTab(file.path))
            if (recordNavigation) currentSemanticLocation()?.let { navigationTrail = navigationTrail.record(departed, it) }
            if (file.path == startupPreview?.source?.path) engine?.let { confirmStartupPreview(file, it) }
            return true
        }
        val current = generation
        val target = layout.groups().firstOrNull {it.id==focused && "files" !in it.tabs}?.id
            ?: layout.groups().firstOrNull {"files" !in it.tabs}?.id ?: focused
        val request = Any()
        newestOpen[target] = request
        if (!forStartup) foregroundNavigation++
        val startingPane = focused
        if (isCurrent == null) focused = target
        fun ownsGroup() = generation == current && newestOpen[target] === request && layout.groups().any { it.id == target } &&
            (isCurrent == null || focused == startingPane && isCurrent())
        return try {
            val buffer = loadBuffer(file, forStartup)
            if (!ownsGroup()) return false
            layout = layout.activate(sourceTab(file.path), target)
            if (isCurrent != null) focused = target
            if (focused == target) {
                status = if (buffer.startupObservation) buffer.saveGuardReason ?: "Waiting for workspace permission." else file.path
                if (recordNavigation) currentSemanticLocation()?.let { navigationTrail = navigationTrail.record(departed, it) }
            }
            focused == target
        } catch(cancelled:CancellationException) {throw cancelled}
        catch(failure:Exception) {
            if (ownsGroup() && (isCurrent != null || focused == target)) status = "${failure.message} Refresh files and try opening ${file.path} again."
            false
        } finally {
            if (newestOpen[target] === request) newestOpen.remove(target)
        }
    }
    fun currentSemanticLocation(): SemanticLocation? {
        val buffer = currentBuffer ?: return null
        val position = languagePosition(buffer.text, buffer.state.selection.min)
        return SemanticLocation(languageURI(buffer.file.path), position.getInt("line"), position.getInt("character"))
    }
    suspend fun navigateHistory(direction: Int) {
        val (next, location) = navigationTrail.step(direction, currentSemanticLocation()); navigationTrail = next
        if (location != null) openSemanticLocation(location, recordHistory = false, allowDirty = true)
    }
    suspend fun openSemanticLocation(location: SemanticLocation, recordHistory: Boolean = true, allowDirty: Boolean = false, expectedSource: String? = null, isCurrent: () -> Boolean = {true}, onRevealed: () -> Unit = {}) {
        if(!isCurrent())return
        val departed = currentSemanticLocation()
        val path = location.path ?: return
        fun find(nodes: List<FileNode>): FileNode? { nodes.forEach { if (it.path == path) return it; find(it.children)?.let { child -> return child } }; return null }
        val file = (buffers[path]?.file ?: find(rootFiles))?.takeUnless { it.dir } ?: run { status = "Location not found: $path. Refresh files and query again."; return }
        val current = generation
        if (!activateFile(file, recordNavigation = false, isCurrent = isCurrent)) return
        if (!isCurrent() || generation != current || active != sourceTab(path)) return
        val buffer = buffers[path] ?: return
        if (buffer.dirty && !allowDirty && departed?.path != path) { status = "Save $path and query again before using a cross-file source position."; return }
        val source = buffer.text
        if (expectedSource != null && source != expectedSource) { status = "Source changed. Refresh the query before navigating."; return }
        val offset = languageOffset(source, org.json.JSONObject().put("line", location.line).put("character", location.character))
            ?: run { status = "Location no longer fits $path. Query again."; return }
        if (recordHistory && expectedSource == null && departed?.path != path && !location.matchesInferredSourceName(source, offset)) {
            status = "Source changed. Refresh the query before navigating."
            return
        }
        if (buffer.selectSourceRange(offset, offset, source)) { if (recordHistory) navigationTrail = navigationTrail.record(departed, location); buffer.focusSource?.invoke(); onRevealed() }
    }
    private var debugRevealTicket=0L
    suspend fun revealDebug(path: String, line: Int) {
        val debug=debugger ?: return;val request=++debugRevealTicket;val current=generation;val group=focused;val departed=active;val locationCurrent=debug.locationGuard(path,line)
        fun find(nodes: List<FileNode>): FileNode? { nodes.forEach { if (it.path == path) return it; find(it.children)?.let { child -> return child } }; return null }
        val original=debug.sourceEditor(path)?.takeIf{editor->buffers.values.any{it===editor}||retainedResourceTabs.values.any{it===editor}}
        val address=original?.let{editor->buffers.entries.firstOrNull{it.value===editor}?.key?.let(::sourceTab) ?: retainedResourceTabs.entries.firstOrNull{it.value===editor}?.key} ?: sourceTab(path)
        val file=(original?.file ?: buffers[path]?.file ?: find(rootFiles))?.takeUnless{it.dir} ?: run{status="Reported debugger source is not in this workspace. Refresh files to inspect it.";return}
        if(!locationCurrent())return
        val buffer=original ?: loadBuffer(file)
        if(generation!=current||debugger!==debug||request!=debugRevealTicket||!locationCurrent()||focused!=group||active!=departed)return
        show(address) // The retained execution editor wins over a new same-spelling occupant.
        val visibleGroup=focused;val selection=buffer.state.selection;val scroll=buffer.sourceScroll.value
        debug.rememberSource(buffer)
        val valid={generation==current&&debugger===debug&&request==debugRevealTicket&&locationCurrent()&&focused==visibleGroup&&active==address&&bufferForTab(address)===buffer&&debug.positionSourceMatches(buffer)&&buffer.state.selection==selection&&buffer.sourceScroll.value==scroll}
        if(!valid()){
            if(generation==current&&request==debugRevealTicket&&active==address)status=if(debug.adapterSourceRequired)"Opened saved path text. Adapter source is unavailable here; its positive source reference forbids a precise disk-source reveal." else if(!debug.positionSourceMatches(buffer))"Opened reported debugger source. Its text is unbound to captured launch/mapping source, so no precise position was revealed." else "Debugger position or editor interaction changed. The editor is retained; choose the current frame again to reveal its captured position."
            return
        }
        val source=buffer.text;buffer.debugRevealSource=source;buffer.debugRevealCurrent={valid()&&buffer.text==source}
        buffer.debugReveal=line to ((buffer.debugReveal?.second ?: 0)+1)
    }
    override fun onCleared() { sourceTestIndex?.stop(); buffers.values.forEach { it.language?.close() }; languageConnection?.close(); debugger?.close(); agent.debugger?.close(); super.onCleared() }
    suspend fun openTerminal() {
        val server = engine ?: run { status = "Connect to a workspace server before opening Terminal."; return }
        show("terminal"); terminal.open(server)
    }
    fun show(tab: String, group: String? = null) {
        if (tab == "files") hiddenFilesPane = null
        val target = group ?: layout.groups().firstOrNull { tab in it.tabs }?.id ?: (if (tab != "files") layout.groups().firstOrNull { it.id == focused && "files" !in it.tabs }?.id ?: layout.groups().firstOrNull { "files" !in it.tabs }?.id else null) ?: focused
        newestOpen.remove(target)
        foregroundNavigation++
        layout = layout.activate(tab, target); focused = target
    }
    internal fun adoptTabLayout(next: PaneLayout, group: String? = null): Boolean {
        if (layout === next) return false
        val groups = next.groups().associateBy { it.id }
        newestOpen.keys.removeAll { id -> id == group || id !in groups || groups[id]?.active != layout.nodes[id]?.active }
        if (group != null || groups[focused]?.active != active) foregroundNavigation++
        layout = next
        focused = group ?: focused.takeIf { it in groups } ?: groups.keys.first()
        return true
    }
    fun close(tab: String) {
        val buffer = bufferForTab(tab)
        if (buffer != null && (buffer.dirty || buffer.saving)) { status = "Unsaved edits retained. Save this document before closing its tab."; return }
        adoptTabLayout(layout.remove(tab))
    }
    fun split(axis: String) {
        val id = UUID.randomUUID().toString(); layout = layout.split(focused, axis, id); focused = "retained-$id"
    }
    fun moveNext() {
        val groups = layout.groups(); val tab = active ?: return
        val index = groups.indexOfFirst { it.id == focused }; if (index < 0) return
        val target = groups[(index + 1) % groups.size].id
        adoptTabLayout(layout.move(tab, target), target)
    }
    fun label(tab: String) = when (tab) {
        "files" -> "Files"; "agent" -> "Agent"; "terminal" -> "Terminal"
        else -> (bufferForTab(tab)?.file?.name ?: tab.substringAfterLast('/')) + (if(bufferForTab(tab)?.resourceDeleted==true) " (deleted)" else "") + if (bufferForTab(tab)?.dirty == true) " •" else ""
    }
    var savingAll = false
    var fileCommand by mutableStateOf<String?>(null)
    var filePath by mutableStateOf("")
    fun newTab() {
        val path = "Untitled-" + UUID.randomUUID()
        buffers[path] = DocumentBuffer(FileNode("Untitled ${buffers.values.count { it.isUntitledDraft } + 1}", path, false, emptyList(), null, null), FileText("", ""),untitledDraft=true); show(sourceTab(path))
    }
    fun chooseFileCommand(command: String) { filePath = ""; fileCommand = command }
    suspend fun performFileCommand() {
        val command = fileCommand ?: return
        val server = engine ?: run { status = "Connect to a workspace server first."; return }
        val path = filePath; val source = currentBuffer; val sourceTab=active;fileCommand = null
        fun find(nodes: List<FileNode>): FileNode? {
            for (node in nodes) { if (node.path == path) return node; find(node.children)?.let { return it } }; return null
        }
        if (command == "Save As") {
            if (source == null || source.saving || path.isEmpty()) return
            if (path == source.file.path) {
                if(source.resourceDeleted){status="The original source is deleted. Choose a different unused path to save its retained text as a copy.";return}
                source.save(server); return
            }
            source.saving = true
            try {
                val copy=createSourceCopy(source,path)
                show(sourceTab(copy.file.path));refresh()
                status=copy.error.ifEmpty{"Created source confirmed. The original saved file was not overwritten."}
                source.saving=false
                if(savingAll)saveAll()
            } catch (cancelled: CancellationException) { throw cancelled }
            catch (failure: Exception) { status = "${failure.message} Original edits are retained. Inspect the destination and Local history before another Save As; an uncertain request is not automatically retried." }
            finally { source.saving = false }
        } else if (command == "Open Directory" && path.isEmpty()) { directoryScope="";refresh(); show("files") }
        else {
            val file = find(files)
            if (file == null || file.dir != (command == "Open Directory")) { status = "Path not found. Refresh and choose a path in this workspace."; return }
            if (file.dir) { directoryScope=file.path;files = file.children; show("files") } else open(file)
        }
    }
    suspend fun saveAll() {
        val server = engine ?: return
        savingAll = true
        for (buffer in buffers.values.toList().filter { !it.isUntitledDraft }) buffer.save(server)
        val draft = buffers.values.firstOrNull { it.isUntitledDraft && it.dirty }
        if (draft != null) { show(sourceTab(draft.file.path)); chooseFileCommand("Save As") } else savingAll = false
    }
    suspend fun save() {
        if (currentBuffer?.isUntitledDraft == true || currentBuffer?.resourceDeleted == true) { chooseFileCommand("Save As"); return }
        engine?.let { currentBuffer?.save(it) }
    }
}
