<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Render recursive panes with fixed tab strips

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

The same recursive split model works on Mac, iPhone and iPad. On narrow screens groups retain their commands; proportions and split directions remain user-controlled. Draggable tab identities and a menu alternative both move the existing buffer.

The first view task connects to the launcher-supplied address and forwards its optional requested file to the same workspace opening path as native menu commands. With no address, or if that connection fails, show the existing connection sheet. An already attached workspace keeps its drafts and layout when the view reappears.

A DropDelegate reads pane-local hover and drop positions while the native text view stays mounted. Its pointer-transparent half-pane overlay does not steal normal editing input. The custom tab payload is loaded asynchronously; docking validates the current source and target again before changing the layout, so an obsolete reply cannot remove a tab. A custom tab type keeps the drag distinct from text insertion; plain text never reaches the docking decoder. A decoded payload that does not name a live tab is ignored.

Native tab drops insert before their destination tab, while unused strip space appends. The insertion marker is an overlay, so dragging never changes tab geometry. Reject text drags that do not name an existing tab; the model revalidates the destination before changing membership.

Reserve a small blank slot at the end of the strip even when tabs overflow, so append remains reachable by horizontal scrolling. It is always present and does not resize during a drag.

File commands act on the connected workspace: path entry opens a file or directory, New Tab keeps an unnamed buffer, and Save As exclusively creates a new destination before a hash-checked write. Existing destinations are never overwritten by Save As. Save All visits each named buffer; unnamed drafts ask for a path and failed saves retain their original edits. Refresh observes the tree without replacing open buffers.

The status-bar Problems control opens a bounded floating view. The workspace constructs source-bound report values and owns navigation; this view only filters, pages and displays them. An unversioned message offers a file open, not a fabricated precise location. Opening the popover never changes the editor's available rectangle.

On iOS the Edit menu offers multiple-selection commands for the mounted source.
The replacement sheet holds the editor, source and selection revisions from when
the command was chosen. Apply checks them again and sends linked fields through
the same snippet edit as UIKit typing, so their mirrors and undo stay together.
Disable prose keyboard substitutions so the replacement reaches both paths exactly.

<ui:copy id="simple-80f4e5a1a0-0">
import SwiftUI
import UniformTypeIdentifiers
import CoreTransferable

private extension UTType { static let hickoryTab = UTType(exportedAs: "com.hickorydocs.simple.tab") }
private struct TabDrag: Codable, Transferable {
    let tab: String
    static var transferRepresentation: some TransferRepresentation { CodableRepresentation(contentType: .hickoryTab) }
}

struct WorkspaceView: View {
    @Environment(\.openWindow) private var openWindow
    @Bindable var workspace: Workspace
    var launchFile = ProcessInfo.processInfo.environment["HICKORY_OPEN_FILE"]
    @State private var connection = false
    #if os(macOS)
    @State private var localHost: LocalWorkspaceHost?
    @State private var openingLocalWorkspace = false
    #endif
    #if os(iOS)
    @State private var selectionEdit: MobileSelectionEdit?
    @State private var selectionReplacement = ""
    #endif
    private var menuCommands:[WorkspaceCommand] {workspace.commands(ui:["file.window":{openWindow(id:"workspace")},"file.connect":{connection=true}])}
    var body: some View {
        let epoch = workspace.sourceNavigationEpoch
        let observeAgent = workspace.agent.liveTurn != nil || workspace.layout.groups().contains { $0.active == "agent" }
        VStack(spacing: 0) {
            #if !os(macOS)
            HStack {
                Menu("File") {ForEach(menuCommands.filter {$0.group=="File"}) {WorkspaceCommandButton(command:$0)}}
                Menu("Edit") {
                    EditorNavigationSettings()
                    ForEach(menuCommands.filter {$0.group=="Edit"}) {WorkspaceCommandButton(command:$0)}
                    #if os(iOS)
                    if let selections = MobileSelectionEdit(workspace) {
                        Button("Edit selections") {
                            guard selections.canEdit(in: workspace) else { return }
                            selectionReplacement = ""
                            selectionEdit = selections
                        }.disabled(!selections.canEdit(in: workspace))
                        Button("Clear selections") {
                            guard selections.isCurrent(in: workspace) else { return }
                            if selections.buffer.snippets.parallel != nil { selections.buffer.snippets.finish() }
                            selections.buffer.clearMulti()
                        }
                    }
                    #endif
                }
                Menu("View") {ForEach(menuCommands.filter {$0.group=="View"}) {WorkspaceCommandButton(command:$0)}}
                Spacer()
            }.padding(.horizontal, 8).frame(height: 32)
            Divider()
            #endif
            PaneView(workspace: workspace, id: workspace.layout.rootID)
            Divider()
            HStack {
                Text(workspace.status).lineLimit(1).help(workspace.status)
                Spacer(minLength: 8)
                if let buffer = workspace.currentBuffer {
                    if let language = buffer.language { Text(language.completionNotice.isEmpty ? language.status:language.completionNotice).lineLimit(1).frame(maxWidth: 310).help(language.completionNotice.isEmpty ? language.status:language.completionNotice) } else if !buffer.snippets.notice.isEmpty {Text(buffer.snippets.notice).lineLimit(1).frame(maxWidth:310).help(buffer.snippets.notice)}
                    Text(buffer.readOnly ? "Read-only" : buffer.saving ? "Saving…" : buffer.dirty ? "Modified" : "Saved")
                }
                Button(workspace.analysis.running ? "Reading \(workspace.analysis.completed)/\(workspace.analysis.scope.files.count)":"Analysis") { workspace.showAnalysis.toggle() }
                    .frame(width:112).lineLimit(1).popover(isPresented:$workspace.showAnalysis) { WorkspaceAnalysisView(workspace:workspace) }
                Button("Problems \(workspace.problems.count)") { workspace.showProblems.toggle() }
                    .popover(isPresented: $workspace.showProblems) { WorkspaceProblemsView(workspace: workspace) }
                Button("⋯") { connection = true }.help("Workspace connection")
            }.font(.system(size: 11)).padding(.horizontal, 8).frame(height: 24)
        }
        #if os(iOS)
        .sheet(item: $selectionEdit) { selections in
            VStack(alignment: .leading, spacing: 12) {
                Text("Edit selections").font(.headline)
                TextField("Insert at selections", text: $selectionReplacement).textFieldStyle(.roundedBorder)
                    .textInputAutocapitalization(.never).autocorrectionDisabled()
                HStack {
                    Button("Cancel") { selectionEdit = nil }
                    Spacer()
                    Button("Apply to selections") {
                        guard selections.canEdit(in: workspace) else { return }
                        let buffer = selections.buffer
                        let applied: Bool
                        if buffer.snippets.parallel != nil {
                            applied = buffer.snippets.inputParallel(source: buffer.text, primaryRange: buffer.selection,
                                                                    text: selectionReplacement)
                        } else {
                            applied = buffer.replaceMultiple(selectionReplacement)
                        }
                        if applied { selectionEdit = nil }
                    }.disabled(!selections.canEdit(in: workspace))
                }
            }.padding().presentationDetents([.height(200)])
        }
        #endif
        .task{await workspace.editorRecovery.captureLoop(workspace)}
        .task(id: epoch.uuidString + String(observeAgent)) {
            guard observeAgent else { return }
            guard let engine = workspace.engine else {
                workspace.agent.error = "Connect to a workspace server to use the agent."
                return
            }
            await workspace.agent.observe(engine) { workspace.sourceNavigationEpoch == epoch }
        }
        .popover(isPresented:Binding(get:{workspace.editorRecovery.showing},set:{if !$0{workspace.editorRecovery.close()}})){EditorRecoveryView(workspace:workspace).onKeyPress(.escape){workspace.editorRecovery.close();return .handled}}
        .popover(isPresented:$workspace.showCoverage) {SimpleCoverageView(workspace:workspace).onKeyPress(.escape){workspace.showCoverage=false;workspace.currentBuffer?.focusSource?();return .handled}}
        .popover(isPresented:$workspace.showRecovery) {SourceRecoveryView(workspace:workspace).onKeyPress(.escape){workspace.showRecovery=false;workspace.currentBuffer?.focusSource?();return .handled}}
        .popover(isPresented:$workspace.showConstruction) {ConstructionReadingView(workspace:workspace).onKeyPress(.escape){workspace.showConstruction=false;workspace.currentBuffer?.focusSource?();return .handled}}
        .popover(isPresented:$workspace.showAttribution) {GitAttributionView(workspace:workspace,model:workspace.gitAttribution).onKeyPress(.escape){workspace.showAttribution=false;workspace.currentBuffer?.focusSource?();return .handled}}
        .popover(isPresented:$workspace.showHistory) {GitHistoryView(workspace:workspace,model:workspace.gitHistory).onKeyPress(.escape){workspace.showHistory=false;workspace.currentBuffer?.focusSource?();return .handled}}
        .popover(isPresented:$workspace.showReview) {GitReviewView(workspace:workspace).onKeyPress(.escape){workspace.showReview=false;workspace.currentBuffer?.focusSource?();return .handled}}
        .popover(isPresented:Binding(get:{workspace.agentProposals.showing},set:{if !$0{workspace.agentProposals.close()}})){AgentProposalView(workspace:workspace).onKeyPress(.escape){workspace.agentProposals.close();workspace.currentBuffer?.focusSource?();return .handled}}
        .popover(isPresented:Binding(get:{workspace.sourceTemplates.showing},set:{if !$0{workspace.sourceTemplates.close()}})) {SourceTemplatesView(workspace:workspace).onKeyPress(.escape){workspace.sourceTemplates.close();workspace.currentBuffer?.focusSource?();return .handled}}
        .popover(isPresented:Binding(get:{workspace.sourceHistory.showing},set:{if !$0{workspace.sourceHistory.close()}})) {SourceHistoryView(workspace:workspace).onKeyPress(.escape){workspace.sourceHistory.close();workspace.currentBuffer?.focusSource?();return .handled}}
        .popover(isPresented:$workspace.showTests) {TestSourcesView(workspace:workspace).onKeyPress(.escape){workspace.showTests=false;workspace.currentBuffer?.focusSource?();return .handled}}
        .popover(isPresented:$workspace.showEverywhere) {SearchEverywhereView(workspace:workspace,commands:menuCommands)}
        .sheet(isPresented: Binding(get: { workspace.fileCommand != nil }, set: { if !$0 { workspace.fileCommand = nil; workspace.savingAll = false } })) {
            VStack(alignment: .leading, spacing: 12) {
                Text(workspace.fileCommand ?? "File").font(.headline)
                Text("Enter a path in the connected workspace. Save As requires an unused path; an empty directory path opens the root.").font(.caption)
                TextField("Workspace path", text: $workspace.filePath).textFieldStyle(.roundedBorder)
                HStack { Button("Cancel") { workspace.fileCommand = nil; workspace.savingAll = false }; Spacer(); Button("OK") { Task { await workspace.performFileCommand() } } }
            }.padding().frame(minWidth: 320)
        }
        .sheet(isPresented: Binding(get: { workspace.changes.mode != nil }, set: { if !$0 { workspace.changes.close(workspace) } })) { WorkspaceChangesView(workspace:workspace) }
        .sheet(isPresented: $workspace.showWorkspaceSearch, onDismiss: { workspace.search.revision = UUID(); workspace.currentBuffer?.focusSource?() }) { WorkspaceSearchView(workspace: workspace) }
        .sheet(isPresented: $connection) {
            VStack(alignment: .leading, spacing: 12) {
                Text("Workspace server").font(.headline)
                TextField("http://server:port", text: $workspace.address).textFieldStyle(.roundedBorder).accessibilityLabel("Workspace server")
                Text("Use the address of the running Hickory backend. On iPhone or iPad, use a host reachable from that device.").font(.caption)
                Text(workspace.status).font(.caption).textSelection(.enabled)
                HStack {
                    Button("Cancel") { connection = false }
                    #if os(macOS)
                    if LocalWorkspaceHost.isAvailable {
                        Button("Open Local Workspace…") { chooseLocalWorkspace() }
                            .disabled(openingLocalWorkspace || workspace.connecting)
                    }
                    #endif
                    Spacer()
                    Button("Connect") { Task { await connectWorkspace() } }.disabled(connectingWorkspace)
                }
            }.padding().frame(minWidth: 280, idealWidth: 420)
        }
        .onReceive(NotificationCenter.default.publisher(for: .simpleConnect)) { _ in connection = true }
        .task {
            if workspace.engine == nil {
                #if os(macOS)
                if workspace.address.isEmpty, let location = LocalWorkspaceHost.takeLaunchLocation() {
                    await openLocalWorkspace(location)
                    if workspace.engine == nil { connection = true }
                    return
                }
                #endif
                if workspace.address.isEmpty { connection = true }
                else { connection = !(await workspace.connect(opening: launchFile)) }
            }
        }
        .onDisappear { workspace.debugger?.closeRecordedFence() }
        #if os(macOS)
        .background(WindowGuard(workspace: workspace))
        .onOpenURL { url in
            if url.isFileURL { Task { await openLocalWorkspace(url) } }
        }
        #endif
    }

    private var connectingWorkspace: Bool {
        #if os(macOS)
        if openingLocalWorkspace { return true }
        #endif
        return workspace.connecting
    }

    private func connectWorkspace() async {
        guard !connectingWorkspace else { return }
        if await workspace.connect() {
            #if os(macOS)
            if let localHost, workspace.engine?.origin.absoluteString != localHost.origin {
                self.localHost = nil
            }
            #endif
            connection = false
        }
    }

    #if os(macOS)
    private func chooseLocalWorkspace() {
        let panel = NSOpenPanel()
        panel.canChooseDirectories = true
        panel.canChooseFiles = true
        panel.prompt = "Open Workspace"
        panel.begin { response in
            if response == .OK, let url = panel.url {
                Task { await openLocalWorkspace(url) }
            }
        }
    }

    private func openLocalWorkspace(_ url: URL) async {
        guard !openingLocalWorkspace, !workspace.connecting else { return }
        guard !workspace.hasUnsavedChanges else {
            workspace.status = "Save documents and send or clear draft messages before opening another workspace."
            return
        }
        openingLocalWorkspace = true
        defer { openingLocalWorkspace = false }
        let previousAddress = workspace.address
        do {
            let location = url.standardizedFileURL.resolvingSymlinksInPath()
            let isDirectory = try location.resourceValues(forKeys: [.isDirectoryKey]).isDirectory == true
            let root = isDirectory ? location : location.deletingLastPathComponent()
            let next = try LocalWorkspaceHost(root: root)
            let address = try await next.address()
            workspace.address = address
            if await workspace.connect(opening: isDirectory ? nil : location.lastPathComponent) {
                localHost = next
                connection = false
            } else {
                workspace.address = previousAddress
            }
        } catch {
            workspace.address = previousAddress
            workspace.status = error.localizedDescription
        }
    }
    #endif
}
#if os(iOS)
@MainActor
private struct MobileSelectionEdit: Identifiable {
    let id = UUID()
    let buffer: DocumentBuffer
    private let workspaceEpoch: UUID
    private let sourceRevision: UUID
    private let selectionRevision: UUID
    private let snippetIntent: Int
    private let editorIdentity: UUID

    init?(_ workspace: Workspace) {
        guard let buffer = workspace.currentBuffer, Self.selectionsMatch(buffer),
              let editor = buffer.sourceEditorState?(), Self.editorMatches(editor, buffer) else { return nil }
        self.buffer = buffer
        workspaceEpoch = workspace.sourceNavigationEpoch
        sourceRevision = buffer.sourceEditIdentity
        selectionRevision = buffer.selectionIdentity
        snippetIntent = buffer.snippets.intent
        editorIdentity = editor.identity
    }

    func isCurrent(in workspace: Workspace) -> Bool { currentEditor(in: workspace) != nil }

    func canEdit(in workspace: Workspace) -> Bool {
        currentEditor(in: workspace)?.editable == true && !buffer.readOnly && !buffer.saving
    }

    private func currentEditor(in workspace: Workspace) -> NativeSourceEditorState? {
        guard workspace.currentBuffer === buffer, workspace.sourceNavigationEpoch == workspaceEpoch,
              buffer.sourceEditIdentity == sourceRevision, buffer.selectionIdentity == selectionRevision,
              buffer.snippets.intent == snippetIntent, Self.selectionsMatch(buffer),
              let editor = buffer.sourceEditorState?(), editor.identity == editorIdentity,
              Self.editorMatches(editor, buffer) else { return nil }
        return editor
    }

    private static func selectionsMatch(_ buffer: DocumentBuffer) -> Bool {
        !buffer.resourceDeleted && buffer.multiRanges.count > 1 && exactUTF8(buffer.multiSource, buffer.text)
    }

    private static func editorMatches(_ editor: NativeSourceEditorState, _ buffer: DocumentBuffer) -> Bool {
        editor.attached && !editor.composing && editor.selection == buffer.selection && exactUTF8(editor.source, buffer.text)
    }
}
#endif
struct PaneMenu: View {
    var workspace: Workspace
    var body: some View {
        Menu("View") {
            Button("Files") { workspace.showUtility(.files) }
            Button("Agent") { workspace.showUtility(.agent) }
            Button("Terminal") { Task { await workspace.openTerminal() } }
            Button("Split right") { workspace.split("horizontal") }
            Button("Split below") { workspace.split("vertical") }
            Button("Move tab to next group") { workspace.moveToNext() }.disabled(workspace.active == nil || workspace.layout.groups().count < 2)
        }
    }
}
extension Notification.Name { static let simpleConnect = Notification.Name("hickory.simple.connect") }
</ui:copy>

A split reserves five points for its resize handle. A leaf owns a fixed-height tab strip and an expanding content rectangle. Menus and popovers do not change neighboring content geometry.

<ui:copy id="simple-80f4e5a1a0-1">
struct PaneView: View {
    var workspace: Workspace
    let id: String
    @Environment(\.accessibilityReduceMotion) private var reducedMotion
    @State private var initialRatio: Double?
    @State private var dropBefore: NativeSourceKey?
    @State private var paneEdge: String?
    @State private var tabFrames: [NativeSourceKey: CGRect] = [:]
    var body: some View {
        if let node = workspace.layout.nodes[id] {
            if let first = node.first, let second = node.second {
                GeometryReader { geometry in
                    let horizontal = node.axis == "horizontal"
                    let length = max(0, (horizontal ? geometry.size.width : geometry.size.height) - 5)
                    if horizontal {
                        HStack(spacing: 0) {
                            AnyView(PaneView(workspace: workspace, id: first)).frame(width: length * node.ratio)
                            handle(node, length: length, horizontal: true).frame(width: 5)
                            AnyView(PaneView(workspace: workspace, id: second))
                        }
                    } else {
                        VStack(spacing: 0) {
                            AnyView(PaneView(workspace: workspace, id: first)).frame(height: length * node.ratio)
                            handle(node, length: length, horizontal: false).frame(height: 5)
                            AnyView(PaneView(workspace: workspace, id: second))
                        }
                    }
                }
            } else {
                VStack(spacing: 0) {
                    ScrollView(.horizontal) {
                        HStack(spacing: 0) {
                            ForEach(node.tabs, id: \.nativeSourceKey) { tab in
                                HStack(spacing: 0) {
                                    #if os(macOS)
                                    MacTabLabel(tab: tab, label: workspace.label(tab), activate: {
                                        workspace.show(tab, in: id)
                                    }, dragging: { workspace.draggingTab = $0 }).fixedSize(horizontal: true, vertical: false).frame(height: 28)
                                    #else
                                    Button(workspace.label(tab)) { workspace.show(tab, in: id) }
                                        .accessibilityAddTraits(exactUTF8(node.active,tab) ? .isSelected : [])
                                    #endif
                                    Button("×") { workspace.close(tab) }.accessibilityLabel("Close \(workspace.label(tab))")
                                }.padding(.horizontal, 5).frame(height: 28)
                                    .animation(SimpleMotion.animation(100, reduced: reducedMotion)) {
                                        $0.background(exactUTF8(node.active,tab) ? Color.accentColor.opacity(0.12) : Color.clear)
                                    }
                                    .overlay(alignment: .top) { if exactUTF8(node.active,tab) { Rectangle().fill(Color.accentColor).frame(height: 2) } }
                                    .overlay(alignment: .leading) { if dropBefore==tab.nativeSourceKey { Rectangle().fill(Color.accentColor).frame(width: 2) } }
                                    #if os(macOS)
                                    .background(GeometryReader { geometry in
                                        Color.clear.preference(key: TabFramePreference.self, value: [tab.nativeSourceKey: geometry.frame(in: .named("tabs-" + id))])
                                    })
                                    #else
                                    .draggable(TabDrag(tab: tab))
                                    .dropDestination(for: TabDrag.self) { tabs, _ in
                                        dropBefore = nil
                                        guard let dragged = tabs.first?.tab, workspace.layout.groups().contains(where: { $0.tabs.containsTab(dragged) }) else { return false }
                                        workspace.moveTab(dragged, to: id, before: tab); return true
                                    } isTargeted: { targeted in
                                        if targeted { dropBefore = tab.nativeSourceKey } else if dropBefore==tab.nativeSourceKey { dropBefore = nil }
                                    }
                                    #endif
                            }
                            Color.clear.frame(width: 20, height: 28).contentShape(Rectangle())
                        }
                    }.frame(height: 29).background(.quaternary.opacity(0.4))
                        #if os(macOS)
                        .coordinateSpace(name: "tabs-" + id)
                        .onPreferenceChange(TabFramePreference.self) { tabFrames = $0 }
                        .overlay {
                            MacDropSurface(hover: { point in
                                    dropBefore = point.flatMap { point in node.tabs.first { tabFrames[$0.nativeSourceKey]?.contains(point) == true }?.nativeSourceKey }
                                }, drop: { tab, point in
                                    let before = node.tabs.first { tabFrames[$0.nativeSourceKey]?.contains(point) == true }
                                    workspace.moveTab(tab, to: id, before: before)
                                    workspace.draggingTab = nil; dropBefore = nil; return true
                                })
                        }
                        #else
                        .dropDestination(for: TabDrag.self) { tabs, _ in
                            guard let tab = tabs.first?.tab, workspace.layout.groups().contains(where: { $0.tabs.containsTab(tab) }) else { return false }
                            dropBefore = nil
                            workspace.moveTab(tab, to: id); return true
                        }
                        #endif
                    Divider()
                    GeometryReader { geometry in
                        content(node.active).frame(width: geometry.size.width, height: geometry.size.height)
                            .contentShape(Rectangle()).simultaneousGesture(TapGesture().onEnded { workspace.focused = id })
                            #if os(macOS)
                            .overlay {
                                MacDropSurface(isPaneEdge: true, hover: { point in
                                        paneEdge = point.map { dropEdge($0.x / max(1, geometry.size.width), $0.y / max(1, geometry.size.height)) ?? "center" }
                                    }, drop: { tab, point in
                                        paneEdge = nil
                                        guard workspace.layout.groups().contains(where: { $0.id == id }), workspace.layout.groups().contains(where: { $0.tabs.containsTab(tab) }) else { return false }
                                        if let edge = dropEdge(point.x / max(1, geometry.size.width), point.y / max(1, geometry.size.height)) {
                                            guard workspace.dockTab(tab, to: id, edge: edge) else { return false }
                                        } else { workspace.moveTab(tab, to: id) }
                                        workspace.draggingTab = nil; return true
                                    })
                            }
                            #else
                            .onDrop(of: [UTType.hickoryTab.identifier], delegate: PaneEdgeDrop(workspace: workspace, group: id, size: geometry.size, edge: $paneEdge))
                            #endif
                            .overlay(alignment: paneEdge == "left" ? .leading : paneEdge == "right" ? .trailing : paneEdge == "top" ? .top : .bottom) {
                                if let edge = paneEdge {
                                    Rectangle().fill(Color.accentColor.opacity(0.18))
                                        .overlay { Rectangle().stroke(Color.accentColor, lineWidth: 2) }
                                        .frame(width: geometry.size.width * ((edge == "left" || edge == "right") ? 0.5 : 1),
                                               height: geometry.size.height * ((edge == "top" || edge == "bottom") ? 0.5 : 1))
                                        .modifier(SimpleReveal(milliseconds: 80))
                                        .allowsHitTesting(false)
                                }
                            }
                    }
                }.buttonStyle(SimpleChromeButtonStyle()).clipped().onChange(of: workspace.draggingTab) { _, tab in
                    if tab == nil { paneEdge = nil; dropBefore = nil }
                }
            }
        }
    }
    private func handle(_ node: PaneNode, length: Double, horizontal: Bool) -> some View {
        Rectangle().fill(.quaternary)
            .gesture(DragGesture(minimumDistance: 0).onChanged { value in
                if initialRatio == nil { initialRatio = node.ratio }
                let displacement = horizontal ? value.translation.width : value.translation.height
                workspace.layout.resize(id, ratio: (initialRatio ?? node.ratio) + displacement / max(1, length))
            }.onEnded { _ in initialRatio = nil })
            .accessibilityLabel("Resize panes")
            .accessibilityAdjustableAction { direction in
                workspace.layout.resize(id, ratio: node.ratio + (direction == .increment ? 0.05 : -0.05))
            }
    }
    @ViewBuilder private func content(_ tab: String?) -> some View {
        if tab.flatMap({NativeTabIdentity(address:$0).utility}) == .files {
            ScrollView([.horizontal, .vertical]) {
                VStack(alignment: .leading, spacing: 0) { FileRows(nodes: workspace.files, workspace: workspace) }.padding(5).frame(maxWidth: .infinity, alignment: .leading)
            }.accessibilityLabel("Files")
        } else if tab.flatMap({NativeTabIdentity(address:$0).utility}) == .terminal {
            TerminalPane(engine: workspace.engine, session: workspace.terminal)
        } else if tab.flatMap({NativeTabIdentity(address:$0).utility}) == .agent {
            let epoch = workspace.sourceNavigationEpoch, root = workspace.sourceRootAddress, id = workspace.sourceWorkspaceID
            AgentPane(engine: workspace.engine, session: workspace.agent, debugger: workspace.debugger, workspaceAddress: root, workspaceID: id, reviewPatch: { buffer, source, line in
                Task { await workspace.agentProposals.prepareRecordedPatch(workspace, message: buffer, source: source, openingLine: line) }
            }, workspaceCurrent: { [weak workspace] in
                guard let workspace else { return false }
                return workspace.sourceNavigationEpoch == epoch && exactUTF8(workspace.sourceRootAddress, root)
                    && exactUTF8(workspace.sourceWorkspaceID, id)
            }).onAppear{workspace.configureAgentContext()}
        } else if let tab, let buffer = workspace.bufferForTab(tab) {
            VStack(spacing: 0) {
                let session = workspace.engine?.session
                LanguageEditor(showRunTerminal: { id in
                    guard let session, workspace.engine?.session === session else { return }
                    workspace.terminal.id = id
                    workspace.showUtility(.terminal)
                }, buffer: buffer, debugger: workspace.debugger).id(buffer.id)
                if !buffer.error.isEmpty { Text(buffer.error).font(.caption).foregroundStyle(.red).padding(6).textSelection(.enabled) }
            }
        } else {
            Text("Open a file or move a tab here").foregroundStyle(.secondary).font(.callout)
        }
    }
}
</ui:copy>

## Revalidate an asynchronously loaded drop

A provider reply may arrive after the destination has changed. Decode its custom tab representation, then ask the current layout to validate that the target is still a leaf and the source still exists. Focus the new group only when docking succeeds. Hover and drop each calculate pane-local edges; the preview never authorizes a split by itself.

<ui:copy id="simple-apple-pane-edge-drop">
@MainActor private struct PaneEdgeDrop: DropDelegate {
    var workspace: Workspace
    var group: String
    var size: CGSize
    @Binding var edge: String?
    private func selected(_ info: DropInfo) -> String? {
        guard size.width > 0, size.height > 0, (0...size.width).contains(info.location.x), (0...size.height).contains(info.location.y) else { return nil }
        return dropEdge(info.location.x / size.width, info.location.y / size.height) ?? "center"
    }
    func validateDrop(info: DropInfo) -> Bool { info.hasItemsConforming(to: [UTType.hickoryTab.identifier]) }
    func dropEntered(info: DropInfo) { edge = selected(info) }
    func dropUpdated(info: DropInfo) -> DropProposal? { edge = selected(info); return DropProposal(operation: .move) }
    func dropExited(info: DropInfo) { edge = nil }
    func performDrop(info: DropInfo) -> Bool {
        edge = nil
        guard let selected = selected(info), let provider = info.itemProviders(for: [UTType.hickoryTab.identifier]).first else { return false }
        provider.loadDataRepresentation(forTypeIdentifier: UTType.hickoryTab.identifier) { data, _ in
            guard let data, let payload = try? JSONDecoder().decode(TabDrag.self, from: data) else { return }
            let tab = payload.tab
            Task { @MainActor in
                guard workspace.layout.groups().contains(where: { $0.id == group }), workspace.layout.groups().contains(where: { $0.tabs.containsTab(tab) }) else { return }
                if selected == "center" { workspace.moveTab(tab, to: group) }
                else { workspace.dockTab(tab, to: group, edge: selected) }
            }
        }
        return true
    }
}
</ui:copy>

## Keep file browsing independent of pane drops

The file list below opens source text through the workspace. Edge docking wraps its existing content without changing file identity or introducing another file-loading path.

<ui:copy id="simple-apple-file-rows">
struct FileRows: View {
    let nodes: [FileNode]
    var workspace: Workspace
    var body: some View {
        ForEach(nodes) { node in
            if node.dir {
                DisclosureGroup(node.name) { AnyView(FileRows(nodes: node.children ?? [], workspace: workspace)).padding(.leading, 8) }.padding(.vertical, 2)
            } else {
                Button(node.name) { Task { await workspace.open(node) } }.help(node.path).frame(height: 23).accessibilityIdentifier(node.path)
            }
        }
    }
}
</ui:copy>

## Route in-window tab drags before embedded editors receive them

The Mac tab label tracks native mouse drag/up events in its own window, with a four-point movement threshold and Escape cancellation. It resolves mounted drop surfaces in window coordinates and calls the existing layout operations on release. This feature moves tabs within one window; it does not export files or use a system pasteboard drag. The weak destination registry does not retain views or editors after layout changes.

Destinations are always mounted and pass normal hit tests through to the editor. Their native preview draws during AppKit tracking without waiting for a SwiftUI render. Tab bounds remain measured in the strip coordinate space, so one destination distinguishes insert-before from append. The destination model still validates source and target identities at release.

<ui:copy id="simple-mac-native-tab-drag">
#if os(macOS)
import AppKit
private struct TabFramePreference: PreferenceKey {
    static var defaultValue: [NativeSourceKey: CGRect] = [:]
    static func reduce(value: inout [NativeSourceKey: CGRect], nextValue: () -> [NativeSourceKey: CGRect]) { value.merge(nextValue(), uniquingKeysWith: { _, new in new }) }
}
// Native destinations stay mounted, but never intercept normal editor clicks.
@MainActor private enum NativeTabDragState {
    static let surfaces = NSHashTable<NativeTabDropView>.weakObjects()
    static func destination(_ point: NSPoint, window: NSWindow) -> (NativeTabDropView, NSPoint)? {
        for view in surfaces.allObjects where view.window === window && !view.isHiddenOrHasHiddenAncestor {
            let local = view.convert(point, from: nil)
            if view.bounds.contains(local) { return (view, local) }
        }
        return nil
    }
    static func update(_ point: NSPoint, window: NSWindow) {
        let target = destination(point, window: window)
        for view in surfaces.allObjects {
            let local = target?.0 === view ? target?.1 : nil
            view.dragPoint = local; view.hover(local); view.needsDisplay = true
        }
        window.displayIfNeeded()
    }
    static func clear() {
        for view in surfaces.allObjects { view.dragPoint = nil; view.hover(nil); view.needsDisplay = true }
    }
}
</ui:copy>

## Track movement and release in the source window

A normal click selects the tab. Crossing four points begins the in-window drag; Escape cancels it. Each move resolves the current mounted destination and each mouse-up resolves it again, so the final location determines split versus tab insertion. End cleanup clears preview state and restores the pointer even when the destination refuses the drop.

<ui:copy id="simple-mac-tab-pointer-source">
struct MacTabLabel: NSViewRepresentable {
    var tab: String
    var label: String
    var activate: () -> Void
    var dragging: (String?) -> Void
    func makeNSView(context: Context) -> NativeTabLabel { NativeTabLabel() }
    func updateNSView(_ view: NativeTabLabel, context: Context) {
        view.tab = tab; view.label = label; view.activate = activate; view.dragging = dragging
        view.setAccessibilityElement(true); view.setAccessibilityRole(.button); view.setAccessibilityLabel(label)
        view.invalidateIntrinsicContentSize(); view.needsDisplay = true
    }
    func sizeThatFits(_ proposal: ProposedViewSize, nsView: NativeTabLabel, context: Context) -> CGSize? { nsView.intrinsicContentSize }
}
final class NativeTabLabel: NSView {
    var tab = "", label = ""
    var activate: () -> Void = {}
    var dragging: (String?) -> Void = { _ in }
    override var isFlipped: Bool { true }
    private var attributes: [NSAttributedString.Key: Any] { [.font: NSFont.systemFont(ofSize: 13), .foregroundColor: NSColor.labelColor] }
    override var intrinsicContentSize: NSSize { NSSize(width: ceil((label as NSString).size(withAttributes: attributes).width) + 4, height: 28) }
    override func acceptsFirstMouse(for event: NSEvent?) -> Bool { true }
    override func draw(_ dirtyRect: NSRect) { (label as NSString).draw(at: NSPoint(x: 2, y: 6), withAttributes: attributes) }
    override func mouseDown(with event: NSEvent) {
        guard let window else { return }
        NSApp.activate(); window.makeKeyAndOrderFront(nil); activate()
        let start = event.locationInWindow, identity = tab
        var moved = false
        defer { if moved { NSCursor.pop() }; NativeTabDragState.clear(); dragging(nil) }
        while let next = window.nextEvent(matching: [.leftMouseDragged, .leftMouseUp, .keyDown]) {
            if next.type == .keyDown { if next.keyCode == 53 { return }; continue }
            let point = next.locationInWindow
            if !moved && hypot(point.x - start.x, point.y - start.y) >= 4 {
                moved = true; dragging(identity); NSCursor.closedHand.push()
            }
            if next.type == .leftMouseUp {
                if moved, let (target, local) = NativeTabDragState.destination(point, window: window) { _ = target.drop(identity, local) }
                return
            }
            if moved { NativeTabDragState.update(point, window: window) }
        }
    }
    override func accessibilityPerformPress() -> Bool { activate(); return true }
}
</ui:copy>

## Keep destinations transparent to normal editing

The destination view returns nil from ordinary hit testing. The drag tracker finds it explicitly by window-space bounds, so NSTextView cannot consume a tab drop and no overlay blocks selection or typing. During tracking it paints either the selected half or the entire pane for a center merge. Weak registration follows native view attachment to the window.

<ui:copy id="simple-mac-tab-drop-surfaces">
struct MacDropSurface: NSViewRepresentable {
    var isPaneEdge = false
    var hover: (CGPoint?) -> Void
    var drop: (String, CGPoint) -> Bool
    func makeNSView(context: Context) -> NativeTabDropView { NativeTabDropView() }
    func updateNSView(_ view: NativeTabDropView, context: Context) { view.isPaneEdge = isPaneEdge; view.hover = hover; view.drop = drop }
    func sizeThatFits(_ proposal: ProposedViewSize, nsView: NativeTabDropView, context: Context) -> CGSize? { CGSize(width: proposal.width ?? 0, height: proposal.height ?? 0) }
}
final class NativeTabDropView: NSView {
    var isPaneEdge = false
    var dragPoint: NSPoint?
    var hover: (CGPoint?) -> Void = { _ in }
    var drop: (String, CGPoint) -> Bool = { _, _ in false }
    override var isFlipped: Bool { true }
    override func hitTest(_ point: NSPoint) -> NSView? { nil }
    override func viewDidMoveToWindow() {
        super.viewDidMoveToWindow(); NativeTabDragState.surfaces.remove(self)
        if window != nil { NativeTabDragState.surfaces.add(self) }
    }
    override func draw(_ dirtyRect: NSRect) {
        guard isPaneEdge, let point = dragPoint else { return }
        let edge = dropEdge(point.x / max(1, bounds.width), point.y / max(1, bounds.height))
        var rect = bounds
        if edge == nil { /* The center previews joining the entire tab group. */ }
        else if edge == "left" || edge == "right" { rect.size.width /= 2; if edge == "right" { rect.origin.x += rect.width } }
        else { rect.size.height /= 2; if edge == "bottom" { rect.origin.y += rect.height } }
        NSColor.controlAccentColor.withAlphaComponent(0.18).setFill(); rect.fill()
        NSColor.controlAccentColor.setStroke(); let path = NSBezierPath(rect: rect.insetBy(dx: 1, dy: 1)); path.lineWidth = 2; path.stroke()
    }
}
#endif
</ui:copy>

## Animate paint independently of source state

A shared curve and durations match the browser and Compose chrome. The button
style changes only its background, preserving compact hit targets and line
anchors. Restrict SwiftUI animation to the background or opacity modifier;
putting it on the pane would also interpolate editor geometry during docking.
Search and drag previews fade on insertion and disappear immediately on removal.
Native popovers and sheets retain their system transitions. Reduce Motion
removes our animation. The modifier never replaces or keys its content, so the
editor, selection and undo owner survive the reveal.

<ui:copy id="simple-apple-motion">

// Keep this paint policy local to chrome; never animate an editor subtree.
enum SimpleMotion {
    static func animation(_ milliseconds: Double, reduced: Bool) -> Animation? {
        reduced ? nil : .timingCurve(0, 0, 0.2, 1, duration: milliseconds / 1000)
    }
}
struct SimpleReveal: ViewModifier {
    var milliseconds: Double = 120
    @Environment(\.accessibilityReduceMotion) private var reduced
    @State private var visible = false
    func body(content: Content) -> some View {
        content.animation(SimpleMotion.animation(milliseconds, reduced: reduced)) {
            $0.opacity(visible || reduced ? 1 : 0)
        }.onAppear { visible = true }
    }
}
struct SimpleChromeButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        SimpleChromeButtonBody(label: configuration.label, pressed: configuration.isPressed)
    }
}
private struct SimpleChromeButtonBody<Label: View>: View {
    let label: Label
    let pressed: Bool
    @Environment(\.isEnabled) private var enabled
    @Environment(\.accessibilityReduceMotion) private var reduced
    @State private var hovered = false
    var body: some View {
        label.contentShape(Rectangle())
            .animation(SimpleMotion.animation(80, reduced: reduced)) {
                $0.background(Color.primary.opacity(enabled ? (pressed ? 0.12 : hovered ? 0.08 : 0) : 0))
            }
            .opacity(enabled ? 1 : 0.45)
            .onHover { hovered = $0 }
    }
}

struct WorkspaceProblemsView: View {
    @Bindable var workspace: Workspace
    @State private var query = ""
    @State private var page = 0
    var body: some View {
        let reports = workspace.problems.filter { query.isEmpty || ($0.path + " " + $0.message).localizedCaseInsensitiveContains(query) }
        let last = max(0, (reports.count + 99) / 100 - 1), selected = min(page, last)
        VStack(alignment: .leading, spacing: 8) {
            HStack { Text("Problems").font(.headline); Spacer(); Button("Close") { workspace.showProblems = false } }
            Text("Language-provider reports · \(workspace.problems.filter { $0.current }.count) current locations").font(.caption)
            TextField("Filter problems", text: $query).textFieldStyle(.roundedBorder).onChange(of: query) { _, _ in page = 0 }
            ScrollView { LazyVStack(alignment: .leading, spacing: 8) {
                if reports.isEmpty { Text("No reported problems match. This is provider information, not a verification result.").font(.caption) }
                ForEach(Array(reports.dropFirst(selected * 100).prefix(100))) { problem in
                    Button(problem.path + " · " + problem.message) { Task { await workspace.openProblem(problem) } }.disabled(!problem.current)
                    if !problem.current {
                        Text("Source location stale or unversioned.").font(.caption)
                        Button("Open file") { Task { await workspace.openProblem(problem, precise: false) } }
                    }
                }
            } }
            if last > 0 { HStack {
                Button("Previous results") { page = selected - 1 }.disabled(selected == 0)
                Text("\(selected + 1)/\(last + 1)").font(.caption)
                Button("Next results") { page = selected + 1 }.disabled(selected == last)
            } }
        }.padding().frame(width: 350, height: 420)
    }
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/apple-simple/Sources/WorkspaceView.swift"><ui:paste select="#simple-80f4e5a1a0-0"/><ui:paste select="#simple-80f4e5a1a0-1"/><ui:paste select="#simple-apple-pane-edge-drop"/><ui:paste select="#simple-apple-file-rows"/><ui:paste select="#simple-mac-native-tab-drag"/><ui:paste select="#simple-mac-tab-pointer-source"/><ui:paste select="#simple-mac-tab-drop-surfaces"/><ui:paste select="#simple-apple-motion"/></ui:file>

</ui:doc>
