import SwiftUI

// MARK: - VSLICE: DriftSignal @App

/// Observable app state holding the FFI engine instance.
///
/// Engine initialization (SQLite setup, sub-app registration) runs on a
/// background thread so the main thread stays responsive.
final class AppState: ObservableObject {
    @Published var engine: FfiDriftSignalEngine?
    @Published var initError: String?

    init() {}

    func bootstrap(dbPath: String, userId: String, deviceId: String) {
        DispatchQueue.global(qos: .userInitiated).async { [weak self] in
            do {
                let eng = try FfiDriftSignalEngine.open(
                    dbPath: dbPath,
                    userId: userId,
                    deviceId: deviceId
                )
                DispatchQueue.main.async {
                    self?.engine = eng
                }
            } catch {
                DispatchQueue.main.async {
                    self?.initError = error.localizedDescription
                }
            }
        }
    }
}
