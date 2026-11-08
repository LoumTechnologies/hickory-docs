<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Launch independently and protect unsaved edits on Mac

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

The new app has its own product identity and window. A terminal launch must explicitly adopt the regular macOS activation policy before launch finishes, then activate once its workspace window appears. Otherwise the process can finish building and run without a discoverable foreground app. Activation runs at startup, independently of the backend connection. Launch Services owns the app process, so a development-only host PID makes the app stop when its launcher exits. That explicit Ctrl-C teardown ends the session; ordinary app Quit still checks unsaved buffers. Mac commands match the browser menu. Native window close and application termination refuse while any buffer is dirty or saving, rather than losing edits behind a closed tab.

<ui:copy id="simple-967795f4aa-0">
import SwiftUI

@main struct HickorySimpleApp: App {
    @State private var workspace = Workspace()
    #if os(macOS)
    @NSApplicationDelegateAdaptor(ApplicationDelegate.self) private var delegate
    #endif
    var body: some Scene {
        #if os(macOS)
        Window("Hickory Simple", id: "workspace") {
            WorkspaceView(workspace: workspace).frame(minWidth: 480, minHeight: 320)
                .onAppear {
                    delegate.workspace = workspace
                    DispatchQueue.main.async {
                        NSApp.windows.first(where: { $0.canBecomeMain })?.makeKeyAndOrderFront(nil)
                        NSApp.activate()
                    }
                }
        }.commands {
            CommandGroup(replacing: .newItem) {
                Button("Connect…") { NotificationCenter.default.post(name: .simpleConnect, object: nil) }
                Button("Refresh files") { Task { await workspace.refresh() } }
            }
            CommandGroup(replacing: .saveItem) {
                Button("Save") { Task { await workspace.save() } }.keyboardShortcut("s").disabled(workspace.currentBuffer == nil || workspace.currentBuffer?.readOnly == true)
                Button("Close tab") { if let active = workspace.active { workspace.close(active) } }.keyboardShortcut("w")
            }
            CommandGroup(after: .textEditing) {
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
        WindowGroup { WorkspaceView(workspace: workspace) }
        #endif
    }
}
#if os(macOS)
import AppKit
import Darwin
@MainActor final class ApplicationDelegate: NSObject, NSApplicationDelegate {
    var workspace: Workspace?
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
        if !developmentHostEnded && workspace?.hasUnsavedChanges == true { workspace?.status = "Save your documents before quitting."; return .terminateCancel }
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
