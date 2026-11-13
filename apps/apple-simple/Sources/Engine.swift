import Foundation
import CryptoKit

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
struct SavedDocument: Decodable { var source: String; var updated_at: String? }
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
    func send(_ prompt: String, parent: String?, provider: String, model: String, effort: String) async throws -> StartedTurn {
        try await request("POST", "/api/docs/workspace/agent", body: ["prompt": prompt, "parent_id": parent as Any? ?? NSNull(), "backend": "python", "provider": provider, "model": model, "effort": effort.isEmpty ? NSNull() : effort as Any])
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
import Observation

struct InlineReceiptEvent: Decodable { var kind: String; var data: String? }
struct InlineReceipt: Decodable, Identifiable {
    var kind: String; var span: [Int]; var command: String?; var status: String?; var transcript: [InlineReceiptEvent]?; var recorded_output: String?
    var id: Int { span.first ?? -1 }
}
struct InlineReceiptPage: Decodable { var document: String; var source_hash: String; var blocks: [InlineReceipt] }
func nativeSourceHash(_ source: String) -> String { SHA256.hash(data: Data(source.utf8)).map { String(format: "%02x", $0) }.joined() }
struct NativeSemanticToken { var range: NSRange; var kind: String }
struct NativeDiagnostic: Identifiable {
    var id: Int; var message: String; var severity: Int; var range: NSRange?
}
func nativeMarkupText(_ value: Any?) -> String {
    if let text = value as? String { return text }
    if let rows = value as? [Any] { return rows.map { nativeMarkupText($0) }.filter { !$0.isEmpty }.joined(separator: "\n\n") }
    if let object = value as? [String: Any] { return object["value"] as? String ?? object["label"] as? String ?? "" }
    return ""
}
func decodeNativeDiagnostics(_ params: [String: Any], source: String, expectedVersion: Int) -> [NativeDiagnostic] {
    // Unversioned diagnostics remain readable messages, never positional authority.
    let located = params["version"] as? Int == expectedVersion
    return (params["diagnostics"] as? [[String: Any]] ?? []).enumerated().map { index, item in
        var range: NSRange?
        if located, let raw = item["range"] as? [String: Any], let start = raw["start"] as? [String: Any], let end = raw["end"] as? [String: Any],
           let from = languageOffset(source, start), let to = languageOffset(source, end), from <= to { range = NSRange(location: from, length: to - from) }
        return NativeDiagnostic(id: index, message: item["message"] as? String ?? "Language diagnostic", severity: item["severity"] as? Int ?? 1, range: range)
    }
}
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
// Rebase display-only spans through the smallest UTF-16 replacement. Discard
// crossed tokens; retain the tentative classification when typing inside one token.
func rebaseNativeTokens(_ tokens: [NativeSemanticToken], from old: String, to new: String) -> [NativeSemanticToken] {
    let before = Array(old.utf16), after = Array(new.utf16)
    var prefix = 0, suffix = 0
    while prefix < min(before.count, after.count), before[prefix] == after[prefix] { prefix += 1 }
    while suffix < min(before.count, after.count) - prefix,
          before[before.count - suffix - 1] == after[after.count - suffix - 1] { suffix += 1 }
    let end = before.count - suffix, shift = after.count - before.count
    return tokens.compactMap { token in
        guard token.range.location >= 0, token.range.length > 0, NSMaxRange(token.range) <= before.count else { return nil }
        if NSMaxRange(token.range) <= prefix { return token }
        if token.range.location <= prefix, NSMaxRange(token.range) >= end,
           token.range.length + shift > 0, !after[prefix..<(after.count - suffix)].contains(10) {
            return NativeSemanticToken(range: NSRange(location: token.range.location, length: token.range.length + shift), kind: token.kind)
        }
        if token.range.location >= end {
            return NativeSemanticToken(range: NSRange(location: token.range.location + shift, length: token.range.length), kind: token.kind)
        }
        return nil
    }
}
@MainActor @Observable final class NativeLanguage {
    var status = "Language server connecting…"
    var tokens: [NativeSemanticToken] = []
    var tokenSource = ""
    var suggestions: [NativeCompletion] = []
    var showingSuggestions = false
    var diagnostics: [NativeDiagnostic] = []
    var diagnosticSource = ""
    var hover = ""
    var signature = ""
    var inlayHints: [String] = []
    var inspectionSource = ""
    var inspectionSelection = NSRange(location: 0, length: 0)
    private var inspectionID = UUID()
    private var completionID = UUID()
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
        diagnostics = []; diagnosticSource = ""; hover = ""; signature = ""; inlayHints = []; inspectionID = UUID()
        epoch = UUID(); receiveTask?.cancel(); refreshTask?.cancel(); sendTask?.cancel()
        socket?.cancel(with: .normalClosure, reason: nil); socket = nil
        let waits = pending; pending.removeAll(); for continuation in waits.values { continuation.resume(throwing: CancellationError()) }
    }
    private func failed(_ error: Error) {
        status = "Language services unavailable: \(error.localizedDescription). Reconnect or retry."
        let waits = pending; pending.removeAll(); for continuation in waits.values { continuation.resume(throwing: error) }
    }
    private func receive(_ object: [String: Any]) {
        if object["method"] as? String == "textDocument/publishDiagnostics", let params = object["params"] as? [String: Any],
           params["uri"] as? String == uri, let buffer, buffer.text == sent {
            if let incoming = params["version"] as? Int, incoming != version { return }
            diagnostics = decodeNativeDiagnostics(params, source: sent, expectedVersion: version); diagnosticSource = sent
            return
        }
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
    private func sync() {
        guard let text = buffer?.text, text != sent else { return }
        sent = text; version += 1
        notify("textDocument/didChange", ["textDocument": ["uri": uri, "version": version], "contentChanges": [["text": text]]])
    }
    func changed() {
        refreshTask?.cancel(); inspectionID = UUID(); completionID = UUID()
        if let buffer {
            tokens = rebaseNativeTokens(tokens, from: tokenSource, to: buffer.text)
            tokenSource = buffer.text
            buffer.renderLanguage?(tokens, buffer.text)
        }
        let current = epoch
        refreshTask = Task { [weak self] in
            try? await Task.sleep(for: .milliseconds(300))
            guard let self, !Task.isCancelled, self.epoch == current else { return }
            self.sync()
            if self.showingSuggestions {
                Task { [weak self] in await self?.inspect() }
                Task { [weak self] in await self?.complete() }
            }
            guard let source = self.buffer?.text else { return }
            for attempt in 0..<5 {
                guard !Task.isCancelled, self.buffer?.text == source, self.epoch == current else { return }
                do {
                    let result = try await self.request("textDocument/semanticTokens/full", ["textDocument": ["uri": self.uri]]) as? [String: Any]
                    guard !Task.isCancelled, self.buffer?.text == source, self.epoch == current else { return }
                    let data = result?["data"] as? [Int]
                    let fresh = decodeNativeTokens(data ?? [], legend: self.legend, source: source)
                    if !fresh.isEmpty || (attempt == 4 && data?.isEmpty == true) {
                        self.tokens = fresh; self.tokenSource = source
                        self.buffer?.renderLanguage?(self.tokens, source)
                    }
                    self.status = self.tokens.isEmpty ? "LSP connected; no semantic colors for this file. Open Language tools for setup or retry." : "LSP · \(self.tokens.count) semantic spans"
                    if !fresh.isEmpty || attempt == 4 { return }
                } catch { if !Task.isCancelled, self.epoch == current { self.status = "Language colors unavailable: \(error.localizedDescription)" }; return }
                try? await Task.sleep(for: .milliseconds(700))
            }
        }
    }
    func inspect() async {
        showingSuggestions = true
        guard let buffer else { return }
        sync(); let source = buffer.text, selection = buffer.selection, current = epoch, id = UUID()
        inspectionID = id; inspectionSource = source; inspectionSelection = selection
        let position = languagePosition(source, selection.location)
        await withTaskGroup(of: Void.self) { group in
            for method in ["textDocument/hover", "textDocument/signatureHelp", "textDocument/inlayHint"] {
                group.addTask { @MainActor [weak self] in
                    guard let self else { return }
                    var params: [String: Any] = ["textDocument": ["uri": self.uri], "position": position]
                    if method == "textDocument/inlayHint" { params["range"] = ["start": ["line": 0, "character": 0], "end": languagePosition(source, source.utf16.count)] }
                    guard let result = try? await self.request(method, params), !(result is NSNull), self.epoch == current, self.inspectionID == id,
                          self.buffer?.text == source, self.buffer?.selection == selection else { return }
                    if method == "textDocument/hover" { self.hover = nativeMarkupText((result as? [String: Any])?["contents"]) }
                    if method == "textDocument/signatureHelp" {
                        self.signature = ((result as? [String: Any])?["signatures"] as? [[String: Any]] ?? []).compactMap { $0["label"] as? String }.joined(separator: "\n")
                    }
                    if method == "textDocument/inlayHint" {
                        self.inlayHints = (result as? [[String: Any]] ?? []).prefix(200).compactMap { hint in
                            guard let pos = hint["position"] as? [String: Any], languageOffset(source, pos) != nil else { return nil }
                            let label = hint["label"] as? String ?? (hint["label"] as? [[String: Any]] ?? []).compactMap { $0["value"] as? String }.joined()
                            return "Line \((pos["line"] as? Int ?? 0) + 1): \(label)"
                        }
                    }
                }
            }
        }
    }
    func reveal(_ diagnostic: NativeDiagnostic) {
        guard let buffer, buffer.text == diagnosticSource, let range = diagnostic.range else { return }
        _ = buffer.selectSourceRange?(range, diagnosticSource)
    }
    func complete() async {
        showingSuggestions = true
        guard let buffer, !buffer.readOnly else { return }
        sync(); let source = buffer.text, selection = buffer.selection, current = epoch, id = UUID()
        completionID = id
        status = "Updating completions; showing last received results…"
        do {
            // The bridge prepares virtual files asynchronously after full-text sync.
            // Keep the popup responsive while typing settles; never use an obsolete position.
            try await Task.sleep(for: .milliseconds(350))
            guard self.buffer?.text == source, self.buffer?.selection == selection, epoch == current, completionID == id else { return }
            let result = try await request("textDocument/completion", ["textDocument": ["uri": uri], "position": languagePosition(source, selection.location)])
            guard self.buffer?.text == source, self.buffer?.selection == selection, epoch == current, completionID == id else { return }
            guard !(result is NSNull) else { return }
            let items = result as? [[String: Any]] ?? (result as? [String: Any])?["items"] as? [[String: Any]] ?? []
            let ns = source as NSString
            var prefix = min(selection.location, ns.length)
            while prefix > 0, let scalar = UnicodeScalar(ns.character(at: prefix - 1)), CharacterSet.alphanumerics.union(CharacterSet(charactersIn: "_")).contains(scalar) { prefix -= 1 }
            let typed = ns.substring(with: NSRange(location: prefix, length: max(0, selection.location - prefix))).lowercased()
            var fresh: [NativeCompletion] = []
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
                fresh.append(NativeCompletion(id: index, label: label, detail: item["detail"] as? String ?? "Language server", range: range, insert: insert, source: source, selection: selection))
                if fresh.count >= 100 { break }
            }
            suggestions = fresh
            status = suggestions.isEmpty ? "No supported completions at this position. Open Language tools for setup or retry." : "LSP · \(suggestions.count) completions"
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
func nativeDebugURI(_ path: String) -> String { "hick:///" + path.trimmingCharacters(in: CharacterSet(charactersIn: "/")) }
struct DebugLine: Equatable { var line: Int; var y: Double; var height: Double }
struct NativeDebugProfile: Decodable, Identifiable {
    var id: String; var name: String; var program: String?; var can_debug: Bool
}
struct NativeDebugTarget: Identifiable { var path: String; var line: Int; var end: Int; var id: String { "\(path):\(line)" } }
struct NativeDebugRender: Decodable {
    struct Block: Decodable { var kind: String; var path: String?; var span: [Int]? }
    var blocks: [Block]
}
struct NativeDebugCatalog: Decodable { var profiles: [NativeDebugProfile]; var fingerprint: String; var viewed: String? }
struct NativeRunOutput: Equatable {
    var category: String; var text: String; var source: String?; var line: Int?
}
@MainActor @Observable final class NativeDebugger {
    var status = "idle"
    var message = ""
    var owner = ""
    var sessionID: String?
    var frames: [[String: Any]] = []
    var variables: [[String: Any]] = []
    var capabilities: [String: Any] = [:]
    var breakpoints: [String: [[String: Any]]] = [:]
    var output: [NativeRunOutput] = []
    var values: [String: String] = [:]
    var watches: [String] = []
    var profiles: [NativeDebugProfile] = []
    var targets: [NativeDebugTarget] = []
    private var launchProgram: String?
    var catalog: NativeDebugCatalog?
    var launchSource = ""
    private var sourceSnapshots: [String: String] = [:]
    var pausedLine: Int?
    var selectedFrame: Int?
    var offer: [String: String]?
    @ObservationIgnored var onLocation: ((String, Int) -> Void)?
    @ObservationIgnored var emitForTest: (([String: Any]) -> Void)?
    private let engine: Engine
    private weak var launchBuffer: DocumentBuffer?
    private var socket: URLSessionWebSocketTask?
    private var receiver: Task<Void, Never>?
    private var sender: Task<Void, Never>?
    private var epoch = UUID()
    private var topVariables: [[String: Any]] = []
    private var stopPending = false
    init(engine: Engine) { self.engine = engine }
    var active: Bool { ["starting", "running", "paused", "stopping"].contains(status) }
    func line(in path: String) -> Int? {
        guard status == "paused" else { return nil }
        let frame = frames.first(where: { $0["id"] as? Int == selectedFrame }) ?? frames.first
        if path == owner, frame?["in_document"] as? Bool == true { return frame?["line"] as? Int }
        if frame?["source"] as? String == path { return frame?["source_line"] as? Int }
        return path == owner && selectedFrame == frames.first?["id"] as? Int ? pausedLine : nil
    }
    func outputPlacements(_ buffer: DocumentBuffer) -> [Int: String] {
        guard sourceMatches(buffer) else { return [:] }
        let context = inlineDebugContext(buffer.text, path: buffer.file.path)
        let last = max(0, buffer.text.components(separatedBy: "\n").count - 1 - (buffer.text.hasSuffix("\n") ? 1 : 0))
        let rows = buffer.text.components(separatedBy: "\n")
        func blockEnd(_ line: Int) -> Int {
            guard let header = context.launch.keys.filter({ $0 <= line }).max(), rows.indices.contains(header) else { return min(last, line) }
            let declaration = rows[header].trimmingCharacters(in: .whitespaces)
            let fence = declaration.hasPrefix("```") ? "`" : declaration.hasPrefix("~~~") ? "~" : ""
            if line < last {
                for candidate in (line + 1)...last {
                    let row = rows[candidate].trimmingCharacters(in: .whitespaces)
                    if !fence.isEmpty && row.count >= 3 && row.allSatisfy({ String($0) == fence }) { return candidate }
                    if fence.isEmpty && row.hasPrefix("</") && row.contains(":file>") { return candidate }
                }
            }
            return last
        }
        var result: [Int: String] = [:]
        for chunk in output {
            var anchor = last
            if let line = chunk.line, chunk.source == buffer.file.path {
                anchor = line
                if context.code[line] != nil { anchor = blockEnd(line) }
            } else if let program = launchProgram, let end = context.code.filter({ $0.value == program }).keys.max() { anchor = blockEnd(end) }
            result[anchor, default: ""] += chunk.text
        }
        return result
    }
    func sourceMatches(_ buffer: DocumentBuffer) -> Bool { !buffer.dirty && buffer.text == (buffer.file.path == owner ? launchSource : sourceSnapshots[buffer.file.path]) }
    func rememberSource(_ buffer: DocumentBuffer) {
        if active && sourceSnapshots[buffer.file.path] == nil { sourceSnapshots[buffer.file.path] = buffer.base }
    }
    func prepare(_ buffer: DocumentBuffer, line: Int = 0, program: String? = nil) async {
        guard !active else { message = "Stop the current debugger before launching another program."; return }
        guard !buffer.dirty, !buffer.saving else { message = "Save this file before debugging so the adapter runs the displayed text."; return }
        close(); let current = epoch
        owner = buffer.file.path; launchBuffer = buffer; launchSource = buffer.text; sourceSnapshots = [owner: launchSource]; message = "Resolving launch targets…"; profiles = []; targets = []
        do {
            let literate = buffer.file.doc_id != nil || ["md", "markdown"].contains((owner as NSString).pathExtension.lowercased())
            var target = program ?? inlineDebugPath(buffer.text, path: owner, line: line, launch: true) ?? inlineDebugPath(buffer.text, path: owner, line: line)
            if literate && target == nil {
                guard let id = buffer.file.doc_id else { message = "Refresh files to discover this document before choosing a file block to debug."; return }
                let rendered: NativeDebugRender = try await engine.request("GET", "/api/docs/" + id + "/render")
                guard epoch == current, buffer.text == launchSource, !buffer.dirty else { return }
                targets = rendered.blocks.compactMap { block in
                    guard block.kind == "file", let path = block.path, let span = block.span, span.count == 2,
                          span[0] >= 0, span[1] >= span[0], span[1] <= launchSource.utf8.count else { return nil }
                    return NativeDebugTarget(path: path, line: launchSource.utf8.prefix(span[0]).filter { $0 == 10 }.count,
                        end: launchSource.utf8.prefix(span[1]).filter { $0 == 10 }.count)
                }
                target = targets.first(where: { $0.line <= line && line <= $0.end })?.path
                guard target != nil else { message = targets.isEmpty ? "No file blocks to debug. Add a supported file block and save this document." : "Choose the file block to debug."; return }
            }
            launchProgram = target
            var query: [String: Any] = ["uri": nativeDebugURI(owner), literate ? "expected_document" : "expected_source": launchSource]
            if let target { query["program"] = target }
            let result: NativeDebugCatalog = try await engine.request("POST", "/api/launch-profiles", body: query)
            guard epoch == current, buffer.text == launchSource, !buffer.dirty else { return }
            catalog = result; profiles = result.profiles.filter(\.can_debug)
            if let automatic = profiles.first(where: { $0.program == (target ?? result.viewed ?? owner) }) ?? (profiles.count == 1 ? profiles.first : nil) { launch(automatic); return }
            message = profiles.isEmpty ? "No debug launch targets. Configure a debug profile in this project and try again." : "Choose a debug target."
        } catch { if epoch == current { message = "\(error.localizedDescription) Save or refresh this file and try debugging again." } }
    }
    func launch(_ profile: NativeDebugProfile) {
        guard let buffer = launchBuffer, let catalog, profiles.contains(where: { $0.id == profile.id }),
              buffer.text == launchSource, !buffer.dirty, !buffer.saving, !active else {
            message = "The source or launch target changed. Save and choose Debug again."; return
        }
        status = "starting"; message = ""; profiles = []; values = [:]; variables = []; frames = []; pausedLine = nil; offer = nil
        var url = URLComponents(url: engine.origin, resolvingAgainstBaseURL: false)!
        url.scheme = engine.origin.scheme == "https" ? "wss" : "ws"; url.path = "/api/ws"; url.queryItems = [URLQueryItem(name: "doc", value: "workspace")]
        let socket = engine.session.webSocketTask(with: url.url!); self.socket = socket; let current = epoch; socket.resume()
        receiver = Task { [weak self] in
            do {
                while !Task.isCancelled {
                    let frame = try await socket.receive()
                    guard let self, self.epoch == current else { return }
                    if case .data(let bytes) = frame, bytes.first == 3,
                       let event = try? JSONSerialization.jsonObject(with: bytes.dropFirst()) as? [String: Any] { self.receive(event) }
                }
            } catch { if let self, self.epoch == current, !Task.isCancelled { self.fail("Debugger disconnected: \(error.localizedDescription). Stop or reconnect explicitly; commands were not retried.") } }
        }
        var request: [String: Any] = ["op": "start", "doc": nativeDebugURI(owner), "breakpoints": breakpoints[owner] ?? [],
            "profile": ["id": profile.id, "fingerprint": catalog.fingerprint]]
        if let launchProgram { request["program"] = launchProgram }
        request["sources"] = Dictionary(uniqueKeysWithValues: breakpoints.filter { $0.key != owner }.map { (nativeDebugURI($0.key), $0.value) })
        send(request)
    }
    private func send(_ request: [String: Any]) {
        if let emitForTest { emitForTest(request); return }
        guard let socket, let bytes = try? JSONSerialization.data(withJSONObject: request) else { return }
        let previous = sender, current = epoch
        sender = Task { [weak self] in
            await previous?.value
            guard !Task.isCancelled, let self, self.epoch == current else { return }
            do { try await socket.send(.data(Data([3]) + bytes)) }
            catch { if self.epoch == current { self.fail("Debugger command failed: \(error.localizedDescription). Reconnect explicitly; it was not retried.") } }
        }
    }
    private func fail(_ detail: String) { status = "failed"; message = detail; sessionID = nil; pausedLine = nil; variables = []; frames = []; values = [:] }
    func close() {
        epoch = UUID(); receiver?.cancel(); sender?.cancel(); socket?.cancel(with: .normalClosure, reason: nil); socket = nil
        stopPending = false; sessionID = nil; status = "idle"; pausedLine = nil; frames = []; variables = []; values = [:]; profiles = []
    }
    // Session-bearing events from an old run cannot paint a new launch.
    func receive(_ event: [String: Any]) {
        guard let kind = event["event"] as? String else { return }
        if ["started", "build"].contains(kind) || (kind == "failed" && event["session"] is NSNull) {
            guard event["doc"] as? String == nativeDebugURI(owner), ["starting", "stopping"].contains(status) else { return }
        } else { guard let id = event["session"] as? String, id == sessionID else { return } }
        switch kind {
        case "started":
            output = []
            sessionID = event["session"] as? String; capabilities = event["capabilities"] as? [String: Any] ?? [:]; status = "running"
            bind(event["breakpoints"] as? [[String: Any]] ?? [], path: owner)
            if stopPending { stopPending = false; stop() }
        case "stopped":
            status = "paused"; frames = event["frames"] as? [[String: Any]] ?? []; variables = event["variables"] as? [[String: Any]] ?? []; topVariables = variables
            pausedLine = event["line"] as? Int; selectedFrame = event["selected_frame"] as? Int ?? frames.first?["id"] as? Int
            if let top = frames.first, let path = top["source"] as? String, let line = top["source_line"] as? Int,
               !path.hasPrefix("/"), !path.contains(":") { onLocation?(path, line) }
            else if let pausedLine { onLocation?(owner, pausedLine) }
            for expression in watches { evaluate(expression, watch: true) }
        case "breakpoints": bind(event["breakpoints"] as? [[String: Any]] ?? [], path: owner)
        case "source_breakpoints":
            if let source = event["source"] as? String, source.hasPrefix("hick:///") { bind(event["breakpoints"] as? [[String: Any]] ?? [], path: String(source.dropFirst(8))) }
        case "run_output":
            output = Array((output + (event["events"] as? [[String: Any]] ?? []).compactMap { item in
                guard let text = item["text"] as? String else { return nil }
                return NativeRunOutput(category: item["category"] as? String ?? "stdout", text: text, source: item["source"] as? String, line: item["line"] as? Int)
            }).suffix(256))
        case "value":
            if status == "paused", let expression = event["expression"] as? String, let value = event["value"] as? String { values[expression] = value }
        case "finished", "ended": status = "finished"; sessionID = nil; pausedLine = nil; variables = []; frames = []; values = [:]
        case "failed":
            offer = event["offer_install"] as? [String: String]
            if sessionID == nil { fail(event["message"] as? String ?? "Debug launch failed. Check the adapter installation and try again.") }
            else { message = (event["message"] as? String ?? "Debug command failed.") + " Review the paused state before trying again." }
        case "build": message = (event["events"] as? [[String: Any]] ?? []).compactMap { $0["data"] as? String }.joined()
        default: break
        }
    }
    private func bind(_ statuses: [[String: Any]], path: String) {
        let held = breakpoints[path] ?? []
        breakpoints[path] = statuses.map { status in
            var result = held.first(where: { ($0["line"] as? Int) == (status["line"] as? Int) }) ?? [:]
            result.merge(status) { _, latest in latest }
            if let moved = status["moved_to"] as? Int { result["line"] = moved }
            return result
        }
    }
    func toggle(_ path: String, line: Int, condition: String? = nil) {
        guard line >= 0 else { return }
        var held = breakpoints[path] ?? []
        if let index = held.firstIndex(where: { $0["line"] as? Int == line }) {
            if let condition { held[index]["condition"] = condition } else { held.remove(at: index) }
        } else { var mark: [String: Any] = ["line": line]; if let condition { mark["condition"] = condition }; held.append(mark) }
        breakpoints[path] = held
        if let sessionID {
            send(path == owner ? ["op": "breakpoints", "session": sessionID, "breakpoints": held] : ["op": "source_breakpoints", "session": sessionID, "source": nativeDebugURI(path), "breakpoints": held])
        }
    }
    func step(_ how: String) {
        guard status == "paused", let sessionID else { return }
        var request: [String: Any] = ["op": "step", "session": sessionID, "how": how]
        if let selectedFrame { request["frame"] = selectedFrame }
        status = "running"; pausedLine = nil; variables = []; values = [:]; send(request)
    }
    func stop() {
        guard let sessionID else {
            if status == "starting" || status == "stopping" { stopPending = true; status = "stopping"; message = "Stopping as soon as the adapter acknowledges launch…" }
            return
        }
        status = "stopping"; pausedLine = nil; variables = []; values = [:]; send(["op": "stop", "session": sessionID])
    }
    func evaluate(_ expression: String, watch: Bool = false) {
        guard status == "paused", let sessionID, !expression.isEmpty else { return }
        if watch && !watches.contains(expression) { watches.append(expression) }
        var request: [String: Any] = ["op": "eval", "session": sessionID, "expression": expression, "context": watch ? "watch" : "repl"]
        if let selectedFrame { request["frame"] = selectedFrame }; send(request)
    }
    func select(_ frame: [String: Any]) {
        selectedFrame = frame["id"] as? Int
        variables = selectedFrame == frames.first?["id"] as? Int ? topVariables : []; values = [:]
        for expression in watches { evaluate(expression, watch: true) }
        if let path = frame["source"] as? String, let line = frame["source_line"] as? Int, !path.hasPrefix("/"), !path.contains(":") { onLocation?(path, line) }
        else if let line = frame["line"] as? Int { onLocation?(owner, line) }
    }
    func install() async {
        guard let offer else { return }; let current = epoch
        do {
            struct Installed: Decodable { var installed: String; var path: String }
            let _: Installed = try await engine.request("POST", "/api/install", body: ["kind": offer["kind"] ?? "dap", "language": offer["language"] ?? ""])
            if epoch == current { self.offer = nil; message = "Debugger installed. Choose Debug again to launch the current saved source." }
        } catch { if epoch == current { message = "\(error.localizedDescription) Check the adapter installation and retry." } }
    }
}
