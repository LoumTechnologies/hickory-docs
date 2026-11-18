import SwiftUI
import Observation

@MainActor @Observable final class DocumentBuffer: Identifiable {
    let id = UUID()
    var debugInspection=UUID()
    var showSnippetTools=false
    @ObservationIgnored lazy var snippets=SourceSnippetController(buffer:self)
    var file: FileNode
    var resourceDeleted = false
    var unmaterializedHistoryDraft = false
    var recoveryViewport:EditorRecoveryViewport?
    var recoveryBaseline:EditorRecoveryBaseline?
    @ObservationIgnored var captureRecoveryViewport:(()->CGPoint)?
    // Source equality alone must not revive a preview after edit/undo.
    var sourceEditIdentity=UUID()
    private(set) var agentSourceRevision:UInt64?=0
    func retireAgentContext(){agentSourceRevision=agentSourceRevision.flatMap{$0 == UInt64.max ? nil:$0+1}}
    @ObservationIgnored var saveHistoryAttempt:SourceSaveAttempt?
    @ObservationIgnored var creationAttempt:SourceCreationAttempt?
    var savedGuard:String?
    var saveGuardReason:String?
    var savedHistoryNotice=""
    private var sourceValue:ExactSourceText {
        didSet {let changed = !exactUTF8(text,base);if dirty != changed {dirty=changed};if sourceValue != oldValue {sourceEditIdentity=UUID();retireAgentContext();snippets.sourceChanged(text);language?.changed();refreshInline()}}
    }
    var text:String {get {sourceValue.text} set {sourceValue=ExactSourceText(newValue)}}
    var sourceRevision:ExactSourceText {sourceValue}
    @ObservationIgnored var base: String {
        didSet { let changed = !exactUTF8(text,base); if dirty != changed { dirty = changed } }
    }
    @ObservationIgnored var navigate: ((Double) -> Void)?
    @ObservationIgnored var renderLanguage: (([NativeSemanticToken], String) -> Void)?
    @ObservationIgnored var canUndoSource:(()->Bool)?
    @ObservationIgnored var canRedoSource:(()->Bool)?
    @ObservationIgnored var undoSource:(()->Bool)?
    @ObservationIgnored var redoSource:(()->Bool)?
    @ObservationIgnored var sourceEditorState:(()->NativeSourceEditorState?)?
    @ObservationIgnored var applyCompletionDraft: ((CompletionDraft,NSRange)->Bool)?
    @ObservationIgnored var applyLanguageCompletion: ((NSRange, String, String, NSRange) -> Bool)?
    // Cursor round trips cannot revive a retired provider or template context.
    @ObservationIgnored var selectionIdentity=UUID()
    @ObservationIgnored var selection = NSRange(location: 0, length: 0) {didSet{if selection != oldValue{selectionIdentity=UUID()}}}
    @ObservationIgnored var receiptEngine: Engine?
    var receiptSource = "" { didSet { renderMarkup?() } }
    var runOutputSource = ""
    var runOutputs: [Int: String] = [:] { didSet { renderMarkup?() } }
    var receipts: [InlineReceipt] = []
    var receiptRevision = ""
    var receiptNotice = ""
    func refreshReceipts() async {
        guard let engine = receiptEngine, let id = file.doc_id else { return }
        guard !dirty else { receiptNotice = "Save current source to view matching recorded output."; return }
        let source = text; receiptNotice = "Loading recorded output…"
        do {
            let page: InlineReceiptPage = try await engine.request("GET", "/api/docs/" + id + "/render")
            guard exactUTF8(text,source), !dirty, page.document == id, page.source_hash.replacingOccurrences(of: "sha256:", with: "") == nativeSourceHash(source) else {
                receiptNotice = "Recorded output belongs to another source revision. Save or refresh before retrying."; return
            }
            receipts = page.blocks.filter { $0.kind == "exec" }; receiptRevision = UUID().uuidString; receiptSource = source; receiptNotice = ""
        } catch { if exactUTF8(text,source) { receiptNotice = "Recorded output unavailable. Retry after checking the server." } }
    }
    // Recorded execution receipts already carry an exact source-hash check.
    // Their byte spans place the read-only transcript after the producing block.
    func recordedOutputPlacements() -> [Int: String] {
        guard exactUTF8(receiptSource,text), !dirty else { return [:] }
        let bytes = Array(text.utf8)
        var result: [Int: String] = [:]
        for receipt in receipts where receipt.span.count == 2 && receipt.span[0] >= 0 && receipt.span[1] > receipt.span[0] && receipt.span[1] <= bytes.count {
            let chunks = (receipt.transcript ?? []).filter { ["out", "err", "stdout", "stderr"].contains($0.kind) }.compactMap(\.data).joined()
            let output = chunks.isEmpty ? receipt.recorded_output ?? "" : chunks
            if !output.isEmpty { result[bytes.prefix(receipt.span[1] - 1).filter { $0 == 10 }.count, default: ""] += output }
        }
        return result
    }
    func receipts(on line: Int) -> [InlineReceipt] {
        guard exactUTF8(receiptSource,text), !dirty else { return [] }
        let bytes = Array(text.utf8)
        return receipts.filter { $0.span.count == 2 && $0.span[0] >= 0 && $0.span[1] >= $0.span[0] && $0.span[1] <= bytes.count && bytes.prefix($0.span[0]).filter { $0 == 10 }.count == line }
    }
    // A source reveal changes presentation only; the editor and its undo stack stay alive.
    @ObservationIgnored var applyTableEdit: ((NativeSourceEdit, String) -> Bool)?
    var sourceTable: Int? { didSet { renderMarkup?() } }
    var tableHeights: [Int: CGFloat] = [:]
    var diagramSizes: [Int: CGSize] = [:]
    @ObservationIgnored var openDiagramFile: ((String) -> Void)?
    var tableSourceVisible = false { didSet { renderMarkup?() } }
    var inlineMarkup = InlineSnapshot(source: "")
    @ObservationIgnored var renderMarkup: (() -> Void)?
    @ObservationIgnored private var inlineTask: Task<Void, Never>?
    func refreshInline() {
        inlineTask?.cancel(); let source = text, path = file.path
        inlineTask = Task { [weak self] in
            try? await Task.sleep(for: .milliseconds(100))
            guard !Task.isCancelled else { return }
            let job = Task.detached { computeInlineMarkup(source, path: path) }
            let result = await withTaskCancellationHandler(operation: { await job.value }, onCancel: { job.cancel() })
            guard !Task.isCancelled, let self, exactUTF8(self.text,source) else { return }
            self.inlineMarkup = result; self.renderMarkup?()
        }
    }
    var visibleDebugLines: [DebugLine] = []
    @ObservationIgnored var focusSource: (() -> Void)?
    func closeFind() { showFind = false; focusSource?() }
    var multiSource = ""
    var multiRanges: [NSRange] = [] {
        didSet {
            if multiRanges != oldValue { selectionIdentity = UUID() }
        }
    }
    var multiRects: [CGRect] = []
    var showMulti = false
    var multiText = ""
    @ObservationIgnored var applyingMulti = false
    func selectRectangle() {
        let start = languagePosition(text, selection.location), end = languagePosition(text, NSMaxRange(selection))
        let rows = text.components(separatedBy: "\n")
        guard end["line"]! - start["line"]! < 256 else { return }
        let left = min(start["character"]!, end["character"]!), right = max(start["character"]!, end["character"]!)
        let ranges = (start["line"]!...end["line"]!).compactMap { row -> NSRange? in
            let from = languageOffset(text, ["line": row, "character": min(left, rows[row].utf16.count)])!
            let to = languageOffset(text, ["line": row, "character": min(right, rows[row].utf16.count)])!
            return NSRange(location: from, length: to - from)
        }
        _ = selectMultiple(ranges, source: text)
    }
    func addCursor(_ direction: Int) {
        let active = exactUTF8(multiSource,text) ? multiRanges : []
        let anchor = (direction > 0 ? active.last : active.first)?.location ?? selection.location
        let pos = languagePosition(text, anchor), rows = text.components(separatedBy: "\n")
        let row = pos["line"]! + direction
        guard rows.indices.contains(row), let offset = languageOffset(text, ["line": row, "character": min(pos["character"]!, rows[row].utf16.count)]) else { return }
        _ = selectMultiple((exactUTF8(multiSource,text) && !multiRanges.isEmpty ? multiRanges : [selection]) + [NSRange(location: offset, length: 0)], source: text)
    }
    func clearMulti() { multiSource = ""; multiRanges = []; multiRects = [] }
    func selectMultiple(_ ranges: [NSRange], source: String) -> Bool {
        let sorted = Array(Set(ranges)).sorted { $0.location < $1.location }
        guard exactUTF8(text,source), sorted.count > 1, sorted.count <= 256,
              validSourceEdits(sorted.map { NativeSourceEdit(range: $0, text: "") }, length: source.utf16.count) else { return false }
        applyingMulti = true; defer { applyingMulti = false }
        guard selectSourceRange?(sorted[0], source) == true else { return false }
        multiSource = source; multiRanges = sorted; return true
    }
    func replaceMultiple(_ replacement: String, affected: NSRange? = nil) -> Bool {
        guard !readOnly, exactUTF8(multiSource,text), multiRanges.count > 1 else { return false }
        let source = text as NSString
        var ranges = multiRanges
        if let affected, let primary = ranges.first, affected != primary {
            guard replacement.isEmpty, primary.length == 0 else { clearMulti(); return false }
            if NSMaxRange(affected) == primary.location {
                ranges = ranges.map { $0.length == 0 && $0.location > 0 ? source.rangeOfComposedCharacterSequence(at: $0.location - 1) : $0 }
            } else if affected.location == primary.location {
                ranges = ranges.map { $0.length == 0 && $0.location < source.length ? source.rangeOfComposedCharacterSequence(at: $0.location) : $0 }
            } else { clearMulti(); return false }
        }
        let edits = ranges.map { NativeSourceEdit(range: $0, text: replacement) }
        guard validSourceEdits(edits, length: source.length) else { return false }
        applyingMulti = true; defer { applyingMulti = false }
        guard applySourceEdits?(edits, text) == true else { return false }
        var delta = 0
        let carets = ranges.map { range -> NSRange in
            let caret = NSRange(location: range.location + delta + replacement.utf16.count, length: 0)
            delta += replacement.utf16.count - range.length; return caret
        }
        multiSource = text; multiRanges = carets
        _ = selectSourceRange?(carets[0], text); return true
    }
    var showFind = false
    var findQuery = ""
    var findReplacement = ""
    var findCase = false
    var findRegex = false
    @ObservationIgnored var selectSourceRange: ((NSRange, String) -> Bool)?
    @ObservationIgnored var canReflectWorkspaceEdits: (([NativeSourceEdit], String) -> Bool)?
    @ObservationIgnored var adoptSourceDraft: ((String,String)->Bool)?
    @ObservationIgnored var reflectWorkspaceEdits: (([NativeSourceEdit], String, String) -> Bool)?
    @ObservationIgnored var applySourceEdits: (([NativeSourceEdit], String) -> Bool)?
    var foldedLines: Set<Int> = []
    @ObservationIgnored var toggleFold: ((Int) -> Void)?
    @ObservationIgnored var revealDebugLine: ((Int) -> Bool)?
    @ObservationIgnored var rememberNavigation: ((SemanticLocation, SemanticLocation) -> Void)?
    @ObservationIgnored var openSemanticLocation: ((SemanticLocation) -> Void)?
    @ObservationIgnored var coverageSource:SimpleCoverageState?
    @ObservationIgnored var inspectCoverage:((Int)->Void)?
    @ObservationIgnored var testIndex:SourceTestIndex?
    @ObservationIgnored var inspectTests:((Int)->Void)?
    var language: NativeLanguage?
    var hash: String
    var saving = false
    var error = ""
    private(set) var dirty = false
    private(set) var isUntitledDraft:Bool
    func confirmSourceAddress(){isUntitledDraft=false}
    var readOnly: Bool { (resourceDeleted && !unmaterializedHistoryDraft) || file.generated_by != nil }
    init(file: FileNode, text: FileText,isUntitledDraft:Bool=false) {
        self.isUntitledDraft=isUntitledDraft
        self.file = file; self.sourceValue = ExactSourceText(text.content); base = text.content; hash = text.hash;savedGuard=text.save_guard;saveGuardReason=text.save_guard_reason; refreshInline()
    }
    func save(using engine: Engine) async {
        guard !resourceDeleted, dirty, !saving, !readOnly else { return }
        if let editor=sourceEditorState?(),editor.composing || (editor.attached && !exactUTF8(editor.source,text)){error="Finish source composition before saving. Your draft is retained.";return}
        guard let guardToken=savedGuard else{error=saveGuardReason ?? "This source has no captured physical save guard. Its text remains editable; reopen or compare the exact file before saving.";return}
        let submitted = text, previous = base, previousHash = hash
        let attempt=saveHistoryAttempt.flatMap{$0.matches(source:submitted,base:previous,hash:previousHash,guardToken:guardToken,revision:sourceEditIdentity) ? $0:nil} ?? SourceSaveAttempt(source:submitted,base:previous,hash:previousHash,guardToken:guardToken,revision:sourceEditIdentity)
        saveHistoryAttempt=attempt
        saving = true; error = ""
        defer { saving = false }
        do { hash = try await engine.saveSourceWithHistory(self,text:submitted,base:previous,hash:previousHash,requestID:attempt.id,guardToken:guardToken);base=submitted;saveHistoryAttempt=nil }
        catch { self.error = "\(error.localizedDescription) Your edits are retained. Copy them before closing; compare with disk before retrying." }
    }
}
@MainActor @Observable final class Workspace {
    var draggingTab: String?
    let changes = WorkspaceChanges()
    var showWorkspaceSearch = false
    var search = WorkspaceSearch()
    func openSearchHit(_ hit: WorkspaceSearchHit) async {
        guard let engine else { return }
        let current = generation, request = search.revision
        func find(_ nodes: [FileNode]) -> FileNode? { for node in nodes { if exactUTF8(node.path,hit.path) { return node }; if let child = find(node.children ?? []) { return child } }; return nil }
        guard let file = buffers[hit.path]?.file ?? find(rootFiles), !file.dir else { search.notice = "File no longer listed. Refresh files and search again."; return }
        do {
            let source: String
            if let buffer = buffers[hit.path] { source = buffer.text } else { source = try await engine.file(file).content }
            let observedHash = await Task.detached { nativeSourceHash(source) }.value
            let rows = source.components(separatedBy: "\n")
            guard generation == current, search.revision == request else { return }
            guard let hash = hit.sourceHash, observedHash == hash.replacingOccurrences(of: "sha256:", with: "").lowercased(), let character = hit.character,
                  hit.line > 0, rows.indices.contains(hit.line - 1), rows[hit.line - 1].trimmingCharacters(in: .newlines) == hit.text.trimmingCharacters(in: .newlines) else { search.notice = "Search source changed or its revision is unknown. Save edits and search again."; return }
            await openSemanticLocation(SemanticLocation(uri: languageURI(hit.path), line: hit.line - 1, character: character), allowDirty: true, expectedSource: source)
            if generation == current, search.revision == request { showWorkspaceSearch = false }
        } catch { if generation == current, search.revision == request { search.notice = error.localizedDescription } }
    }
    var analysis = WorkspaceAnalysis()
    var sourceTestIndex:SourceTestIndex?
    var agentProposals=AgentProposalModel()
    var sourceTemplates=SourceTemplatesModel()
    var sourceHistory=SourceHistoryModel()
    var editorRecovery=EditorRecoveryModel()
    var sourceRootAddress:String?
    var sourceWorkspaceID:String?
    var testSources=TestSourcesModel()
    var coverage=SimpleCoverageState();var showCoverage=false
    var showTests=false {didSet{if !showTests{testSources.closeDebugPreparation(self)}}}
    var showReview=false
    var gitReview=GitReviewModel()
    var gitResolution=GitResolutionModel()
    var showHistory=false
    var showRecovery=false
    var showConstruction=false
    var showAttribution=false
    var gitAttribution=GitAttributionModel(path:"")
    var gitHistory=GitHistoryModel()
    var showEverywhere = false
    var showAnalysis = false
    func startAnalysis() {
        let model=analysis, current=generation, files=rootFiles
        Task { _ = await model.run(files,load:{file in
            guard self.generation==current else {throw CancellationError()}
            return try await self.loadBuffer(file)
        },outline:{file,source in
            guard self.generation==current,let service=self.buffers[file.path]?.language else {throw EngineError(message:"Language service unavailable. Retry analysis.")}
            return try await service.readOutline(source:source)
        }) }
    }
    var showProblems = false
    var problems: [WorkspaceProblem] {
        buffers.values.sorted { $0.file.path < $1.file.path }.flatMap { buffer -> [WorkspaceProblem] in
            guard let language = buffer.language else { return [] }
            let matchesSource = exactUTF8(language.diagnosticSource,buffer.text)
            return language.diagnostics.map { diagnostic in
                let source: String? = diagnostic.location == nil ? nil : language.diagnosticSource
                let current = source != nil && matchesSource
                let location = current ? diagnostic.location : nil
                return WorkspaceProblem(id: "\(buffer.file.path):\(diagnostic.id)", path: buffer.file.path, message: diagnostic.message, severity: diagnostic.severity, location: location, source: source, current: current)
            }
        }
    }
    func openProblem(_ problem: WorkspaceProblem, precise: Bool = true) async {
        if !precise { if let buffer = buffers[problem.path] { await open(buffer.file) }; return }
        guard problem.current, let location = problem.location, let source = problem.source, exactUTF8(buffers[problem.path]?.text,source) else {
            status = "Problem source changed or its location is unversioned. Refresh diagnostics."; showProblems = true; return
        }
        await openSemanticLocation(location, allowDirty: true, expectedSource: source); showProblems = false
    }
    func navigateProblem(_ direction: Int) async {
        guard let problem = nextWorkspaceProblem(problems, current: currentSemanticLocation(), direction: direction) else {
            status = "No current source locations. Open Problems to read provider messages."; showProblems = true; return
        }
        await openProblem(problem)
    }
    var navigationTrail = NavigationTrail()
    var layout = PaneLayout()
    var focused = "document-group"
    var rootFiles: [FileNode] = []
    var files: [FileNode] = []
    var buffers: NativeSourceMap<DocumentBuffer> = [:]
    let resourceRestoration=ResourceRestoration()
    var retainedResourceEditors:[String:DocumentBuffer]=[:]
    func retainedResourceTab(_ buffer:DocumentBuffer)->String {NativeTabIdentity.retained(buffer.id).address}
    func bufferForTab(_ tab:String)->DocumentBuffer? {
        switch NativeTabIdentity(address:tab){case .source(let key):return buffers[key.string];case .retained(let id):return retainedResourceEditors.values.first{$0.id==id};case .utility,.unavailable:return nil}
    }
    func captureResourceEditors(_ patches:[ResourceEditorPatch],restoring:[String:DocumentBuffer]=[:])->()->Bool {
        let session=generation
        let paths=patches.map{($0.path,buffers[$0.path])}
        let snapshots=(paths.compactMap{$0.1}+Array(restoring.values)).map{($0,$0.text,$0.base,$0.hash,$0.file.path)}
        return { [weak self] in
            guard let self,self.generation==session else{return false}
            return paths.allSatisfy { self.buffers[$0.0] === $0.1 } && snapshots.allSatisfy {
                exactUTF8($0.0.text,$0.1) && exactUTF8($0.0.base,$0.2) && exactUTF8($0.0.hash,$0.3) && exactUTF8($0.0.file.path,$0.4) && !$0.0.saving
            }
        }
    }
    func adoptResourceReceipt(_ patches:[ResourceEditorPatch],receipt:ResourcePublication<FileNode>,request:String,workspace:String,context:String,expected:[String:ResourceSaved],current:()->Bool,restoring:[String:DocumentBuffer]=[:],absentOriginal:String?=nil) throws->[ResourceEditorRoute<DocumentBuffer>] {
        guard current(),let engine else{throw EngineError(message:"Editor drafts or baselines changed after review. Preserve them and inspect Source recovery.")}
        let files=try confirmedResourceFiles(patches,receipt:receipt,request:request,workspace:workspace,context:context,expected:expected){($0.path,!$0.dir && $0.generated_by==nil)}
        var live:[String:ResourceEditorCapture<DocumentBuffer>]=[:]
        for patch in patches {if let before=patch.before {guard let buffer=buffers[patch.path],!buffer.saving,!buffer.readOnly else{throw EngineError(message:"Source editor unavailable. Preserve drafts and inspect recovery.")};live[before.identity]=ResourceEditorCapture(editor:buffer,source:buffer.text)}}
        guard restoring.values.allSatisfy({buffer in buffer.resourceDeleted && retainedResourceEditors.values.contains(where:{$0 === buffer}) && !buffers.values.contains(where:{$0 === buffer})}) else{throw EngineError(message:"Restoration must select a retained deleted editor. Inspect Source recovery.")}
        let editorPatches=patches.filter{!exactUTF8($0.path,absentOriginal) || $0.before != nil || $0.after != nil || buffers[$0.path] != nil}
        let routes=try routeResourceEditors(editorPatches,live:live,retained:restoring.mapValues{ResourceEditorCapture(editor:$0,source:$0.text)})
        let targets=routes.map{route->(ResourceEditorRoute<DocumentBuffer>,DocumentBuffer,[NativeSourceEdit]) in
            let entry=route.to.flatMap{files[Data($0.utf8)]}
            let buffer=route.editor ?? DocumentBuffer(file:entry!.file,text:FileText(path:entry!.file.path,content:route.after!,hash:entry!.hash))
            let edits=route.after.flatMap{exactUTF8(buffer.text,$0) ? nil:[NativeSourceEdit(range:NSRange(location:0,length:buffer.text.utf16.count),text:$0)]} ?? []
            return (route,buffer,edits)
        }
        for (route,buffer,edits) in targets {
            if let path=route.to,!patches.contains(where:{exactUTF8($0.path,path)&&$0.before != nil}),buffers[path] != nil {throw EngineError(message:"A destination editor appeared after review. Inspect recovery.")}
            let probe=edits.isEmpty ? [NativeSourceEdit(range:NSRange(location:0,length:0),text:"")]:edits
            if !(buffer.resourceDeleted && edits.isEmpty),buffer.applySourceEdits != nil,buffer.canReflectWorkspaceEdits?(probe,buffer.text) != true {throw EngineError(message:"Finish composition before adopting source editors. Inspect recovery.")}
        }
        var addresses:[Data:String]=[:]
        for (route,buffer,_) in targets {
            let from=route.from.map(nativeSourceTab) ?? retainedResourceTab(buffer)
            addresses[Data(from.utf8)]=route.to.map(nativeSourceTab) ?? retainedResourceTab(buffer)
        }
        let adoptedLayout=try layout.remappingTabs(addresses)
        debugger?.resourceNamespaceChanged(routes)
        generation=UUID();loading.values.forEach{$0.cancel()};loading=[:]
        // Release every old URI before attaching any destination: swaps otherwise
        // collide with the other editor's still-live language subscription.
        for (_,buffer,_) in targets {buffer.language?.close();buffer.language=nil}
        for (route,_,_) in targets {if let path=route.from {buffers.removeValue(forKey:path)}}
        for (route,buffer,edits) in targets {
            guard let path=route.to,let entry=files[Data(path.utf8)] else{buffer.resourceDeleted=true;retainedResourceEditors[route.identity]=buffer;continue}
            let before=buffer.text
            if !edits.isEmpty {
                if let adopt=buffer.adoptSourceDraft {guard adopt(before,route.after!) else{throw EngineError(message:"Native source adoption failed. Preserve drafts and inspect recovery.")}}
                else{buffer.text=route.after!;_ = buffer.reflectWorkspaceEdits?(edits,before,route.after!)}
            }
            buffer.file=entry.file;buffer.base=entry.saved.text!;buffer.hash=entry.hash;buffer.savedGuard=entry.save_guard;buffer.saveGuardReason=entry.save_guard==nil ? "Resource publication did not provide a physical save guard. Preserve the draft and read the exact saved source before another ordinary Save.":nil;buffer.saveHistoryAttempt=nil;buffer.resourceDeleted=false;buffer.unmaterializedHistoryDraft=false;buffer.error=""
            buffers[path]=buffer;retainedResourceEditors = retainedResourceEditors.filter{$0.value !== buffer}
            _ = retainBuffer(entry.file,text:FileText(path:path,content:route.after!,hash:entry.hash),engine:engine,existing:buffer)
        }
        layout=adoptedLayout
        return routes
    }
    func adoptCreatedSource(_ source:DocumentBuffer,file:FileNode,text:FileText,engine:Engine)throws->DocumentBuffer {
        guard buffers[file.path]==nil || (buffers[file.path] === source && source.isUntitledDraft),exactUTF8(source.text,text.content) else{throw EngineError(message:"Creation editor changed before adoption. Preserve its source and inspect the confirmed destination.")}
        let old=source.file.path,from=buffers[old] === source ? nativeSourceTab(old):retainedResourceTab(source)
        let adopted=try layout.remappingTabs([Data(from.utf8):nativeSourceTab(file.path)])
        debugger?.resourceNamespaceChanged([ResourceEditorRoute(identity:source.id.uuidString,from:source.resourceDeleted ? nil:old,to:file.path,before:source.text,after:source.text,editor:source)])
        sourceHistory.close();generation=UUID();loading.values.forEach{$0.cancel()};loading=[:]
        source.language?.close();source.language=nil
        if buffers[old] === source{buffers.removeValue(forKey:old)}
        retainedResourceEditors=retainedResourceEditors.filter{$0.value !== source}
        source.file=file;source.base=text.content;source.hash=text.hash;source.savedGuard=text.save_guard;source.saveGuardReason=nil;source.resourceDeleted=false;source.unmaterializedHistoryDraft=false;source.saveHistoryAttempt=nil;source.creationAttempt=nil;source.error=""
        _ = retainBuffer(file,text:text,engine:engine,existing:source);layout=adopted
        return source
    }

    var status = "Connect to a Hickory workspace server"
    var debugger: NativeDebugger?
    var engine: Engine?
    var address = ProcessInfo.processInfo.environment["HICKORY_API_ORIGIN"] ?? ""
    var connecting = false
    private var loading: NativeSourceMap<Task<DocumentBuffer, Error>> = [:]
    private var languageConnection: NativeLanguageConnection?
    private var newestOpen: [String: NativeSourceKey] = [:]
    let agent = AgentSession()
    func configureAgentContext(){agent.captureContext={ [weak self] in guard let self else{throw EngineError(message:"Original agent workspace closed.")};return try nativeAgentContext(self)}}
    let terminal = TerminalSession()
    private var generation = UUID()
    var sourceNavigationEpoch:UUID {generation}
    var activeTab:String? {layout.nodes[focused]?.active}
    var active: String? {activeTab.map{NativeTabIdentity(address:$0).sourcePath ?? $0}}
    var currentBuffer: DocumentBuffer? {activeTab.flatMap{bufferForTab($0)}}
    var hasUnsavedChanges: Bool { gitResolution.hasPending || gitResolution.hasDraft || resourceRestoration.busy || resourceRestoration.hasPending || !agent.prompt.isEmpty || agent.running || (Array(buffers.values)+Array(retainedResourceEditors.values)).contains { $0.dirty || $0.saving } || debugger?.active == true }
    func connect() async -> Bool {
        guard !connecting else { return false }
        guard !hasUnsavedChanges else { status = "Save documents and send or clear draft messages before changing workspace servers."; return false }
        connecting = true; defer { connecting = false }
        do {
            let next = try Engine(address: address)
            let tree = try await next.files()
            // The terminal renderer owns screen contents; reconnect by clearing the session ID.
            agent.turns = []; agent.error = ""; terminal.id = nil; terminal.error = ""
            debugger?.close()
            for buffer in buffers.values { buffer.language?.close() }
            languageConnection?.close();languageConnection=nil
            sourceHistory.close();sourceRootAddress=nil;sourceWorkspaceID=nil;sourceTemplates.close();analysis.stop(); analysis=WorkspaceAnalysis(); showAnalysis=false; showEverywhere=false;showTests=false;showReview=false;showHistory=false;showAttribution=false;showConstruction=false;showRecovery=false;gitAttribution.close();gitAttribution=GitAttributionModel(path:"");gitHistory.close();gitHistory=GitHistoryModel();gitReview.close();gitReview=GitReviewModel();gitResolution.reset();showCoverage=false;coverage.stop();coverage=SimpleCoverageState();testSources=TestSourcesModel();sourceTestIndex?.stop();sourceTestIndex=nil
            generation = UUID(); changes.reset(); search.invalidate(); showWorkspaceSearch = false; engine = next; debugger = NativeDebugger(engine: next)
            if let debug=debugger {debug.sourceBuffers={ [weak self] in guard let self else{return []};return Array(self.buffers.values)+Array(self.retainedResourceEditors.values)};debug.onLocation={ [weak self,weak debug] location in guard let self,let debug else{return};let scope=self.generation;Task {[weak self,weak debug] in guard let self,let debug,self.generation==scope else{return};await self.revealDebug(location,debug:debug)}}}; sourceRootAddress=tree.root_path;sourceWorkspaceID=tree.workspace_id;files = tree.tree; rootFiles = tree.tree; navigationTrail = NavigationTrail(); retainedResourceEditors = [:]; buffers = [:]; loading.values.forEach { $0.cancel() }; loading = [:]
            layout = PaneLayout(); focused = "document-group"; status = tree.root
            return true
        } catch { status = error.localizedDescription; return false }
    }
    func refresh() async {
        guard let engine else { return }
        let current = generation
        do { let tree = try await engine.files(); if current == generation { sourceRootAddress=tree.root_path;sourceWorkspaceID=tree.workspace_id;files = tree.tree; rootFiles = tree.tree; status = tree.root } }
        catch { if current == generation { status = error.localizedDescription } }
    }
    func loadBuffer(_ file: FileNode) async throws -> DocumentBuffer {
        if let buffer = buffers[file.path] { return buffer }
        if let pending = loading[file.path] { return try await pending.value }
        guard !file.dir, let engine else { throw EngineError(message:"Connect to the workspace and choose a file.") }
        let current = generation
        let pending = Task { @MainActor [weak self] () async throws -> DocumentBuffer in
            let text = try await engine.file(file)
            guard exactUTF8(text.path,file.path) else{throw EngineError(message:"File reply names a different source spelling. Its text was not attached to another editor; reopen the exact file.")}
            guard let self, self.generation == current else { throw CancellationError() }
            if let available = self.buffers[file.path] { return available }
            return self.retainBuffer(file, text:text, engine:engine)
        }
        loading[file.path] = pending
        defer { if generation == current { loading.removeValue(forKey:file.path) } }
        return try await pending.value
    }
    func retainBuffer(_ file: FileNode, text: FileText, engine: Engine, existing:DocumentBuffer?=nil) -> DocumentBuffer {
        let buffer = existing ?? DocumentBuffer(file: file, text: text)
        buffer.confirmSourceAddress()
        buffers[file.path] = buffer
        buffer.openDiagramFile = { [weak self, weak buffer] href in
            guard let self, let buffer, let path = diagramFilePath(href, relativeTo: buffer.file.path) else { return }
            Task {
                func find(_ nodes: [FileNode]) -> FileNode? {
                    for node in nodes { if exactUTF8(node.path,path) { return node }; if let file = find(node.children ?? []) { return file } }
                    return nil
                }
                if find(self.files) == nil { await self.refresh() }
                guard let file = find(self.files), !file.dir else { self.status = "File not found: \(path). Refresh files and check the diagram link."; return }
                await self.open(file)
            }
        }
        buffer.rememberNavigation = { [weak self] from, to in self?.navigationTrail.record(from: from, to: to) }
        buffer.openSemanticLocation = { [weak self] location in
            Task { await self?.openSemanticLocation(location) }
        }
        if sourceTestIndex==nil {sourceTestIndex=SourceTestIndex {path,source in try await engine.request("POST","/api/tests/source-candidates",body:["path":path,"source":source])}}
        buffer.coverageSource=coverage
        buffer.inspectCoverage={ [weak self,weak buffer] line in guard let self,let buffer,!buffer.coverageRows().filter({$0.line==line}).isEmpty else{return};Task {await self.open(buffer.file);self.showCoverage=true}}
        buffer.testIndex=sourceTestIndex
        buffer.inspectTests={ [weak self,weak buffer] line in guard let self,let buffer else {return};Task {await self.inspectTestSource(buffer,line:line)}}
        buffer.receiptEngine = engine
        if languageConnection == nil {languageConnection=NativeLanguageConnection(engine:engine)}
        buffer.language = NativeLanguage(engine:engine,buffer:buffer,connection:languageConnection); buffer.language?.start()
        return buffer
    }
    func open(_ file: FileNode, recordNavigation: Bool = true) async {
        let departed = currentSemanticLocation()
        if buffers[file.path] != nil {
            showSource(file.path)
            if recordNavigation, let arrived = currentSemanticLocation() { navigationTrail.record(from:departed,to:arrived) }
            return
        }
        let current = generation
        let target = layout.groups().first(where: { $0.id == focused && !$0.tabs.containsUtility(.files) })?.id
            ?? layout.groups().first(where: { !$0.tabs.containsUtility(.files) })?.id ?? focused
        newestOpen[target] = file.path.nativeSourceKey; focused = target
        do {
            _ = try await loadBuffer(file)
            guard current == generation else { return }
            if newestOpen[target] == file.path.nativeSourceKey {
                let live = layout.groups().first(where: { $0.id == target })?.id ?? layout.groups()[0].id
                layout.activateSource(file.path,in:live); focused = live; status = file.path
                if recordNavigation, let arrived = currentSemanticLocation() { navigationTrail.record(from:departed,to:arrived) }
            }
        } catch is CancellationError { return }
        catch { if current == generation { status = "\(error.localizedDescription) Refresh files and try opening \(file.path) again." } }
    }
    func currentSemanticLocation() -> SemanticLocation? {
        guard let buffer = currentBuffer else { return nil }
        let position = languagePosition(buffer.text, buffer.selection.location)
        return SemanticLocation(uri: languageURI(buffer.file.path), line: position["line"]!, character: position["character"]!)
    }
    func navigateHistory(_ direction: Int) async {
        guard let location = navigationTrail.step(direction, current: currentSemanticLocation()) else { return }
        await openSemanticLocation(location, recordHistory: false, allowDirty: true)
    }
    func openSemanticLocation(_ location: SemanticLocation, recordHistory: Bool = true, allowDirty: Bool = false, expectedSource: String? = nil, isCurrent: () -> Bool = {true}, onRevealed: () -> Void = {}) async {
        guard isCurrent() else {return}
        let departed = currentSemanticLocation()
        guard let path = location.path else { return }
        func find(_ nodes: [FileNode]) -> FileNode? {
            for node in nodes { if exactUTF8(node.path,path) { return node }; if let child = find(node.children ?? []) { return child } }; return nil
        }
        guard let file = buffers[path]?.file ?? find(rootFiles), !file.dir else { status = "Location not found: \(path). Refresh files and query again."; return }
        let current = generation
        await open(file, recordNavigation: false)
        guard isCurrent(), generation == current, exactUTF8(active,path), let buffer = buffers[path] else { return }
        if buffer.dirty && !allowDirty && departed?.path != path { status = "Save \(path) and query again before using a cross-file source position."; return }
        let source = buffer.text
        if let expectedSource, !exactUTF8(source,expectedSource) { status = "Source changed. Refresh the query before navigating."; return }
        guard let offset = languageOffset(source, ["line": location.line, "character": location.character]) else { status = "Location no longer fits \(path). Query again."; return }
        for _ in 0..<20 {
            guard isCurrent(), generation == current, exactUTF8(buffer.text,source), (!buffer.dirty || allowDirty || exactUTF8(departed?.path,path)) else { return }
            if buffer.selectSourceRange?(NSRange(location: offset, length: 0), source) == true { if recordHistory { navigationTrail.record(from: departed, to: location) }; buffer.focusSource?(); onRevealed(); return }
            try? await Task.sleep(for: .milliseconds(25))
        }
        status = "Opened \(path); editor could not reveal the position. Query again once it is ready."
    }
    private var debugReveal=UUID()
    func revealDebug(_ location:NativeDebugLocation,debug:NativeDebugger)async {
        guard debugger === debug,debug.locationCurrent(location) else{return}
        let path=location.path,scope=generation,ticket=UUID(),originFocus=focused,originActive=active,originBuffer=currentBuffer,originSelection=currentBuffer?.selection,originText=currentBuffer?.text
        let departed=currentSemanticLocation()
        debugReveal=ticket
        let current={self.generation==scope && self.debugReveal==ticket && self.debugger === debug && debug.locationCurrent(location)}
        let originCurrent={self.focused==originFocus && exactUTF8(self.active,originActive) && self.currentBuffer === originBuffer && self.currentBuffer?.selection==originSelection && exactUTF8(self.currentBuffer?.text,originText)}
        guard current() else{return}
        func find(_ nodes:[FileNode])->FileNode? {for node in nodes {if exactUTF8(node.path,path){return node};if let child=find(node.children ?? []){return child}};return nil}
        guard let file=buffers.values.first(where:{exactUTF8($0.file.path,path)})?.file ?? find(rootFiles),!file.dir else{status="Debugger source is not listed: \(path). Refresh files to read it.";return}
        do {
            let buffer=try await loadBuffer(file)
            guard current(),originCurrent() else{return}
            guard exactUTF8(buffer.file.path,path) else{status="Debugger source identity conflicts with a retained native address. Preserve drafts before reconciling its spelling.";return}
            showSource(path);status=path
            if let arrived=currentSemanticLocation(){navigationTrail.record(from:departed,to:arrived)}
            guard let proof=location.sourceHash,proof.range(of:"^[0-9a-f]{64}$",options:.regularExpression) != nil else{status=location.sourceReference.map{$0>0 ? "Opened \(path) without a position; this frame uses adapter-provided source reference \($0), not saved file bytes.":"Opened \(path) without a position; this frame has no captured source revision."} ?? "Opened \(path) without a position; this frame has no captured source revision.";return}
            guard debug.admitSourceReading(buffer,path:path,proof:proof),debug.sourceMatches(buffer),nativeSourceHash(buffer.text)==proof else{status="Opened \(path) without a position; current source identity or bytes are not a confirmed original launch/construction source.";return}
            let source=buffer.text,selection=buffer.selection,targetFocus=focused
            guard location.line>=0,languageOffset(source,["line":location.line,"character":0]) != nil else{status="Opened \(path); stopped position does not fit its captured source.";return}
            for _ in 0..<20 {
                guard current(),focused==targetFocus,exactUTF8(active,path),buffers[path] === buffer,exactUTF8(buffer.text,source),buffer.selection==selection,debug.sourceMatches(buffer) else{return}
                if buffer.revealDebugLine?(location.line)==true {status="Opened \(path); captured position accepted by the editor.";return}
                try? await Task.sleep(for:.milliseconds(25))
            }
            if current(){status="Opened \(path); captured position is waiting for an editor. Readable text is retained."}
        }catch{if current(){status=error.localizedDescription+" Debug source could not be opened; other buffers are retained."}}
    }
    func openTerminal() async {
        guard let engine else { status = "Connect to a workspace server before opening Terminal."; return }
        showUtility(.terminal); await terminal.open(using: engine)
    }
    func showSource(_ path:String){show(nativeSourceTab(path))}
    func showUtility(_ utility:NativeUtilityTab){show(utility.rawValue)}
    func show(_ tab: String) {
        let target = layout.groups().first(where: { $0.tabs.containsTab(tab) })?.id ?? (tab != "files" ? layout.groups().first(where: { $0.id == focused && !$0.tabs.containsUtility(.files) })?.id ?? layout.groups().first(where: { !$0.tabs.containsUtility(.files) })?.id : nil) ?? focused
        layout.activate(tab, in: target); focused = target
    }
    func close(_ tab: String) {
        if let buffer = bufferForTab(tab), buffer.dirty || buffer.saving {
            status = "Unsaved edits retained. Save this document before closing its tab."; return
        }
        if let buffer=bufferForTab(tab){buffer.retireAgentContext();sourceHistory.sourceClosed(buffer);sourceTemplates.sourceClosed(buffer);testSources.sourceClosed(buffer,workspace:self)}
        layout.remove(tab)
        if !layout.groups().contains(where: { $0.id == focused }) { focused = layout.groups()[0].id }
    }
    func split(_ axis: String) {
        if let retained = layout.split(focused, axis: axis, newID: UUID().uuidString) { focused = retained }
    }
    func moveToNext() {
        let groups = layout.groups()
        guard let active=activeTab, let index = groups.firstIndex(where: { $0.id == focused }) else { return }
        let target = groups[(index + 1) % groups.count].id
        layout.move(active, to: target); focused = target
    }
    func label(_ tab: String) -> String {
        switch NativeTabIdentity(address:tab).utility{case .files:return "Files";case .agent:return "Agent";case .terminal:return "Terminal";case nil:break}
        return (bufferForTab(tab)?.file.name ?? (tab as NSString).lastPathComponent) + (bufferForTab(tab)?.resourceDeleted == true ? " (retained)" : "") + (bufferForTab(tab)?.dirty == true ? " •" : "")
    }
    var savingAll = false
    var fileCommand: String?
    private var filePathKey=NativeSourceKey("")
    var filePath:String {get{filePathKey.string}set{filePathKey=NativeSourceKey(newValue)}}
    func newTab() {
        let path = "Untitled-" + UUID().uuidString
        buffers[path] = DocumentBuffer(file: FileNode(name: "Untitled \(buffers.values.filter{$0.isUntitledDraft}.count + 1)", path: path, dir: false), text: FileText(path: path, content: "", hash: ""),isUntitledDraft:true)
        showSource(path)
    }
    func chooseFileCommand(_ command: String) { filePath = ""; fileCommand = command }
    func performFileCommand() async {
        guard let command = fileCommand, let engine else { status = "Connect to a workspace server first."; return }
        let path = filePath; let source = currentBuffer; fileCommand = nil
        func find(_ nodes: [FileNode]) -> FileNode? {
            for node in nodes { if exactUTF8(node.path,path) { return node }; if let found = find(node.children ?? []) { return found } }; return nil
        }
        if command == "Save As" {
            guard let source, !source.saving, !path.isEmpty else { return }
            if exactUTF8(path,source.file.path),!source.resourceDeleted,!source.isUntitledDraft { await source.save(using: engine); return }
            let submitted = source.text
            source.saving = true; defer { source.saving = false }
            do {
                let created=try await createSourceCopy(source,path:path)
                showSource(created.file.path);await refresh()
                source.saving = false
                if savingAll { await saveAll() }
            } catch { status = "\(error.localizedDescription) Original edits are retained. Choose an unused path and retry Save As." }
        } else if command == "Open Directory" && path.isEmpty { await refresh(); showUtility(.files) }
        else if let file = find(files), file.dir == (command == "Open Directory") {
            if file.dir { files = file.children ?? []; showUtility(.files) } else { await open(file) }
        } else { status = "Path not found. Refresh and choose a path in this workspace." }
    }
    func saveAll() async {
        guard let engine else { return }
        savingAll = true
        for buffer in Array(buffers.values) where !buffer.isUntitledDraft { await buffer.save(using: engine) }
        if let draft = buffers.values.first(where: { $0.isUntitledDraft && $0.dirty }) {
            showSource(draft.file.path); chooseFileCommand("Save As")
        } else { savingAll = false }
    }
    func save() async {
        if currentBuffer?.isUntitledDraft == true || currentBuffer?.resourceDeleted == true { chooseFileCommand("Save As"); return }
        if let engine, let buffer = currentBuffer { await buffer.save(using: engine) }
    }
}
