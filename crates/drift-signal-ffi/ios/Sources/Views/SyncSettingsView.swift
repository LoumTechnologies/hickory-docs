import SwiftUI

// MARK: - VSLICE: DriftSignalSync @Views

struct SyncSettingsView: View {
    let engine: FfiDriftSignalEngine

    @State private var connectionStatus = "offline"
    @State private var nodeId: String?
    @State private var linkToken: String?
    @State private var isSyncing = false
    @State private var errorMessage: String?

    var body: some View {
        List {
            Section("Connection") {
                HStack {
                    Text("Status")
                    Spacer()
                    Text(connectionStatus)
                        .foregroundStyle(statusColor)
                }

                if let nodeId {
                    HStack {
                        Text("Node ID")
                        Spacer()
                        Text(String(nodeId.prefix(12)) + "...")
                            .font(.caption)
                            .monospaced()
                    }
                }

                if isSyncing {
                    Button("Stop Sync", role: .destructive) {
                        stopSync()
                    }
                } else {
                    Button("Start Sync") {
                        startSync()
                    }
                }
            }

            Section("Device Linking") {
                if let linkToken {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Scan this on your other device:")
                            .font(.caption)
                        Text(linkToken)
                            .font(.caption2)
                            .monospaced()
                            .textSelection(.enabled)
                    }
                }

                Button("Generate Link Token") {
                    generateLinkToken()
                }
            }

            if let errorMessage {
                Section {
                    Text(errorMessage)
                        .foregroundStyle(.red)
                        .font(.caption)
                }
            }
        }
        .navigationTitle("Sync")
        .onAppear { refreshStatus() }
    }

    private var statusColor: Color {
        switch connectionStatus {
        case "connected": .green
        case "connecting", "reconnecting": .orange
        default: .secondary
        }
    }

    private func refreshStatus() {
        do {
            connectionStatus = try engine.connectionStatus()
            nodeId = try engine.nodeId()
            isSyncing = connectionStatus != "offline"
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    private func startSync() {
        DispatchQueue.global(qos: .userInitiated).async {
            do {
                try engine.startSync(relayUrl: nil)
                DispatchQueue.main.async { refreshStatus() }
            } catch {
                DispatchQueue.main.async {
                    errorMessage = error.localizedDescription
                }
            }
        }
    }

    private func stopSync() {
        DispatchQueue.global(qos: .userInitiated).async {
            do {
                try engine.stopSync()
                DispatchQueue.main.async { refreshStatus() }
            } catch {
                DispatchQueue.main.async {
                    errorMessage = error.localizedDescription
                }
            }
        }
    }

    private func generateLinkToken() {
        do {
            linkToken = try engine.generateLinkToken()
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}
