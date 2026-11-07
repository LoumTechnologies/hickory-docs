import SwiftUI
import Observation

@MainActor @Observable final class DocumentBuffer: Identifiable {
    let id = UUID()
    let file: FileNode
    @ObservationIgnored var text: String {
        didSet { let changed = text != base; if dirty != changed { dirty = changed } }
    }
    @ObservationIgnored var base: String {
        didSet { let changed = text != base; if dirty != changed { dirty = changed } }
    }
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
    var layout = PaneLayout()
    var focused = "document-group"
    var files: [FileNode] = []
    var buffers: [String: DocumentBuffer] = [:]
    var status = "Connect to a Hickory workspace server"
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
    var hasUnsavedChanges: Bool { !agent.prompt.isEmpty || !terminal.input.isEmpty || agent.running || terminal.sending || buffers.values.contains { $0.dirty || $0.saving } }
    func connect() async -> Bool {
        guard !connecting else { return false }
        guard !hasUnsavedChanges else { status = "Save documents and send or clear draft messages before changing workspace servers."; return false }
        connecting = true; defer { connecting = false }
        do {
            let next = try Engine(address: address)
            let tree = try await next.files()
            agent.turns = []; agent.error = ""; terminal.id = nil; terminal.screen = ""; terminal.error = ""
            generation = UUID(); engine = next; files = tree.tree; buffers = [:]; opening = []
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
            buffers[file.path] = DocumentBuffer(file: file, text: text)
            if newestOpen[target] == file.path { layout.activate(file.path, in: target); status = file.path }
        } catch { if current == generation { status = "\(error.localizedDescription) Refresh files and try opening \(file.path) again." } }
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
