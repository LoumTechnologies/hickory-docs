import SwiftUI
import Observation
import WebKit

@MainActor @Observable final class TerminalSession {
    var id: String?
    var opening = false
    var error = ""
    func open(using engine: Engine) async {
        guard id == nil, !opening else { return }
        opening = true; error = ""; defer { opening = false }
        do { id = try await engine.openTerminal().id }
        catch { self.error = "\(error.localizedDescription) Check the terminal backend, then choose Terminal again." }
    }
}
// Only bundled local content can reach this bridge. The socket is native so
// file-origin WebKit content never needs permission to contact workspace APIs.
@MainActor final class TerminalCoordinator: NSObject, WKNavigationDelegate, WKScriptMessageHandler {
    let engine: Engine
    let session: TerminalSession
    let id: String
    var socket: URLSessionWebSocketTask?
    var receiving: Task<Void, Never>?
    var sending: Task<Void, Never>?
    var active = true
    init(engine: Engine, session: TerminalSession, id: String) { self.engine = engine; self.session = session; self.id = id }
    func fail(_ error: Error) {
        guard active else { return }
        session.error = "\(error.localizedDescription) Terminal disconnected. Reopen the tab to reconnect; input was not retried."
        socket?.cancel(with: .goingAway, reason: nil); socket = nil
    }
    func webView(_ view: WKWebView, didFinish navigation: WKNavigation!) {
        guard active, socket == nil else { return }
        var url = URLComponents(url: engine.origin, resolvingAgainstBaseURL: false)!
        url.scheme = url.scheme == "https" ? "wss" : "ws"; url.path = "/api/terminals/ws"
        url.queryItems = [URLQueryItem(name: "session", value: id)]
        let connection = engine.session.webSocketTask(with: url.url!); socket = connection; connection.resume()
        receiving = Task { [weak self, weak view] in
            guard let self, let view else { return }
            do {
                _ = try await view.evaluateJavaScript("globalThis.startInteractiveTerminal(); true")
                while !Task.isCancelled && active {
                    let message = try await connection.receive()
                    guard active else { return }
                    session.error = ""
                    let bytes: Data
                    switch message { case .data(let data): bytes = data; case .string(let text): bytes = Data(text.utf8); @unknown default: continue }
                    _ = try await view.evaluateJavaScript("globalThis.writeInteractiveTerminal('\(bytes.base64EncodedString())'); true")
                }
            } catch { if !Task.isCancelled { fail(error) } }
        }
    }
    func userContentController(_ controller: WKUserContentController, didReceive message: WKScriptMessage) {
        guard active, message.frameInfo.isMainFrame, message.frameInfo.request.url?.isFileURL == true,
              let body = message.body as? [String: Any] else { return }
        if body["kind"] as? String == "input", let data = body["data"] as? String {
            // Each send waits for the previous one. A failed write cancels the
            // connection; queued keys must never be retried on a new session.
            let previous = sending
            sending = Task { [weak self] in
                await previous?.value
                guard let self, active, let socket else { return }
                do { try await socket.send(.string(data)) } catch { fail(error) }
            }
        } else if body["kind"] as? String == "resize", let rows = body["rows"] as? Int, let cols = body["cols"] as? Int,
                  (1...1000).contains(rows), (1...1000).contains(cols) {
            Task { [weak self] in
                guard let self, active else { return }
                do { try await engine.terminalResize(id, rows: rows, cols: cols) }
                catch { if active { session.error = "\(error.localizedDescription) Resize the pane to retry its terminal size." } }
            }
        }
    }
    func webView(_ view: WKWebView, decidePolicyFor action: WKNavigationAction, decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
        decisionHandler(action.request.url?.isFileURL == true ? .allow : .cancel)
    }
    func webViewWebContentProcessDidTerminate(_ webView: WKWebView) {
        fail(EngineError(message: "Terminal renderer stopped. Choose another tab and reopen Terminal."))
    }
    func stop(_ view: WKWebView) {
        active = false; receiving?.cancel(); sending?.cancel(); socket?.cancel(with: .goingAway, reason: nil)
        view.configuration.userContentController.removeScriptMessageHandler(forName: "terminal"); view.stopLoading()
    }
    func makeView() -> WKWebView {
        let configuration = WKWebViewConfiguration(); configuration.websiteDataStore = .nonPersistent()
        configuration.userContentController.add(self, name: "terminal")
        let view = WKWebView(frame: .zero, configuration: configuration); view.navigationDelegate = self
        let owner = Bundle(for: MathCoordinator.self)
        let roots = [Bundle.main.resourceURL, Bundle.main.bundleURL, owner.resourceURL, owner.bundleURL,
            URL(fileURLWithPath: CommandLine.arguments[0]).deletingLastPathComponent()].compactMap { $0 }
        if let url = nativeMathResource(roots: roots) { view.loadFileURL(url, allowingReadAccessTo: url.deletingLastPathComponent()) }
        else { session.error = "Terminal resources are missing. Prepare bundled assets and rebuild the app." }
        return view
    }
}
#if os(macOS)
struct InteractiveTerminal: NSViewRepresentable {
    let engine: Engine; let session: TerminalSession; let id: String
    func makeCoordinator() -> TerminalCoordinator { TerminalCoordinator(engine: engine, session: session, id: id) }
    func makeNSView(context: Context) -> WKWebView { context.coordinator.makeView() }
    func updateNSView(_ view: WKWebView, context: Context) {}
    static func dismantleNSView(_ view: WKWebView, coordinator: TerminalCoordinator) { coordinator.stop(view) }
}
#else
struct InteractiveTerminal: UIViewRepresentable {
    let engine: Engine; let session: TerminalSession; let id: String
    func makeCoordinator() -> TerminalCoordinator { TerminalCoordinator(engine: engine, session: session, id: id) }
    func makeUIView(context: Context) -> WKWebView { context.coordinator.makeView() }
    func updateUIView(_ view: WKWebView, context: Context) {}
    static func dismantleUIView(_ view: WKWebView, coordinator: TerminalCoordinator) { coordinator.stop(view) }
}
#endif
struct TerminalPane: View {
    let engine: Engine?
    @Bindable var session: TerminalSession
    var body: some View {
        VStack(spacing: 0) {
            if let engine, let id = session.id { InteractiveTerminal(engine: engine, session: session, id: id).id(id) }
            else { Text(session.opening ? "Opening Terminal…" : "Choose Terminal to connect.").frame(maxWidth: .infinity, maxHeight: .infinity) }
            if !session.error.isEmpty { Text(session.error).font(.caption).foregroundStyle(.red).padding(6) }
        }
    }
}
