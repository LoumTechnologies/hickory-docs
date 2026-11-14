<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Launch independently and protect unsaved edits on Mac

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

The new app has its own product identity and window. A terminal launch must explicitly adopt the regular macOS activation policy before launch finishes, then activate once its workspace window appears. Otherwise the process can finish building and run without a discoverable foreground app. Activation runs at startup, independently of the backend connection. Launch Services owns the app process, so a development-only host PID makes the app stop when its launcher exits. That explicit Ctrl-C teardown ends the session; ordinary app Quit still checks unsaved buffers. Mac commands match the browser menu. Cmd+N creates a tab; Cmd+Shift+N creates a window. Native window close and application termination refuse while any buffer is dirty or saving, rather than losing edits behind a closed tab.

File commands act on the connected workspace: path entry opens a file or directory, New Tab keeps an unnamed buffer, and Save As exclusively creates a new destination before a hash-checked write. Existing destinations are never overwritten by Save As. Save All visits each named buffer; unnamed drafts ask for a path and failed saves retain their original edits. Refresh observes the tree without replacing open buffers.

<ui:copy id="simple-967795f4aa-0">
import SwiftUI

@main struct HickorySimpleApp: App {
    @State private var fallbackWorkspace = Workspace()
    @FocusedValue(\.simpleWorkspace) private var focusedWorkspace
    private var workspace: Workspace { focusedWorkspace ?? fallbackWorkspace }
    @Environment(\.openWindow) private var openWindow
    #if os(macOS)
    @NSApplicationDelegateAdaptor(ApplicationDelegate.self) private var delegate
    #endif
    private var menuCommands:[WorkspaceCommand] {workspace.commands(ui:["file.window":{openWindow(id:"workspace")},"file.connect":{NotificationCenter.default.post(name:.simpleConnect,object:nil)}])}
    var body: some Scene {
        #if os(macOS)
        WindowGroup("Hickory Simple", id: "workspace") {
            SimpleWindowRoot().frame(minWidth: 480, minHeight: 320)
                .onAppear {
                    DispatchQueue.main.async {
                        NSApp.windows.first(where: { $0.canBecomeMain })?.makeKeyAndOrderFront(nil)
                        NSApp.activate()
                    }
                }
        }.commands {
            CommandGroup(replacing: .newItem) {
                ForEach(menuCommands.filter {["file.connect","file.open","file.directory","file.new","file.window"].contains($0.id)}) {WorkspaceCommandButton(command:$0)}
            }
            CommandGroup(replacing: .saveItem) {
                ForEach(menuCommands.filter {["file.save","file.saveAs","file.saveAll","file.refresh","file.close"].contains($0.id)}) {WorkspaceCommandButton(command:$0)}
            }
            CommandGroup(after: .textEditing) {
                EditorNavigationSettings()
                ForEach(menuCommands.filter {$0.group=="Edit"}) {WorkspaceCommandButton(command:$0)}
            }
            CommandMenu("Panes") {
                ForEach(menuCommands.filter {$0.group=="View"}) {WorkspaceCommandButton(command:$0)}
            }
        }
        #else
        WindowGroup(id: "workspace") { SimpleWindowRoot() }
        #endif
    }
}
private struct SimpleWorkspaceKey: FocusedValueKey { typealias Value = Workspace }
extension FocusedValues {
    var simpleWorkspace: Workspace? {
        get { self[SimpleWorkspaceKey.self] }
        set { self[SimpleWorkspaceKey.self] = newValue }
    }
}
@MainActor private enum WorkspaceWindows { static let values = NSHashTable<Workspace>.weakObjects() }
private struct SimpleWindowRoot: View {
    @State private var workspace = Workspace()
    var body: some View {
        WorkspaceView(workspace: workspace).focusedSceneValue(\.simpleWorkspace, workspace)
            .onAppear { WorkspaceWindows.values.add(workspace) }
    }
}
#if os(macOS)
import AppKit
import Darwin
@MainActor final class ApplicationDelegate: NSObject, NSApplicationDelegate {
    private var developmentHostEnded = false
    func applicationWillFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.regular)
    }
    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.activate()
        if let value = ProcessInfo.processInfo.environment["HICKORY_SIMPLE_HOST_PID"],
           let host = Int32(value), host > 1 {
            Task { @MainActor [weak self] in
                while Darwin.kill(host, 0) == 0 || errno == EPERM {
                    try? await Task.sleep(for: .milliseconds(250))
                }
                // Ctrl-C intentionally ends this development session, matching
                // termination of an app supervised as a direct child process.
                self?.developmentHostEnded = true
                NSApp.terminate(nil)
            }
        }
    }
    func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
        if !developmentHostEnded && WorkspaceWindows.values.allObjects.contains(where: { $0.hasUnsavedChanges }) {
            for workspace in WorkspaceWindows.values.allObjects where workspace.hasUnsavedChanges { workspace.status = "Save your documents before quitting." }
            return .terminateCancel
        }
        return .terminateNow
    }
}
struct WindowGuard: NSViewRepresentable {
    let workspace: Workspace
    func makeCoordinator() -> Coordinator { Coordinator(workspace) }
    func makeNSView(context: Context) -> NSView {
        let view = NSView()
        DispatchQueue.main.async { view.window?.delegate = context.coordinator }
        return view
    }
    func updateNSView(_ view: NSView, context: Context) {}
    final class Coordinator: NSObject, NSWindowDelegate {
        let workspace: Workspace
        init(_ workspace: Workspace) { self.workspace = workspace }
        func windowShouldClose(_ sender: NSWindow) -> Bool {
            if workspace.hasUnsavedChanges { workspace.status = "Save your documents before closing this window."; return false }
            return true
        }
    }
}
#endif
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/apple-simple/Sources/HickorySimpleApp.swift"><ui:paste select="#simple-967795f4aa-0"/></ui:file>

</ui:doc>
