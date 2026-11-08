import SwiftUI
import UniformTypeIdentifiers
import CoreTransferable

private extension UTType { static let hickoryTab = UTType(exportedAs: "com.hickorydocs.simple.tab") }
private struct TabDrag: Codable, Transferable {
    let tab: String
    static var transferRepresentation: some TransferRepresentation { CodableRepresentation(contentType: .hickoryTab) }
}

struct WorkspaceView: View {
    @Bindable var workspace: Workspace
    @State private var connection = false
    var body: some View {
        VStack(spacing: 0) {
            #if !os(macOS)
            HStack {
                Menu("File") {
                    Button("Connect…") { connection = true }
                    Button("Refresh files") { Task { await workspace.refresh() } }
                    Button("Save") { Task { await workspace.save() } }.disabled(workspace.currentBuffer == nil)
                    Button("Close tab") { if let active = workspace.active { workspace.close(active) } }
                }
                PaneMenu(workspace: workspace)
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
                    if let language = buffer.language { Text(language.status).lineLimit(1).frame(maxWidth: 310).help(language.status) }
                    Text(buffer.readOnly ? "Read-only" : buffer.saving ? "Saving…" : buffer.dirty ? "Modified" : "Saved")
                }
                Button("⋯") { connection = true }.help("Workspace connection")
            }.font(.system(size: 11)).padding(.horizontal, 8).frame(height: 24)
        }
        .sheet(isPresented: $connection) {
            VStack(alignment: .leading, spacing: 12) {
                Text("Workspace server").font(.headline)
                TextField("http://server:port", text: $workspace.address).textFieldStyle(.roundedBorder).accessibilityLabel("Workspace server")
                Text("Use the address of the running Hickory backend. On iPhone or iPad, use a host reachable from that device.").font(.caption)
                Text(workspace.status).font(.caption).textSelection(.enabled)
                HStack {
                    Button("Cancel") { connection = false }
                    Spacer()
                    Button("Connect") { Task { if await workspace.connect() { connection = false } } }.disabled(workspace.connecting)
                }
            }.padding().frame(minWidth: 280, idealWidth: 420)
        }
        .onReceive(NotificationCenter.default.publisher(for: .simpleConnect)) { _ in connection = true }
        .task {
            if workspace.engine == nil {
                if workspace.address.isEmpty { connection = true }
                else { connection = !(await workspace.connect()) }
            }
        }
        #if os(macOS)
        .background(WindowGuard(workspace: workspace))
        #endif
    }
}
struct PaneMenu: View {
    var workspace: Workspace
    var body: some View {
        Menu("View") {
            Button("Files") { workspace.show("files") }
            Button("Agent") { workspace.show("agent") }
            Button("Terminal") { Task { await workspace.openTerminal() } }
            Button("Split right") { workspace.split("horizontal") }
            Button("Split below") { workspace.split("vertical") }
            Button("Move tab to next group") { workspace.moveToNext() }.disabled(workspace.active == nil || workspace.layout.groups().count < 2)
        }
    }
}
extension Notification.Name { static let simpleConnect = Notification.Name("hickory.simple.connect") }
struct PaneView: View {
    var workspace: Workspace
    let id: String
    @State private var initialRatio: Double?
    @State private var dropBefore: String?
    @State private var paneEdge: String?
    @State private var tabFrames: [String: CGRect] = [:]
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
                            ForEach(node.tabs, id: \.self) { tab in
                                HStack(spacing: 0) {
                                    #if os(macOS)
                                    MacTabLabel(tab: tab, label: workspace.label(tab), activate: {
                                        workspace.layout.activate(tab, in: id); workspace.focused = id
                                    }, dragging: { workspace.draggingTab = $0 }).fixedSize(horizontal: true, vertical: false).frame(height: 28)
                                    #else
                                    Button(workspace.label(tab)) { workspace.layout.activate(tab, in: id); workspace.focused = id }
                                        .accessibilityAddTraits(node.active == tab ? .isSelected : [])
                                    #endif
                                    Button("×") { workspace.close(tab) }.accessibilityLabel("Close \(workspace.label(tab))")
                                }.padding(.horizontal, 5).frame(height: 28)
                                    .background(node.active == tab ? Color.accentColor.opacity(0.12) : Color.clear)
                                    .overlay(alignment: .top) { if node.active == tab { Rectangle().fill(Color.accentColor).frame(height: 2) } }
                                    .overlay(alignment: .leading) { if dropBefore == tab { Rectangle().fill(Color.accentColor).frame(width: 2) } }
                                    #if os(macOS)
                                    .background(GeometryReader { geometry in
                                        Color.clear.preference(key: TabFramePreference.self, value: [tab: geometry.frame(in: .named("tabs-" + id))])
                                    })
                                    #else
                                    .draggable(TabDrag(tab: tab))
                                    .dropDestination(for: TabDrag.self) { tabs, _ in
                                        dropBefore = nil
                                        guard let dragged = tabs.first?.tab, workspace.layout.groups().contains(where: { $0.tabs.contains(dragged) }) else { return false }
                                        workspace.layout.move(dragged, to: id, before: tab); workspace.focused = id; return true
                                    } isTargeted: { targeted in
                                        if targeted { dropBefore = tab } else if dropBefore == tab { dropBefore = nil }
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
                                    dropBefore = point.flatMap { point in node.tabs.first { tabFrames[$0]?.contains(point) == true } }
                                }, drop: { tab, point in
                                    let before = node.tabs.first { tabFrames[$0]?.contains(point) == true }
                                    workspace.layout.move(tab, to: id, before: before); workspace.focused = id
                                    workspace.draggingTab = nil; dropBefore = nil; return true
                                })
                        }
                        #else
                        .dropDestination(for: TabDrag.self) { tabs, _ in
                            guard let tab = tabs.first?.tab, workspace.layout.groups().contains(where: { $0.tabs.contains(tab) }) else { return false }
                            dropBefore = nil
                            workspace.layout.move(tab, to: id); workspace.focused = id; return true
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
                                        guard workspace.layout.groups().contains(where: { $0.id == id }), workspace.layout.groups().contains(where: { $0.tabs.contains(tab) }) else { return false }
                                        if let edge = dropEdge(point.x / max(1, geometry.size.width), point.y / max(1, geometry.size.height)) {
                                            guard let focused = workspace.layout.dock(tab, to: id, edge: edge, newID: UUID().uuidString) else { return false }
                                            workspace.focused = focused
                                        } else { workspace.layout.move(tab, to: id); workspace.focused = id }
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
                                        .allowsHitTesting(false)
                                }
                            }
                    }
                }.buttonStyle(.plain).clipped().onChange(of: workspace.draggingTab) { _, tab in
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
        if tab == "files" {
            ScrollView([.horizontal, .vertical]) {
                VStack(alignment: .leading, spacing: 0) { FileRows(nodes: workspace.files, workspace: workspace) }.padding(5).frame(maxWidth: .infinity, alignment: .leading)
            }.accessibilityLabel("Files")
        } else if tab == "terminal" {
            TerminalPane(engine: workspace.engine, session: workspace.terminal)
        } else if tab == "agent" {
            AgentPane(engine: workspace.engine, session: workspace.agent)
        } else if let tab, let buffer = workspace.buffers[tab] {
            VStack(spacing: 0) {
                LanguageEditor(buffer: buffer).id(buffer.id)
                if !buffer.error.isEmpty { Text(buffer.error).font(.caption).foregroundStyle(.red).padding(6).textSelection(.enabled) }
            }
        } else {
            Text("Open a file or move a tab here").foregroundStyle(.secondary).font(.callout)
        }
    }
}
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
                guard workspace.layout.groups().contains(where: { $0.id == group }), workspace.layout.groups().contains(where: { $0.tabs.contains(tab) }) else { return }
                if selected == "center" { workspace.layout.move(tab, to: group); workspace.focused = group }
                else if let focused = workspace.layout.dock(tab, to: group, edge: selected, newID: UUID().uuidString) { workspace.focused = focused }
            }
        }
        return true
    }
}
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
#if os(macOS)
import AppKit
private struct TabFramePreference: PreferenceKey {
    static var defaultValue: [String: CGRect] = [:]
    static func reduce(value: inout [String: CGRect], nextValue: () -> [String: CGRect]) { value.merge(nextValue(), uniquingKeysWith: { _, new in new }) }
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
