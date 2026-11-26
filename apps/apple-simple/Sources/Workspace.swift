import SwiftUI
import Observation

struct StartupSourceObservation {
    var origin: URL
    var root: String
    var workspace: String?
}

@MainActor @Observable final class DocumentBuffer: Identifiable {
    let id = UUID()
    var gitComparison: GitEditorComparison?
    var gitComparisonChanges: (comparison: UUID, revision: UUID, rows: [Int: GitEditorComparison.Line]?)?
    var gitChanges: [Int: GitEditorComparison.Line]? {
        guard let result = gitComparisonChanges, let gitComparison, result.comparison == gitComparison.id,
              result.revision == sourceEditIdentity, exactUTF8(file.path, gitComparison.path) else { return nil }
        return result.rows
    }
    var debugInspection=UUID()
    var showSnippetTools=false
    @ObservationIgnored lazy var snippets=SourceSnippetController(buffer:self)
    var file: FileNode
    var startupObservation: StartupSourceObservation?
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
    var selection = NSRange(location: 0, length: 0) {didSet{if selection != oldValue{selectionIdentity=UUID()}}}
    @ObservationIgnored var receiptEngine: Engine?
    var receiptSource = "" { didSet { renderMarkup?() } }
    var terminalOutputRevision: UUID?
    var terminalOutput: [Int: String] = [:] { didSet { renderMarkup?() } }
    var runOutputSource = ""
    var runOutputLinks: [String: Int] = [:]
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
            receipts = page.blocks.filter { $0.kind == "exec" }; receiptRevision = UUID().uuidString; receiptSource = source
            receiptNotice = receipts.isEmpty ? "This document has no recorded execution blocks." : ""
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
    #if os(iOS)
    @ObservationIgnored var releaseSourceFocus: (() -> Void)?
    #endif
    func closeFind() { showFind = false; focusSource?() }
    var multiSource = ""
    var multiRanges: [NSRange] = [] {
        didSet {
            if multiRanges != oldValue { selectionIdentity = UUID() }
        }
    }
    var multiRects: [CGRect] = []
    #if os(macOS)
    var showMulti = false
    var multiText = ""
    #endif
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
    @ObservationIgnored var adoptSourceDraft: ((String, String, SourceReplacement) -> Bool)?
    @ObservationIgnored var reflectWorkspaceEdits: (([NativeSourceEdit], String, String) -> Bool)?
    @ObservationIgnored var applySourceEdits: (([NativeSourceEdit], String) -> Bool)?
    var foldedLines: Set<Int> = []
    @ObservationIgnored var toggleFold: ((Int) -> Void)?
    @ObservationIgnored var revealDebugLine: ((Int) -> Bool)?
    @ObservationIgnored var rememberNavigation: ((SemanticLocation, SemanticLocation) -> Void)?
    @ObservationIgnored var openSemanticLocation: ((SemanticLocation, @escaping () -> Bool) -> Void)?
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
    var readOnly: Bool { startupObservation != nil || (resourceDeleted && !unmaterializedHistoryDraft) || file.generated_by != nil }
    init(file: FileNode, text: FileText,isUntitledDraft:Bool=false, startupObservation: StartupSourceObservation? = nil) {
        self.isUntitledDraft=isUntitledDraft
        self.startupObservation = startupObservation
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
    @ObservationIgnored private var searchOpening = UUID()
    func openSearchHit(_ hit: WorkspaceSearchHit) async {
        guard let server = engine else { return }
        let prepared = beginSourceNavigationPreparation()
        let current = generation, request = search.revision, choice = UUID()
        searchOpening = choice
        let stillSearching = { self.generation == current && self.search.revision == request && self.searchOpening == choice }
        func find(_ nodes: [FileNode]) -> FileNode? { for node in nodes { if exactUTF8(node.path,hit.path) { return node }; if let child = find(node.children ?? []) { return child } }; return nil }
        guard let file = buffers[hit.path]?.file ?? find(rootFiles), !file.dir else { search.notice = "File no longer listed. Refresh files and search again."; return }
        do {
            let buffer = try await loadBuffer(file)
            let source = buffer.text
            let observedHash = await Task.detached { nativeSourceHash(source) }.value
            let rows = source.components(separatedBy: "\n")
            guard stillSearching(), prepared() else { return }
            guard let hash = hit.sourceHash, observedHash == hash.replacingOccurrences(of: "sha256:", with: "").lowercased(), let character = hit.character,
                  hit.line > 0, rows.indices.contains(hit.line - 1), rows[hit.line - 1].trimmingCharacters(in: .newlines) == hit.text.trimmingCharacters(in: .newlines) else { search.notice = "Search source changed or its revision is unknown. Compare the editor with disk and search again."; return }
            // The cached editor can still match a result after an external write.
            let saved: FileText = try await server.request("GET", "/api/file", query: [URLQueryItem(name: "path", value: hit.path)])
            guard stillSearching(), prepared() else { return }
            guard exactUTF8(saved.path, hit.path), exactUTF8(saved.content, source) else {
                search.notice = "The search source changed on disk. Search again before opening this result."
                return
            }
            guard buffers[hit.path] === buffer, exactUTF8(buffer.text, source) else {
                search.notice = "Search source changed in the editor. Compare it with disk and search again."
                return
            }
            await openSemanticLocation(SemanticLocation(uri: languageURI(hit.path), line: hit.line - 1, character: character),
                allowDirty: true, expectedSource: source, isCurrent: stillSearching, onRevealed: { showWorkspaceSearch = false })
        } catch { if stillSearching(), prepared() { search.notice = error.localizedDescription } }
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
    var showTests = false {
        didSet {
            if oldValue != showTests { testSourcesPresentation = UUID() }
            if !showTests { testSources.closeDebugPreparation(self) }
        }
    }
    @ObservationIgnored private(set) var testSourcesPresentation = UUID()
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
    var layout = PaneLayout.workspaceDefault()
    private var hiddenFilesPane: String?
    private var visibleDocumentPane: PaneNode? {
        layout.groups().first { group in
            group.active.flatMap { bufferForTab($0) } != nil
        }
    }
    var separateFilesPane: String? {
        guard visibleDocumentPane != nil else { return nil }
        return layout.groups().first { $0.tabs.containsUtility(.files) && $0.tabs.allSatisfy { $0 == "files" || $0 == "agent-history" } }?.id
    }
    var collapsedFilesPane: String? {
        guard hiddenFilesPane == separateFilesPane else { return nil }
        return hiddenFilesPane
    }
    func toggleFilesPane() {
        guard let files = separateFilesPane else { return }
        hiddenFilesPane = collapsedFilesPane == nil ? files : nil
        if hiddenFilesPane != nil, focused == files, let document = visibleDocumentPane {
            focused = document.id
        }
    }
    var focused = "document-group" {
        didSet { if oldValue != focused { paneInteraction = UUID() } }
    }
    @ObservationIgnored private var paneInteraction = UUID()
    var rootFiles: [FileNode] = []
    var files: [FileNode] = []
    private var displayedFolder: NativeSourceKey?
    private func treeEntry(_ path: String, in nodes: [FileNode], parent: FileNode? = nil) -> (file: FileNode, parent: FileNode?)? {
        for node in nodes {
            if exactUTF8(node.path, path) { return (node, parent) }
            if let children = node.children, let nested = treeEntry(path, in: children, parent: node) { return nested }
        }
        return nil
    }
    func workspaceFile(_ path: String) -> FileNode? { treeEntry(path, in: rootFiles)?.file }
    private func displayFolder(_ folder: FileNode?) {
        displayedFolder = folder?.id
        if let folder { files = folder.children ?? [] }
        else { files = rootFiles }
    }
    var buffers: NativeSourceMap<DocumentBuffer> = [:]
    let resourceRestoration=ResourceRestoration()
    var retainedResourceEditors:[String:DocumentBuffer]=[:]
    func retainedResourceTab(_ buffer:DocumentBuffer)->String {NativeTabIdentity.retained(buffer.id).address}
    func bufferForTab(_ tab:String)->DocumentBuffer? {
        switch NativeTabIdentity(address:tab){case .source(let key):return buffers[key.string];case .retained(let id):return retainedResourceEditors.values.first{$0.id==id};case .utility,.conversation,.unavailable:return nil}
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
        let targets=routes.map{route->(ResourceEditorRoute<DocumentBuffer>,DocumentBuffer,SourceReplacement?) in
            let entry=route.to.flatMap{files[Data($0.utf8)]}
            let buffer=route.editor ?? DocumentBuffer(file:entry!.file,text:FileText(path:entry!.file.path,content:route.after!,hash:entry!.hash))
            let edit = route.after.flatMap { reviewedSourceReplacement(buffer.text, $0) }
            return (route,buffer,edit)
        }
        for (route,buffer,edit) in targets {
            if let path=route.to,!patches.contains(where:{exactUTF8($0.path,path)&&$0.before != nil}),buffers[path] != nil {throw EngineError(message:"A destination editor appeared after review. Inspect recovery.")}
            let probe = NativeSourceEdit(range: NSRange(location: edit?.from ?? 0, length: edit.map { $0.to - $0.from } ?? 0), text: edit?.text ?? "")
            if !(buffer.resourceDeleted && edit == nil),buffer.applySourceEdits != nil,buffer.canReflectWorkspaceEdits?([probe],buffer.text) != true {throw EngineError(message:"Finish composition before adopting source editors. Inspect recovery.")}
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
        for (route,buffer,edit) in targets {
            guard let path=route.to,let entry=files[Data(path.utf8)] else{buffer.resourceDeleted=true;retainedResourceEditors[route.identity]=buffer;continue}
            let before=buffer.text
            if let edit, let after = route.after {
                if let adopt = buffer.adoptSourceDraft {
                    guard adopt(before, after, edit) else { throw EngineError(message:"Native source adoption failed. Preserve drafts and inspect recovery.") }
                } else {
                    buffer.selection = edit.reviewedSelection(buffer.selection)
                    buffer.text = after
                    let replacement = NativeSourceEdit(range: NSRange(location: edit.from, length: edit.to - edit.from), text: edit.text)
                    _ = buffer.reflectWorkspaceEdits?([replacement], before, after)
                }
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

    var status = ""
    var debugger: NativeDebugger?
    var engine: Engine?
    var address = ProcessInfo.processInfo.environment["HICKORY_API_ORIGIN"] ?? ""
    var connecting = false
    private var loading: NativeSourceMap<Task<DocumentBuffer, Error>> = [:]
    private var languageConnection: NativeLanguageConnection?
    private var newestOpen: [String: UUID] = [:]
    let agent = AgentSession()
    var conversations: [String: AgentSession] = [:]
    var allAgentSessions: [AgentSession] { [agent] + Array(conversations.values) }
    func agentSession(_ tab: String?) -> AgentSession? {
        guard let tab else { return nil }
        return tab == "agent" ? agent : conversations[tab]
    }
    func openConversation(_ tip: String? = nil, fork: Bool? = nil) {
        let session = AgentSession()
        session.turns = agent.turns; session.messageForks = agent.messageForks
        session.selection = .init(id: tip, userFork: fork)
        session.provider = agent.provider; session.model = agent.model; session.effort = agent.effort
        session.captureContext = { [weak self] in
            guard let self else { throw EngineError(message: "Original agent workspace closed.") }
            return try nativeAgentContext(self)
        }
        let address = NativeTabIdentity.conversation(UUID()).address
        conversations[address] = session
        show(address)
    }
    func configureAgentContext(){agent.captureContext={ [weak self] in guard let self else{throw EngineError(message:"Original agent workspace closed.")};return try nativeAgentContext(self)}}
    let terminal = TerminalSession()
    private var generation = UUID()
    var sourceNavigationEpoch:UUID {generation}
    var activeTab:String? {layout.nodes[focused]?.active}
    var active: String? {activeTab.map{NativeTabIdentity(address:$0).sourcePath ?? $0}}
    var currentBuffer: DocumentBuffer? {activeTab.flatMap{bufferForTab($0)}}
    var hasUnsavedChanges: Bool { gitResolution.hasPending || gitResolution.hasDraft || resourceRestoration.busy || resourceRestoration.hasPending || allAgentSessions.contains { !$0.prompt.isEmpty || $0.running } || (Array(buffers.values)+Array(retainedResourceEditors.values)).contains { $0.dirty || $0.saving } || debugger?.active == true }
    private func startRequestedReading(_ path: String?, engine next: Engine,
                                       isCurrent: @escaping () -> Bool) -> Task<Void, Never>? {
        guard let path, !path.isEmpty, engine == nil, buffers.isEmpty else { return nil }
        return Task { [weak self] in
            guard let reply = try? await next.startupSource(path), !Task.isCancelled,
                  let self, self.connecting, isCurrent(), self.buffers.isEmpty,
                  self.engine == nil || self.engine?.origin == next.origin else { return }
            if self.engine != nil {
                guard exactUTF8(self.sourceRootAddress, reply.root_path),
                      reply.workspace_id.map({ exactUTF8(self.sourceWorkspaceID, $0) }) ?? true else { return }
            }
            let file = FileNode(name: (path as NSString).lastPathComponent, path: path, dir: false)
            let reading = StartupSourceObservation(origin: next.origin, root: reply.root_path, workspace: reply.workspace_id)
            self.buffers[path] = DocumentBuffer(file: file, text: FileText(path: path, content: reply.content, hash: ""), startupObservation: reading)
            self.showSource(path)
            self.status = "\(path) · waiting for workspace permissions"
        }
    }
    private func bindRequestedReading(_ buffer: DocumentBuffer, file: FileNode) async {
        guard buffer.startupObservation != nil, let engine else { return }
        let scope = generation
        let before = buffer.text, root = sourceRootAddress, workspace = sourceWorkspaceID
        let current = {
            self.generation == scope && self.engine?.origin == engine.origin
                && exactUTF8(self.sourceRootAddress, root) && exactUTF8(self.sourceWorkspaceID, workspace)
                && self.buffers[file.path] === buffer && buffer.startupObservation != nil && exactUTF8(buffer.text, before)
        }
        do {
            let text = try await engine.file(file)
            if current() { acceptRequestedReading(text, in: buffer, file: file, engine: engine) }
        } catch {
            if current() && currentBuffer === buffer {
                status = "\(error.localizedDescription) The file remains readable. Open it again in Files to retry permissions."
            }
        }
    }
    private func acceptRequestedReading(_ text: FileText, in buffer: DocumentBuffer, file: FileNode,
                                        engine: Engine) {
        guard let reading = buffer.startupObservation else { return }
        func notice(_ message: String) { if currentBuffer === buffer { status = message } }
        let root = sourceRootAddress, workspace = sourceWorkspaceID
        guard engine.origin == reading.origin, exactUTF8(root, reading.root),
              reading.workspace.map({ exactUTF8($0, workspace) }) ?? true else {
            notice("Workspace identity changed while opening \(file.path). Reconnect to read its current source.")
            return
        }
        guard exactUTF8(text.path, file.path), !buffer.dirty else {
            notice("Source identity changed while opening. Open the exact file in Files.")
            return
        }
        if let edit = reviewedSourceReplacement(buffer.text, text.content) {
            buffer.selection = edit.reviewedSelection(buffer.sourceEditorState?()?.selection ?? buffer.selection)
            buffer.base = text.content
            buffer.text = text.content
        }
        guard file.generated_by != nil || text.save_guard?.isEmpty == false else {
            let reason = text.save_guard_reason ?? "Source permission is unconfirmed. Refresh Files to retry."
            buffer.saveGuardReason = reason
            notice(reason)
            return
        }
        buffer.file = file
        buffer.hash = text.hash
        buffer.savedGuard = text.save_guard
        buffer.saveGuardReason = text.save_guard_reason
        buffer.startupObservation = nil
        _ = retainBuffer(file, text: text, engine: engine, existing: buffer)
        // The first text frame may have mounted before these readers were attached.
        // Confirming the same bytes does not emit another source revision.
        buffer.readTestSources()
        buffer.readCoverageSource()
        notice(file.path)
    }
    func connect(opening explicitPath: String? = nil) async -> Bool {
        guard !connecting else { return false }
        guard !hasUnsavedChanges else { status = "Save documents and send or clear draft messages before changing workspace servers."; return false }
        connecting = true; defer { connecting = false }
        do {
            let next = try Engine(address: address)
            let heldReading = engine == nil ? buffers.values.first(where: { $0.startupObservation?.origin == next.origin }) : nil
            let requestedPath = explicitPath ?? heldReading?.file.path
            var requestedScope = generation, requestedInteraction = paneInteraction
            let startupIsCurrent = { self.generation == requestedScope && self.paneInteraction == requestedInteraction }
            let startup = startRequestedReading(requestedPath, engine: next, isCurrent: startupIsCurrent)
            defer { startup?.cancel() }
            let tree: FileTree
            do { tree = try await next.files() }
            catch {
                await startup?.value
                if let requestedPath, buffers[requestedPath]?.startupObservation?.origin == next.origin {
                    status = "\(error.localizedDescription) \(requestedPath) remains readable. Use Workspace Connection to retry the file tree."
                    return true
                }
                throw error
            }
            let requestedTabIsCurrent = startupIsCurrent()
            let preview = requestedPath.flatMap { buffers[$0] }.flatMap { $0.startupObservation?.origin == next.origin ? $0 : nil }
            // The terminal renderer owns screen contents; reconnect by clearing the session ID.
            for session in conversations.values { session.resetHistory() }; conversations.removeAll()
            agent.resetHistory(); agent.totals = nil; agent.activity = [:]; agent.stopPending = false; agent.error = ""; terminal.id = nil; terminal.error = ""
            debugger?.close()
            for buffer in buffers.values { buffer.language?.close() }
            languageConnection?.close();languageConnection=nil
            sourceHistory.close();sourceRootAddress=nil;sourceWorkspaceID=nil;sourceTemplates.close();analysis.stop(); analysis=WorkspaceAnalysis(); showAnalysis=false; showEverywhere=false;showTests=false;showReview=false;showHistory=false;showAttribution=false;showConstruction=false;showRecovery=false;gitAttribution.close();gitAttribution=GitAttributionModel(path:"");gitHistory.close();gitHistory=GitHistoryModel();gitReview.close();gitReview=GitReviewModel();gitResolution.reset();showCoverage=false;coverage.stop();coverage=SimpleCoverageState();testSources=TestSourcesModel();sourceTestIndex?.stop();sourceTestIndex=nil
            generation = UUID(); changes.reset(); search.invalidate(); showWorkspaceSearch = false; engine = next; debugger = NativeDebugger(engine: next)
            if let debug=debugger {debug.sourceBuffers={ [weak self] in guard let self else{return []};return Array(self.buffers.values)+Array(self.retainedResourceEditors.values)};debug.onLocation={ [weak self,weak debug] location in guard let self,let debug else{return};let scope=self.generation;Task {[weak self,weak debug] in guard let self,let debug,self.generation==scope else{return};await self.revealDebug(location,debug:debug)}}}; sourceRootAddress=tree.root_path;sourceWorkspaceID=tree.workspace_id;rootFiles = tree.tree; navigationTrail = NavigationTrail(); retainedResourceEditors = [:]; loading.values.forEach { $0.cancel() }; loading = [:]
            if preview == nil { buffers = [:]; layout = PaneLayout.workspaceDefault(); focused = "document-group" }
            if requestedTabIsCurrent { requestedScope = generation; requestedInteraction = paneInteraction }
            status = tree.root
            displayFolder(nil)
            if let requestedPath, !requestedPath.isEmpty {
                if let entry = treeEntry(requestedPath, in: rootFiles), !entry.file.dir {
                    displayFolder(entry.parent)
                    if let preview { await bindRequestedReading(preview, file: entry.file) }
                    else if startupIsCurrent() {
                        let opened = await activateFile(entry.file, recordNavigation: false, isCurrent: startupIsCurrent, forStartup: true)
                        if !opened && startupIsCurrent() { await startup?.value }
                    }
                } else {
                    status = "Requested file not found: \(requestedPath). Choose a file from the workspace tree."
                }
            }
            return true
        } catch { status = error.localizedDescription; return false }
    }
    @ObservationIgnored private var savedSourceRead = UUID()
    func refreshCurrentSource() async {
        guard let engine, let root = sourceRootAddress, let workspace = sourceWorkspaceID,
              let buffer = currentBuffer, buffer.startupObservation == nil,
              !buffer.saving, !buffer.resourceDeleted, !buffer.isUntitledDraft, buffer.gitComparison == nil,
              let file = treeEntry(buffer.file.path, in: rootFiles)?.file, !file.dir,
              file.doc_id == buffer.file.doc_id, file.generated_by == buffer.file.generated_by else { return }
        let request = UUID(), scope = generation, revision = buffer.sourceEditIdentity
        let base = buffer.base, guardToken = buffer.savedGuard
        savedSourceRead = request
        func current() -> Bool {
            !Task.isCancelled && generation == scope && savedSourceRead == request
                && exactUTF8(sourceRootAddress, root) && exactUTF8(sourceWorkspaceID, workspace)
                && currentBuffer === buffer && buffers[file.path] === buffer
                && buffer.sourceEditIdentity == revision && exactUTF8(buffer.base, base)
                && exactUTF8(buffer.savedGuard, guardToken) && !buffer.saving && !buffer.resourceDeleted
        }
        do {
            let saved = try await engine.file(file)
            guard current(), exactUTF8(saved.path, file.path) else { return }
            if buffer.dirty {
                if !exactUTF8(saved.content, base) || !exactUTF8(saved.save_guard, guardToken) {
                    status = "\(file.path) changed on disk. Your unsaved draft is still open."
                }
                return
            }
            if let change = reviewedSourceReplacement(base, saved.content) {
                if let state = buffer.sourceEditorState?(), state.composing || !exactUTF8(state.source, base) { return }
                if let adopt = buffer.adoptSourceDraft, !buffer.readOnly {
                    guard adopt(base, saved.content, change) else { return }
                } else {
                    buffer.selection = change.reviewedSelection(buffer.sourceEditorState?()?.selection ?? buffer.selection)
                    buffer.text = saved.content
                }
                status = "Reloaded \(file.path) from disk."
            }
            buffer.base = saved.content
            buffer.hash = saved.hash
            buffer.savedGuard = saved.save_guard
            buffer.saveGuardReason = saved.save_guard_reason
        } catch {
            if current() { status = "Could not check \(file.path) on disk. \(error.localizedDescription)" }
        }
    }
    func refresh() async {
        guard let engine else { return }
        let current = generation
        do {
            let tree = try await engine.files()
            guard current == generation else { return }
            sourceRootAddress = tree.root_path
            sourceWorkspaceID = tree.workspace_id
            rootFiles = tree.tree
            let folder = displayedFolder.flatMap { treeEntry($0.string, in: rootFiles)?.file }
            displayFolder(folder?.dir == true ? folder : nil)
            status = tree.root
            if !connecting, let buffer = buffers.values.first(where: { $0.startupObservation != nil }),
               let entry = treeEntry(buffer.file.path, in: rootFiles), !entry.file.dir {
                await bindRequestedReading(buffer, file: entry.file)
            } else {
                await refreshCurrentSource()
            }
        }
        catch { if current == generation { status = error.localizedDescription } }
    }
    func loadBuffer(_ file: FileNode, forStartup: Bool = false) async throws -> DocumentBuffer {
        if let buffer = buffers[file.path] { return buffer }
        if let pending = loading[file.path] { return try await pending.value }
        guard !file.dir, let engine else { throw EngineError(message:"Connect to the workspace and choose a file.") }
        let current = generation
        let root = sourceRootAddress, workspace = sourceWorkspaceID
        let pending = Task { @MainActor [weak self] () async throws -> DocumentBuffer in
            let text = try await engine.file(file)
            guard exactUTF8(text.path,file.path) else{throw EngineError(message:"File reply names a different source spelling. Its text was not attached to another editor; reopen the exact file.")}
            guard let self, self.generation == current else { throw CancellationError() }
            if forStartup {
                guard exactUTF8(self.sourceRootAddress, root), exactUTF8(self.sourceWorkspaceID, workspace) else { throw CancellationError() }
            }
            if let available = self.buffers[file.path] {
                self.acceptRequestedReading(text, in: available, file: file, engine: engine)
                return available
            }
            if forStartup, file.generated_by == nil, text.save_guard?.isEmpty != false, let root = self.sourceRootAddress {
                let reading = StartupSourceObservation(origin: engine.origin, root: root, workspace: self.sourceWorkspaceID)
                let buffer = DocumentBuffer(file: file, text: text, startupObservation: reading)
                buffer.saveGuardReason = text.save_guard_reason ?? "Source permission is unconfirmed. Refresh Files to retry."
                self.buffers[file.path] = buffer
                return buffer
            }
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
            guard let self, let buffer, self.buffers[buffer.file.path] === buffer else { return }
            guard let target = workspaceLinkTarget(href, relativeTo: buffer.file.path) else {
                self.status = "This link must point to a file in the workspace."
                return
            }
            let path = target.path, scope = self.generation, revision = buffer.sourceEditIdentity
            Task {
                if self.treeEntry(path, in: self.rootFiles) == nil { await self.refresh() }
                @MainActor func current() -> Bool {
                    self.generation == scope && self.buffers[buffer.file.path] === buffer && buffer.sourceEditIdentity == revision
                }
                guard current() else { return }
                guard let file = self.treeEntry(path, in: self.rootFiles)?.file, !file.dir else {
                    self.status = "File not found: \(path). Refresh Files and check the link."
                    return
                }
                if let line = target.line {
                    await self.openSemanticLocation(SemanticLocation(uri: nativeDebugURI(path), line: line, character: 0), allowDirty: true, isCurrent: current)
                } else { _ = await self.activateFile(file, recordNavigation: true, isCurrent: current) }
            }
        }
        buffer.rememberNavigation = { [weak self] from, to in self?.navigationTrail.record(from: from, to: to) }
        buffer.openSemanticLocation = { [weak self, weak buffer] location, current in
            Task {
                guard let self, let buffer else { return }
                await self.openSemanticLocation(location, isCurrent: {
                    self.buffers[buffer.file.path] === buffer && current()
                })
            }
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
    // A read can populate the buffer cache after its pane selects another tab.
    // Only the pane's current open request can activate the newly read buffer.
    func openGitComparison(_ comparison: GitEditorComparison, current: @escaping () -> Bool) async -> Bool {
        guard let file = workspaceFile(comparison.path), !file.dir,
              await activateFile(file, recordNavigation: true, isCurrent: current), current(),
              let buffer = currentBuffer, exactUTF8(buffer.file.path, comparison.path) else { return false }
        buffer.gitComparison = comparison
        return true
    }

    func open(_ file: FileNode, recordNavigation: Bool = true) async {
        _ = await activateFile(file, recordNavigation: recordNavigation)
    }
    private func activateFile(_ file: FileNode, recordNavigation: Bool, isCurrent: (() -> Bool)? = nil,
                              forStartup: Bool = false) async -> Bool {
        guard isCurrent?() != false else { return false }
        let departed = currentSemanticLocation()
        if let existing = buffers[file.path] {
            showSource(file.path)
            let interaction = paneInteraction
            if recordNavigation, let arrived = currentSemanticLocation() { navigationTrail.record(from:departed,to:arrived) }
            if existing.startupObservation != nil { await bindRequestedReading(existing, file: file) }
            return isCurrent?() != false && paneInteraction == interaction && currentBuffer === existing
        }
        let current = generation
        let target = layout.groups().first(where: { $0.id == focused && !$0.tabs.containsUtility(.files) })?.id
            ?? layout.groups().first(where: { !$0.tabs.containsUtility(.files) })?.id ?? focused
        let request = UUID(), startingInteraction = paneInteraction
        newestOpen[target] = request
        if isCurrent == nil {
            paneInteraction = UUID()
            focused = target
        }
        func ownsPane() -> Bool {
            generation == current && newestOpen[target] == request && layout.groups().contains { $0.id == target }
                && (isCurrent == nil || (paneInteraction == startingInteraction && isCurrent?() == true))
        }
        defer { if newestOpen[target] == request { newestOpen.removeValue(forKey: target) } }
        do {
            let buffer = try await loadBuffer(file, forStartup: forStartup)
            guard ownsPane() else { return false }
            if isCurrent != nil { focused = target }
            if focused == target { paneInteraction = UUID() }
            layout.activateSource(file.path, in: target)
            if focused == target {
                status = buffer.startupObservation == nil ? file.path : (buffer.saveGuardReason ?? "Waiting for workspace permissions.")
                if recordNavigation, let arrived = currentSemanticLocation() { navigationTrail.record(from:departed,to:arrived) }
            }
            return focused == target
        } catch is CancellationError { return false }
        catch {
            if ownsPane(), isCurrent != nil || focused == target { status = "\(error.localizedDescription) Refresh files and try opening \(file.path) again." }
            return false
        }
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
    @ObservationIgnored private var semanticReveal = UUID()
    @ObservationIgnored private var sourcePreparation = UUID()
    func beginSourceNavigationPreparation() -> () -> Bool {
        let choice = UUID(), scope = generation, interaction = paneInteraction
        let origin = currentBuffer, address = activeTab
        let owner = origin?.id, revision = origin?.sourceEditIdentity, selection = origin?.selection
        let viewport = origin?.sourceEditorState?()?.attached == true ? origin?.captureRecoveryViewport?() : nil
        sourcePreparation = choice
        semanticReveal = UUID()
        return { [weak self] in
            guard let self, !Task.isCancelled, self.generation == scope,
                  self.sourcePreparation == choice, self.paneInteraction == interaction else { return false }
            let visible = self.currentBuffer
            return exactUTF8(self.activeTab, address) && visible?.id == owner
                && visible?.sourceEditIdentity == revision && visible?.selection == selection
                && (viewport == nil || visible?.captureRecoveryViewport?() == viewport)
        }
    }
    private func matchesReportedIdentifier(_ name: String, in source: String, at offset: Int) -> Bool {
        func continuesIdentifier(_ scalar: Unicode.Scalar) -> Bool {
            scalar.properties.isIDContinue || scalar == "_" || scalar == "$"
        }
        let nameScalars = name.unicodeScalars
        guard let first = nameScalars.first,
              first.properties.isIDStart || first == "_" || first == "$",
              nameScalars.dropFirst().allSatisfy(continuesIdentifier) else { return true }
        let units = source.utf16, scalars = source.unicodeScalars
        guard let startUnit = units.index(units.startIndex, offsetBy: offset, limitedBy: units.endIndex),
              let endUnit = units.index(startUnit, offsetBy: name.utf16.count, limitedBy: units.endIndex),
              let start = startUnit.samePosition(in: scalars), let end = endUnit.samePosition(in: scalars),
              scalars[start..<end].elementsEqual(nameScalars) else { return false }
        let startsIdentifier = start == scalars.startIndex || !continuesIdentifier(scalars[scalars.index(before: start)])
        let endsIdentifier = end == scalars.endIndex || !continuesIdentifier(scalars[end])
        return startsIdentifier && endsIdentifier
    }
    func openSemanticLocation(_ location: SemanticLocation, recordHistory: Bool = true, allowDirty: Bool = false, expectedSource: String? = nil, isCurrent: @escaping () -> Bool = {true}, onRevealed: () -> Void = {}) async {
        guard isCurrent(), !Task.isCancelled else { return }
        sourcePreparation = UUID()
        let departed = currentSemanticLocation()
        guard let path = location.path else { return }
        func find(_ nodes: [FileNode]) -> FileNode? {
            for node in nodes { if exactUTF8(node.path,path) { return node }; if let child = find(node.children ?? []) { return child } }; return nil
        }
        guard let file = buffers[path]?.file ?? find(rootFiles), !file.dir else { status = "Location not found: \(path). Refresh files and query again."; return }
        let current = generation, ticket = UUID()
        semanticReveal = ticket
        func ownsRequest() -> Bool {
            isCurrent() && !Task.isCancelled && generation == current && semanticReveal == ticket
        }
        guard await activateFile(file, recordNavigation: false, isCurrent: ownsRequest),
              ownsRequest(), exactUTF8(active,path), let buffer = buffers[path] else { return }
        if buffer.dirty && !allowDirty && departed?.path != path { status = "Save \(path) and query again before using a cross-file source position."; return }
        let source = buffer.text
        if let expectedSource, !exactUTF8(source,expectedSource) { status = "Source changed. Refresh the query before navigating."; return }
        guard let offset = languageOffset(source, ["line": location.line, "character": location.character]) else { status = "Location no longer fits \(path). Query again."; return }
        if recordHistory, expectedSource == nil, !exactUTF8(departed?.path, path), let name = location.name,
           !matchesReportedIdentifier(name, in: source, at: offset) {
            status = "Opened \(path). The reported symbol no longer matches. Query again."
            return
        }
        let interaction = paneInteraction, selection = buffer.selection, revision = buffer.sourceEditIdentity
        let viewport = buffer.sourceEditorState?()?.attached == true ? buffer.captureRecoveryViewport?() : nil
        func ownsEditor() -> Bool {
            ownsRequest() && paneInteraction == interaction && currentBuffer === buffer && buffers[path] === buffer
                && buffer.sourceEditIdentity == revision && exactUTF8(buffer.text, source) && buffer.selection == selection
                && (viewport == nil || buffer.captureRecoveryViewport?() == viewport)
                && (!buffer.dirty || allowDirty || exactUTF8(departed?.path, path))
        }
        for _ in 0..<20 {
            guard ownsEditor() else { return }
            if buffer.selectSourceRange?(NSRange(location: offset, length: 0), source) == true { if recordHistory { navigationTrail.record(from: departed, to: location) }; buffer.focusSource?(); onRevealed(); return }
            try? await Task.sleep(for: .milliseconds(25))
        }
        if ownsEditor() { status = "Opened \(path); editor could not reveal the position. Query again once it is ready." }
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
    func showUtility(_ utility:NativeUtilityTab){if utility == .agent {openConversation()} else {show(utility.rawValue)}}
    func show(_ tab: String, in group: String? = nil) {
        let history = tab == NativeUtilityTab.agentHistory.rawValue
        if tab == NativeUtilityTab.files.rawValue || history { hiddenFilesPane = nil }
        let groups = layout.groups()
        let target = group ?? groups.first(where: { $0.tabs.containsTab(tab) })?.id
            ?? (history ? groups.first(where: { $0.tabs.containsUtility(.files) })?.id ?? groups.first?.id : nil)
            ?? (tab != "files" ? groups.first(where: { $0.id == focused && !$0.tabs.containsUtility(.files) && !$0.tabs.containsUtility(.agentHistory) })?.id
                ?? groups.first(where: { !$0.tabs.containsUtility(.files) && !$0.tabs.containsUtility(.agentHistory) })?.id : nil) ?? focused
        guard groups.contains(where: { $0.id == target }) else { return }
        newestOpen.removeValue(forKey: target)
        paneInteraction = UUID()
        layout.activate(tab, in: target); focused = target
    }
    private func adoptTabLayout(_ next: PaneLayout, focus group: String? = nil) {
        let groups = next.groups()
        let retained = Set(groups.map(\.id))
        newestOpen = newestOpen.filter { id, _ in
            id != group && retained.contains(id) && exactUTF8(next.nodes[id]?.active, layout.nodes[id]?.active)
        }
        if group != nil || !exactUTF8(activeTab, next.nodes[focused]?.active) { paneInteraction = UUID() }
        layout = next
        focused = group ?? (retained.contains(focused) ? focused : groups[0].id)
    }
    func moveTab(_ tab: String, to group: String, before: String? = nil) {
        var next = layout
        next.move(tab, to: group, before: before)
        guard next.nodes != layout.nodes else { return }
        adoptTabLayout(next, focus: group)
    }
    @discardableResult func dockTab(_ tab: String, to group: String, edge: String) -> Bool {
        var next = layout
        guard let retained = next.dock(tab, to: group, edge: edge, newID: UUID().uuidString) else { return false }
        adoptTabLayout(next, focus: retained)
        return true
    }
    struct TabClosePrompt {
        let tab: String
        let bufferID: UUID
        let revision: UUID
        let name: String
        let saving: Bool
        let needsPath: Bool
    }
    var tabClosePrompt: TabClosePrompt?
    private var closeAfterSaveAs: TabClosePrompt?

    private func hasPendingNativeText(_ buffer: DocumentBuffer) -> Bool {
        guard let editor = buffer.sourceEditorState?(), editor.attached else { return false }
        return editor.composing || !exactUTF8(editor.source, buffer.text)
    }

    func close(_ tab: String) {
        if let session = agentSession(tab), !session.prompt.isEmpty || session.sending || session.canStop {
            status = session.canStop ? "Stop this request or wait for it to finish before closing its tab." : "Send or clear this conversation's draft before closing its tab."
            session.error = status
            return
        }
        if let buffer = bufferForTab(tab), buffer.saving || buffer.dirty || hasPendingNativeText(buffer) {
            tabClosePrompt = TabClosePrompt(tab: tab, bufferID: buffer.id, revision: buffer.sourceEditIdentity,
                                            name: buffer.file.name, saving: buffer.saving,
                                            needsPath: buffer.isUntitledDraft || buffer.resourceDeleted)
            return
        }
        removeTab(tab)
        conversations.removeValue(forKey: tab)
    }

    private func closeTarget(_ prompt: TabClosePrompt) -> DocumentBuffer? {
        guard layout.groups().contains(where: { $0.tabs.containsTab(prompt.tab) }),
              let buffer = bufferForTab(prompt.tab), buffer.id == prompt.bufferID,
              buffer.sourceEditIdentity == prompt.revision, !buffer.saving else {
            status = "The tab or its edits changed. Review the tab and try closing again."
            return nil
        }
        return buffer
    }

    func discardAndClose(_ prompt: TabClosePrompt) {
        guard !prompt.saving, let buffer = closeTarget(prompt) else { return }
        removeTab(prompt.tab)
        buffer.language?.close()
        buffer.language = nil
        if buffers[buffer.file.path] === buffer { buffers.removeValue(forKey: buffer.file.path) }
        retainedResourceEditors = retainedResourceEditors.filter { $0.value !== buffer }
    }

    func saveAndClose(_ prompt: TabClosePrompt) async {
        guard !prompt.saving, let buffer = closeTarget(prompt) else { return }
        guard let engine else {
            status = "Connect to a workspace server to save \(prompt.name). The tab is still open."
            return
        }
        guard !hasPendingNativeText(buffer) else {
            status = "Finish editing \(prompt.name) before saving. The tab is still open."
            return
        }
        if buffer.isUntitledDraft || buffer.resourceDeleted {
            chooseFileCommand("Save As")
            closeAfterSaveAs = prompt
            return
        }
        await buffer.save(using: engine)
        guard closeTarget(prompt) === buffer, !buffer.dirty,
              !hasPendingNativeText(buffer), buffer.error.isEmpty else {
            if !buffer.error.isEmpty { status = buffer.error }
            else { status = "Review \(prompt.name) before closing. The tab is still open." }
            return
        }
        removeTab(prompt.tab)
    }

    private func removeTab(_ tab: String) {
        if let buffer = bufferForTab(tab) {
            buffer.retireAgentContext()
            sourceHistory.sourceClosed(buffer)
            sourceTemplates.sourceClosed(buffer)
            testSources.sourceClosed(buffer, workspace: self)
        }
        var next = layout
        next.remove(tab)
        adoptTabLayout(next)
    }
    func split(_ axis: String) {
        var next = layout
        if let retained = next.split(focused, axis: axis, newID: UUID().uuidString) {
            adoptTabLayout(next, focus: retained)
        }
    }
    func moveToNext() {
        let groups = layout.groups()
        guard let active=activeTab, let index = groups.firstIndex(where: { $0.id == focused }) else { return }
        let target = groups[(index + 1) % groups.count].id
        moveTab(active, to: target)
    }
    func tabIcon(_ tab: String) -> String {
        if agentSession(tab) != nil { return "sparkles" }
        switch NativeTabIdentity(address: tab).utility {
        case .files: return "folder"
        case .agentHistory: return "clock.arrow.circlepath"
        case .terminal: return "terminal"
        default: return "doc.text"
        }
    }
    func label(_ tab: String) -> String {
        if let session = agentSession(tab) { return session.title }
        switch NativeTabIdentity(address:tab).utility{case .files:return "Files";case .agent:return "Agent";case .agentHistory:return "Agent History";case .terminal:return "Terminal";case nil:break}
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
    func chooseFileCommand(_ command: String) {
        closeAfterSaveAs = nil
        filePath = ""
        fileCommand = command
    }
    func cancelFileCommand() {
        closeAfterSaveAs = nil
        fileCommand = nil
        savingAll = false
    }
    func performFileCommand() async {
        guard let command = fileCommand, let engine else { status = "Connect to a workspace server first."; return }
        let path = filePath
        let closeRequest = closeAfterSaveAs
        closeAfterSaveAs = nil
        fileCommand = nil
        if command == "Save As" {
            let source = if let closeRequest { closeTarget(closeRequest) } else { currentBuffer }
            guard let source, !source.saving, !path.isEmpty else { return }
            if closeRequest == nil, exactUTF8(path,source.file.path),!source.resourceDeleted,!source.isUntitledDraft { await source.save(using: engine); return }
            source.saving = true; defer { source.saving = false }
            do {
                let created=try await createSourceCopy(source,path:path)
                if closeRequest != nil {
                    if !created.dirty, !hasPendingNativeText(created) {
                        removeTab(nativeSourceTab(created.file.path))
                    } else {
                        status = "Review \(created.file.name) before closing. The tab is still open."
                    }
                } else {
                    showSource(created.file.path)
                }
                await refresh()
                source.saving = false
                if savingAll { await saveAll() }
            } catch { status = "\(error.localizedDescription) Original edits are retained. Choose an unused path and retry Save As." }
        } else if command == "Open Directory" && path.isEmpty {
            displayFolder(nil); await refresh(); showUtility(.files)
        } else if let entry = treeEntry(path, in: rootFiles), entry.file.dir == (command == "Open Directory") {
            if entry.file.dir { displayFolder(entry.file); showUtility(.files) }
            else {
                if currentBuffer == nil { displayFolder(entry.parent) }
                await open(entry.file)
            }
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
