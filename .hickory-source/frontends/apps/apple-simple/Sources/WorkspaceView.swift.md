<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Render recursive panes with fixed tab strips

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

The same recursive split model works on Mac, iPhone and iPad. On narrow screens groups retain their commands; proportions and split directions remain user-controlled. Draggable tab identities and a menu alternative both move the existing buffer.

The first view task connects to a launcher-supplied address. With no address, or if that connection fails, show the existing connection sheet; do not reconnect an already attached workspace when views reappear.

<ui:copy id="simple-80f4e5a1a0-0">
import SwiftUI

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
            PaneView(workspace: workspace, id: "root")
            Divider()
            HStack {
                Text(workspace.status).lineLimit(1).help(workspace.status)
                Spacer(minLength: 8)
                if let buffer = workspace.currentBuffer {
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
</ui:copy>

A split reserves five points for its resize handle. A leaf owns a fixed-height tab strip and an expanding content rectangle. Menus and popovers do not change neighboring content geometry.

<ui:copy id="simple-80f4e5a1a0-1">
struct PaneView: View {
    var workspace: Workspace
    let id: String
    @State private var initialRatio: Double?
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
                                    Button(workspace.label(tab)) { workspace.layout.activate(tab, in: id); workspace.focused = id }
                                        .accessibilityAddTraits(node.active == tab ? .isSelected : [])
                                    Button("×") { workspace.close(tab) }.accessibilityLabel("Close \(workspace.label(tab))")
                                }.padding(.horizontal, 5).frame(height: 28)
                                    .background(node.active == tab ? Color.accentColor.opacity(0.12) : Color.clear)
                                    .overlay(alignment: .top) { if node.active == tab { Rectangle().fill(Color.accentColor).frame(height: 2) } }
                                    .draggable(tab)
                            }
                        }
                    }.frame(height: 29).background(.quaternary.opacity(0.4))
                        .dropDestination(for: String.self) { tabs, _ in
                            guard let tab = tabs.first else { return false }
                            workspace.layout.move(tab, to: id); workspace.focused = id; return true
                        }
                    Divider()
                    content(node.active).frame(maxWidth: .infinity, maxHeight: .infinity)
                        .contentShape(Rectangle()).simultaneousGesture(TapGesture().onEnded { workspace.focused = id })
                }.buttonStyle(.plain).clipped()
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

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/apple-simple/Sources/WorkspaceView.swift"><ui:paste select="#simple-80f4e5a1a0-0"/><ui:paste select="#simple-80f4e5a1a0-1"/></ui:file>

</ui:doc>
