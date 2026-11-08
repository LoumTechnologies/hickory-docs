import SwiftUI
import Observation

@MainActor @Observable final class DocumentBuffer: Identifiable {
    let id = UUID()
    let file: FileNode
    var text: String {
        didSet { let changed = text != base; if dirty != changed { dirty = changed }; if text != oldValue { language?.changed() } }
    }
    @ObservationIgnored var base: String {
        didSet { let changed = text != base; if dirty != changed { dirty = changed } }
    }
    @ObservationIgnored var navigate: ((Double) -> Void)?
    @ObservationIgnored var renderLanguage: (([NativeSemanticToken], String) -> Void)?
    @ObservationIgnored var applyLanguageCompletion: ((NSRange, String, String, NSRange) -> Bool)?
    @ObservationIgnored var selection = NSRange(location: 0, length: 0)
    var visibleDebugLines: [DebugLine] = []
    @ObservationIgnored var revealDebugLine: ((Int) -> Void)?
    var language: NativeLanguage?
    var hash: String
    var saving = false
    var error = ""
    private(set) var dirty = false
    var readOnly: Bool { file.generated_by != nil }
    init(file: FileNode, text: FileText) {
        self.file = file; self.text = text.content; base = text.content; hash = text.hash
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
        let target = layout.groups().first(where: { $0.tabs.contains(tab) })?.id ?? focused
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
        return (tab as NSString).lastPathComponent + (buffers[tab]?.dirty == true ? " •" : "")
    }
    func save() async { if let engine, let buffer = currentBuffer { await buffer.save(using: engine) } }
}
