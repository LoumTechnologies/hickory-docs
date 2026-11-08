<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Use the existing engine protocol from Swift

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Decode only the fields this frontend needs. The address is explicit runtime configuration, validated once. Requests have a timeout and never retry a mutation after an ambiguous failure. The browser uses the existing generated TypeScript client for these same routes.

Read managed Markdown through its document ID, using the source response without requesting rendering. Plain text retains the hash-based file endpoint. Document saves continue through the room-aware document endpoint with the loaded source baseline.

<ui:copy id="simple-ff9425b170-0">
import Foundation

struct FileNode: Codable, Identifiable {
    var name: String
    var path: String
    var dir: Bool
    var children: [FileNode]?
    var doc_id: String?
    var generated_by: String?
    var id: String { path }
}
struct FileTree: Decodable { var root: String; var tree: [FileNode] }
struct FileText: Decodable { var path: String; var content: String; var hash: String }
struct FileSaved: Decodable { var path: String; var hash: String }
struct SavedDocument: Decodable { var source: String }
struct AgentTurn: Decodable, Identifiable {
    var id: String
    var prompt: String
    var answer: String?
    var error: String?
    var status: String
}
struct AgentHistory: Decodable { var turns: [AgentTurn] }
struct TerminalOpened: Decodable { var id: String }
struct TerminalPage: Decodable { var bytes: [UInt8]; var next: Int?; var revision: String }
struct TerminalOK: Decodable { var ok: Bool }
struct StartedTurn: Decodable { var session_id: String }
struct EngineError: LocalizedError {
    var message: String
    var errorDescription: String? { message }
}
struct Engine {
    let origin: URL
    var session: URLSession = .shared
    init(address: String, session: URLSession = .shared) throws {
        guard let url = URL(string: address), ["http", "https"].contains(url.scheme), url.host != nil,
              url.user == nil, url.password == nil, url.query == nil, url.fragment == nil,
              url.path.isEmpty || url.path == "/" else {
            throw EngineError(message: "Enter the workspace server's http:// or https:// address, without a path or password.")
        }
        origin = url; self.session = session
    }
    func request<T: Decodable>(_ method: String, _ path: String, query: [URLQueryItem] = [], body: [String: Any]? = nil) async throws -> T {
        var components = URLComponents(url: origin, resolvingAgainstBaseURL: false)!
        components.path = path; components.queryItems = query.isEmpty ? nil : query
        var request = URLRequest(url: components.url!); request.httpMethod = method; request.timeoutInterval = 30
        if let body { request.httpBody = try JSONSerialization.data(withJSONObject: body); request.setValue("application/json", forHTTPHeaderField: "Content-Type") }
        let (data, response) = try await session.data(for: request)
        guard let response = response as? HTTPURLResponse else { throw EngineError(message: "No server response. Check the workspace address and reconnect.") }
        guard (200..<300).contains(response.statusCode) else {
            let object = try? JSONSerialization.jsonObject(with: data) as? [String: Any]
            let detail = object?["error"] as? String ?? String(data: data.prefix(600), encoding: .utf8) ?? "Request refused"
            throw EngineError(message: "\(detail) (HTTP \(response.statusCode)). Check the server and retry after reviewing its current state.")
        }
        return try JSONDecoder().decode(T.self, from: data)
    }
</ui:copy>

Save requests carry the exact baseline from the original read. Agent sends select the Python backend explicitly, continue the last observed turn and never grant implicit editor write authority.

<ui:copy id="simple-ff9425b170-1">
    func files() async throws -> FileTree { try await request("GET", "/api/files") }
    func file(_ file: FileNode) async throws -> FileText {
        if let id = file.doc_id {
            let document: SavedDocument = try await request("GET", "/api/docs/" + id)
            return FileText(path: file.path, content: document.source, hash: "")
        }
        return try await request("GET", "/api/file", query: [URLQueryItem(name: "path", value: file.path)])
    }
    @MainActor func save(_ buffer: DocumentBuffer, text: String, base: String, hash: String) async throws -> String {
        if let id = buffer.file.doc_id {
            let _: SavedDocument = try await request("PUT", "/api/docs/" + id, body: ["source": text, "expected": ["source": base, "disk": base]])
            return hash
        }
        let result: FileSaved = try await request("PUT", "/api/file", body: ["path": buffer.file.path, "content": text, "base_hash": hash])
        return result.hash
    }
    func turns() async throws -> AgentHistory { try await request("GET", "/api/docs/workspace/agent/turns") }
    func send(_ prompt: String, parent: String?) async throws -> StartedTurn {
        try await request("POST", "/api/docs/workspace/agent", body: ["prompt": prompt, "parent_id": parent as Any? ?? NSNull(), "backend": "python"])
    }
    func stop() async throws {
        struct Stopped: Decodable { var stopping: String }
        let _: Stopped = try await request("POST", "/api/docs/workspace/agent/stop", body: [:])
    }
    func openTerminal() async throws -> TerminalOpened { try await request("POST", "/api/terminals", body: ["title": "Terminal"]) }
    func terminalInput(_ id: String, data: String) async throws {
        let _: TerminalOK = try await request("POST", "/api/terminals/" + id + "/input", body: ["data": data])
    }
    func terminalResize(_ id: String, rows: Int, cols: Int) async throws {
        let _: TerminalOK = try await request("POST", "/api/terminals/" + id + "/resize", body: ["rows": rows, "cols": cols])
    }
    func terminalScreen(_ id: String) async throws -> String {
        var bytes: [UInt8] = [], offset = 0, revision = ""
        while true {
            let page: TerminalPage = try await request("GET", "/api/terminals/" + id + "/output", query: [
                URLQueryItem(name: "view", value: "screen"), URLQueryItem(name: "offset", value: String(offset)),
                URLQueryItem(name: "limit", value: "16384"), URLQueryItem(name: "revision", value: revision)
            ])
            bytes += page.bytes; revision = page.revision
            guard let next = page.next else { return String(decoding: bytes, as: UTF8.self) }
            offset = next
        }
    }

}
</ui:copy>

## Attach language services after readable text is available

Each document owns a read-only language channel over the existing backend WebSocket framing. Channel 2 carries JSON-RPC; the backend owns initialize and child-server discovery. Serialize outgoing notifications and requests so a completion cannot overtake its full-text synchronization. Source snapshots and selection positions guard every response and insertion. A failed or missing server reports its state without blocking text, saving or pane movement.

Semantic-token ranges use UTF-16, including surrogate pairs. Decode against the server's advertised legend and reject ranges outside the actual line rather than painting unrelated text. Completion items requiring snippets or additional edits are excluded from this first native slice; accepting a simple item still goes through the native editor's permission, source and selection checks.

<ui:copy id="simple-apple-language-transport">
import Observation

struct NativeSemanticToken { var range: NSRange; var kind: String }
struct NativeCompletion: Identifiable {
    var id: Int
    var label: String
    var detail: String
    var range: NSRange
    var insert: String
    var source: String
    var selection: NSRange
}
func languageURI(_ path: String) -> String {
    var parts = URLComponents(); parts.scheme = "hick"; parts.host = ""; parts.path = "/" + path.trimmingCharacters(in: CharacterSet(charactersIn: "/"))
    return parts.url!.absoluteString
}
func languageOffset(_ source: String, _ position: [String: Any]) -> Int? {
    guard let line = position["line"] as? Int, let character = position["character"] as? Int, line >= 0, character >= 0 else { return nil }
    let lines = source.components(separatedBy: "\n")
    guard line < lines.count, character <= lines[line].utf16.count else { return nil }
    return lines.prefix(line).reduce(0) { $0 + $1.utf16.count + 1 } + character
}
func languagePosition(_ source: String, _ offset: Int) -> [String: Int] {
    let text = source as NSString, end = min(text.length, max(0, offset))
    var line = 0, start = 0
    for index in 0..<end where text.character(at: index) == 10 { line += 1; start = index + 1 }
    return ["line": line, "character": end - start]
}
func decodeNativeTokens(_ data: [Int], legend: [String], source: String) -> [NativeSemanticToken] {
    guard data.count % 5 == 0 else { return [] }
    let lines = source.components(separatedBy: "\n")
    var starts = [0]
    for line in lines.dropLast() { starts.append(starts.last! + line.utf16.count + 1) }
    var row = 0, column = 0, result: [NativeSemanticToken] = []
    for index in stride(from: 0, to: data.count, by: 5) {
        let delta = data[index], start = data[index + 1], length = data[index + 2], type = data[index + 3]
        guard delta >= 0, start >= 0, length > 0, delta <= lines.count, start <= source.utf16.count else { return [] }
        row += delta; column = delta == 0 ? column + start : start
        guard row < lines.count, column <= lines[row].utf16.count, length <= lines[row].utf16.count - column,
              type >= 0, type < legend.count else { continue }
        result.append(NativeSemanticToken(range: NSRange(location: starts[row] + column, length: length), kind: legend[type]))
    }
    return result
}
</ui:copy>

## Own the optional socket and pending requests

The session opens only after the buffer exists. Closing or reconnecting retires its epoch, cancels background tasks and fails pending requests; an old socket cannot publish into the replacement session. The server capability announcement supplies the semantic legend. Outgoing frames stay ordered on channel 2, while other backend channels are ignored.

<ui:copy id="simple-apple-language-session">
@MainActor @Observable final class NativeLanguage {
    var status = "Language server connecting…"
    var tokens: [NativeSemanticToken] = []
    var suggestions: [NativeCompletion] = []
    var showingSuggestions = false
    private weak var buffer: DocumentBuffer?
    private let engine: Engine
    private let uri: String
    private var socket: URLSessionWebSocketTask?
    private var receiveTask: Task<Void, Never>?
    private var sendTask: Task<Void, Never>?
    private var refreshTask: Task<Void, Never>?
    private var pending: [Int: CheckedContinuation<Any, Error>] = [:]
    private var nextID = 1
    private var version = 1
    private var sent = ""
    private var legend: [String] = []
    private var epoch = UUID()
    init(engine: Engine, buffer: DocumentBuffer) { self.engine = engine; self.buffer = buffer; uri = languageURI(buffer.file.path) }
    func start() {
        close(); epoch = UUID(); let current = epoch
        var url = URLComponents(url: engine.origin, resolvingAgainstBaseURL: false)!
        url.scheme = engine.origin.scheme == "https" ? "wss" : "ws"; url.path = "/api/ws"; url.queryItems = [URLQueryItem(name: "doc", value: "workspace")]
        let socket = engine.session.webSocketTask(with: url.url!); self.socket = socket
        status = "Language server connecting…"; socket.resume()
        receiveTask = Task { [weak self] in
            do {
                while !Task.isCancelled {
                    let message = try await socket.receive()
                    guard let self, self.epoch == current else { return }
                    if case .data(let bytes) = message, bytes.first == 2,
                       let object = try JSONSerialization.jsonObject(with: bytes.dropFirst()) as? [String: Any] { self.receive(object) }
                }
            } catch { if let self, self.epoch == current, !Task.isCancelled { self.failed(error) } }
        }
        sent = buffer?.text ?? ""; version = 1
        notify("textDocument/didOpen", ["textDocument": ["uri": uri, "languageId": "hick", "version": version, "text": sent]])
        changed()
    }
    func close() {
        epoch = UUID(); receiveTask?.cancel(); refreshTask?.cancel(); sendTask?.cancel()
        socket?.cancel(with: .normalClosure, reason: nil); socket = nil
        let waits = pending; pending.removeAll(); for continuation in waits.values { continuation.resume(throwing: CancellationError()) }
    }
    private func failed(_ error: Error) {
        status = "Language services unavailable: \(error.localizedDescription). Reconnect or retry."
        let waits = pending; pending.removeAll(); for continuation in waits.values { continuation.resume(throwing: error) }
    }
    private func receive(_ object: [String: Any]) {
        if let id = object["id"] as? Int, let continuation = pending.removeValue(forKey: id) {
            if let error = object["error"] as? [String: Any] { continuation.resume(throwing: EngineError(message: error["message"] as? String ?? "Language request refused")) }
            else { continuation.resume(returning: object["result"] ?? NSNull()) }
        } else if object["method"] as? String == "hick/serverCapabilities",
                  let params = object["params"] as? [String: Any], let caps = params["capabilities"] as? [String: Any] {
            legend = ((caps["semanticTokensProvider"] as? [String: Any])?["legend"] as? [String: Any])?["tokenTypes"] as? [String] ?? []
            status = "LSP connected; checking semantic colors…"; changed()
        }
    }
    private func send(_ message: [String: Any]) {
        guard let socket, let bytes = try? JSONSerialization.data(withJSONObject: message) else { return }
        let previous = sendTask, current = epoch
        sendTask = Task { [weak self] in
            await previous?.value
            guard !Task.isCancelled else { return }
            do { try await socket.send(.data(Data([2]) + bytes)) }
            catch { if let self, self.epoch == current { self.failed(error) } }
        }
    }
    private func notify(_ method: String, _ params: [String: Any]) { send(["jsonrpc": "2.0", "method": method, "params": params]) }
    private func request(_ method: String, _ params: [String: Any]) async throws -> Any {
        guard socket != nil else { throw EngineError(message: "Language connection is closed. Retry language services.") }
        let id = nextID; nextID += 1
        return try await withCheckedThrowingContinuation { continuation in
            pending[id] = continuation; send(["jsonrpc": "2.0", "id": id, "method": method, "params": params])
            Task { [weak self] in
                try? await Task.sleep(for: .seconds(12))
                self?.pending.removeValue(forKey: id)?.resume(throwing: EngineError(message: "Language request timed out. Check the server installation and retry."))
            }
        }
    }
</ui:copy>

## Apply results only to the source and selection that requested them

Typing clears old colors and schedules a quiet-period refresh. Providers can start late, so semantic reads retry a bounded number of times without delaying text opening. Completion requests are debounced after synchronization, filter unsupported edit shapes and retain their exact source/selection snapshot. The native edit callback makes the final permission and revision check. No language result saves a file.

<ui:copy id="simple-apple-language-snapshots">
    private func sync() {
        guard let text = buffer?.text, text != sent else { return }
        sent = text; version += 1
        notify("textDocument/didChange", ["textDocument": ["uri": uri, "version": version], "contentChanges": [["text": text]]])
    }
    func changed() {
        refreshTask?.cancel(); tokens = []; suggestions = []
        if let buffer { buffer.renderLanguage?([], buffer.text) }
        let current = epoch
        refreshTask = Task { [weak self] in
            try? await Task.sleep(for: .milliseconds(300))
            guard let self, !Task.isCancelled, self.epoch == current else { return }
            self.sync()
            guard let source = self.buffer?.text else { return }
            for attempt in 0..<5 {
                guard !Task.isCancelled, self.buffer?.text == source, self.epoch == current else { return }
                do {
                    let result = try await self.request("textDocument/semanticTokens/full", ["textDocument": ["uri": self.uri]]) as? [String: Any]
                    guard !Task.isCancelled, self.buffer?.text == source, self.epoch == current else { return }
                    self.tokens = decodeNativeTokens(result?["data"] as? [Int] ?? [], legend: self.legend, source: source)
                    self.buffer?.renderLanguage?(self.tokens, source)
                    self.status = self.tokens.isEmpty ? "LSP: no semantic tokens. Check language-server installation." : "LSP · \(self.tokens.count) semantic spans"
                    if !self.tokens.isEmpty || attempt == 4 { return }
                } catch { if !Task.isCancelled, self.epoch == current { self.status = "Language colors unavailable: \(error.localizedDescription)" }; return }
                try? await Task.sleep(for: .milliseconds(700))
            }
        }
    }
    func complete() async {
        showingSuggestions = true; suggestions = []
        guard let buffer, !buffer.readOnly else { return }
        sync(); let source = buffer.text, selection = buffer.selection, current = epoch
        status = "Requesting LSP completion…"
        do {
            // The bridge prepares virtual files asynchronously after full-text sync.
            // Keep the popup responsive while typing settles; never use an obsolete position.
            try await Task.sleep(for: .milliseconds(350))
            guard self.buffer?.text == source, self.buffer?.selection == selection, epoch == current else { return }
            let result = try await request("textDocument/completion", ["textDocument": ["uri": uri], "position": languagePosition(source, selection.location)])
            guard self.buffer?.text == source, self.buffer?.selection == selection, epoch == current else { return }
            let items = result as? [[String: Any]] ?? (result as? [String: Any])?["items"] as? [[String: Any]] ?? []
            let ns = source as NSString
            var prefix = min(selection.location, ns.length)
            while prefix > 0, let scalar = UnicodeScalar(ns.character(at: prefix - 1)), CharacterSet.alphanumerics.union(CharacterSet(charactersIn: "_")).contains(scalar) { prefix -= 1 }
            let typed = ns.substring(with: NSRange(location: prefix, length: max(0, selection.location - prefix))).lowercased()
            for (index, item) in items.enumerated() {
                guard let label = item["label"] as? String, label.lowercased().hasPrefix(typed), item["insertTextFormat"] as? Int != 2,
                      (item["additionalTextEdits"] as? [Any] ?? []).isEmpty else { continue }
                var range = NSRange(location: prefix, length: NSMaxRange(selection) - prefix)
                var insert = item["insertText"] as? String ?? label
                if let edit = item["textEdit"] as? [String: Any] {
                    guard let raw = edit["range"] as? [String: Any], let start = raw["start"] as? [String: Any], let end = raw["end"] as? [String: Any],
                          let from = languageOffset(source, start), let to = languageOffset(source, end), from <= to, let replacement = edit["newText"] as? String else { continue }
                    range = NSRange(location: from, length: to - from); insert = replacement
                }
                suggestions.append(NativeCompletion(id: index, label: label, detail: item["detail"] as? String ?? "Language server", range: range, insert: insert, source: source, selection: selection))
                if suggestions.count >= 100 { break }
            }
            status = suggestions.isEmpty ? "LSP returned no simple completions here. Check cursor and server installation." : "LSP · \(suggestions.count) completions"
        } catch { if epoch == current { status = "Completion unavailable: \(error.localizedDescription)" } }
    }
    func accept(_ completion: NativeCompletion) {
        guard let buffer, !buffer.readOnly, buffer.text == completion.source, buffer.selection == completion.selection,
              buffer.applyLanguageCompletion?(completion.range, completion.insert, completion.source, completion.selection) == true else {
            status = "Suggestion is out of date. Request completions again."; return
        }
        showingSuggestions = false
    }
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/apple-simple/Sources/Engine.swift"><ui:paste select="#simple-ff9425b170-0"/><ui:paste select="#simple-ff9425b170-1"/><ui:paste select="#simple-apple-language-transport"/><ui:paste select="#simple-apple-language-session"/><ui:paste select="#simple-apple-language-snapshots"/></ui:file>

</ui:doc>
