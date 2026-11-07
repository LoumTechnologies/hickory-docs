import SwiftUI
import Observation

@MainActor @Observable final class TerminalSession {
    var id: String?
    var opening = false
    var input = ""
    var screen = ""
    var error = ""
    var sending = false
    func open(using engine: Engine) async {
        guard id == nil, !opening else { return }
        opening = true; error = ""; defer { opening = false }
        do { id = try await engine.openTerminal().id }
        catch { self.error = "\(error.localizedDescription) Check the terminal backend, then choose Terminal again." }
    }
    func send(using engine: Engine, data: String? = nil) async {
        guard let id, !sending else { return }
        let submitted = input
        sending = true; error = ""; defer { sending = false }
        do { try await engine.terminalInput(id, data: data ?? submitted + "\r"); if data == nil && input == submitted { input = "" } }
        catch { self.error = "\(error.localizedDescription) Input was not retried. Check the screen before sending again." }
    }
}
struct TerminalPane: View {
    let engine: Engine?
    @Bindable var session: TerminalSession
    var body: some View {
        VStack(spacing: 0) {
            GeometryReader { geometry in
                ScrollView([.horizontal, .vertical]) {
                    Text(session.screen).font(.system(size: 13, design: .monospaced)).textSelection(.enabled)
                        .frame(maxWidth: .infinity, alignment: .topLeading).padding(6).accessibilityLabel("Terminal screen")
                }.task(id: "\(session.id ?? ""):\(Int(geometry.size.width)):\(Int(geometry.size.height))") {
                    guard let engine, let id = session.id else { return }
                    do { try await engine.terminalResize(id, rows: max(1, min(60, Int(geometry.size.height / 17))), cols: max(1, min(160, Int(geometry.size.width / 8)))) }
                    catch { session.error = "\(error.localizedDescription) Resize the pane to retry its terminal size." }
                }
            }
            if !session.error.isEmpty { Text(session.error).font(.caption).foregroundStyle(.red).padding(6) }
            Divider()
            HStack {
                TextField("Terminal input", text: $session.input).accessibilityLabel("Terminal input").onSubmit { send() }
                Button("Send") { send() }.disabled(session.id == nil || session.sending)
                Button("Interrupt") { send("\u{03}") }.disabled(session.id == nil || session.sending)
                Button("EOF") { send("\u{04}") }.disabled(session.id == nil || session.sending)
            }.padding(6)
        }.task(id: session.id) {
            guard let engine, let id = session.id else { return }
            while !Task.isCancelled {
                do { let screen = try await engine.terminalScreen(id); guard !Task.isCancelled else { return }; if session.id == id { session.screen = screen } }
                catch { if Task.isCancelled { return }; session.error = "\(error.localizedDescription) Reopen Terminal to reconnect to this session." }
                do { try await Task.sleep(for: .milliseconds(250)) } catch { return }
            }
        }
    }
    private func send(_ data: String? = nil) {
        guard let engine else { return }
        Task { await session.send(using: engine, data: data) }
    }
}
