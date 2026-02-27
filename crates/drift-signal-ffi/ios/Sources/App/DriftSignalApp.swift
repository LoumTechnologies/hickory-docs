import SwiftUI

// MARK: - VSLICE: DriftSignal @App

@main
struct DriftSignalApp: App {
    @StateObject private var appState = AppState()

    init() {
        #if DEBUG
        initLogging(sink: DriftSignalLogSink())
        #endif
    }

    var body: some Scene {
        WindowGroup {
            Group {
                if let error = appState.initError {
                    VStack(spacing: 16) {
                        Image(systemName: "exclamationmark.triangle")
                            .font(.largeTitle)
                            .foregroundColor(.red)
                        Text("Failed to start engine")
                            .font(.headline)
                        Text(error)
                            .font(.caption)
                            .foregroundColor(.secondary)
                            .multilineTextAlignment(.center)
                            .padding(.horizontal)
                    }
                } else if let engine = appState.engine {
                    MainTabView(engine: engine)
                } else {
                    VStack(spacing: 12) {
                        ProgressView()
                        Text("Starting Drift Signal...")
                            .font(.subheadline)
                            .foregroundColor(.secondary)
                    }
                }
            }
            .onAppear {
                guard appState.engine == nil, appState.initError == nil else { return }
                let dbPath = Self.databasePath()
                let deviceId = Self.deviceIdentifier()
                appState.bootstrap(dbPath: dbPath, userId: "local-user", deviceId: deviceId)
            }
            #if os(macOS)
            .frame(minWidth: 800, minHeight: 600)
            #endif
        }
    }

    private static func databasePath() -> String {
        #if os(macOS)
        let dir = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
            .appendingPathComponent("DriftSignal", isDirectory: true)
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir.appendingPathComponent("drift_signal.db").path
        #else
        let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
        return docs.appendingPathComponent("drift_signal.db").path
        #endif
    }

    private static func deviceIdentifier() -> String {
        let key = "dev.drift-signal.device-id"
        if let existing = UserDefaults.standard.string(forKey: key) {
            return existing
        }
        let id = "device-\(UUID().uuidString.lowercased())"
        UserDefaults.standard.set(id, forKey: key)
        return id
    }
}
