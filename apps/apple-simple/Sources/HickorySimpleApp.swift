import SwiftUI

struct SimpleWindowLaunch: Codable, Hashable, Identifiable {
    var id = UUID()
    var suggestedServer: String? = nil
    var localFolder: URL? = nil
}

@main struct HickorySimpleApp: App {
    @State private var fallbackWorkspace = Workspace()
    @FocusedValue(\.simpleWorkspace) private var focusedWorkspace
    private var workspace: Workspace { focusedWorkspace ?? fallbackWorkspace }
    @Environment(\.openWindow) private var openWindow
    #if os(macOS)
    @Environment(\.dismissWindow) private var dismissWindow
    @NSApplicationDelegateAdaptor(ApplicationDelegate.self) private var delegate
    private static let scaffoldProviders: [any ScaffoldProvider] = [DotnetScaffoldProvider()]
    @FocusedValue(\.simpleOpenDirectory) private var openDirectory
    @FocusedValue(\.simpleNewWindow) private var newWindow
    private var menuCommands:[WorkspaceCommand] {
        var actions: [String: () -> Void] = [
            "file.window": newWindow ?? { openWindow(id: "workspace", value: SimpleWindowLaunch()) },
            "file.connect": { NotificationCenter.default.post(name: .simpleConnect, object: nil) },
        ]
        actions["file.directory"] = openDirectory
        return workspace.commands(ui: actions)
    }
    #endif
    var body: some Scene {
        #if os(macOS)
        WindowGroup("Hickory Docs", id: "workspace", for: SimpleWindowLaunch.self) { $launch in
            SimpleWindowRoot(launch: launch).frame(minWidth: 480, minHeight: 320)
                .onAppear {
                    // A regular startup may originate from the command line.
                    // Let SwiftUI order an explicitly opened scene itself.
                    guard launch == nil else { return }
                    DispatchQueue.main.async {
                        NSApp.windows.first(where: { $0.canBecomeMain })?.makeKeyAndOrderFront(nil)
                        NSApp.activate()
                    }
                }
        }.commands {
            CommandGroup(replacing: .newItem) {
                Menu("New") {
                    ForEach(menuCommands.filter { ["file.new", "file.window"].contains($0.id) }) {
                        WorkspaceCommandButton(command: $0)
                    }
                    Divider()
                    ForEach(Self.scaffoldProviders, id: \.id) { provider in
                        Button(provider.title) { openWindow(id: "scaffold", value: provider.id) }
                    }
                }
                ForEach(menuCommands.filter { ["file.connect", "file.open", "file.directory"].contains($0.id) }) {
                    WorkspaceCommandButton(command: $0)
                }
            }
            CommandGroup(replacing: .saveItem) {
                ForEach(menuCommands.filter {["file.save","file.saveAs","file.saveAll","file.refresh","file.close"].contains($0.id)}) {WorkspaceCommandButton(command:$0)}
            }
            CommandGroup(after: .textEditing) {
                EditorNavigationSettings()
                ForEach(menuCommands.filter {$0.group=="Edit"}) {WorkspaceCommandButton(command:workspace.menuPresentation($0))}
                Button("Insert element…") { workspace.currentBuffer?.openElementForm() }
                    .disabled(workspace.currentBuffer?.canAuthorElements != true)
                    .keyboardShortcut("i", modifiers: .command)
            }
            CommandMenu("Panes") {
                ForEach(menuCommands.filter {$0.group=="View"}) {WorkspaceCommandButton(command:$0)}
                if let focusedWorkspace { WorkspaceStatusMenu(workspace: focusedWorkspace) }
            }
        }
        WindowGroup("New Project", id: "scaffold", for: String.self) { $providerID in
            if let provider = Self.scaffoldProviders.first(where: { $0.id == providerID }) {
                ScaffoldProjectView(provider: provider) { folder in
                    if let folder {
                        if LocalWorkspaceHost.isAvailable {
                            openWindow(id: "workspace", value: SimpleWindowLaunch(localFolder: folder))
                        } else {
                            NSWorkspace.shared.activateFileViewerSelecting([folder])
                        }
                    }
                    dismissWindow(id: "scaffold", value: provider.id)
                }
            }
        }
        .defaultSize(width: 760, height: 580)
        #else
        WindowGroup("Hickory Docs", id: "workspace", for: SimpleWindowLaunch.self) { $launch in
            SimpleWindowRoot(launch: launch)
        }
        #endif
    }
}
private struct SimpleWorkspaceKey: FocusedValueKey { typealias Value = Workspace }
#if os(macOS)
private struct SimpleOpenDirectoryKey: FocusedValueKey { typealias Value = () -> Void }
private struct SimpleNewWindowKey: FocusedValueKey { typealias Value = () -> Void }
#endif
extension FocusedValues {
    #if os(macOS)
    var simpleNewWindow: (() -> Void)? {
        get { self[SimpleNewWindowKey.self] }
        set { self[SimpleNewWindowKey.self] = newValue }
    }
    var simpleOpenDirectory: (() -> Void)? {
        get { self[SimpleOpenDirectoryKey.self] }
        set { self[SimpleOpenDirectoryKey.self] = newValue }
    }
    #endif
    var simpleWorkspace: Workspace? {
        get { self[SimpleWorkspaceKey.self] }
        set { self[SimpleWorkspaceKey.self] = newValue }
    }
}
@MainActor private enum WorkspaceWindows { static let values = NSHashTable<Workspace>.weakObjects() }
struct SimpleWindowRoot: View {
    var launch: SimpleWindowLaunch? = nil
    var closeWindow: (() -> Void)? = nil
    @State private var workspace = Workspace(exclusiveWindow: true)
    var body: some View {
        WorkspaceView(workspace: workspace, launch: launch, closeWindow: closeWindow)
            .focusedSceneValue(\.simpleWorkspace, workspace.windowConnectionLost ? nil : workspace)
            .onAppear { WorkspaceWindows.values.add(workspace) }
    }
}
#if os(macOS)
import AppKit
import Darwin

final class LocalWorkspaceHost {
    @MainActor static var launchLocation: URL? = CommandLine.arguments
        .dropFirst().first.flatMap { $0.hasPrefix("/") ? URL(fileURLWithPath: $0) : nil }

    @MainActor static func takeLaunchLocation() -> URL? {
        defer { launchLocation = nil }
        return launchLocation
    }

    private static var launchCommand: (executable: URL, assets: URL?)? {
        let bundled = Bundle.main.executableURL?.deletingLastPathComponent().appendingPathComponent("hickory-simple-host")
        if let bundled, FileManager.default.isExecutableFile(atPath: bundled.path),
           let resources = Bundle.main.resourceURL {
            return (bundled, resources.appendingPathComponent("HickoryWeb"))
        }
        if let path = ProcessInfo.processInfo.environment["HICKORY_SIMPLE_WORKSPACE_HOST"],
           FileManager.default.isExecutableFile(atPath: path) {
            return (URL(fileURLWithPath: path), nil)
        }
        return nil
    }
    static var isAvailable: Bool { launchCommand != nil }
    private let owner = Pipe()
    private let output = Pipe()
    private let process = Process()

    init(root: URL) throws {
        guard let command = Self.launchCommand else {
            throw EngineError(message: "This build needs a workspace server. Install the packaged Mac app to open local folders.")
        }
        process.executableURL = command.executable
        var arguments = ["--workspace-host", root.path]
        if let assets = command.assets { arguments.append(assets.path) }
        arguments.append("0")
        process.arguments = arguments
        process.standardInput = owner
        process.standardOutput = output
        try process.run()
    }

    private(set) var origin: String?

    func address() async throws -> String {
        let reader = output.fileHandleForReading
        let address = try await Task.detached {
            // The same host serves KDE and WinForms. Its single startup line
            // reports the assigned loopback port only after engine attachment.
            var line = Data()
            while line.count < 64, let byte = try reader.read(upToCount: 1), !byte.isEmpty {
                if byte == Data([10]) { break }
                line.append(byte)
            }
            let fields = String(decoding: line, as: UTF8.self).split(separator: " ")
            guard fields.count == 2, fields[0] == "HICKORY_SIMPLE_PORT",
                  let port = UInt16(fields[1]), port != 0 else {
                throw EngineError(message: "Could not start the local workspace. Check that the folder is readable and try again.")
            }
            return "http://127.0.0.1:\(port)"
        }.value
        origin = address
        return address
    }

    deinit { try? owner.fileHandleForWriting.close() }
}
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
