import SwiftUI

@main struct HickorySimpleApp: App {
    @State private var fallbackWorkspace = Workspace()
    @FocusedValue(\.simpleWorkspace) private var focusedWorkspace
    private var workspace: Workspace { focusedWorkspace ?? fallbackWorkspace }
    @Environment(\.openWindow) private var openWindow
    #if os(macOS)
    @NSApplicationDelegateAdaptor(ApplicationDelegate.self) private var delegate
    #endif
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
                Button("Connect…") { NotificationCenter.default.post(name: .simpleConnect, object: nil) }
                Button("Open File") { workspace.chooseFileCommand("Open File") }.keyboardShortcut("o")
                Button("Open Directory") { workspace.chooseFileCommand("Open Directory") }.keyboardShortcut("o", modifiers: [.command, .shift])
                Button("New Tab") { workspace.newTab() }.keyboardShortcut("n")
                Button("New Window") { openWindow(id: "workspace") }.keyboardShortcut("n", modifiers: [.command, .shift])
            }
            CommandGroup(replacing: .saveItem) {
                Button("Save") { Task { await workspace.save() } }.keyboardShortcut("s").disabled(workspace.currentBuffer == nil || workspace.currentBuffer?.readOnly == true)
                Button("Save As") { workspace.chooseFileCommand("Save As") }.keyboardShortcut("s", modifiers: [.command, .shift]).disabled(workspace.currentBuffer == nil)
                Button("Save All") { Task { await workspace.saveAll() } }
                Button("Refresh") { Task { await workspace.refresh() } }
                Button("Close tab") { if let active = workspace.active { workspace.close(active) } }.keyboardShortcut("w")
            }
            CommandGroup(after: .textEditing) {
                EditorNavigationSettings()
                Button("Find and replace") { workspace.currentBuffer?.showFind = true }.keyboardShortcut("f").disabled(workspace.currentBuffer == nil)
                Button("Language tools") {
                    Task { await workspace.currentBuffer?.language?.inspect() }
                    Task { await workspace.currentBuffer?.language?.complete() }
                }.keyboardShortcut(.space, modifiers: .control).disabled(workspace.currentBuffer == nil)
                Button("Column selection") { workspace.currentBuffer?.selectRectangle() }.disabled(workspace.currentBuffer == nil)
                Button("Add cursor below") { workspace.currentBuffer?.addCursor(1) }.keyboardShortcut(.downArrow, modifiers: [.command, .option]).disabled(workspace.currentBuffer == nil)
                Button("Add cursor above") { workspace.currentBuffer?.addCursor(-1) }.keyboardShortcut(.upArrow, modifiers: [.command, .option]).disabled(workspace.currentBuffer == nil)
            }
            CommandMenu("Panes") {
                Button("Files") { workspace.show("files") }; Button("Agent") { workspace.show("agent") }; Button("Terminal") { Task { await workspace.openTerminal() } }
                Button("Split right") { workspace.split("horizontal") }; Button("Split below") { workspace.split("vertical") }
                Button("Move tab to next group") { workspace.moveToNext() }
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
