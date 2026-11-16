<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Use the existing engine protocol from Swift

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Decode only the fields this frontend needs. The address is explicit runtime configuration, validated once. Requests have a timeout and never retry a mutation after an ambiguous failure. The browser uses the existing generated TypeScript client for these same routes.

Read managed Markdown through its document ID, using the source response without requesting rendering. Plain text retains the hash-based file endpoint. Document saves continue through the room-aware document endpoint with the loaded source baseline.

Source revisions below use the [exact UTF-8 mechanism](../../../../mechanisms/native-source-revisions.md). Swift linguistic equality does not establish identical program bytes. The buffer and language snapshots retain their String-facing renderer API, while exact revision values drive observation and source-derived work.

<ui:copy id="simple-ff9425b170-0">
import Foundation
import CryptoKit

struct FileNode: Codable, Identifiable {
    var name: String
    var path: String
    var dir: Bool
    var children: [FileNode]?
    var doc_id: String?
    var generated_by: String?
    var language: String?
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
</ui:copy>

Save requests carry the exact baseline from the original read. Agent sends select the Python backend explicitly, continue the last observed turn and never grant implicit editor write authority.

<ui:copy id="simple-ff9425b170-1">
    func find(_ query: String, regex: Bool, matchCase: Bool, wholeWord: Bool) async throws -> WorkspaceSearchResult {
        struct Occurrence: Decodable { var column: Int; var length: Int }
        struct Row: Decodable { var line: Int; var text: String; var at: [Occurrence] }
        struct File: Decodable { var path: String; var source_hash: String?; var generated_by: String?; var matches: [Row] }
        struct Result: Decodable { var files: [File]; var truncated: Bool }
        let result: Result = try await request("GET", "/api/find", query: [URLQueryItem(name: "q", value: query), URLQueryItem(name: "regex", value: String(regex)), URLQueryItem(name: "case", value: String(matchCase)), URLQueryItem(name: "whole_word", value: String(wholeWord))])
        return WorkspaceSearchResult(hits: result.files.flatMap { file in file.matches.flatMap { row in row.at.map { occurrence in WorkspaceSearchHit(path: file.path, line: row.line, text: row.text, column: occurrence.column, sourceHash: file.source_hash, generatedBy: file.generated_by) } } }, truncated: result.truncated)
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
</ui:copy>

## Attach language services after readable text is available

Each document owns a read-only language channel over the existing backend WebSocket framing. Channel 2 carries JSON-RPC; the backend owns initialize and child-server discovery. Serialize outgoing notifications and requests so a completion cannot overtake its full-text synchronization. Source snapshots and selection positions guard every response and insertion. A failed or missing server reports its state without blocking text, saving or pane movement.

Semantic-token ranges use UTF-16, including surrogate pairs. Decode against the server's advertised legend and reject ranges outside the actual line rather than painting unrelated text. Completion items requiring snippets or additional edits are excluded from this first native slice; accepting a simple item still goes through the native editor's permission, source and selection checks.

<ui:copy id="simple-apple-language-transport">
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
    var location: SemanticLocation? = nil
}
func nativeMarkupText(_ value: Any?) -> String {
    if let text = value as? String { return text }
    if let rows = value as? [Any] { return rows.map { nativeMarkupText($0) }.filter { !$0.isEmpty }.joined(separator: "\n\n") }
    if let object = value as? [String: Any] { return object["value"] as? String ?? object["label"] as? String ?? "" }
    return ""
}
func decodeNativeDiagnostics(_ params: [String: Any], source: String, expectedVersion: Int) -> [NativeDiagnostic] {
    // Unversioned diagnostics remain readable messages, never positional authority.
    let items = params["diagnostics"] as? [[String:Any]] ?? []
    let hasEvidence = items.contains { ($0["data"] as? [String:Any])?["hickory"] != nil }
    let digest = hasEvidence ? "sha256:" + nativeSourceHash(source) : ""
    return items.enumerated().map { index, item in
        let data = item["data"] as? [String:Any]
        let envelope = data?["hickory"] as? [String:Any]
        let located = data?["hickory"] != nil ? (envelope?["source_hash"] as? String)?.lowercased() == digest : params["version"] as? Int == expectedVersion
        var range: NSRange?; var location: SemanticLocation?
        if located, let raw = item["range"] as? [String: Any], let start = raw["start"] as? [String: Any], let end = raw["end"] as? [String: Any],
           let from = languageOffset(source, start), let to = languageOffset(source, end), from <= to {
            range = NSRange(location: from, length: to - from)
            if let uri = params["uri"] as? String, let line = start["line"] as? Int, let character = start["character"] as? Int {
                location = SemanticLocation(uri: uri, line: line, character: character)
            }
        }
        return NativeDiagnostic(id: index, message: item["message"] as? String ?? "Language diagnostic", severity: item["severity"] as? Int ?? 1, range: range, location: location)
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
    var original:[String:Any]?=nil
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

Versioned diagnostic ranges are usable only when the server version equals the native source version. The bridge also forwards unversioned diagnostics: show their messages without clickable positions, since receiving one after an edit cannot prove which source it describes. Edits clear observations immediately. Hover, signature and inlay requests run independently and retain source, selection, connection and request-generation guards.

## Own the optional socket and pending requests

The session opens only after the buffer exists. Closing or reconnecting retires its epoch, cancels background tasks and fails pending requests; an old socket cannot publish into the replacement session. The server capability announcement supplies the semantic legend. Outgoing frames stay ordered on channel 2, while other backend channels are ignored.

## Ask semantic questions without owning their presentation

The pure query vocabulary and location decoder are constructed in the shared [intelligence model](../../../intelligence.md). This adapter owns connection and buffer snapshots, not presentation or symbol normalization.

A provider can answer with locations, nested document symbols, flat workspace symbols or call-hierarchy items. Normalize those shapes to named source locations once. Outlines inherit the queried document identity; workspace symbols and calls must carry their own identity. Caller/callee reads prepare provider items before asking for incoming/outgoing calls. Bound ambiguous preparation and nesting, retain explicit limits, and do not infer absent call relationships.

Definition, usages, type and implementation are the same operation: ask a provider about a captured source position and return navigable locations. Keep the vocabulary and Location/LocationLink normalization independent of SwiftUI. Preserve unsupported URI identities for display, but admit only workspace-relative authored paths for opening. A result is information, never permission to modify a file.



<ui:copy id="simple-apple-language-session">
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
    private var symbolQueryRevision=ExactSourceText("")
    var symbolQuery:String {get {symbolQueryRevision.text} set {symbolQueryRevision=ExactSourceText(newValue)}}
    var locations: [SemanticLocation] = []
    var locationStatus = ""
    var locationSymbolQuery: String?
    private var locationSourceRevision=ExactSourceText("")
    var locationSource:String {get {locationSourceRevision.text} set {locationSourceRevision=ExactSourceText(newValue)}}
    var locationSelection = NSRange(location: 0, length: 0)
    private var navigationID = UUID()
    var status = "Language server connecting…"
    var tokens: [NativeSemanticToken] = []
    private var tokenSourceRevision=ExactSourceText("")
    var tokenSource:String {get {tokenSourceRevision.text} set {tokenSourceRevision=ExactSourceText(newValue)}}
    var suggestions: [NativeCompletion] = []
    var showingSuggestions = false
    var diagnostics: [NativeDiagnostic] = []
    private var diagnosticSourceRevision=ExactSourceText("")
    var diagnosticSource:String {get {diagnosticSourceRevision.text} set {diagnosticSourceRevision=ExactSourceText(newValue)}}
    var hover = ""
    var signature = ""
    var inlayHints: [String] = []
    private var inspectionSourceRevision=ExactSourceText("")
    var inspectionSource:String {get {inspectionSourceRevision.text} set {inspectionSourceRevision=ExactSourceText(newValue)}}
    var inspectionSelection = NSRange(location: 0, length: 0)
    private var diagnosticID = UUID()
    private var inspectionID = UUID()
    private var completionID = UUID()
    private weak var buffer: DocumentBuffer?
    let connection: NativeLanguageConnection
    private let ownsConnection: Bool
    private var subscription: UUID?
    private let uri: String
    private var refreshTask: Task<Void, Never>?
    private var version = 1
    private var sent = ""
    private var legend: [String] = []
    private var epoch = UUID()
    init(engine: Engine, buffer: DocumentBuffer, connection: NativeLanguageConnection? = nil) {
        self.connection = connection ?? NativeLanguageConnection(engine:engine); ownsConnection = connection == nil
        self.buffer = buffer; uri = languageURI(buffer.file.path)
    }
    func start() {
        close(); epoch = UUID(); sent = buffer?.text ?? ""; version = 1; status = "Language server connecting…"
        subscription = connection.attach(uri:uri,source:sent,version:version,
            receive:{ [weak self] in self?.receive($0) },failed:{ [weak self] in self?.failed($0) })
        changed()
    }
    func close() {
        cancelCompletion();completionNotice=""
        navigationID = UUID(); locations = []; locationStatus = ""
        diagnostics = []; diagnosticSource = ""; hover = ""; signature = ""; inlayHints = []; inspectionID = UUID()
        epoch = UUID(); refreshTask?.cancel()
        if let token=subscription { subscription=nil;connection.detach(token) }
        if ownsConnection { connection.close() }
    }
    private func failed(_ error: Error) {
        epoch=UUID();inspectionID=UUID();completionID=UUID();navigationID=UUID();diagnosticID=UUID()
        status = "Language services unavailable: \(error.localizedDescription). Reconnect or retry."
        if !locations.isEmpty {locationStatus="Provider connection changed. Run the query again."}
    }
    private func receive(_ object: [String: Any]) {
        if object["method"] as? String == "textDocument/publishDiagnostics", let params = object["params"] as? [String: Any],
           params["uri"] as? String == uri, let buffer {
            let items = params["diagnostics"] as? [[String:Any]] ?? []
            let evidence = items.contains { ($0["data"] as? [String:Any])?["hickory"] != nil }
            if !evidence {
                guard exactUTF8(buffer.text,sent) else { return }
                if let incoming = params["version"] as? Int, incoming != version { return }
            }
            let source = buffer.text, expectedVersion = version, identity = epoch, request = UUID()
            diagnosticID = request; diagnosticSource = ""
            diagnostics = items.enumerated().map { index,item in NativeDiagnostic(id:index,message:item["message"] as? String ?? "Language diagnostic",severity:item["severity"] as? Int ?? 1,range:nil) }
            Task { [weak self, weak buffer] in
                let decoded = await Task.detached { decodeNativeDiagnostics(params,source:source,expectedVersion:expectedVersion) }.value
                guard let self, let buffer, self.epoch == identity, self.diagnosticID == request, exactUTF8(buffer.text,source), evidence || exactUTF8(self.sent,source) else { return }
                self.diagnostics = decoded; self.diagnosticSource = source
            }
            return
        }
        if object["method"] as? String == "hick/serverCapabilities",
                  let params = object["params"] as? [String: Any], let caps = params["capabilities"] as? [String: Any] {
            legend = ((caps["semanticTokensProvider"] as? [String: Any])?["legend"] as? [String: Any])?["tokenTypes"] as? [String] ?? []
            status = "LSP connected; checking semantic colors…"; changed()
        }
    }
    private func request(_ method: String, _ params: [String:Any]) async throws -> Any {
        guard let owner=subscription else {throw EngineError(message:"Language source is closed. Retry language services.")}
        return try await connection.request(method,params:params,owner:owner)
    }
</ui:copy>

## Apply results only to the source and selection that requested them

An empty semantic-token or completion response establishes only that the bridge answered; it does not establish whether a child server is installed. Keep these statuses descriptive and point to Language tools for the workspace setup command.

Typing rebases cached colors onto unchanged UTF-16 regions and schedules a quiet-period refresh. A single-line edit contained within one token keeps that tentative classification with a resized range; edits crossing token boundaries discard affected spans. Unchanged prefix and suffix colors stay visible. A null/error response retains that cache; an explicit empty token array can replace it after the bounded startup retries. Language-tool text and suggestions remain visible as cached results, while exact source/selection guards still control navigation and completion acceptance. Inspection replies replace each capability independently; the panel labels their contents as last received information rather than claiming that cached text describes the current cursor. Providers can start late, so semantic reads retry a bounded number of times without delaying text opening. Completion requests are debounced after synchronization, filter unsupported edit shapes and retain their exact source/selection snapshot. The native edit callback makes the final permission and revision check. No language result saves a file.

<ui:copy id="simple-apple-language-snapshots">
    private func sync() {
        guard let text = buffer?.text, !exactUTF8(text,sent) else { return }
        sent = text; version += 1
        if let owner=subscription {connection.change(owner:owner,source:text,version:version)}
    }
    func changed() {
        refreshTask?.cancel(); inspectionID = UUID(); completionID = UUID(); navigationID = UUID()
        if locationStatus == "Searching…" { locationStatus = "Source or provider changed. Run the query again." }
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
                guard !Task.isCancelled, exactUTF8(self.buffer?.text,source), self.epoch == current else { return }
                do {
                    let result = try await self.request("textDocument/semanticTokens/full", ["textDocument": ["uri": self.uri]]) as? [String: Any]
                    guard !Task.isCancelled, exactUTF8(self.buffer?.text,source), self.epoch == current else { return }
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
    // Background reads depend on source and connection, never the current caret or popup.
    func readOutline(source: String) async throws -> [SemanticLocation] {
        guard let buffer, exactUTF8(buffer.text,source) else { throw EngineError(message:"Source changed before analysis. Run again.") }
        sync(); let current = epoch
        let result = try await request("textDocument/documentSymbol", ["textDocument":["uri":uri]])
        guard epoch == current, exactUTF8(buffer.text,source) else { throw EngineError(message:"Source or provider changed during analysis. Run again.") }
        return semanticLocations(result,defaultURI:uri).filter {$0.uri == uri}
    }
    func navigate(_ query: SemanticQuery) async {
        showingSuggestions = true
        guard let buffer else { return }
        sync(); let source = buffer.text, selection = buffer.selection, current = epoch, id = UUID()
        navigationID = id; locations = []; locationStatus = "Searching…"
        locationSymbolQuery = query == .symbols ? symbolQuery : nil
        locationSource = source; locationSelection = selection
        var params: [String: Any] = ["textDocument": ["uri": uri], "position": languagePosition(source, selection.location)]
        if query == .references { params["context"] = ["includeDeclaration": true] }
        do {
            if query == .symbols { params = ["query": locationSymbolQuery ?? ""] }
            if query == .outline { params = ["textDocument": ["uri": uri]] }
            var result = try await request(query.method, params)
            if let followup=query.hierarchyFollowup {
                let items = result as? [[String: Any]] ?? []
                guard items.count <= 32 else { throw EngineError(message: "Too many call targets. Choose a more specific symbol.") }
                var answers: [Any] = []
                for item in items {
                    guard navigationID == id, epoch == current, exactUTF8(buffer.text,source), buffer.selection == selection else {
                        if navigationID==id,epoch==current{locationStatus="Source or cursor changed. Run the query again."};return
                    }
                    answers.append(try await request(followup, ["item": item]))
                }
                result = answers
            }
            guard navigationID == id, epoch == current, exactUTF8(buffer.text,source), buffer.selection == selection else {
                if navigationID == id, epoch == current { locationStatus = "Source or cursor changed. Run the query again." }; return
            }
            locations = semanticLocations(result, defaultURI: query == .outline ? uri : nil)
            locationStatus = locations.isEmpty ? "No results. The provider may still be indexing or may not support this query. Retry or choose another symbol." : "\(locations.count) locations\(locations.count >= 10000 ? "; narrow the query for more" : "")"
        } catch {
            if navigationID == id, epoch == current, exactUTF8(buffer.text,source), buffer.selection == selection {
                locationStatus = "\(query.label) unavailable: \(error.localizedDescription). Retry language services."
            }
        }
    }
    func open(_ location: SemanticLocation) {
        guard let buffer, exactUTF8(buffer.text,locationSource), buffer.selection == locationSelection,
              locations.contains(location), location.path != nil, locationSymbolQuery == nil || exactUTF8(locationSymbolQuery,symbolQuery) else {
            locationStatus = "Location is stale or outside authored workspace files. Run the query again."; return
        }
        if location.path == buffer.file.path {
            guard let offset = languageOffset(buffer.text, ["line": location.line, "character": location.character]),
                  buffer.selectSourceRange?(NSRange(location: offset, length: 0), buffer.text) == true else {
                locationStatus = "Location no longer fits this source. Query again."; return
            }
            let position = languagePosition(locationSource, locationSelection.location)
            buffer.rememberNavigation?(SemanticLocation(uri: uri, line: position["line"]!, character: position["character"]!), location)
            buffer.focusSource?()
        } else { buffer.openSemanticLocation?(location) }
        showingSuggestions = false
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
                          exactUTF8(self.buffer?.text,source), self.buffer?.selection == selection else { return }
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
        guard let buffer, exactUTF8(buffer.text,diagnosticSource), let range = diagnostic.range else { return }
        _ = buffer.selectSourceRange?(range, diagnosticSource)
    }
    var proposalIdentity: UUID { epoch }
    func propose(_ kind: String, name: String) async throws -> Any {
        guard let buffer, !buffer.readOnly, subscription != nil else { throw EngineError(message: "Choose writable source with a connected language provider.") }
        sync(); let source = buffer.text, selection = buffer.selection, identity = epoch
        var params: [String:Any] = ["textDocument":["uri":uri]]
        let method: String
        switch kind {
        case "rename": method = "textDocument/rename"; params["position"] = languagePosition(source,selection.location); params["newName"] = name
        case "references": method = "textDocument/references"; params["position"] = languagePosition(source,selection.location); params["context"] = ["includeDeclaration":true]
        case "format": method = "textDocument/formatting"; params["options"] = ["tabSize":4,"insertSpaces":!source.components(separatedBy:"\n").contains(where:{$0.hasPrefix("\t")})]
        default: method = "textDocument/codeAction"; params["range"] = ["start":languagePosition(source,selection.location),"end":languagePosition(source,NSMaxRange(selection))]; params["context"] = ["diagnostics":[]]
        }
        let result = try await request(method,params)
        guard epoch == identity, exactUTF8(buffer.text,source), buffer.selection == selection else { throw EngineError(message: "Source, selection or provider changed. Prepare again.") }
        return kind == "format" ? ["changes":[uri:result]] : result
    }
</ui:copy>

## Resolve held provider actions on apple

Resolution synchronizes the originating document and retains its exact source, selection and provider identity across the request. Eager legacy edits need no unsupported RPC. The complete resolved value returns to the change model; this language adapter never executes a command or applies source.

<ui:copy id="native-provider-action-resolution-apple">
    func resolveProposal(_ action:[String:Any]) async throws->[String:Any] {
        let data=action["data"] as? [String:Any]
        if data?["hickory_action"] as? String == nil && ((action["edit"] != nil && !(action["edit"] is NSNull)) || action["data"] == nil || action["data"] is NSNull) {return action}
        guard let buffer,!buffer.readOnly,subscription != nil else{throw EngineError(message:"Choose writable source with a connected language provider.")}
        sync();let source=buffer.text,selection=buffer.selection,identity=epoch
        var params=action;params["textDocument"]=["uri":uri]
        let result=try await request("codeAction/resolve",params)
        guard epoch==identity,exactUTF8(buffer.text,source),buffer.selection==selection else{throw EngineError(message:"Source, selection or provider changed. Prepare actions again.")}
        guard let value=result as? [String:Any],value["title"] is String else{throw EngineError(message:"Provider did not resolve this action. Prepare actions again.")};return value
    }
</ui:copy>

Continue the independent completion service after the action-resolution boundary.

<ui:copy id="native-language-after-action-resolution-apple">
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
            guard exactUTF8(self.buffer?.text,source), self.buffer?.selection == selection, epoch == current, completionID == id else { return }
            let result = try await request("textDocument/completion", ["textDocument": ["uri": uri], "position": languagePosition(source, selection.location)])
            guard exactUTF8(self.buffer?.text,source), self.buffer?.selection == selection, epoch == current, completionID == id else { return }
            guard !(result is NSNull) else { return }
            let items = result as? [[String: Any]] ?? (result as? [String: Any])?["items"] as? [[String: Any]] ?? []
            let ns = source as NSString
            var prefix = min(selection.location, ns.length)
            while prefix > 0, let scalar = UnicodeScalar(ns.character(at: prefix - 1)), CharacterSet.alphanumerics.union(CharacterSet(charactersIn: "_")).contains(scalar) { prefix -= 1 }
            let typed = ns.substring(with: NSRange(location: prefix, length: max(0, selection.location - prefix))).lowercased()
            var fresh: [NativeCompletion] = []
            for (index, item) in items.enumerated() {
                guard let label=item["label"] as? String,(item["filterText"] as? String ?? label).lowercased().hasPrefix(typed) else{continue}
                let range=NSRange(location:prefix,length:NSMaxRange(selection)-prefix)
                fresh.append(NativeCompletion(id:index,label:label,detail:item["detail"] as? String ?? "Language server",range:range,insert:item["insertText"] as? String ?? label,source:source,selection:selection,original:item))
                if fresh.count >= 100 { break }
            }
            suggestions = fresh
            status = suggestions.isEmpty ? "No supported completions at this position. Open Language tools for setup or retry." : "LSP · \(suggestions.count) completions"
        } catch { if epoch == current,completionID==id,exactUTF8(buffer.text,source),buffer.selection==selection { status = "Completion unavailable: \(error.localizedDescription)" } }
    }
</ui:copy>

## Accept complete apple source proposals

Retain each offered provider item and its source/selection. Resolve before planning, keep a latest acceptance identity and reject changed source, selection, provider, saving or permissions. Dismissal retires pending acceptance. Commit the complete source/import edit and resulting selection through the existing editor batch boundary; no command or save is executed. Source-local legacy suggestions retain their guarded eager path.

<ui:copy id="native-completion-acceptance-apple">
    var completionNotice=""
    private var acceptingCompletion=UUID()
    private var completionPending=false
    func cancelCompletion() {
        acceptingCompletion=UUID()
        if completionPending{completionNotice="Completion cancelled."}
        completionPending=false
    }
    func resolveCompletion(_ item:[String:Any]) async throws->[String:Any] {
        if item["data"]==nil || item["data"] is NSNull{return item}
        guard let buffer,!buffer.readOnly,subscription != nil else{throw EngineError(message:"Choose writable source with a connected language provider.")}
        sync();let source=buffer.text,selection=buffer.selection,identity=epoch
        var params=item;params["textDocument"]=["uri":uri]
        let result=try await request("completionItem/resolve",params)
        guard epoch==identity,exactUTF8(buffer.text,source),buffer.selection==selection else{throw EngineError(message:"Source, cursor or provider changed. Request completions again.")}
        guard let value=result as? [String:Any],value["label"] is String else{throw EngineError(message:"Provider did not resolve this completion. Request again.")};return value
    }
    @discardableResult func accept(_ completion:NativeCompletion,resolution:(([String:Any]) async throws->[String:Any])?=nil) async->Bool {
        guard let buffer,!buffer.readOnly,!buffer.saving,exactUTF8(buffer.text,completion.source),buffer.selection==completion.selection else{completionNotice="Suggestion is out of date. Request completions again.";return false}
        if exactUTF8(buffer.multiSource,buffer.text),buffer.multiRanges.count>1{completionNotice="Choose one caret before accepting this source completion.";return false}
        guard let original=completion.original else {
            cancelCompletion()
            let accepted=buffer.snippets.active != nil ? buffer.snippets.input(source:completion.source,range:completion.range,text:completion.insert)==true:buffer.applyLanguageCompletion?(completion.range,completion.insert,completion.source,completion.selection)==true
            guard accepted else{completionNotice="Editor refused the completion. Finish composition or request again.";return false}
            completionNotice="Completion selected";showingSuggestions=false;return true
        }
        let request=UUID(),identity=epoch,snippetIntent=buffer.snippets.intent;acceptingCompletion=request;completionPending=true;completionNotice="Resolving completion…"
        defer{if acceptingCompletion==request{completionPending=false}}
        do {
            let resolved=try await (resolution ?? {try await self.resolveCompletion($0)})(original)
            guard acceptingCompletion==request else{return false}
            guard epoch==identity,buffer.snippets.intent==snippetIntent,buffer.language===self,!buffer.readOnly,!buffer.saving,exactUTF8(buffer.text,completion.source),buffer.selection==completion.selection,!(exactUTF8(buffer.multiSource,buffer.text)&&buffer.multiRanges.count>1) else{throw EngineError(message:"Source, cursor or provider changed. Request completions again.")}
            let initialDraft=try planCompletion(uri,item:resolved,snapshot:SourceSnapshot(text:completion.source,writable:!buffer.readOnly && !buffer.saving),from:completion.range.location,to:NSMaxRange(completion.range),expansion:try completionExpansion(resolved))
            let draft=try buffer.snippets.prepareCompletion(initialDraft)
            guard exactUTF8(draft.path,buffer.file.path),buffer.applyCompletionDraft?(draft,completion.selection)==true else{throw EngineError(message:"Editor refused the complete completion. Finish composition or request again.")}
            completionPending=false;completionNotice=draft.plan.patches.isEmpty ? "Completion selected":"Completion applied · draft";showingSuggestions=false;return true
        }catch{if acceptingCompletion==request{completionNotice=error.localizedDescription};return false}
    }
</ui:copy>

Resume the remaining platform services after the completion acceptance boundary.

<ui:copy id="native-completion-acceptance-apple-tail">

}
</ui:copy>

## Keep one native debugger above the panes

The native client uses the engine's binary 0x03 channel, whose line numbers already refer to document source. Launch resolution sends the displayed saved bytes to the existing profile endpoint. A socket generation and session identity reject late messages. Sending a control never retries a mutation; disconnects clear live observations. A workspace-owned controller survives tab switches, and cross-file stops ask the workspace to reveal the adapter's source line. The transport starts only on an explicit launch, independently of readable text.

The gutter has already selected a file block. After validating its saved bytes, launch the matching program immediately; a sole application also needs no confirmation. Keep the picker only when discovery cannot identify that program and several applications remain. Automatic launch goes through the same fingerprint, buffer and session guards as a manual choice.

The thread reader keeps a stopped generation and selected runtime thread separate from frame selection. Each list or selection request carries a fresh identity; only its matching response at the current stop can replace the floating list. Selection requests retain the existing stack until the engine confirms a new stopped observation, then admit the paired thread list. Late stops, retired lists and old failures cannot repaint a newer pause. Independent read failures leave frames, source and the editor alive. Thread names/counts are adapter reports, not verification results.

Evaluation carries the exact stop, selected thread/frame and a fresh per-expression request identity. A newer request for the same expression retires the older answer, even within one pause. New stops and frame selections clear prior values and evaluation notices. Evaluation failure remains separate from the session state, so an unsupported provider frame cannot remove the controls needed to choose another thread. Rendered frame rows carry their own snapshot identity, preventing a late click from selecting a recycled frame identifier.

Selecting a frame requests its locals through the same engine scope checks, with a separate latest request identity. Replies must match the stopped generation, thread and selected frame; switching frames retires the previous read without waiting for it. A provider refusal is separate from an observed empty variable list. A failed same-stop stack refresh keeps earlier frames readable as historical context, disables their navigation/evaluation and leaves thread selection and runtime Stop available. Only a successful fresh stack re-enables those positional actions.

<ui:copy id="simple-apple-debug-session">
func nativeDebugURI(_ path:String)->String {languageURI(path)}
struct DebugLine: Equatable { var line: Int; var y: Double; var height: Double }
struct NativeDebugProfile: Decodable, Identifiable {
    var id: String; var name: String; var program: String?; var can_debug: Bool
}
struct NativeDebugTarget: Identifiable { var path: String; var line: Int; var end: Int; var id: String { "\(path):\(line)" } }
struct NativeDebugRender: Decodable {
    struct Block: Decodable { var kind: String; var path: String?; var span: [Int]? }
    var blocks: [Block]
}
struct NativeDebugCatalog: Decodable {var profiles:[NativeDebugProfile];var fingerprint:String;var viewed:String?;var debug_protocol:Int?}
struct NativeRunOutput: Equatable {
    var category: String; var text: String; var source: String?; var line: Int?
}
import CoreFoundation
private struct NativeDebugID:Decodable,Equatable {var value:Int32;init(from decoder:Decoder)throws{value=try decoder.singleValueContainer().decode(Int32.self)}}
private func nativeDebugID(_ raw:Any?)->Int? {
    guard let number=raw as? NSNumber,CFGetTypeID(number) != CFBooleanGetTypeID() else{return nil}
    let value=number.doubleValue
    guard value.isFinite,value.rounded()==value,value>=Double(Int32.min),value<=Double(Int32.max) else{return nil}
    return Int(value)
}
private func nativeDebugLine(_ raw:Any?)->Int? {guard let line=nativeDebugID(raw),line>=0 else{return nil};return line}
struct NativeDebugThread:Decodable,Identifiable,Equatable {
    private var runtimeID:NativeDebugID;var name:String;var id:Int64 {Int64(runtimeID.value)}
    private enum CodingKeys:String,CodingKey {case runtimeID="id",name}
}
struct NativeDebugLocation {var path:String;var line:Int;var session:String?;var stop:UInt64?;var thread:Int64?;var frame:Int?;var snapshot:UUID;var sourceHash:String?;var sourceReference:Int64?}
private struct NativeDebugThreadPause:Decodable {
    var stop:UInt64;private var runtimeThread:NativeDebugID;var thread:Int64 {Int64(runtimeThread.value)}
    private enum CodingKeys:String,CodingKey {case stop,runtimeThread="thread"}
}
private struct NativeDebugThreadsEvent:Decodable {
    var session:String;var stop:UInt64;private var runtimeSelected:NativeDebugID;var selected:Int64 {Int64(runtimeSelected.value)};var threads:[NativeDebugThread];var request_id:String?
    private enum CodingKeys:String,CodingKey {case session,stop,runtimeSelected="selected",threads,request_id}
}
private struct NativeDebugThreadRequest {var id:String;var stop:UInt64;var thread:Int64?}
private struct NativeDebugLaunch {var id:String;var doc:String}
private struct NativeDebugEvaluation {var id:String;var stop:UInt64;var thread:Int64;var frame:Int?}
struct NativeDebugExpressionValue:Identifiable {var expression:String;var value:String;var id:Data {Data(expression.utf8)}}
struct NativeDebugValues:ExpressibleByDictionaryLiteral {
    private var entries:[Data:NativeDebugExpressionValue]=[:]
    init(){}
    init(dictionaryLiteral elements:(String,String)...){for (expression,value) in elements{self[expression]=value}}
    subscript(_ expression:String)->String? {
        get{entries[Data(expression.utf8)]?.value}
        set{let key=Data(expression.utf8);if let value=newValue{entries[key]=NativeDebugExpressionValue(expression:expression,value:value)}else{entries.removeValue(forKey:key)}}
    }
    var isEmpty:Bool {entries.isEmpty}
    var readings:[NativeDebugExpressionValue] {entries.values.sorted{$0.id.lexicographicallyPrecedes($1.id)}}
    var keys:[String] {readings.map{$0.expression}}
    @discardableResult mutating func removeValue(forKey expression:String)->String? {entries.removeValue(forKey:Data(expression.utf8))?.value}
}
private struct NativeDebugExpressionRequest {var expression:String;var request:NativeDebugEvaluation}

private struct NativeDebugValue:Decodable {
    var expression:String;var value:String;var stop:UInt64;private var runtimeThread:NativeDebugID?;private var runtimeFrame:NativeDebugID?;var thread:Int64? {runtimeThread.map{Int64($0.value)}};var frame:Int? {runtimeFrame.map{Int($0.value)}};var request_id:String?
    private enum CodingKeys:String,CodingKey {case expression,value,stop,runtimeThread="thread",runtimeFrame="frame",request_id}
}
private struct NativeDebugVariable:Decodable {var name:String;var value:String;var type_name:String?;var variables_reference:Int64?}
private struct NativeDebugFrameVariables:Decodable {
    var session:String;var stop:UInt64;private var runtimeThread:NativeDebugID?;private var runtimeFrame:NativeDebugID?;var thread:Int64? {runtimeThread.map{Int64($0.value)}};var frame:Int? {runtimeFrame.map{Int($0.value)}};var request_id:String?;var variables:[NativeDebugVariable]
    private enum CodingKeys:String,CodingKey {case session,stop,runtimeThread="thread",runtimeFrame="frame",request_id,variables}
}

</ui:copy>

## Own the workspace session before sending commands

Keep renderer handles out of the runtime model. It retains exact source relationships, source-reference limits and a bounded set of owned launch identities. Store a fresh launch identity before any socket send; old same-document startup replies cannot adopt a newer launch, and cleanup only names known retired launches. Typed location callbacks describe evidence without acquiring authority from newly opened disk.

<ui:copy id="native-debug-workspace-session-apple">
@MainActor @Observable final class NativeDebugger {
    var status = "idle"
    var message = ""
    var owner = ""
    var sessionID: String?
    private(set) var threads:[NativeDebugThread]=[]
    private(set) var selectedThread:Int64?
    private(set) var threadStop:UInt64?
    private(set) var threadSnapshot=UUID()
    private(set) var threadBusy=false
    private(set) var refreshingPause=false
    private(set) var threadNotice=""
    private(set) var evaluationNotice=""
    private(set) var frameNotice=""
    private(set) var stackNotice=""
    private(set) var frameBusy=false
    private(set) var frameVariablesCurrent=false
    private var frameRequest:NativeDebugEvaluation?
    var threadPage=0
    var threadPages:Int {max(1,(threads.count+99)/100)}
    var visibleThreads:[NativeDebugThread] {Array(threads.dropFirst(min(max(0,threadPage),threadPages-1)*100).prefix(100))}
    var canReadThreads:Bool {status=="paused" && pauseIdentityKnown && latestThreadStop != nil && !threadBusy && !refreshingPause}
    var canSelectThreads:Bool {status=="paused" && pauseIdentityKnown && !threadBusy && !refreshingPause && threadStop != nil && threadStop==latestThreadStop}
    var canResume:Bool {status=="paused" && pauseIdentityKnown && !refreshingPause && threadRequest?.thread==nil}
    var canInspectFrame:Bool {canResume && stackNotice.isEmpty}
    var canReadFrameLocals:Bool {canInspectFrame && latestThreadStop != nil && selectedThread != nil && selectedFrame != nil && !frameBusy}
    var canEvaluate:Bool {canInspectFrame && latestThreadStop != nil && selectedThread != nil}
    private(set) var frameSnapshot=UUID()
    private var evaluations:[Data:NativeDebugExpressionRequest]=[:]
    private var pauseIdentityKnown=false
    private var latestThreadStop:UInt64?
    private var threadRequest:NativeDebugThreadRequest?
    var frames: [[String: Any]] = []
    var variables: [[String: Any]] = []
    var capabilities: [String: Any] = [:]
    var breakpoints: [String: [[String: Any]]] = [:]
    var output: [NativeRunOutput] = []
    var values=NativeDebugValues()
    var watches: [String] = []
    var profiles: [NativeDebugProfile] = []
    var targets: [NativeDebugTarget] = []
    private var launchProgram: String?
    var catalog: NativeDebugCatalog?
    var launchSource = ""
    private var sourceHashes:[Data:String]=[:]
    private var ambiguousSources=Set<Data>()
    var pausedLine: Int?
    var selectedFrame: Int?
    var offer: [String: String]?
    @ObservationIgnored var onLocation: ((NativeDebugLocation) -> Void)?
    @ObservationIgnored var emitForTest: (([String: Any]) -> Void)?
    private let engine: Engine
    private weak var launchBuffer: DocumentBuffer?
    private var socket: URLSessionWebSocketTask?
    private var receiver: Task<Void, Never>?
    private var sender: Task<Void, Never>?
    private var epoch = UUID()
    private var topVariables: [[String: Any]] = []
    private var stopPending = false
    private var pendingLaunch:NativeDebugLaunch?
    private var activeLaunch:NativeDebugLaunch?
    private var retiredLaunches:[NativeDebugLaunch]=[]
    var launchRequestID:String? {pendingLaunch?.id ?? activeLaunch?.id}
    private func retireLaunches() {
        for value in [pendingLaunch,activeLaunch].compactMap({$0}) where !retiredLaunches.contains(where:{$0.id==value.id}) {retiredLaunches.append(value)}
        retiredLaunches=Array(retiredLaunches.suffix(64));pendingLaunch=nil;activeLaunch=nil
    }
    @discardableResult func beginLaunchRequest()->String {
        retireLaunches();let id=UUID().uuidString;pendingLaunch=NativeDebugLaunch(id:id,doc:nativeDebugURI(owner));sessionID=nil;stopPending=false;message="";status="starting";return id
    }
    private func stopRetiredLaunch(_ event:[String:Any])->Bool {
        guard event["event"] as? String=="started",let id=event["request_id"] as? String,let held=retiredLaunches.first(where:{$0.id==id}),event["doc"] as? String==held.doc,let session=event["session"] as? String,!session.isEmpty,session != sessionID else{return false}
        send(["op":"stop","session":session]);return true
    }
    init(engine: Engine) { self.engine = engine }
    var active: Bool { ["starting", "running", "paused", "stopping"].contains(status) }
    var hasSessionReading:Bool {active || sessionID != nil || status=="finished" || status=="failed" || status=="unavailable" || !output.isEmpty}
    func line(in path: String) -> Int? {
        guard status == "paused",stackNotice.isEmpty,let reference=selectedSourceReference,reference<=0 else { return nil }
        let frame = frames.first(where: { nativeDebugID($0["id"]) == selectedFrame }) ?? frames.first
        if exactUTF8(path,owner), frame?["in_document"] as? Bool == true { return nativeDebugLine(frame?["line"]) }
        if exactUTF8(frame?["source"] as? String,path) { return nativeDebugLine(frame?["source_line"]) }
        return exactUTF8(path,owner) && selectedFrame == nativeDebugID(frames.first?["id"]) ? pausedLine : nil
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
            if let line = chunk.line, exactUTF8(chunk.source,buffer.file.path) {
                anchor = line
                if context.code[line] != nil { anchor = blockEnd(line) }
            } else if let program = launchProgram, let end = context.code.filter({ $0.value == program }).keys.max() { anchor = blockEnd(end) }
            result[anchor, default: ""] += chunk.text
        }
        return result
    }
    private func sourceReference(_ frame:[String:Any])->Int? {guard let raw=frame["source_reference"],!(raw is NSNull) else{return 0};return nativeDebugID(raw)}
    private var selectedSourceReference:Int? {guard let frame=frames.first(where:{nativeDebugID($0["id"])==selectedFrame}) else{return 0};return sourceReference(frame)}
    var sourceReferenceNotice:String {
        guard let reference=selectedSourceReference else{return "Selected frame has a malformed adapter source identity. Workspace positions are unavailable."}
        return reference>0 ? "Selected frame uses adapter-provided source reference \(reference). Workspace file text cannot establish its position.":""
    }
    private func sourceProof(_ path:String)->String? {
        let key=Data(path.utf8)
        guard let reference=selectedSourceReference,reference<=0,!ambiguousSources.contains(key) else{return nil}
        if exactUTF8(path,owner){return nativeSourceHash(launchSource)}
        guard let frame=frames.first(where:{nativeDebugID($0["id"])==selectedFrame}),exactUTF8(frame["source"] as? String,path),let hash=frame["source_hash"] as? String,hash.range(of:"^[0-9a-f]{64}$",options:.regularExpression) != nil,sourceHashes[key]==hash else{return nil}
        return hash
    }
    func sourceMatches(_ buffer:DocumentBuffer)->Bool {
        guard !buffer.dirty,let proof=sourceProof(buffer.file.path) else{return false}
        if exactUTF8(buffer.file.path,owner){return exactUTF8(buffer.text,launchSource)}
        return proof==nativeSourceHash(buffer.text)
    }
    func locationCurrent(_ location:NativeDebugLocation)->Bool {status=="paused" && stackNotice.isEmpty && location.snapshot==frameSnapshot && location.session==sessionID && location.stop==latestThreadStop && location.thread==selectedThread && location.frame==selectedFrame}
    private func sourcePath(_ path:String)->Bool {!path.isEmpty && !path.contains("\0") && path.split(separator:"/",omittingEmptySubsequences:false).allSatisfy{!$0.isEmpty && $0 != "." && $0 != ".."}}
    private func retainSourceProofs() {
        for frame in frames {
            guard let reference=sourceReference(frame),reference<=0,let path=frame["source"] as? String,sourcePath(path),let hash=frame["source_hash"] as? String,hash.range(of:"^[0-9a-f]{64}$",options:.regularExpression) != nil else{continue}
            let key=Data(path.utf8),held=exactUTF8(path,owner) ? nativeSourceHash(launchSource):sourceHashes[key]
            if let held,held != hash {sourceHashes.removeValue(forKey:key);ambiguousSources.insert(key)}
            else if !ambiguousSources.contains(key){sourceHashes[key]=hash}
        }
    }
    func sourceLocation(path:String,line:Int)->NativeDebugLocation? {
        guard status=="paused",stackNotice.isEmpty,sourcePath(path),line>=0 else{return nil}
        let proof=sourceProof(path)
        return NativeDebugLocation(path:path,line:line,session:sessionID,stop:latestThreadStop,thread:selectedThread,frame:selectedFrame,snapshot:frameSnapshot,sourceHash:proof,sourceReference:selectedSourceReference.map{Int64($0)})
    }
    private func revealLocation(path:String,line:Int) {if let location=sourceLocation(path:path,line:line){onLocation?(location)}}
    private func compatibleDebugger(_ catalog:NativeDebugCatalog)->Bool {
        guard (catalog.debug_protocol ?? 0)>=2 else{self.catalog=nil;profiles=[];status="unavailable";message="Scoped debugging is unavailable from this engine. Update/reconnect an engine with debug protocol 2 before Debug. No Start command was sent.";return false}
        return true
    }
    func prepare(_ buffer: DocumentBuffer, line: Int = 0, program: String? = nil) async {
        guard !active else { message = "Stop the current debugger before launching another program."; return }
        guard !buffer.dirty, !buffer.saving else { message = "Save this file before debugging so the adapter runs the displayed text."; return }
        close(); let current = epoch
        owner = buffer.file.path; launchBuffer = buffer; launchSource = buffer.text; sourceHashes = [Data(owner.utf8):nativeSourceHash(launchSource)];ambiguousSources=[]; message = "Resolving launch targets…"; profiles = []; targets = []
        do {
            let literate = buffer.file.doc_id != nil || ["md", "markdown"].contains((owner as NSString).pathExtension.lowercased())
            var target = program ?? inlineDebugPath(buffer.text, path: owner, line: line, launch: true) ?? inlineDebugPath(buffer.text, path: owner, line: line)
            if literate && target == nil {
                guard let id = buffer.file.doc_id else { message = "Refresh files to discover this document before choosing a file block to debug."; return }
                let rendered: NativeDebugRender = try await engine.request("GET", "/api/docs/" + id + "/render")
                guard epoch == current, exactUTF8(buffer.text,launchSource), !buffer.dirty else { return }
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
            guard epoch == current, exactUTF8(buffer.text,launchSource), !buffer.dirty else { return }
            guard compatibleDebugger(result) else{return}
            catalog = result; profiles = result.profiles.filter(\.can_debug)
            if let automatic = profiles.first(where: { $0.program == (target ?? result.viewed ?? owner) }) ?? (profiles.count == 1 ? profiles.first : nil) { launch(automatic); return }
            message = profiles.isEmpty ? "No debug launch targets. Configure a debug profile in this project and try again." : "Choose a debug target."
        } catch { if epoch == current { message = "\(error.localizedDescription) Save or refresh this file and try debugging again." } }
    }
    func launch(_ profile: NativeDebugProfile) {
        guard let catalog else{message="Resolve a current scoped launch profile before Debug. No Start command was sent.";return}
        guard compatibleDebugger(catalog) else{return}
        guard let buffer = launchBuffer, profiles.contains(where: { $0.id == profile.id }),
              exactUTF8(buffer.text,launchSource), !buffer.dirty, !buffer.saving, !active else {
            message = "The source or launch target changed. Save and choose Debug again."; return
        }
        let launch=beginLaunchRequest();message = ""; profiles = []; values = [:]; variables = []; frames = []; pausedLine = nil; offer = nil
        var url = URLComponents(url: engine.origin, resolvingAgainstBaseURL: false)!
        url.scheme = engine.origin.scheme == "https" ? "wss" : "ws"; url.path = "/api/ws"; url.queryItems = [URLQueryItem(name: "doc", value: "workspace")]
        let socket = engine.session.webSocketTask(with: url.url!); self.socket = socket; let current = epoch; socket.resume()
        receiver = Task { [weak self] in
            do {
                while !Task.isCancelled {
                    let frame = try await socket.receive()
                    guard let self else{return}
                    if case .data(let bytes)=frame,bytes.first==3,let event=try? JSONSerialization.jsonObject(with:bytes.dropFirst()) as? [String:Any] {
                        if self.epoch==current {self.receive(event)}else{_ = self.stopRetiredLaunch(event);return}
                    }else if self.epoch != current{return}
                }
            } catch { if let self, self.epoch == current, !Task.isCancelled { self.fail("Debugger disconnected: \(error.localizedDescription). Stop or reconnect explicitly; commands were not retried.") } }
        }
        var request: [String: Any] = ["op": "start", "request_id":launch,"doc": nativeDebugURI(owner), "breakpoints": breakpoints[owner] ?? [],
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
    private func fail(_ detail: String) { retireLaunches();resetThreads();status = "failed"; message = detail; sessionID = nil; pausedLine = nil; variables = []; frames = []; values = [:] }
    func close() {
        retireLaunches()
        epoch = UUID(); receiver?.cancel(); sender?.cancel(); socket?.cancel(with: .normalClosure, reason: nil); socket = nil
        resetThreads();stopPending = false; sessionID = nil; status = "idle"; pausedLine = nil; frames = []; variables = []; values = [:]; profiles = []
    }
</ui:copy>

## Admit replies at their observed scope

Session, launch, stopped generation, thread, selected frame and latest request identities are independent guards. A duplicate or malformed ID cannot stand in for an absent identity. Admit source hashes only from the selected frame’s captured mapping; an adapter source reference always retires disk fallback. Keep read errors distinct from observed empty lists, and preserve readable historical frames without offering unsafe navigation.

<ui:copy id="native-debug-reply-admission-apple">
    // Session-bearing events from an old run cannot paint a new launch.
    func receive(_ event: [String: Any]) {
        guard let kind = event["event"] as? String else { return }
        if kind=="started" {
            if stopRetiredLaunch(event){return}
            guard let pending=pendingLaunch,event["request_id"] as? String==pending.id,event["doc"] as? String==pending.doc,let session=event["session"] as? String,!session.isEmpty,["starting","stopping"].contains(status) else{return}
        }else if kind=="build" || kind=="failed" && (event["session"] is NSNull || event["about"] as? String=="start") {
            guard let held=pendingLaunch ?? activeLaunch,event["request_id"] as? String==held.id else{return}
            if kind=="build" || event["session"] is NSNull {guard event["doc"] as? String==held.doc else{return}}
            else {guard event["session"] as? String==sessionID else{return}}
        }else {guard let id=event["session"] as? String,id==sessionID else{return}}
        switch kind {
        case "started":
            activeLaunch=pendingLaunch;pendingLaunch=nil
            resetThreads();output = []
            sessionID = event["session"] as? String; capabilities = event["capabilities"] as? [String: Any] ?? [:]; status = "running"
            bind(event["breakpoints"] as? [[String: Any]] ?? [], path: owner)
            if stopPending { stopPending = false; stop() }
        case "stopped":
            guard let bytes=try? JSONSerialization.data(withJSONObject:event),let pause=try? JSONDecoder().decode(NativeDebugThreadPause.self,from:bytes) else{pauseIdentityKnown=false;stackNotice="Stopped response has no valid integer thread/stop identity. Earlier frames are historical; refresh through a current engine.";threadNotice=stackNotice;values=[:];evaluations=[:];frameRequest=nil;frameBusy=false;frameVariablesCurrent=false;threadRequest=nil;threadBusy=false;return}
            guard status != "stopping" else{return}
            if let latestThreadStop {guard pause.stop>=latestThreadStop,status != "running" || pause.stop>latestThreadStop else{return}}
            let previousFrames=frames,samePause=pause.stop==latestThreadStop && pause.thread==selectedThread
            pauseIdentityKnown=true;refreshingPause=false;admitThreadPause(pause);values=[:];evaluations=[:];evaluationNotice="";frameRequest=nil;frameBusy=false;frameSnapshot=UUID()
            stackNotice=(event["stack_error"] as? String).map{"Provider stack read refused: "+$0} ?? ""
            frameNotice=(event["variables_error"] as? String).map{"Provider variables read refused: "+$0} ?? ""
            status = "paused"; frames = event["frames"] as? [[String: Any]] ?? []
            let ids=frames.compactMap{nativeDebugID($0["id"])}
            if ids.count != frames.count || Set(ids).count != ids.count {frames=[];stackNotice="Provider frame identities are malformed, duplicated or outside signed int32."}
            if !stackNotice.isEmpty,samePause {frames=previousFrames;stackNotice += " Earlier frames from this stop remain readable; refresh before selecting them."}
            variables = stackNotice.isEmpty ? event["variables"] as? [[String:Any]] ?? []:[];topVariables=variables
            frameVariablesCurrent=stackNotice.isEmpty && event["variables_error"] as? String==nil && !frames.isEmpty
            if variables.isEmpty,frameNotice.isEmpty{frameNotice=stackNotice.isEmpty ? "No variables were reported for the top frame.":"Variables were not read because the current stack is unavailable."}
            pausedLine=nativeDebugLine(event["line"]);selectedFrame=nativeDebugID(event["selected_frame"]) ?? nativeDebugID(frames.first?["id"])
            if let raw=event["selected_frame"],!(raw is NSNull),nativeDebugID(raw)==nil {selectedFrame=nil;stackNotice="Provider selected-frame identity is malformed or outside signed int32.";frameVariablesCurrent=false;variables=[]}
            if stackNotice.isEmpty {
                retainSourceProofs()
                if let top = frames.first, let path = top["source"] as? String, let line = nativeDebugLine(top["source_line"]) { revealLocation(path:path,line:line) }
                else if let pausedLine { revealLocation(path:owner,line:pausedLine) }
            }
            for expression in watches { evaluate(expression, watch: true) }
        case "threads":receiveThreads(event)
        case "frame_variables":receiveFrameVariables(event)
        case "breakpoints": bind(event["breakpoints"] as? [[String: Any]] ?? [], path: owner)
        case "source_breakpoints":
            if let source = event["source"] as? String, source.hasPrefix("hick:///") { bind(event["breakpoints"] as? [[String: Any]] ?? [], path: String(source.dropFirst(8))) }
        case "run_output":
            output = Array((output + (event["events"] as? [[String: Any]] ?? []).compactMap { item in
                guard let text = item["text"] as? String else { return nil }
                return NativeRunOutput(category: item["category"] as? String ?? "stdout", text: text, source: item["source"] as? String, line: item["line"] as? Int)
            }).suffix(256))
        case "value":
            guard status=="paused",let bytes=try? JSONSerialization.data(withJSONObject:event),let value=try? JSONDecoder().decode(NativeDebugValue.self,from:bytes),let request=evaluations[Data(value.expression.utf8)]?.request,value.request_id==request.id,value.stop==request.stop,value.stop==latestThreadStop,value.thread==request.thread,value.thread==selectedThread,value.frame==request.frame,value.frame==selectedFrame else{return}
            evaluations.removeValue(forKey:Data(value.expression.utf8));values[value.expression]=value.value
        case "finished", "ended": retireLaunches();resetThreads();status = "finished"; sessionID = nil; pausedLine = nil; variables = []; frames = []; values = [:]
        case "failed":
            if event["about"] as? String=="state",let id=event["request_id"] as? String {
                guard let request=frameRequest,request.id==id else{return}
                frameRequest=nil;frameBusy=false;frameVariablesCurrent=false;frameNotice=(event["message"] as? String ?? "Frame variables could not be read.")+" The selected stack and source are retained.";return
            }
            if event["about"] as? String=="state",refreshingPause {refreshingPause=false;threadNotice=(event["message"] as? String ?? "Paused state could not be refreshed.")+" Previous frames remain readable.";return}
            if event["about"] as? String=="eval",status=="paused" {
                guard let id=event["request_id"] as? String,let key=evaluations.first(where:{$0.value.request.id==id})?.key,let failed=evaluations.removeValue(forKey:key) else{return}
                values.removeValue(forKey:failed.expression);evaluationNotice=(event["message"] as? String ?? "Evaluation failed.")+" The current stopped frame is retained.";return
            }
            if ["threads","select_thread"].contains(event["about"] as? String ?? ""),status=="paused" {
                guard let request=threadRequest,event["request_id"] as? String==request.id else{return}
                threadRequest=nil;threadBusy=false;threadNotice=(event["message"] as? String ?? "Thread request failed.")+" Paused frames and source are retained.";return
            }
            offer = event["offer_install"] as? [String: String]
            if sessionID == nil { fail(event["message"] as? String ?? "Debug launch failed. Check the adapter installation and try again.") }
            else { message = (event["message"] as? String ?? "Debug command failed.") + " The current session is retained; inspect its state before retrying." }
        case "build": message = (event["events"] as? [[String: Any]] ?? []).compactMap { $0["data"] as? String }.joined()
        default: break
        }
    }
</ui:copy>

## Read one current inspection without replacing the editor

Read-only thread and local-variable requests retain separate tickets. New selections supersede earlier reads; a thread selection keeps the old stack until the engine confirms its new stopped observation, then admits the paired list. Counts and pages describe reported adapter data. Provider refusal changes its own notice, leaving source, the paused session and the escape route to another thread intact.

<ui:copy id="native-debug-inspection-requests-apple">
    private func resetThreads() {
        threads=[];threadStop=nil;selectedThread=nil;threadRequest=nil;threadBusy=false;refreshingPause=false;threadNotice="";pauseIdentityKnown=false;latestThreadStop=nil;threadSnapshot=UUID();frameSnapshot=UUID();evaluations=[:];evaluationNotice="";frameRequest=nil;frameBusy=false;frameVariablesCurrent=false;frameNotice="";stackNotice="";threadPage=0
    }
    private func retireThreadChoices() {
        threads=[];threadStop=nil;threadRequest=nil;threadBusy=false;refreshingPause=false;threadSnapshot=UUID();frameSnapshot=UUID();evaluations=[:];evaluationNotice="";frameRequest=nil;frameBusy=false;frameVariablesCurrent=false;frameNotice="";stackNotice="";threadPage=0
    }
    private func admitThreadPause(_ pause:NativeDebugThreadPause?) {
        guard let pause else{resetThreads();threadNotice="This stopped response has no thread observation. Update the engine before selecting threads.";return}
        let changed=latestThreadStop != pause.stop || selectedThread != pause.thread
        latestThreadStop=pause.stop;selectedThread=pause.thread
        if changed {
            threads=[];threadStop=nil;threadSnapshot=UUID();threadPage=0;threadNotice="Read reported threads for this stopped state."
            // A confirmed selection arrives as Stopped followed by its correlated
            // Threads response. Keep only that handoff; all older reads retire.
            if var request=threadRequest,request.thread==pause.thread,pause.stop>request.stop {request.stop=pause.stop;threadRequest=request}
            else if threadRequest?.stop != pause.stop {threadRequest=nil;threadBusy=false}
        }
    }
    private func receiveThreads(_ event:[String:Any]) {
        guard status=="paused",let request=threadRequest,event["request_id"] as? String==request.id else{return}
        guard let bytes=try? JSONSerialization.data(withJSONObject:event),let result=try? JSONDecoder().decode(NativeDebugThreadsEvent.self,from:bytes),result.session==sessionID,result.stop==latestThreadStop,result.stop==request.stop,result.selected==selectedThread else{threadRequest=nil;threadBusy=false;threadNotice="The thread response no longer matches this pause. Refresh the paused state before selecting threads.";return}
        guard result.threads.count<=10000,Set(result.threads.map{$0.id}).count==result.threads.count else{threadRequest=nil;threadBusy=false;threadNotice="Thread response is duplicate or exceeds 10,000 entries. Paused frames are retained; refresh or choose a narrower debugger scope.";return}
        threads=result.threads;threadStop=result.stop;threadRequest=nil;threadBusy=false;threadSnapshot=UUID();threadPage=0
        threadNotice=threads.contains(where:{$0.id==selectedThread}) ? "\(threads.count) adapter-reported threads. Select a reported thread to request its stopped stack.":"\(threads.count) adapter-reported threads; the selected thread is not in this list. Refresh the paused state."
        if request.thread != nil {for expression in watches{evaluate(expression,watch:true)}}
    }
    private func receiveFrameVariables(_ event:[String:Any]) {
        guard status=="paused",let request=frameRequest,event["request_id"] as? String==request.id else{return}
        guard let bytes=try? JSONSerialization.data(withJSONObject:event),let value=try? JSONDecoder().decode(NativeDebugFrameVariables.self,from:bytes),value.session==sessionID,value.stop==request.stop,value.stop==latestThreadStop,value.thread==request.thread,value.thread==selectedThread,value.frame==request.frame,value.frame==selectedFrame,value.variables.count<=10000 else{frameRequest=nil;frameBusy=false;frameVariablesCurrent=false;frameNotice="Frame variables response is malformed, stale or exceeds the response limit. Current frames are retained; refresh and read again.";return}
        frameRequest=nil;frameBusy=false;frameVariablesCurrent=true
        variables=value.variables.map{item in ["name":item.name,"value":item.value,"type_name":item.type_name as Any? ?? NSNull(),"variables_reference":item.variables_reference ?? 0]}
        if selectedFrame==nativeDebugID(frames.first?["id"]) {topVariables=variables}
        frameNotice=variables.isEmpty ? "No variables were reported for this frame.":"Variables reported for the selected stopped frame."
    }
    func readFrameLocals() {
        guard canReadFrameLocals,let sessionID,let stop=latestThreadStop,let thread=selectedThread,let frame=selectedFrame else{return}
        let id=UUID().uuidString;frameRequest=NativeDebugEvaluation(id:id,stop:stop,thread:thread,frame:frame);frameBusy=true;frameVariablesCurrent=false;frameNotice="Reading selected-frame variables…"
        send(["op":"state","session":sessionID,"stop":stop,"thread":thread,"frame":frame,"request_id":id])
    }
    func refreshPausedState() {
        guard status=="paused",let sessionID,!threadBusy,!refreshingPause else{return}
        retireThreadChoices();refreshingPause=true;threadNotice="Refreshing the current paused stack…";send(["op":"state","session":sessionID])
    }
    func readThreads() {
        guard canReadThreads,let sessionID,let stop=latestThreadStop else{return}
        let id=UUID().uuidString;threadRequest=NativeDebugThreadRequest(id:id,stop:stop);threadBusy=true;threadNotice="Reading reported threads…"
        send(["op":"threads","session":sessionID,"request_id":id])
    }
    func selectThread(_ thread:Int64,snapshot:UUID) {
        guard canSelectThreads,let sessionID,let stop=threadStop,snapshot==threadSnapshot,threads.contains(where:{$0.id==thread}) else{threadNotice="Thread choices changed. Read the current stopped threads again.";return}
        guard thread != selectedThread else{return}
        let id=UUID().uuidString;threadRequest=NativeDebugThreadRequest(id:id,stop:stop,thread:thread);threadBusy=true;threadNotice="Requesting the selected thread’s stopped stack…"
        send(["op":"select_thread","session":sessionID,"thread":thread,"stop":stop,"request_id":id])
    }
</ui:copy>

## Guard explicit runtime commands and expression identity

Breakpoint changes, stepping, stopping and evaluation remain explicit model operations. Frame rows carry their rendered snapshot. Evaluate only at a known pause and retain per-expression request identities with exact UTF-8 keys; canonically equivalent Swift strings may be different runtime expressions. Removing a watch retires only that exact expression and its pending reply. Source navigation compares captured text and never claims that runtime inputs were frozen.

<ui:copy id="native-debug-session-commands-apple">
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
        guard canResume,let sessionID,stackNotice.isEmpty || how=="continue" else { return }
        var request: [String: Any] = ["op": "step", "session": sessionID, "how": how]
        if stackNotice.isEmpty,let selectedFrame { request["frame"] = selectedFrame }
        retireThreadChoices();status = "running"; pausedLine = nil; variables = []; values = [:]; send(request)
    }
    func stop() {
        guard let sessionID else {
            if status == "starting" || status == "stopping" { stopPending = true; status = "stopping"; message = "Stopping as soon as the adapter acknowledges launch…" }
            return
        }
        retireThreadChoices();status = "stopping"; pausedLine = nil; variables = []; values = [:]; send(["op": "stop", "session": sessionID])
    }
    func isWatched(_ expression:String)->Bool {watches.contains{exactUTF8($0,expression)}}
    func removeWatch(_ expression:String) {watches.removeAll{exactUTF8($0,expression)};values.removeValue(forKey:expression);evaluations.removeValue(forKey:Data(expression.utf8))}
    func evaluate(_ expression: String, watch: Bool = false) {
        guard canEvaluate,let sessionID,let stop=latestThreadStop,let thread=selectedThread,!expression.isEmpty else{return}
        if watch && !isWatched(expression) {watches.append(expression)}
        evaluationNotice="";let id=UUID().uuidString;evaluations[Data(expression.utf8)]=NativeDebugExpressionRequest(expression:expression,request:NativeDebugEvaluation(id:id,stop:stop,thread:thread,frame:selectedFrame))
        var request:[String:Any]=["op":"eval","session":sessionID,"expression":expression,"context":watch ? "watch":"repl","stop":stop,"thread":thread,"request_id":id]
        if let selectedFrame {request["frame"]=selectedFrame};send(request)
    }
    func select(_ frame:[String:Any],snapshot:UUID?=nil) {
        guard status=="paused",snapshot==nil || snapshot==frameSnapshot,let id=nativeDebugID(frame["id"]),frames.contains(where:{nativeDebugID($0["id"])==id}),threadRequest?.thread==nil,!refreshingPause,stackNotice.isEmpty else{return}
        selectedFrame=id;evaluations=[:];evaluationNotice="";frameRequest=nil;frameBusy=false;frameVariablesCurrent=false
        variables = selectedFrame == nativeDebugID(frames.first?["id"]) ? topVariables : []; values = [:]
        readFrameLocals()
        for expression in watches { evaluate(expression, watch: true) }
        if let path = frame["source"] as? String, let line = nativeDebugLine(frame["source_line"]) { revealLocation(path:path,line:line) }
        else if let line = nativeDebugLine(frame["line"]) { revealLocation(path:owner,line:line) }
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
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/apple-simple/Sources/Engine.swift"><ui:paste select="#simple-ff9425b170-0"/><ui:paste select="#simple-ff9425b170-1"/><ui:paste select="#simple-apple-language-transport"/><ui:paste select="#simple-apple-language-session"/><ui:paste select="#simple-apple-language-snapshots"/><ui:paste select="#native-provider-action-resolution-apple"/><ui:paste select="#native-language-after-action-resolution-apple"/><ui:paste select="#native-completion-acceptance-apple"/><ui:paste select="#native-completion-acceptance-apple-tail"/><ui:paste select="#simple-apple-debug-session"/><ui:paste select="#native-debug-workspace-session-apple"/><ui:paste select="#native-debug-reply-admission-apple"/><ui:paste select="#native-debug-inspection-requests-apple"/><ui:paste select="#native-debug-session-commands-apple"/></ui:file>

</ui:doc>
