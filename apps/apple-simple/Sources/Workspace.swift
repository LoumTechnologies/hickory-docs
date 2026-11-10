import SwiftUI
import Observation

@MainActor @Observable final class DocumentBuffer: Identifiable {
    let id = UUID()
    let file: FileNode
    var text: String {
        didSet { let changed = text != base; if dirty != changed { dirty = changed }; if text != oldValue { language?.changed(); refreshInline() } }
    }
    @ObservationIgnored var base: String {
        didSet { let changed = text != base; if dirty != changed { dirty = changed } }
    }
    @ObservationIgnored var navigate: ((Double) -> Void)?
    @ObservationIgnored var renderLanguage: (([NativeSemanticToken], String) -> Void)?
    @ObservationIgnored var applyLanguageCompletion: ((NSRange, String, String, NSRange) -> Bool)?
    @ObservationIgnored var selection = NSRange(location: 0, length: 0)
    @ObservationIgnored var receiptEngine: Engine?
    var receiptSource = ""
    var receipts: [InlineReceipt] = []
    var receiptNotice = ""
    func refreshReceipts() async {
        guard let engine = receiptEngine, let id = file.doc_id else { return }
        guard !dirty else { receiptNotice = "Save current source to view matching recorded output."; return }
        let source = text; receiptNotice = "Loading recorded output…"
        do {
            let page: InlineReceiptPage = try await engine.request("GET", "/api/docs/" + id + "/render")
            guard text == source, !dirty, page.document == id, page.source_hash.replacingOccurrences(of: "sha256:", with: "") == nativeSourceHash(source) else {
                receiptNotice = "Recorded output belongs to another source revision. Save or refresh before retrying."; return
            }
            receipts = page.blocks.filter { $0.kind == "exec" }; receiptSource = source; receiptNotice = ""
        } catch { if text == source { receiptNotice = "Recorded output unavailable. Retry after checking the server." } }
    }
    func receipts(on line: Int) -> [InlineReceipt] {
        guard receiptSource == text, !dirty else { return [] }
        let bytes = Array(text.utf8)
        return receipts.filter { $0.span.count == 2 && $0.span[0] >= 0 && $0.span[1] >= $0.span[0] && $0.span[1] <= bytes.count && bytes.prefix($0.span[0]).filter { $0 == 10 }.count == line }
    }
    // A source reveal changes presentation only; the editor and its undo stack stay alive.
    @ObservationIgnored var applyTableEdit: ((NativeSourceEdit, String) -> Bool)?
    var sourceTable: Int? { didSet { renderMarkup?() } }
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
            guard !Task.isCancelled, let self, self.text == source else { return }
            self.inlineMarkup = result; self.renderMarkup?()
        }
    }
    var visibleDebugLines: [DebugLine] = []
    @ObservationIgnored var focusSource: (() -> Void)?
    func closeFind() { showFind = false; focusSource?() }
    var multiSource = ""
    var multiRanges: [NSRange] = []
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
        let active = multiSource == text ? multiRanges : []
        let anchor = (direction > 0 ? active.last : active.first)?.location ?? selection.location
        let pos = languagePosition(text, anchor), rows = text.components(separatedBy: "\n")
        let row = pos["line"]! + direction
        guard rows.indices.contains(row), let offset = languageOffset(text, ["line": row, "character": min(pos["character"]!, rows[row].utf16.count)]) else { return }
        _ = selectMultiple((multiSource == text && !multiRanges.isEmpty ? multiRanges : [selection]) + [NSRange(location: offset, length: 0)], source: text)
    }
    func clearMulti() { multiSource = ""; multiRanges = []; multiRects = [] }
    func selectMultiple(_ ranges: [NSRange], source: String) -> Bool {
        let sorted = Array(Set(ranges)).sorted { $0.location < $1.location }
        guard text == source, sorted.count > 1, sorted.count <= 256,
              validSourceEdits(sorted.map { NativeSourceEdit(range: $0, text: "") }, length: source.utf16.count) else { return false }
        applyingMulti = true; defer { applyingMulti = false }
        guard selectSourceRange?(sorted[0], source) == true else { return false }
        multiSource = source; multiRanges = sorted; return true
    }
    func replaceMultiple(_ replacement: String, affected: NSRange? = nil) -> Bool {
        guard !readOnly, multiSource == text, multiRanges.count > 1 else { return false }
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
    @ObservationIgnored var applySourceEdits: (([NativeSourceEdit], String) -> Bool)?
    var foldedLines: Set<Int> = []
    @ObservationIgnored var toggleFold: ((Int) -> Void)?
    @ObservationIgnored var revealDebugLine: ((Int) -> Void)?
    var language: NativeLanguage?
    var hash: String
    var saving = false
    var error = ""
    private(set) var dirty = false
    var readOnly: Bool { file.generated_by != nil }
    init(file: FileNode, text: FileText) {
        self.file = file; self.text = text.content; base = text.content; hash = text.hash; refreshInline()
    }
    func save(using engine: Engine) async {
        guard dirty, !saving, !readOnly else { return }
        let submitted = text, previous = base, previousHash = hash
        saving = true; error = ""
        defer { saving = false }
        do { hash = try await engine.save(self, text: submitted, base: previous, hash: previousHash); base = submitted }
        catch { self.error = "\(error.localizedDescription) Your edits are retained. Copy them before closing; compare with disk before retrying." }
    }
}
@MainActor @Observable final class Workspace {
    var draggingTab: String?
    var layout = PaneLayout()
    var focused = "document-group"
    var files: [FileNode] = []
    var buffers: [String: DocumentBuffer] = [:]
    var status = "Connect to a Hickory workspace server"
    var debugger: NativeDebugger?
    var engine: Engine?
    var address = ProcessInfo.processInfo.environment["HICKORY_API_ORIGIN"] ?? ""
    var connecting = false
    private var opening: Set<String> = []
    private var newestOpen: [String: String] = [:]
    let agent = AgentSession()
    let terminal = TerminalSession()
    private var generation = UUID()
    var active: String? { layout.nodes[focused]?.active }
    var currentBuffer: DocumentBuffer? { active.flatMap { buffers[$0] } }
    var hasUnsavedChanges: Bool { !agent.prompt.isEmpty || !terminal.input.isEmpty || agent.running || terminal.sending || buffers.values.contains { $0.dirty || $0.saving } || debugger?.active == true }
    func connect() async -> Bool {
        guard !connecting else { return false }
        guard !hasUnsavedChanges else { status = "Save documents and send or clear draft messages before changing workspace servers."; return false }
        connecting = true; defer { connecting = false }
        do {
            let next = try Engine(address: address)
            let tree = try await next.files()
            agent.turns = []; agent.error = ""; terminal.id = nil; terminal.screen = ""; terminal.error = ""
            debugger?.close()
            for buffer in buffers.values { buffer.language?.close() }
            generation = UUID(); engine = next; debugger = NativeDebugger(engine: next)
            debugger?.onLocation = { [weak self] path, line in Task { await self?.revealDebug(path, line: line) } }; files = tree.tree; buffers = [:]; opening = []
            layout = PaneLayout(); focused = "document-group"; status = tree.root
            return true
        } catch { status = error.localizedDescription; return false }
    }
    func refresh() async {
        guard let engine else { return }
        let current = generation
        do { let tree = try await engine.files(); if current == generation { files = tree.tree; status = tree.root } }
        catch { if current == generation { status = error.localizedDescription } }
    }
    func open(_ file: FileNode) async {
        if buffers[file.path] != nil { show(file.path); return }
        guard let engine, !opening.contains(file.path) else { return }
        let current = generation
        let target = layout.groups().first(where: { $0.id == focused && !$0.tabs.contains("files") })?.id
            ?? layout.groups().first(where: { !$0.tabs.contains("files") })?.id ?? focused
        newestOpen[target] = file.path; focused = target
        opening.insert(file.path)
        defer { if current == generation { opening.remove(file.path) } }
        do {
            let text = try await engine.file(file)
            guard current == generation else { return }
            let buffer = DocumentBuffer(file: file, text: text)
            buffers[file.path] = buffer
            buffer.receiptEngine = engine
            buffer.language = NativeLanguage(engine: engine, buffer: buffer); buffer.language?.start()
            if newestOpen[target] == file.path {
                // A close or move can retire the requested pane while text loads.
                let live = layout.groups().first(where: { $0.id == target })?.id ?? layout.groups()[0].id
                layout.activate(file.path, in: live); focused = live; status = file.path
            }
        } catch { if current == generation { status = "\(error.localizedDescription) Refresh files and try opening \(file.path) again." } }
    }
    func revealDebug(_ path: String, line: Int) async {
        func find(_ nodes: [FileNode]) -> FileNode? {
            for node in nodes { if node.path == path { return node }; if let child = find(node.children ?? []) { return child } }; return nil
        }
        guard let file = find(files) else { return }
        await open(file)
        if let buffer = buffers[path] { debugger?.rememberSource(buffer) }
        // Mount the existing editor before applying a source-line reveal.
        for _ in 0..<20 {
            if let reveal = buffers[path]?.revealDebugLine { reveal(line); return }
            try? await Task.sleep(for: .milliseconds(25))
        }
    }
    func openTerminal() async {
        guard let engine else { status = "Connect to a workspace server before opening Terminal."; return }
        show("terminal"); await terminal.open(using: engine)
    }
    func show(_ tab: String) {
        let target = layout.groups().first(where: { $0.tabs.contains(tab) })?.id ?? (tab != "files" ? layout.groups().first(where: { $0.id == focused && !$0.tabs.contains("files") })?.id ?? layout.groups().first(where: { !$0.tabs.contains("files") })?.id : nil) ?? focused
        layout.activate(tab, in: target); focused = target
    }
    func close(_ tab: String) {
        if let buffer = buffers[tab], buffer.dirty || buffer.saving {
            status = "Unsaved edits retained. Save this document before closing its tab."; return
        }
        layout.remove(tab)
        if !layout.groups().contains(where: { $0.id == focused }) { focused = layout.groups()[0].id }
    }
    func split(_ axis: String) {
        if let retained = layout.split(focused, axis: axis, newID: UUID().uuidString) { focused = retained }
    }
    func moveToNext() {
        let groups = layout.groups()
        guard let active, let index = groups.firstIndex(where: { $0.id == focused }) else { return }
        let target = groups[(index + 1) % groups.count].id
        layout.move(active, to: target); focused = target
    }
    func label(_ tab: String) -> String {
        if tab == "files" { return "Files" }
        if tab == "agent" { return "Agent" }
        if tab == "terminal" { return "Terminal" }
        return (buffers[tab]?.file.name ?? (tab as NSString).lastPathComponent) + (buffers[tab]?.dirty == true ? " •" : "")
    }
    var savingAll = false
    var fileCommand: String?
    var filePath = ""
    func newTab() {
        let path = "Untitled-" + UUID().uuidString
        buffers[path] = DocumentBuffer(file: FileNode(name: "Untitled \(buffers.keys.filter { $0.hasPrefix("Untitled-") }.count + 1)", path: path, dir: false), text: FileText(path: path, content: "", hash: ""))
        show(path)
    }
    func chooseFileCommand(_ command: String) { filePath = ""; fileCommand = command }
    func performFileCommand() async {
        guard let command = fileCommand, let engine else { status = "Connect to a workspace server first."; return }
        let path = filePath; let source = currentBuffer; fileCommand = nil
        func find(_ nodes: [FileNode]) -> FileNode? {
            for node in nodes { if node.path == path { return node }; if let found = find(node.children ?? []) { return found } }; return nil
        }
        if command == "Save As" {
            guard let source, !source.saving, !path.isEmpty else { return }
            if path == source.file.path { await source.save(using: engine); return }
            let submitted = source.text
            source.saving = true; defer { source.saving = false }
            func finish() {
                if source.text == submitted { layout.remove(source.file.path); buffers.removeValue(forKey: source.file.path) }
            }
            do {
                struct Created: Decodable { var path: String }
                if path.lowercased().hasSuffix(".md") {
                    let _: SavedDocument = try await engine.request("POST", "/api/projects/local/docs", body: ["path": path, "source": submitted])
                    await refresh()
                    func created(_ nodes: [FileNode]) -> FileNode? {
                        for node in nodes { if node.path == path { return node }; if let found = created(node.children ?? []) { return found } }; return nil
                    }
                    if let file = created(files) { await open(file); finish() }
                    source.saving = false
                    if savingAll { await saveAll() }; return
                }
                let _: Created = try await engine.request("POST", "/api/files/op", body: ["op": "create", "path": path])
                let file = FileNode(name: (path as NSString).lastPathComponent, path: path, dir: false)
                let empty = try await engine.file(file)
                guard empty.content.isEmpty else { status = "Destination changed after creation. Original edits retained."; return }
                let copy = DocumentBuffer(file: file, text: empty); copy.text = submitted
                await copy.save(using: engine)
                guard copy.error.isEmpty else { status = copy.error; return }
                await open(file); finish(); await refresh()
                source.saving = false
                if savingAll { await saveAll() }
            } catch { status = "\(error.localizedDescription) Original edits are retained. Choose an unused path and retry Save As." }
        } else if command == "Open Directory" && path.isEmpty { await refresh(); show("files") }
        else if let file = find(files), file.dir == (command == "Open Directory") {
            if file.dir { files = file.children ?? []; show("files") } else { await open(file) }
        } else { status = "Path not found. Refresh and choose a path in this workspace." }
    }
    func saveAll() async {
        guard let engine else { return }
        savingAll = true
        for buffer in Array(buffers.values) where !buffer.file.path.hasPrefix("Untitled-") { await buffer.save(using: engine) }
        if let draft = buffers.values.first(where: { $0.file.path.hasPrefix("Untitled-") && $0.dirty }) {
            show(draft.file.path); chooseFileCommand("Save As")
        } else { savingAll = false }
    }
    func save() async {
        if currentBuffer?.file.path.hasPrefix("Untitled-") == true { chooseFileCommand("Save As"); return }
        if let engine, let buffer = currentBuffer { await buffer.save(using: engine) }
    }
}
