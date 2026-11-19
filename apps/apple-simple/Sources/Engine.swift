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
    var id: NativeSourceKey { NativeSourceKey(path) }
}
struct FileTree: Decodable { var root: String; var root_path: String? = nil; var workspace_id: String? = nil; var tree: [FileNode] }
struct NativeStartupSource: Decodable {
    var version: Int
    var path: String
    var content: String
    var root_path: String
    var workspace_id: String?
}
struct FileText: Decodable { var path: String; var content: String; var hash: String; var save_guard: String? = nil; var save_guard_reason: String? = nil }
struct FileSaved: Decodable { var path: String; var hash: String; var save_guard: String? = nil }
struct SavedDocument: Decodable { var source: String; var updated_at: String?; var save_guard: String? = nil; var save_guard_reason: String? = nil }
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
        if path=="/api/launch-profiles"{try validateNativeJSON(data,canonicalVersionKeys:["debug_protocol"])}
        if path.hasSuffix("/fence-debug"){try validateNativeReviewJSON(data,maxBytes:4096,versionKeys:["version","opening_line"],label:"Recorded fence Debug")}
        if path.hasSuffix("/fence-construction") {
            try validateNativeReviewJSON(data, maxBytes: 2 * 1024 * 1024,
                                         versionKeys: ["version", "opening_line", "opening_byte", "bytes"],
                                         label: "Recorded construction")
        }
        if path.hasSuffix("/fence-run"){try validateNativeReviewJSON(data,maxBytes:32768,versionKeys:["version","opening_line"],label:"Recorded fence Run")}
        if path == "/api/startup-source" { try validateNativeReviewJSON(data, maxBytes: 64 * 1024 * 1024, versionKeys: ["version"], label: "Startup source") }
        if path.hasPrefix("/api/workspace/recovery"){try validateNativeReviewJSON(data,maxBytes:128*1024*1024,versionKeys:["version","recovery_protocol"],label:"Private recovery")}
        if path.hasPrefix("/api/agent-proposals") || path.hasSuffix("/fence-patch"){try validateNativeAgentProposalResponse(data)}
        if path.hasPrefix("/api/tests/") || path.hasPrefix("/api/templates/") || path.hasPrefix("/api/source-history") || path.hasPrefix("/api/behavior/") || path.hasPrefix("/api/changes/") || path.hasPrefix("/api/git/conflict-resolution/"){try validateSourceTestJSON(data)}
        return try JSONDecoder().decode(T.self, from: data)
    }
    func find(_ query: String, regex: Bool, matchCase: Bool, wholeWord: Bool) async throws -> WorkspaceSearchResult {
        struct Occurrence: Decodable { var column: Int; var length: Int }
        struct Row: Decodable { var line: Int; var text: String; var at: [Occurrence] }
        struct File: Decodable { var path: String; var source_hash: String?; var generated_by: String?; var matches: [Row] }
        struct Result: Decodable { var files: [File]; var truncated: Bool }
        let result: Result = try await request("GET", "/api/find", query: [URLQueryItem(name: "q", value: query), URLQueryItem(name: "regex", value: String(regex)), URLQueryItem(name: "case", value: String(matchCase)), URLQueryItem(name: "whole_word", value: String(wholeWord))])
        return WorkspaceSearchResult(hits: result.files.flatMap { file in file.matches.flatMap { row in row.at.map { occurrence in WorkspaceSearchHit(path: file.path, line: row.line, text: row.text, column: occurrence.column, sourceHash: file.source_hash, generatedBy: file.generated_by) } } }, truncated: result.truncated)
    }
    func files() async throws -> FileTree { try await request("GET", "/api/files") }
    func startupSource(_ path: String) async throws -> NativeStartupSource {
        let reply: NativeStartupSource = try await request("GET", "/api/startup-source", query: [URLQueryItem(name: "path", value: path)])
        guard reply.version == 1, exactUTF8(path, reply.path), !reply.root_path.isEmpty else {
            throw EngineError(message: "The server returned a different startup source. Open its exact path in Files.")
        }
        return reply
    }
    func file(_ file: FileNode) async throws -> FileText {
        if let id = file.doc_id {
            let document: SavedDocument = try await request("GET", "/api/docs/" + id)
            return FileText(path: file.path, content: document.source, hash: "", save_guard: document.save_guard, save_guard_reason: document.save_guard_reason)
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
    func send(_ prompt: String, parent: String?, provider: String, model: String, effort: String, context:[String:Any]?=nil) async throws -> StartedTurn {
        try await request("POST", "/api/docs/workspace/agent", body: ["prompt": prompt, "parent_id": parent as Any? ?? NSNull(), "backend": "python", "provider": provider, "model": model, "effort": effort.isEmpty ? NSNull() : effort as Any,"context":context as Any? ?? ["live":false,"buffers":[]]])
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
    var sourceIdentity:UUID?=nil
    var selectionIdentity:UUID?=nil
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
struct NativeSourcePreparationAck:Decodable {
 let processed:Bool;let current:Bool;let parsed:Bool;let open_files:UInt64;let source_hash:String?;let source_revision:String?;let source_files_modified:Bool
}
func validateNativeSourcePreparation(_ raw:Any,source:String,requireParsed:Bool=false)throws {
 guard JSONSerialization.isValidJSONObject(raw),let bytes=try? JSONSerialization.data(withJSONObject:raw),let value=try? JSONDecoder().decode(NativeSourcePreparationAck.self,from:bytes),value.processed,value.current,!value.source_files_modified,!requireParsed||value.parsed,value.source_hash=="sha256:"+nativeSourceHash(source),let revision=value.source_revision,revision.range(of:"^[a-f0-9]{64}$",options:.regularExpression) != nil else{throw EngineError(message:"Source preparation is unsupported, superseded or incomplete. Retry language services and prepare again.")}
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
    private var locationOpening = UUID()
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
        guard let buffer, exactUTF8(languageURI(buffer.file.path),uri), exactUTF8(buffer.text,source) else { throw EngineError(message:"Source changed before analysis. Run again.") }
        sync(); let current = epoch
        let ack=try await request("hick/captureDocument",["textDocument":["uri":uri,"languageId":"hick","version":version,"text":source]])
        guard epoch==current,exactUTF8(languageURI(buffer.file.path),uri),exactUTF8(buffer.text,source) else{throw EngineError(message:"Source or provider changed during preparation. Run again.")}
        try validateNativeSourcePreparation(ack,source:source,requireParsed:buffer.file.doc_id != nil)
        let result = try await request("textDocument/documentSymbol", ["textDocument":["uri":uri]])
        guard epoch == current, exactUTF8(languageURI(buffer.file.path),uri), exactUTF8(buffer.text,source) else { throw EngineError(message:"Source or provider changed during analysis. Run again.") }
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
        let opening = UUID()
        locationOpening = opening
        if exactUTF8(location.path,buffer.file.path) {
            guard let offset = languageOffset(buffer.text, ["line": location.line, "character": location.character]),
                  buffer.selectSourceRange?(NSRange(location: offset, length: 0), buffer.text) == true else {
                locationStatus = "Location no longer fits this source. Query again."; return
            }
            let position = languagePosition(locationSource, locationSelection.location)
            buffer.rememberNavigation?(SemanticLocation(uri: uri, line: position["line"]!, character: position["character"]!), location)
            buffer.focusSource?()
        } else {
            // A cross-file result can wait on disk after the menu accepts it.
            let query = navigationID, provider = epoch, revision = buffer.sourceEditIdentity
            buffer.openSemanticLocation?(location) { [weak self, weak buffer] in
                guard let self, let buffer else { return false }
                return self.locationOpening == opening && self.navigationID == query && self.epoch == provider
                    && exactUTF8(languageURI(buffer.file.path), self.uri) && buffer.sourceEditIdentity == revision
                    && exactUTF8(buffer.text, self.locationSource) && buffer.selection == self.locationSelection
                    && self.locations.contains(location)
                    && (self.locationSymbolQuery == nil || exactUTF8(self.locationSymbolQuery, self.symbolQuery))
            }
        }
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
        guard let buffer, !buffer.readOnly, !buffer.resourceDeleted, !buffer.saving, exactUTF8(languageURI(buffer.file.path),uri), subscription != nil else { throw EngineError(message: "Choose writable source with a connected language provider.") }
        sync(); let source = buffer.text, selection = buffer.selection, identity = epoch,revision=buffer.sourceEditIdentity,selectionRevision=buffer.selectionIdentity
        func current()->Bool {self.buffer === buffer && epoch==identity && buffer.sourceEditIdentity==revision && buffer.selectionIdentity==selectionRevision && exactUTF8(languageURI(buffer.file.path),uri) && exactUTF8(buffer.text,source) && buffer.selection==selection && !buffer.readOnly && !buffer.resourceDeleted && !buffer.saving}
        var params: [String:Any] = ["textDocument":["uri":uri]]
        let method: String
        switch kind {
        case "rename": method = "hick/rename"; params["position"] = languagePosition(source,selection.location); params["newName"] = name
        case "references": method = "textDocument/references"; params["position"] = languagePosition(source,selection.location); params["context"] = ["includeDeclaration":true]
        case "format": method = "hick/formatting"; params["options"] = ["tabSize":4,"insertSpaces":!source.components(separatedBy:"\n").contains(where:{$0.hasPrefix("\t")})]
        case "fixAll": method = "textDocument/codeAction"; params["range"] = ["start":languagePosition(source,0),"end":languagePosition(source,source.utf16.count)]; params["context"] = ["diagnostics":[],"only":["source.fixAll"]]
        case "sourceActions": method = "textDocument/codeAction"; params["range"] = ["start":languagePosition(source,0),"end":languagePosition(source,source.utf16.count)]; params["context"] = ["diagnostics":[],"only":["source"]]
        default: method = "textDocument/codeAction"; params["range"] = ["start":languagePosition(source,selection.location),"end":languagePosition(source,NSMaxRange(selection))]; params["context"] = ["diagnostics":[]]
        }
        if kind=="rename" || kind=="format" {
            let ack=try await request("hick/captureDocument",["textDocument":["uri":uri,"languageId":"hick","version":version,"text":source]])
            guard current() else{throw EngineError(message:"Source, selection or provider changed during preparation. Prepare again.")}
            try validateNativeSourcePreparation(ack,source:source,requireParsed:buffer.file.doc_id != nil)
            guard let captured=ack as? [String:Any],let hash=captured["source_hash"] as? String,let prepared=captured["source_revision"] as? String else{throw EngineError(message:"Source preparation proof is unavailable. Prepare again.")}
            params["expected"]=["source_hash":hash,"source_revision":prepared]
        }
        let result = try await request(method,params)
        guard current() else { throw EngineError(message: "Source, selection or provider changed. Prepare again.") }
        return result
    }
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
    func complete() async {
        showingSuggestions = true
        guard let buffer, !buffer.readOnly else { return }
        sync(); let source = buffer.text, selection = buffer.selection, current = epoch, revision=buffer.sourceEditIdentity,selectionRevision=buffer.selectionIdentity, id = UUID()
        completionID = id
        status = "Updating completions; showing last received results…"
        do {
            // The bridge prepares virtual files asynchronously after full-text sync.
            // Keep the popup responsive while typing settles; never use an obsolete position.
            try await Task.sleep(for: .milliseconds(350))
            guard self.buffer === buffer,buffer.sourceEditIdentity==revision,buffer.selectionIdentity==selectionRevision,!buffer.resourceDeleted,exactUTF8(languageURI(buffer.file.path),uri),exactUTF8(self.buffer?.text,source), self.buffer?.selection == selection, epoch == current, completionID == id else { return }
            let result = try await request("hick/completion", ["textDocument": ["uri": uri], "position": languagePosition(source, selection.location),"transform_program_versions":supportedSnippetTransformPrograms])
            guard self.buffer === buffer,buffer.sourceEditIdentity==revision,buffer.selectionIdentity==selectionRevision,!buffer.resourceDeleted,exactUTF8(languageURI(buffer.file.path),uri),exactUTF8(self.buffer?.text,source), self.buffer?.selection == selection, epoch == current, completionID == id else { return }
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
                fresh.append(NativeCompletion(id:index,label:label,detail:item["detail"] as? String ?? "Language server",range:range,insert:item["insertText"] as? String ?? label,source:source,selection:selection,original:item,sourceIdentity:revision,selectionIdentity:selectionRevision))
                if fresh.count >= 100 { break }
            }
            suggestions = fresh
            status = suggestions.isEmpty ? "No supported completions at this position. Open Language tools for setup or retry." : "LSP · \(suggestions.count) completions"
        } catch { if epoch == current,completionID==id,buffer.sourceEditIdentity==revision,buffer.selectionIdentity==selectionRevision,exactUTF8(buffer.text,source),buffer.selection==selection { status = "Completion unavailable: \(error.localizedDescription)" } }
    }
    var completionNotice=""
    private var acceptingCompletion=UUID()
    private var completionPending=false
    func cancelCompletion() {
        acceptingCompletion=UUID()
        if completionPending{completionNotice="Completion cancelled."}
        completionPending=false
    }
    var snippetClipboardReader:(()->String?)?=nil
    func resolveCompletion(_ item:[String:Any]) async throws->[String:Any] {
        if item["data"]==nil || item["data"] is NSNull{return item}
        guard let buffer,!buffer.readOnly,!buffer.saving,!buffer.resourceDeleted,subscription != nil else{throw EngineError(message:"Choose writable source with a connected language provider.")}
        sync();let source=buffer.text,selection=buffer.selection,identity=epoch,revision=buffer.sourceEditIdentity,selectionRevision=buffer.selectionIdentity,acceptance=acceptingCompletion
        func current()->Bool{self.buffer === buffer && epoch==identity && acceptingCompletion==acceptance && buffer.sourceEditIdentity==revision && buffer.selectionIdentity==selectionRevision && !buffer.readOnly && !buffer.saving && !buffer.resourceDeleted && exactUTF8(languageURI(buffer.file.path),uri) && exactUTF8(buffer.text,source) && buffer.selection==selection}
        return try await resolveSnippetCompletion(item,source:source,from:selection.location,to:NSMaxRange(selection),current:current,request:{item in var params=item;params["textDocument"]=["uri":self.uri];let result=try await self.request("completionItem/resolve",params);guard let value=result as? [String:Any],value["label"] is String else{throw EngineError(message:"Provider did not resolve this completion. Request again.")};return value},clipboard:snippetClipboardReader)
    }
    @discardableResult func accept(_ completion:NativeCompletion,resolution:(([String:Any]) async throws->[String:Any])?=nil) async->Bool {
        guard let buffer,!buffer.readOnly,!buffer.saving,!buffer.resourceDeleted,completion.sourceIdentity.map({$0==buffer.sourceEditIdentity}) ?? true,completion.selectionIdentity.map({$0==buffer.selectionIdentity}) ?? true,exactUTF8(buffer.text,completion.source),buffer.selection==completion.selection else{completionNotice="Suggestion is out of date. Request completions again.";return false}
        if exactUTF8(buffer.multiSource,buffer.text),buffer.multiRanges.count>1{completionNotice="Choose one caret before accepting this source completion.";return false}
        guard let original=completion.original else {
            cancelCompletion()
            let accepted=buffer.snippets.active != nil ? buffer.snippets.input(source:completion.source,range:completion.range,text:completion.insert)==true:buffer.applyLanguageCompletion?(completion.range,completion.insert,completion.source,completion.selection)==true
            guard accepted else{completionNotice="Editor refused the completion. Finish composition or request again.";return false}
            completionNotice="Completion selected";showingSuggestions=false;return true
        }
        let request=UUID(),identity=epoch,revision=buffer.sourceEditIdentity,selectionRevision=buffer.selectionIdentity,snippetIntent=buffer.snippets.intent;acceptingCompletion=request;completionPending=true;completionNotice="Resolving completion…"
        defer{if acceptingCompletion==request{completionPending=false}}
        do {
            let resolved=try await (resolution ?? {try await self.resolveCompletion($0)})(original)
            guard acceptingCompletion==request else{return false}
            guard self.buffer === buffer,epoch==identity,buffer.sourceEditIdentity==revision,buffer.selectionIdentity==selectionRevision,!buffer.resourceDeleted,exactUTF8(languageURI(buffer.file.path),uri),buffer.snippets.intent==snippetIntent,buffer.language===self,!buffer.readOnly,!buffer.saving,exactUTF8(buffer.text,completion.source),buffer.selection==completion.selection,!(exactUTF8(buffer.multiSource,buffer.text)&&buffer.multiRanges.count>1) else{throw EngineError(message:"Source, cursor or provider changed. Request completions again.")}
            let contextNotice=try snippetContextNotice(resolved)
            let initialDraft=try planCompletion(uri,item:resolved,snapshot:SourceSnapshot(text:completion.source,writable:!buffer.readOnly && !buffer.saving),from:completion.range.location,to:NSMaxRange(completion.range),expansion:try completionExpansion(resolved))
            let draft=try buffer.snippets.prepareCompletion(initialDraft)
            guard exactUTF8(draft.path,buffer.file.path),buffer.applyCompletionDraft?(draft,completion.selection)==true else{throw EngineError(message:"Editor refused the complete completion. Finish composition or request again.")}
            completionPending=false;completionNotice=(draft.plan.patches.isEmpty ? "Completion selected":"Completion applied · draft")+(contextNotice.isEmpty ? "":" · "+contextNotice);showingSuggestions=false;return true
        }catch{if acceptingCompletion==request{completionNotice=error.localizedDescription};return false}
    }

}
func nativeDebugURI(_ path:String)->String {path.hasPrefix("hick://") ? path:languageURI(path)}
struct DebugLine: Equatable { var line: Int; var y: Double; var height: Double }
struct NativeDebugAttach:Decodable {
    var port:UInt16;var arguments:ResourceJSON
    private enum CodingKeys:String,CodingKey{case port,arguments}
    init(from decoder:Decoder)throws{
        let values=try decoder.container(keyedBy:CodingKeys.self);port=try values.decode(UInt16.self,forKey:.port);arguments=try values.decode(ResourceJSON.self,forKey:.arguments)
        guard port>0,case .object=arguments else{throw DecodingError.dataCorruptedError(forKey:.port,in:values,debugDescription:"Attach requires a nonzero loopback port and object arguments.")}
    }
}
struct NativeDebugProfile: Decodable, Identifiable {
    var id: String; var name: String; var program: String?; var can_debug: Bool
    var attach:NativeDebugAttach?=nil;var default_profile:Bool?=nil
    private enum CodingKeys:String,CodingKey{case id,name,program,can_debug,attach;case default_profile="default"}
    var exactID:Data {Data(id.utf8)}
    var actionLabel:String {attach == nil ? "Debug \(name)":"Attach \(name)"}
    var attachNotice:String? {attach.map{"Attach · loopback port \($0.port) · external source revision unverified"}}
    var runUnavailableReason:String? {attach == nil ? nil:"Attach profiles connect an existing loopback process; Run is unavailable."}
}
struct NativeDebugTarget: Identifiable { var path: String; var line: Int; var end: Int; var id: NativeSourceKey {NativeSourceKey("\(path):\(line)")} }
struct NativeDebugRender: Decodable {
    struct Block: Decodable { var kind: String; var path: String?; var span: [Int]? }
    var blocks: [Block]
}
struct NativeDebugCatalog: Decodable {var profiles:[NativeDebugProfile];var fingerprint:String;var viewed:String?;var debug_protocol:Int?;var notices:[String]?}
enum RecordedMessageSide: String {
    case user, assistant
    var bufferSuffix: String { self == .user ? "user" : "agent" }
}
struct NativeRecordedMessageSource {
    var turn: String
    var side: RecordedMessageSide
    var workspace: String?
    var workspaceID: String? = nil
    var current: @MainActor () -> Bool
    var actionRoot: String? {
        guard (1...128).contains(turn.utf8.count), turn != ".", turn != "..",
              !turn.contains("/"), !turn.contains("\\"),
              !turn.unicodeScalars.contains(where: CharacterSet.controlCharacters.contains) else { return nil }
        return "/api/docs/workspace/agent/turns/" + turn
    }
}
private struct NativeRecordedDebugReceipt: Decodable {
    var version: Int
    var id: String
    var uri: String
    var program: String
    var source_hash: String
    var opening_line: Int
    var workspace: String
    var started: Bool
    var turn: String
    var side: String
}
private struct NativeRecordedDebugLaunch {
    var receipt: NativeRecordedDebugReceipt
    var fence: RecordedDebugFence
    var current: @MainActor () -> Bool
    var holdsCapture = true
}
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
    var expression:String;var value:String;var stop:UInt64;private var runtimeThread:NativeDebugID?;private var runtimeFrame:NativeDebugID?;var thread:Int64? {runtimeThread.map{Int64($0.value)}};var frame:Int? {runtimeFrame.map{Int($0.value)}};var request_id:String?;var reference:Int64?
    private enum CodingKeys:String,CodingKey {case expression,value,stop,runtimeThread="thread",runtimeFrame="frame",request_id,reference}
}
struct NativeDebugVariableRow:Identifiable {var id:UUID;var depth:Int;var name:String;var value:String;var reference:Int64;var expanded=false;var pending=false;var retryable=false;var notice="";var children:[UUID]=[]}
private struct NativeDebugExpansion {var request:NativeDebugEvaluation;var node:UUID;var reference:Int64}
private struct NativeDebugSourceIdentity {var buffer:NativeDebugBufferRef;var id:UUID;var path:String;var confirmed:Bool}
private struct NativeDebugBreakpointRequest {var id:String;var source:String}
private final class NativeDebugBufferRef {weak var buffer:DocumentBuffer?;init(_ buffer:DocumentBuffer){self.buffer=buffer}}
private struct NativeDebugVariable:Decodable {var name:String;var value:String;var type_name:String?;var variables_reference:Int64?}
private struct NativeDebugFrameVariables:Decodable {
    var session:String;var stop:UInt64;private var runtimeThread:NativeDebugID?;private var runtimeFrame:NativeDebugID?;var thread:Int64? {runtimeThread.map{Int64($0.value)}};var frame:Int? {runtimeFrame.map{Int($0.value)}};var request_id:String?;var variables:[NativeDebugVariable]
    private enum CodingKeys:String,CodingKey {case session,stop,runtimeThread="thread",runtimeFrame="frame",request_id,variables}
}

@MainActor @Observable final class NativeDebugger {
    var status = "idle"
    var message = ""
    private var ownerKey=NativeSourceKey("")
    var owner:String {get{ownerKey.string}set{ownerKey=NativeSourceKey(newValue)}}
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
    var stoppedObservation:UInt64? {status=="paused" && pauseIdentityKnown ? latestThreadStop:nil}
    var canInspectFrame:Bool {canResume && stackNotice.isEmpty}
    var canReadFrameLocals:Bool {canInspectFrame && latestThreadStop != nil && selectedThread != nil && selectedFrame != nil && !frameBusy}
    var canEvaluate:Bool {canInspectFrame && latestThreadStop != nil && selectedThread != nil}
    private(set) var frameSnapshot=UUID()
    private var evaluations:[Data:NativeDebugExpressionRequest]=[:]
    private var pauseIdentityKnown=false
    private var latestThreadStop:UInt64?
    private var threadRequest:NativeDebugThreadRequest?
    let adapterSources=NativeAdapterSourceModel()
    var adapterSourceBinding:NativeAdapterSourceBinding? {guard canInspectFrame,let sessionID,let stop=latestThreadStop,let thread=selectedThread,let frame=selectedFrame,let selected=frames.first(where:{nativeDebugID($0["id"])==frame}),let reference=nativeDebugID(selected["source_reference"]),reference>0 else{return nil};return NativeAdapterSourceBinding(session:sessionID,stop:stop,thread:thread,frame:frame,reference:reference)}
    var canReadAdapterSource:Bool {guard let number=capabilities["source_content"] as? NSNumber else{return false};return CFGetTypeID(number)==CFBooleanGetTypeID()&&number.boolValue&&adapterSourceBinding != nil}
    func readAdapterSource(_ inspection:UUID){if let request=adapterSources.begin(adapterSourceBinding,capability:capabilities["source_content"],inspection:inspection){send(request)}}
    var frames: [[String: Any]] = []
    var variables: [[String: Any]] = []
    var capabilities: [String: Any] = [:]
    private(set) var breakpoints: NativeSourceMap<[[String:Any]]> = [:]
    private var intentSources:[Data:String]=[:]
    private var boundSources:[Data:String]=[:]
    private var breakpointRequests:[Data:NativeDebugBreakpointRequest]=[:]
    private var breakpointBuffers:[Data:NativeDebugBufferRef]=[:]
    var breakpointNotice=""
    private var retainedBreakpointIntents:[Data:[[String:Any]]]=[:]
    func retainedBreakpointReading(_ path:String)->[String] {
        let key=Data(path.utf8)
        func render(_ mark:[String:Any],earlier:Bool)->String {let raw=mark["source_hash"] as? String ?? "";let hash=raw.range(of:"^[0-9a-f]{64}$",options:.regularExpression) != nil ? String(raw.prefix(8)):"hash unconfirmed";return "\(earlier ? "Earlier":"Retained") source \(hash): line \((mark["line"] as? Int ?? 0)+1)"+((mark["condition"] as? String).map{" if \($0)"} ?? "")}
        let previous=(retainedBreakpointIntents[key] ?? []).map{render($0,earlier:true)}
        let same=breakpointBuffers[key]?.buffer.map{buffer in intentOwners[key]==buffer.id&&intentSources[key].map{exactUTF8($0,buffer.text)}==true} ?? false
        let current=breakpointIntents(path).map{mark->String in var row=mark;if let source=intentSources[key],boundSources[key].map({exactUTF8($0,source)})==true,let hash=sourceHashes[key]{row["source_hash"]=hash};return render(row,earlier:!same)}
        return previous+current
    }
    private var variableNodes:[UUID:NativeDebugVariableRow]=[:]
    private var variableRoots:[UUID]=[]
    private var evaluationRoots:[Data:UUID]=[:]
    private var expansions:[String:NativeDebugExpansion]=[:]
    private var movementRequest:(id:String,stackNotice:String)?
    private(set) var pauseUnconfirmed=false
    var visibleVariableRows:[NativeDebugVariableRow] {var result:[NativeDebugVariableRow]=[];func visit(_ id:UUID){guard let node=variableNodes[id] else{return};result.append(node);if node.expanded {for child in node.children{visit(child)}}};for id in variableRoots{visit(id)};return result}
    var output: [NativeRunOutput] = []
    var values=NativeDebugValues()
    var watches: [String] = []
    var profiles: [NativeDebugProfile] = []
    var targets: [NativeDebugTarget] = []
    private var launchProgramKey:NativeSourceKey?
    private var launchProgram:String? {get{launchProgramKey?.string}set{launchProgramKey=newValue.map(NativeSourceKey.init)}}
    var catalog: NativeDebugCatalog?
    var catalogNotices:[String]=[]
    private(set) var attachReading:String?
    @ObservationIgnored var sourceBuffers:(()->[DocumentBuffer])?
    private var sourceIdentities:[Data:NativeDebugSourceIdentity]=[:]
    private var intentOwners:[Data:UUID]=[:]
    private var sourceNamespace:UInt64=0
    private var launchNamespace:UInt64=0
    private var preparationNamespace:UInt64=0
    private var launchText=ExactSourceText("")
    var launchSource:String {get{launchText.text}set{launchText=ExactSourceText(newValue)}}
    private var launchBoundHash:String?
    private var sourceHashes:[Data:String]=[:]
    private var ambiguousSources=Set<Data>()
    var pausedLine: Int?
    var selectedFrame: Int?
    var offer: [String: String]?
    @ObservationIgnored var onLocation: ((NativeDebugLocation) -> Void)?
    @ObservationIgnored var observeRequestForTest: (([String:Any])->Void)?
    @ObservationIgnored var observeEventForTest: (([String:Any])->Void)?
    @ObservationIgnored var emitForTest: (([String: Any]) -> Void)?
    private let engine: Engine
    private weak var launchBuffer: DocumentBuffer?
    private var recordedLaunch: NativeRecordedDebugLaunch?
    private var socket: URLSessionWebSocketTask?
    private var receiver: Task<Void, Never>?
    private var sender: Task<Void, Never>?
    private var epoch = UUID()
    private var topVariables: [[String: Any]] = []
    private var stopPending = false
    private var pendingLaunch:NativeDebugLaunch?
    private var pendingSourceTest:(plan:SourceTestDebugPlan,facts:SourceTestScan,current:()->Bool,admitted:(String)->Void,rejected:(String)->Void,outcome:(String,String,Int?)->Void)?
    private var sourceTestDeadline:Task<Void,Never>?
    private var sourceTestPreparing=false
    private var testOutcomes:[String:(String,String,Int?)->Void]=[:]
    var retainedSessionID:String?
    private var activeLaunch:NativeDebugLaunch?
    private var retiredLaunches:[NativeDebugLaunch]=[]
    var launchRequestID:String? {pendingLaunch?.id ?? activeLaunch?.id}
    private func retireLaunches() {
        clearSourceTestDeadline()
        for value in [pendingLaunch,activeLaunch].compactMap({$0}) where !retiredLaunches.contains(where:{$0.id==value.id}) {retiredLaunches.append(value)}
        retiredLaunches=Array(retiredLaunches.suffix(64));pendingLaunch=nil;activeLaunch=nil
    }
    // A plan read participates in the same latest-choice lifetime as ordinary profile reads,
    // without discarding the previous inactive runtime reading or guessing a launch target.
    func claimTestPreparation()->UUID? {guard !active else{return nil};epoch=UUID();profiles=[];return epoch}
    func testPreparationCurrent(_ preparation:UUID)->Bool {epoch==preparation}
    func sourceInspectionBuffer()->DocumentBuffer? {launchBuffer}
    @discardableResult func beginLaunchRequest(document: String? = nil)->String {
        retainedSessionID=nil
        launchNamespace=sourceNamespace;sourceIdentities=[:]
        for buffer in (sourceBuffers?() ?? [])+[launchBuffer].compactMap({$0})+breakpointBuffers.values.compactMap({$0.buffer}) where !buffer.resourceDeleted {let key=Data(buffer.file.path.utf8);if sourceIdentities[key]==nil{sourceIdentities[key]=NativeDebugSourceIdentity(buffer:NativeDebugBufferRef(buffer),id:buffer.id,path:buffer.file.path,confirmed:true)}}
        launchBoundHash=nil;boundSources=[:];breakpointRequests=[:];sourceHashes=[Data(owner.utf8):nativeSourceHash(launchSource)];ambiguousSources=[]
        retireLaunches();let id=UUID().uuidString;pendingLaunch=NativeDebugLaunch(id:id,doc:document ?? nativeDebugURI(owner));sessionID=nil;stopPending=false;message="";status="starting";return id
    }
    private func stopRetiredLaunch(_ event:[String:Any])->Bool {
        guard event["event"] as? String=="started",let id=event["request_id"] as? String,
              let index=retiredLaunches.firstIndex(where:{$0.id==id}),
              exactUTF8(event["doc"] as? String,retiredLaunches[index].doc),
              let session=event["session"] as? String,!session.isEmpty,session != sessionID else{return false}
        retiredLaunches.remove(at:index)
        send(["op":"stop","session":session]);return true
    }
    init(engine: Engine) { self.engine = engine }
    var active: Bool { ["starting", "running", "unconfirmed", "paused", "stopping"].contains(status) }
    var hasSessionReading:Bool {active || sessionID != nil || status=="finished" || status=="failed" || status=="unavailable" || !output.isEmpty || adapterSources.reading != nil}
    func line(in path: String) -> Int? {
        guard status == "paused",stackNotice.isEmpty,let reference=selectedSourceReference,reference<=0 else { return nil }
        let frame = frames.first(where: { nativeDebugID($0["id"]) == selectedFrame }) ?? frames.first
        if let recordedLaunch {
            guard exactUTF8(path, owner), let frame else { return nil }
            return recordedFrameLine(frame, launch: recordedLaunch)
        }
        if exactUTF8(path,owner), frame?["in_document"] as? Bool == true { return nativeDebugLine(frame?["line"]) }
        if exactUTF8(frame?["source"] as? String,path) { return nativeDebugLine(frame?["source_line"]) }
        return exactUTF8(path,owner) && selectedFrame == nativeDebugID(frames.first?["id"]) ? pausedLine : nil
    }
    func outputPlacements(_ buffer: DocumentBuffer) -> [Int: String] {
        guard sourceMatches(buffer) else { return [:] }
        if let recordedLaunch, exactUTF8(buffer.file.path, owner), !output.isEmpty {
            return [recordedLaunch.fence.closing: output.map(\.text).joined()]
        }
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
        if exactUTF8(path,owner){return launchBoundHash==nativeSourceHash(launchSource) ? launchBoundHash:nil}
        guard let frame=frames.first(where:{nativeDebugID($0["id"])==selectedFrame}),exactUTF8(frame["source"] as? String,path),let hash=frame["source_hash"] as? String,hash.range(of:"^[0-9a-f]{64}$",options:.regularExpression) != nil,sourceHashes[key]==hash else{return nil}
        return hash
    }
    private func identityMatches(_ buffer:DocumentBuffer,path:String,confirmed:Bool)->Bool {guard !buffer.resourceDeleted,exactUTF8(buffer.file.path,path),let identity=sourceIdentities[Data(path.utf8)],identity.id==buffer.id,identity.buffer.buffer === buffer else{return false};return !confirmed || identity.confirmed}
    @discardableResult func admitSourceReading(_ buffer:DocumentBuffer,path:String,proof:String)->Bool {
        guard !buffer.dirty,!buffer.saving,!buffer.resourceDeleted,exactUTF8(buffer.file.path,path),sourceProof(path)==proof,nativeSourceHash(buffer.text)==proof else{return false}
        let key=Data(path.utf8)
        if let known=sourceIdentities[key] {return known.id==buffer.id && known.buffer.buffer === buffer && known.confirmed}
        guard sourceNamespace==launchNamespace else{return false}
        sourceIdentities[key]=NativeDebugSourceIdentity(buffer:NativeDebugBufferRef(buffer),id:buffer.id,path:path,confirmed:true);return true
    }
    func sourceMatches(_ buffer:DocumentBuffer)->Bool {
        guard !buffer.dirty,!buffer.saving,identityMatches(buffer,path:buffer.file.path,confirmed:true),let proof=sourceProof(buffer.file.path) else{return false}
        if let recordedLaunch, exactUTF8(owner, buffer.file.path), !recordedLaunch.current() { return false }
        return proof==nativeSourceHash(buffer.text)
    }
    func resourceNamespaceChanged(_ routes:[ResourceEditorRoute<DocumentBuffer>]) {
        if sourceNamespace==launchNamespace {for route in routes {if let path=route.from,let buffer=route.editor,exactUTF8(buffer.file.path,path),sourceIdentities[Data(path.utf8)]==nil{sourceIdentities[Data(path.utf8)]=NativeDebugSourceIdentity(buffer:NativeDebugBufferRef(buffer),id:buffer.id,path:path,confirmed:false)}}}
        sourceNamespace &+= 1;frameSnapshot=UUID();breakpointRequests=[:];boundSources=[:]
        for key in sourceIdentities.keys {sourceIdentities[key]?.confirmed=false}
        breakpointNotice="Resource ownership changed. Captured runtime controls/history remain; source positions and breakpoint receipts need fresh original-identity proof."
    }
    private func confirmKnownSource(_ path:String,hash:String) {
        let key=Data(path.utf8);guard var identity=sourceIdentities[key],let buffer=identity.buffer.buffer,identity.id==buffer.id,!buffer.resourceDeleted,!buffer.dirty,!buffer.saving,exactUTF8(buffer.file.path,identity.path),nativeSourceHash(buffer.text)==hash else{return}
        identity.confirmed=true;sourceIdentities[key]=identity
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
    private func recordedFrameLine(_ frame: [String: Any], launch: NativeRecordedDebugLaunch) -> Int? {
        guard frame["in_document"] as? Bool == true,
              exactUTF8(frame["source"] as? String, launch.receipt.uri),
              exactUTF8(frame["source_hash"] as? String, launch.receipt.source_hash),
              let reference = sourceReference(frame), reference <= 0,
              let line = nativeDebugLine(frame["line"]), launch.fence.contains(line) else { return nil }
        return line
    }
    private func revealLocation(path:String,line:Int) {
        guard recordedLaunch == nil else { return }
        if let location=sourceLocation(path:path,line:line){onLocation?(location)}
    }
    private func revealRecordedFrame(_ frame: [String: Any], launch: NativeRecordedDebugLaunch) {
        guard let buffer = launchBuffer, sourceMatches(buffer), let line = recordedFrameLine(frame, launch: launch) else { return }
        _ = buffer.revealDebugLine?(line)
    }
    private func releaseRecordedCapture(_ id: String) {
        guard id.range(of: "^[0-9a-f]{16}$", options: .regularExpression) != nil else { return }
        let engine = self.engine
        Task {
            struct Removed: Decodable { var removed: Bool }
            let _: Removed? = try? await engine.request("DELETE", "/api/project-contexts/" + id)
        }
    }
    private func releaseOwnedRecordedCapture() {
        guard let launch = recordedLaunch, launch.holdsCapture else { return }
        recordedLaunch?.holdsCapture = false
        releaseRecordedCapture(launch.receipt.id)
    }
    private func retireRecordedCapture() {
        releaseOwnedRecordedCapture()
        recordedLaunch = nil
    }
    func closeRecordedFence() {
        if recordedLaunch != nil || (active && owner.hasPrefix("hick-agent-message:///")) { close() }
    }
    func prepareRecordedFence(_ buffer: DocumentBuffer, source: NativeRecordedMessageSource, fence: RecordedDebugFence) async {
        guard !active else { message = "Stop the current debugger before launching another program."; return }
        guard let workspace = source.workspace, let endpoint = source.actionRoot,
              source.current(), buffer.readOnly, !buffer.dirty, !buffer.saving, !buffer.resourceDeleted,
              exactUTF8(buffer.inlineMarkup.source, buffer.text), buffer.inlineMarkup.debug.recorded[fence.opening] == fence else {
            message = "The recorded message is unavailable or changed. Open its current code fence and try Debug again."
            return
        }
        close()
        let preparation = epoch, text = buffer.text, path = buffer.file.path
        let hash = nativeSourceHash(text)
        let namespace = sourceNamespace
        owner = path
        status = "starting"
        message = "Preparing recorded fence…"
        do {
            let receipt: NativeRecordedDebugReceipt = try await engine.request("POST", endpoint + "/fence-debug", body: [
                "side": source.side.rawValue, "source_hash": hash, "opening_line": fence.opening,
            ])
            let valid = receipt.version == 1 && !receipt.started && receipt.opening_line == fence.opening
                && receipt.id.range(of: "^[0-9a-f]{16}$", options: .regularExpression) != nil
                && exactUTF8(receipt.uri, "hick-project:///\(receipt.id)/message.md")
                && exactUTF8(receipt.turn, source.turn) && exactUTF8(receipt.side, source.side.rawValue)
                && exactUTF8(receipt.workspace, workspace) && exactUTF8(receipt.source_hash, hash)
            guard epoch == preparation else {
                if valid { releaseRecordedCapture(receipt.id) }
                return
            }
            guard valid else {
                fail("The engine did not confirm this recorded message and fence. No Start was sent. Refresh the conversation and try Debug again.")
                return
            }
            guard status == "starting", source.current(), sourceNamespace == namespace, exactUTF8(buffer.file.path, path),
                  exactUTF8(buffer.text, text), !buffer.dirty, !buffer.saving, !buffer.resourceDeleted else {
                releaseRecordedCapture(receipt.id)
                fail("The recorded message or launch choice changed. No Start was sent. Open its current code fence to try Debug again.")
                return
            }
            launchBuffer = buffer
            launchSource = text
            launchProgram = receipt.program
            recordedLaunch = NativeRecordedDebugLaunch(receipt: receipt, fence: fence, current: source.current)
            breakpointBuffers[Data(path.utf8)] = NativeDebugBufferRef(buffer)
            let launch = beginLaunchRequest(document: receipt.uri)
            values = [:]; variables = []; frames = []; pausedLine = nil; offer = nil; catalogNotices = []
            openTransport()
            var request: [String: Any] = ["op": "start", "doc": receipt.uri, "request_id": launch]
            appendLaunchBreakpoints(launch, to: &request)
            send(request)
        } catch {
            guard epoch == preparation else { return }
            fail("\(error.localizedDescription) Check the workspace connection and choose Debug on the current recorded fence.")
        }
    }
    private func compatibleDebugger(_ catalog:NativeDebugCatalog)->Bool {
        guard (catalog.debug_protocol ?? 0)>=3 else{self.catalog=nil;profiles=[];status="unavailable";message="Scoped debugging is unavailable from this engine. Update/reconnect an engine with debug protocol 3 before Debug. No Start command was sent.";return false}
        return true
    }
    func prepare(_ buffer: DocumentBuffer, line: Int = 0, program: String? = nil,chooseProfile:Bool=false) async {
        guard !active else { message = "Stop the current debugger before launching another program."; return }
        guard buffer.startupObservation == nil else { message = "Wait for this file’s workspace authority before Debug, or reopen it in Files."; return }
        guard !buffer.resourceDeleted else{message="Deleted source is retained history. Restore its proven identity or save a copy before a new launch.";return}
        guard !buffer.dirty, !buffer.saving else { message = "Save this file before debugging so the adapter runs the displayed text."; return }
        close(); let current = epoch;preparationNamespace=sourceNamespace;catalogNotices=[]
        owner = buffer.file.path; launchBuffer = buffer;breakpointBuffers[Data(owner.utf8)]=NativeDebugBufferRef(buffer); launchSource = buffer.text; sourceHashes = [Data(owner.utf8):nativeSourceHash(launchSource)];ambiguousSources=[]; message = "Resolving launch targets…"; profiles = []; targets = []
        do {
            let literate = buffer.file.doc_id != nil || ["md", "markdown"].contains((owner as NSString).pathExtension.lowercased())
            var target = program ?? inlineDebugPath(buffer.text, path: owner, line: line, launch: true) ?? inlineDebugPath(buffer.text, path: owner, line: line)
            if literate && target == nil {
                guard let id = buffer.file.doc_id else { message = "Refresh files to discover this document before choosing a file block to debug."; return }
                let rendered: NativeDebugRender = try await engine.request("GET", "/api/docs/" + id + "/render")
                guard epoch==current else{return}
                guard preparationNamespace==sourceNamespace,!buffer.resourceDeleted,exactUTF8(buffer.file.path,owner),exactUTF8(buffer.text,launchSource),!buffer.dirty,!buffer.saving else{message="Source identity or text changed during launch preparation. Choose Debug on the current saved source.";return}
                targets = rendered.blocks.compactMap { block in
                    guard block.kind == "file", let path = block.path, let span = block.span, span.count == 2,
                          span[0] >= 0, span[1] >= span[0], span[1] <= launchSource.utf8.count else { return nil }
                    return NativeDebugTarget(path: path, line: launchSource.utf8.prefix(span[0]).filter { $0 == 10 }.count,
                        end: launchSource.utf8.prefix(span[1]).filter { $0 == 10 }.count)
                }
                target = targets.first(where: { $0.line <= line && line <= $0.end })?.path
                guard target != nil || chooseProfile else { message = targets.isEmpty ? "No file blocks to debug. Add a supported file block and save this document." : "Choose the file block to debug."; return }
            }
            launchProgram = target
            var query: [String: Any] = ["uri": nativeDebugURI(owner), literate ? "expected_document" : "expected_source": launchSource]
            if let target { query["program"] = target }
            let result: NativeDebugCatalog = try await engine.request("POST", "/api/launch-profiles", body: query)
            guard epoch==current else{return}
                guard preparationNamespace==sourceNamespace,!buffer.resourceDeleted,exactUTF8(buffer.file.path,owner),exactUTF8(buffer.text,launchSource),!buffer.dirty,!buffer.saving else{message="Source identity or text changed during launch preparation. Choose Debug on the current saved source.";return}
            guard compatibleDebugger(result) else{return}
            catalogNotices=result.notices ?? [];catalog = result; profiles = result.profiles.filter(\.can_debug)
            if !chooseProfile,let automatic = profiles.first(where: { exactUTF8($0.program,target ?? result.viewed ?? owner) }) ?? profiles.first(where:{$0.default_profile==true && $0.attach != nil}) ?? (profiles.count == 1 ? profiles.first : nil) { launch(automatic); return }
            message = profiles.isEmpty ? "No debug launch targets. Configure a debug profile in this project and try again." : "Choose a debug target."
        } catch { if epoch == current { message = "\(error.localizedDescription) Save or refresh this file and try debugging again." } }
    }
    func launch(_ profile: NativeDebugProfile) {
        guard let catalog else{message="Resolve a current scoped launch profile before Debug. No Start command was sent.";return}
        guard compatibleDebugger(catalog) else{return}
        guard let buffer = launchBuffer, profiles.contains(where: { exactUTF8($0.id,profile.id) && exactUTF8($0.program,profile.program) }),
              preparationNamespace==sourceNamespace,!buffer.resourceDeleted,exactUTF8(buffer.file.path,owner),exactUTF8(buffer.text,launchSource), !buffer.dirty, !buffer.saving, !active else {
            message = "The source or launch target changed. Save and choose Debug again."; return
        }
        attachReading=profiles.first(where:{exactUTF8($0.id,profile.id)})?.attachNotice
        let launch=beginLaunchRequest();message = ""; profiles = []; values = [:]; variables = []; frames = []; pausedLine = nil; offer = nil
        openTransport()
        var request: [String: Any] = ["op": "start", "request_id":launch,"doc": nativeDebugURI(owner), "breakpoints": breakpoints[owner] ?? [],
            "profile": ["id": profile.id, "fingerprint": catalog.fingerprint]]
        if let launchProgram,attachReading==nil { request["program"] = launchProgram }
        appendLaunchBreakpoints(launch,to:&request)
        send(request)
    }
    private func appendLaunchBreakpoints(_ launch:String,to request:inout [String:Any]) {
        let primaryKey=Data(owner.utf8)
        if intentSources[primaryKey]==nil {if breakpointIntents(owner).isEmpty{intentSources[primaryKey]=launchSource}else{breakpointNotice="Breakpoint intentions without a captured source remain unbound. Place them explicitly on current text before launch binding."}}
        if let source=intentSources[primaryKey],!exactUTF8(source,launchSource){breakpointNotice="Earlier source intentions are retained. This launch omitted stale positions; place intentions explicitly on current source."}
        request["source_hash"]=nativeSourceHash(launchSource)
        let primaryOwned=intentOwners[primaryKey]==launchBuffer?.id
        request["breakpoints"]=primaryOwned && intentSources[primaryKey].map{exactUTF8($0,launchSource)}==true ? launchBreakpointIntents(owner):[]
        if (primaryOwned || breakpointIntents(owner).isEmpty) && intentSources[primaryKey].map({exactUTF8($0,launchSource)})==true{breakpointRequests[primaryKey]=NativeDebugBreakpointRequest(id:launch,source:launchSource)}
        if recordedLaunch != nil { return }
        var sources:[String:[[String:Any]]]=[:],hashes:[String:String]=[:]
        for (path,_) in breakpoints where !exactUTF8(path,owner) {let key=Data(path.utf8);if let source=intentSources[key],let buffer=breakpointBuffers[key]?.buffer,buffer.file.generated_by != "conversation",!buffer.dirty,!buffer.saving,!buffer.resourceDeleted,exactUTF8(buffer.file.path,path),intentOwners[key]==buffer.id,exactUTF8(source,buffer.text){sources[nativeDebugURI(path)]=breakpointIntents(path);hashes[nativeDebugURI(path)]=nativeSourceHash(source);breakpointRequests[key]=NativeDebugBreakpointRequest(id:launch,source:source)}}
        request["sources"]=sources;request["source_hashes"]=hashes
    }
    // The initial deadline waits for the engine to read the saved choice. One
    // matching preparation receipt enters a longer wait for actual Started.
    // A response for another ticket, edited choice or retired launch cannot
    // extend either wait or create a session.
    func launchSourceTest(_ buffer:DocumentBuffer,body:[String:Any],plan:SourceTestDebugPlan,facts:SourceTestScan,current:@escaping ()->Bool,admitted:@escaping (String)->Void,rejected:@escaping (String)->Void,outcome:@escaping (String,String,Int?)->Void={_,_,_ in}) {
        guard !active,current(),plan.available,!buffer.dirty,!buffer.saving,!buffer.resourceDeleted else{rejected("Test Debug context changed or another debugger is active. No Start was sent.");return}
        close();owner=buffer.file.path;launchBuffer=buffer;launchSource=buffer.text;preparationNamespace=sourceNamespace
        breakpointBuffers[Data(owner.utf8)]=NativeDebugBufferRef(buffer)
        let launch=beginLaunchRequest();pendingSourceTest=(plan,facts,current,admitted,rejected,outcome);catalogNotices=plan.notices
        values=[:];variables=[];frames=[];pausedLine=nil;profiles=[];offer=nil
        openTransport()
        var request:[String:Any]=["op":"start_test","request_id":launch,"doc":plan.source_uri,"source_test":body]
        appendLaunchBreakpoints(launch,to:&request);waitForSourceTest(launch,milliseconds:30_000,preparing:false);send(request)
    }
    private func clearSourceTestDeadline() {
        sourceTestDeadline?.cancel()
        sourceTestDeadline=nil
        sourceTestPreparing=false
    }
    private func waitForSourceTest(_ request:String,milliseconds:Int,preparing:Bool) {
        sourceTestDeadline?.cancel()
        sourceTestPreparing=preparing
        sourceTestDeadline=Task{[weak self] in
            do { try await Task.sleep(for:.milliseconds(milliseconds)) } catch { return }
            self?.expireSourceTestPreparation(request,preparing:preparing)
        }
    }
    func expireSourceTestPreparation(_ request:String,preparing:Bool=false) {
        guard pendingLaunch?.id==request,sourceTestPreparing==preparing,let pending=pendingSourceTest else{return}
        pendingSourceTest=nil;retireLaunches();status="failed"
        message=(preparing ? "Test Debug session was not confirmed within the preparation wait." :
            "Test Debug admission was not confirmed within 30 seconds.") + " No request was retried. A late owned session will be stopped."
        pending.rejected(message)
    }
    private func admitsSourceTest(_ event:[String:Any],_ plan:SourceTestDebugPlan,_ facts:SourceTestScan)->Bool {
        guard let raw=event["source_test"] as? [String:Any],JSONSerialization.isValidJSONObject(raw),
              let bytes=try? JSONSerialization.data(withJSONObject:raw),
              let binding=try? JSONDecoder().decode(SourceTestDebugBinding.self,from:bytes) else{return false}
        return plan.admits(binding,facts)
    }
    func retireSourceTestPreparation() {
        guard pendingSourceTest != nil else{return}
        pendingSourceTest=nil;retireLaunches();status="idle";message="Test Debug preparation closed. No replacement run was requested."
    }
    private func openTransport() {
        if emitForTest != nil{return}
        var url = URLComponents(url: engine.origin, resolvingAgainstBaseURL: false)!
        url.scheme = engine.origin.scheme == "https" ? "wss" : "ws"; url.path = "/api/ws"; url.queryItems = [URLQueryItem(name: "doc", value: "workspace")]
        let socket = engine.session.webSocketTask(with: url.url!); self.socket = socket; let current = epoch; socket.resume()
        receiver = Task { [weak self] in
            do {
                while !Task.isCancelled {
                    let frame = try await socket.receive()
                    guard let self else{return}
                    if case .data(let bytes)=frame,bytes.first==3 {
                        let json=Data(bytes.dropFirst());try validateSourceTestJSON(json)
                        if let event=try? JSONSerialization.jsonObject(with:json) as? [String:Any]{if self.epoch==current {self.receive(event)}else{_ = self.stopRetiredLaunch(event);return}}
                    }else if self.epoch != current{return}
                }
            } catch { if let self, self.epoch == current, !Task.isCancelled { self.fail("Debugger disconnected: \(error.localizedDescription). Stop or reconnect explicitly; commands were not retried.") } }
        }
    }
    private func send(_ request: [String: Any]) {
        observeRequestForTest?(request)
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
    private func retireTestReadings(_ notice:String){let readings=testOutcomes;testOutcomes=[:];for (session,callback) in readings{callback(session,notice,nil)}}
    private func fail(_ detail: String) { retireRecordedCapture();retireTestReadings("debugger reading failed / process outcome unknown");let test=pendingSourceTest;pendingSourceTest=nil;test?.rejected(detail);retireLaunches();resetThreads();status = "failed"; message = detail; sessionID = nil; pausedLine = nil; variables = []; frames = []; values = [:] }
    func close() {
        retireRecordedCapture()
        retireTestReadings("debugger reading closed / process outcome unknown")
        pendingSourceTest=nil
        attachReading=nil
        retireLaunches()
        epoch = UUID(); receiver?.cancel(); sender?.cancel(); socket?.cancel(with: .normalClosure, reason: nil); socket = nil
        resetThreads();stopPending = false; sessionID = nil; status = "idle"; pausedLine = nil; frames = []; variables = []; values = [:]; profiles = []
    }
    // Session-bearing events from an old run cannot paint a new launch.
    func receive(_ event: [String: Any]) {
        observeEventForTest?(event)
        guard let kind = event["event"] as? String else { return }
        if kind=="test_preparing" {
            guard let launch=pendingLaunch,let test=pendingSourceTest,!sourceTestPreparing,
                  ["starting","stopping"].contains(status),test.current(),
                  exactUTF8(event["request_id"] as? String,launch.id),exactUTF8(event["doc"] as? String,launch.doc),
                  event["session"]==nil,let wait=nativeDebugID(event["wait_ms"]),(30_001...180_000).contains(wait),
                  admitsSourceTest(event,test.plan,test.facts) else{return}
            waitForSourceTest(launch.id,milliseconds:wait+5_000,preparing:true)
            if status=="starting" {message="Preparing the saved test. Waiting for the debugger session…"}
            return
        }
        if kind=="started" {
            if stopRetiredLaunch(event){return}
            guard let pending=pendingLaunch,event["request_id"] as? String==pending.id,event["doc"] as? String==pending.doc,let session=event["session"] as? String,!session.isEmpty,["starting","stopping"].contains(status) else{return}
        }else if kind=="build" || kind=="failed" && (event["session"] is NSNull || event["about"] as? String=="start") {
            guard let held=pendingLaunch ?? activeLaunch,event["request_id"] as? String==held.id else{return}
            if kind=="build" || event["session"] is NSNull {guard event["doc"] as? String==held.doc else{return}}
            else {guard event["session"] as? String==sessionID else{return}}
        }else {guard let id=event["session"] as? String,id==sessionID else{return}}
        if kind=="adapter_source" {Task{[weak self] in guard let self else{return};_ = await self.adapterSources.receive(event,binding:{self.adapterSourceBinding})};return}
        if kind=="failed",event["about"] as? String=="source" {if let request=event["request_id"] as? String{_ = adapterSources.refuse(request,event["message"] as? String ?? "Adapter source request refused.")};return}
        switch kind {
        case "started":
            if let recordedLaunch,
               !recordedLaunch.current() || !exactUTF8(event["source_hash"] as? String, recordedLaunch.receipt.source_hash) {
                retireLaunches()
                _ = stopRetiredLaunch(event)
                fail("The debugger did not confirm the recorded message revision. Its owned session was stopped.")
                return
            }
            if let pending=pendingSourceTest {
                guard pending.current(),admitsSourceTest(event,pending.plan,pending.facts),
                      exactUTF8(event["source_hash"] as? String,pending.plan.source_hash) else {
                    pendingSourceTest=nil;retireLaunches();_ = stopRetiredLaunch(event)
                    status="failed";message="Test Debug admission no longer matches the selected saved candidate. A Stop was sent for its owned session.";pending.rejected(message);return
                }
                clearSourceTestDeadline()
                message=""
            }
            launchBoundHash=(event["source_hash"] as? String)==nativeSourceHash(launchSource) ? event["source_hash"] as? String:nil
            if launchBoundHash==nil {message="The engine did not confirm the captured primary source hash. Runtime controls remain available; primary source positions, Jump and Run to caret are unbound."}
            pauseUnconfirmed=false;movementRequest=nil;activeLaunch=pendingLaunch;pendingLaunch=nil
            resetThreads();output = []
            sessionID = event["session"] as? String; capabilities = event["capabilities"] as? [String: Any] ?? [:]; status = "running"
            retainedSessionID=sessionID
            if let pending=pendingSourceTest,let session=sessionID {pendingSourceTest=nil;testOutcomes[session]=pending.outcome;pending.admitted(session)}
            receiveBreakpointReceipt(event,path:owner)
            if stopPending { stopPending = false; stop() }
        case "stopped":
            guard let bytes=try? JSONSerialization.data(withJSONObject:event),let pause=try? JSONDecoder().decode(NativeDebugThreadPause.self,from:bytes) else{adapterSources.retire("Stopped identity is unavailable. Captured adapter text is historical.");clearVariableTree();pauseIdentityKnown=false;stackNotice="Stopped response has no valid integer thread/stop identity. Earlier frames are historical; refresh through a current engine.";threadNotice=stackNotice;values=[:];evaluations=[:];frameRequest=nil;frameBusy=false;frameVariablesCurrent=false;threadRequest=nil;threadBusy=false;return}
            guard status != "stopping" else{return}
            if let latestThreadStop {guard pause.stop>=latestThreadStop,!["running","unconfirmed"].contains(status) || pause.stop>latestThreadStop else{return}}
            adapterSources.retire();pauseUnconfirmed=false;movementRequest=nil
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
                for frame in frames {if let path=frame["source"] as? String,let hash=frame["source_hash"] as? String{confirmKnownSource(path,hash:hash)}}
                if let recordedLaunch, let top = frames.first { revealRecordedFrame(top, launch: recordedLaunch) }
                else if let top = frames.first, let path = top["source"] as? String, let line = nativeDebugLine(top["source_line"]) { revealLocation(path:path,line:line) }
                else if let pausedLine { revealLocation(path:owner,line:pausedLine) }
            }
            resetVariableTree()
            for expression in watches { evaluate(expression, watch: true) }
        case "threads":receiveThreads(event)
        case "frame_variables":receiveFrameVariables(event)
        case "children":receiveChildren(event)
        case "breakpoints":receiveBreakpointReceipt(event,path:owner)
        case "source_breakpoints":
            guard let source=event["source"] as? String,let ticket=event["request_id"] as? String,let key=breakpointRequests.first(where:{entry in entry.value.id==ticket && String(data:entry.key,encoding:.utf8).map{exactUTF8(nativeDebugURI($0),source)}==true})?.key,let path=String(data:key,encoding:.utf8) else{return}
            receiveBreakpointReceipt(event,path:path)
        case "run_output":
            output = Array((output + (event["events"] as? [[String: Any]] ?? []).compactMap { item in
                guard let text = item["text"] as? String else { return nil }
                return NativeRunOutput(category: item["category"] as? String ?? "stdout", text: text, source: item["source"] as? String, line: item["line"] as? Int)
            }).suffix(256))
        case "value":
            guard status=="paused",let bytes=try? JSONSerialization.data(withJSONObject:event),let value=try? JSONDecoder().decode(NativeDebugValue.self,from:bytes),let request=evaluations[Data(value.expression.utf8)]?.request,value.request_id==request.id,value.stop==request.stop,value.stop==latestThreadStop,value.thread==request.thread,value.thread==selectedThread,value.frame==request.frame,value.frame==selectedFrame else{return}
            evaluations.removeValue(forKey:Data(value.expression.utf8));values[value.expression]=value.value
            if let reference=value.reference,reference>0,reference<=Int64(Int32.max) {let node=variableNode(["name":value.expression,"value":value.value,"variables_reference":reference],depth:0);evaluationRoots[Data(value.expression.utf8)]=node.id;variableNodes[node.id]=node;variableRoots.append(node.id)}
        case "finished", "ended":
            if let session=sessionID{testOutcomes.removeValue(forKey:session)?(session,kind=="finished" ? "process finished":"debugger ended / process verdict uncollected",nativeDebugID(event["exit_code"]))}
            releaseOwnedRecordedCapture()
            retireLaunches();resetThreads();status = "finished"; sessionID = nil; pausedLine = nil; variables = []; frames = []; values = [:]
        case "failed":
            if let pending=pendingSourceTest,event["request_id"] as? String==pendingLaunch?.id {pendingSourceTest=nil;pending.rejected(event["message"] as? String ?? "Test Debug admission refused.")}
            if ["breakpoints","source_breakpoints"].contains(event["about"] as? String ?? "") {guard let id=event["request_id"] as? String,let key=breakpointRequests.first(where:{$0.value.id==id})?.key else{return};breakpointRequests.removeValue(forKey:key);boundSources=[:];breakpointNotice="Breakpoint binding refused: "+(event["message"] as? String ?? "Provider could not bind current source.");return}
            if ["step","jump","run_to"].contains(event["about"] as? String ?? "") {guard status=="running",let id=event["request_id"] as? String,let request=movementRequest,id==request.id else{return};movementRequest=nil
                let retired=event["scope_retired"].map{raw->Bool in guard let number=raw as? NSNumber,CFGetTypeID(number)==CFBooleanGetTypeID() else{return true};return number.boolValue} ?? false
                if retired {retireThreadChoices();pauseIdentityKnown=false;pauseUnconfirmed=true;status="unconfirmed";pausedLine=nil;variables=[];values=[:];stackNotice="Pause not confirmed after movement. Earlier stack frames are historical; positions, evaluation and further movement are unavailable until an actual newer stop is reported.";message=(event["message"] as? String ?? "Next stop could not be observed.")+" The current session and Stop remain available."}
                else {stackNotice=request.stackNotice;status="paused";message=(event["message"] as? String ?? "Runtime movement refused.")+" The previous paused observation is retained; refresh it before another movement."};return}
            if event["about"] as? String=="children",let id=event["request_id"] as? String {guard let request=expansions.removeValue(forKey:id),scopeCurrent(request.request),var node=variableNodes[request.node] else{return};node.pending=false;node.retryable=true;node.notice="Provider expansion refused: "+(event["message"] as? String ?? "Unknown error");variableNodes[node.id]=node;return}
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
    private func scopeCurrent(_ request:NativeDebugEvaluation)->Bool {status=="paused" && request.stop==latestThreadStop && request.thread==selectedThread && request.frame==selectedFrame && canInspectFrame}
    private func clearVariableTree(){variableNodes=[:];variableRoots=[];evaluationRoots=[:];expansions=[:]}
    private func retireExpressionRoot(_ expression:String){guard let id=evaluationRoots.removeValue(forKey:Data(expression.utf8)) else{return};func remove(_ id:UUID){if let node=variableNodes.removeValue(forKey:id){for child in node.children{remove(child)}};expansions=expansions.filter{$0.value.node != id}};remove(id);variableRoots.removeAll{$0==id}}
    private func variableNode(_ value:[String:Any],depth:Int)->NativeDebugVariableRow {NativeDebugVariableRow(id:UUID(),depth:depth,name:value["name"] as? String ?? "",value:value["value"] as? String ?? "",reference:nativeDebugID(value["variables_reference"]).map(Int64.init) ?? 0)}
    private func resetVariableTree(){clearVariableTree();for value in variables.prefix(10000){let node=variableNode(value,depth:0);variableNodes[node.id]=node;variableRoots.append(node.id)}}
    func toggleVariable(_ id:UUID){guard canInspectFrame,var node=variableNodes[id],node.reference>0 else{return};if node.expanded {node.expanded=false;variableNodes[id]=node;return};node.expanded=true;variableNodes[id]=node;if !node.children.isEmpty || node.notice=="No child variables reported."{return};if node.depth>=32 || variableNodes.count>=10000 {node.notice="Expansion display limit reached (32 levels / 10,000 variables).";variableNodes[id]=node;return};readChildren(id)}
    func readChildren(_ id:UUID){guard canInspectFrame,let sessionID,let stop=latestThreadStop,let thread=selectedThread,let frame=selectedFrame,var node=variableNodes[id],node.reference>0,!node.pending,node.depth<32,variableNodes.count<10000 else{return};let ticket=UUID().uuidString;node.pending=true;node.retryable=false;node.notice="";variableNodes[id]=node;expansions[ticket]=NativeDebugExpansion(request:NativeDebugEvaluation(id:ticket,stop:stop,thread:thread,frame:frame),node:id,reference:node.reference);send(["op":"children","session":sessionID,"stop":stop,"thread":thread,"frame":frame,"reference":node.reference,"request_id":ticket])}
    private func receiveChildren(_ event:[String:Any]) {guard let ticket=event["request_id"] as? String,let request=expansions[ticket],scopeCurrent(request.request),var node=variableNodes[request.node] else{return}
        guard let bytes=try? JSONSerialization.data(withJSONObject:event),let scope=try? JSONDecoder().decode(NativeDebugFrameVariables.self,from:bytes),scope.stop==request.request.stop,scope.thread==request.request.thread,scope.frame==request.request.frame,nativeDebugID(event["reference"]).map(Int64.init)==request.reference else{expansions.removeValue(forKey:ticket);node.pending=false;node.retryable=true;node.notice="Child response did not confirm the current stopped frame/reference scope. Earlier values and controls are retained.";variableNodes[node.id]=node;return}
        expansions.removeValue(forKey:ticket);node.pending=false;node.retryable=false
        guard let children=event["variables"] as? [[String:Any]],children.count<=10000-variableNodes.count,children.allSatisfy({$0["name"] is String && $0["value"] is String && ($0["variables_reference"]==nil || nativeDebugID($0["variables_reference"]).map{$0>=0}==true)}) else{node.notice="Expansion is malformed or exceeds the 10,000-variable display limit.";variableNodes[node.id]=node;return}
        for value in children {let child=variableNode(value,depth:node.depth+1);variableNodes[child.id]=child;node.children.append(child.id)};node.notice=children.isEmpty ? "No child variables reported.":"";variableNodes[node.id]=node
    }
    private func resetThreads() {adapterSources.retire();
        pauseUnconfirmed=false;clearVariableTree();threads=[];threadStop=nil;selectedThread=nil;threadRequest=nil;threadBusy=false;refreshingPause=false;threadNotice="";pauseIdentityKnown=false;latestThreadStop=nil;threadSnapshot=UUID();frameSnapshot=UUID();evaluations=[:];evaluationNotice="";frameRequest=nil;frameBusy=false;frameVariablesCurrent=false;frameNotice="";stackNotice="";threadPage=0
    }
    private func retireThreadChoices() {adapterSources.retire();
        clearVariableTree();threads=[];threadStop=nil;threadRequest=nil;threadBusy=false;refreshingPause=false;threadSnapshot=UUID();frameSnapshot=UUID();evaluations=[:];evaluationNotice="";frameRequest=nil;frameBusy=false;frameVariablesCurrent=false;frameNotice="";stackNotice="";threadPage=0
    }
    private func admitThreadPause(_ pause:NativeDebugThreadPause?) {
        guard let pause else{resetThreads();threadNotice="This stopped response has no thread observation. Update the engine before selecting threads.";return}
        let changed=latestThreadStop != pause.stop || selectedThread != pause.thread
        latestThreadStop=pause.stop;selectedThread=pause.thread
        if changed {
            adapterSources.retire();clearVariableTree();threads=[];threadStop=nil;threadSnapshot=UUID();threadPage=0;threadNotice="Read reported threads for this stopped state."
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
        variables=value.variables.map{item in ["name":item.name,"value":item.value,"type_name":item.type_name as Any? ?? NSNull(),"variables_reference":item.variables_reference ?? 0]};resetVariableTree()
        if selectedFrame==nativeDebugID(frames.first?["id"]) {topVariables=variables}
        frameNotice=variables.isEmpty ? "No variables were reported for this frame.":"Variables reported for the selected stopped frame."
    }
    func readFrameLocals() {
        guard canReadFrameLocals,let sessionID,let stop=latestThreadStop,let thread=selectedThread,let frame=selectedFrame else{return}
        clearVariableTree();let id=UUID().uuidString;frameRequest=NativeDebugEvaluation(id:id,stop:stop,thread:thread,frame:frame);frameBusy=true;frameVariablesCurrent=false;frameNotice="Reading selected-frame variables…"
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
        adapterSources.retire();let id=UUID().uuidString;threadRequest=NativeDebugThreadRequest(id:id,stop:stop,thread:thread);threadBusy=true;threadNotice="Requesting the selected thread’s stopped stack…"
        send(["op":"select_thread","session":sessionID,"thread":thread,"stop":stop,"request_id":id])
    }
    private func breakpointIntents(_ path:String)->[[String:Any]] {(breakpoints[path] ?? []).map{mark in var clean:[String:Any]=["line":mark["line"] ?? 0];if let condition=mark["condition"] as? String{clean["condition"]=condition};return clean}}
    private func launchBreakpointIntents(_ path: String) -> [[String: Any]] {
        let intentions = breakpointIntents(path)
        guard let recordedLaunch, exactUTF8(path, owner) else { return intentions }
        return intentions.filter { nativeDebugLine($0["line"]).map(recordedLaunch.fence.contains) == true }
    }
    func sourceChanged(_ buffer:DocumentBuffer) {let key=Data(buffer.file.path.utf8);breakpointBuffers[key]=NativeDebugBufferRef(buffer);guard !(breakpoints[buffer.file.path] ?? []).isEmpty || breakpointRequests[key] != nil || boundSources[key] != nil else{return};if buffer.dirty || intentSources[key].map({!exactUTF8($0,buffer.text)})==true {breakpointRequests.removeValue(forKey:key);boundSources.removeValue(forKey:key);breakpointNotice="Breakpoint intentions belong to earlier text. Save and explicitly update them before binding new positions."}}
    func breakpointMarkers(_ buffer: DocumentBuffer, recorded: Bool = false) -> [[String: Any]] {
        let path = buffer.file.path, key = Data(path.utf8)
        let confirmed = identityMatches(buffer, path: path, confirmed: true)
        guard (confirmed || recorded && breakpointBuffers[key]?.buffer === buffer),
              intentOwners[key] == buffer.id, !buffer.dirty,
              let source = intentSources[key], exactUTF8(source, buffer.text) else { return [] }
        let bound = confirmed && sessionID != nil && active && boundSources[key].map { exactUTF8($0, buffer.text) } == true
        let fence = exactUTF8(path, owner) ? recordedLaunch?.fence : nil
        return (breakpoints[path] ?? []).map { mark in
            var item = mark
            let applicable = fence.map { selected in nativeDebugLine(mark["line"]).map(selected.contains) == true } ?? true
            if bound && applicable {
                item["line"] = item["display_line"] ?? item["line"]
            } else { item["state"] = "pending" }
            return item
        }
    }
    private func receiveBreakpointReceipt(_ event:[String:Any],path:String) {let key=Data(path.utf8);guard let request=breakpointRequests[key],event["request_id"] as? String==request.id else{return};guard event["source_hash"] as? String==nativeSourceHash(request.source),intentSources[key].map({exactUTF8($0,request.source)})==true else{breakpointRequests.removeValue(forKey:key);boundSources.removeValue(forKey:key);breakpointNotice="Breakpoint receipt did not confirm the captured intention source. Positions remain unbound.";return};if let buffer=breakpointBuffers[key]?.buffer {guard identityMatches(buffer,path:path,confirmed:false),intentOwners[key]==buffer.id,!buffer.dirty,!buffer.saving,exactUTF8(buffer.file.path,path),exactUTF8(buffer.text,request.source) else{breakpointRequests.removeValue(forKey:key);boundSources.removeValue(forKey:key);breakpointNotice="Editor ownership/text changed before the breakpoint receipt. Positions remain unbound.";return}}
        let requestedMarks=launchBreakpointIntents(path),lines=request.source.components(separatedBy:"\n").count
        let fence = exactUTF8(path, owner) ? recordedLaunch?.fence : nil
        guard let statuses=event["breakpoints"] as? [[String:Any]],statuses.count==requestedMarks.count,statuses.count<=4096 else{breakpointNotice="Incomplete breakpoint receipt refused; intentions retained and positions unbound.";return}
        let requested=Set(requestedMarks.compactMap{nativeDebugLine($0["line"])}),reported=statuses.compactMap{nativeDebugLine($0["line"])}
        guard requested.count==requestedMarks.count,reported.count==statuses.count,Set(reported)==requested,Set(reported).count==reported.count,statuses.allSatisfy({status in
            guard let state=status["state"] as? String,["pending","bound","refused"].contains(state) else{return false}
            if let message=status["message"],!(message is NSNull),!(message is String){return false}
            guard let moved=status["moved_to"],!(moved is NSNull) else{return true};return nativeDebugLine(moved).map{$0<lines && (fence?.contains($0) ?? true)}==true
        }) else{breakpointNotice="Malformed breakpoint receipt refused; intentions retained and positions unbound.";return};var held=breakpoints[path] ?? []
        for index in held.indices {if let status=statuses.first(where:{$0["line"] as? Int==held[index]["line"] as? Int}) {held[index]["state"]=status["state"];held[index]["display_line"]=nativeDebugLine(status["moved_to"]) ?? nativeDebugLine(held[index]["line"]);held[index]["message"]=status["message"]}}
        breakpoints[path]=held;if let buffer=breakpointBuffers[key]?.buffer {confirmKnownSource(path,hash:nativeSourceHash(request.source));guard identityMatches(buffer,path:path,confirmed:true) else{return}};boundSources[key]=request.source;if !ambiguousSources.contains(key){sourceHashes[key]=nativeSourceHash(request.source)};breakpointRequests.removeValue(forKey:key);breakpointNotice="Breakpoint statuses confirmed for captured source."
    }
    func toggle(_ buffer:DocumentBuffer,line:Int,condition:String?=nil) {
        guard line>=0,line<buffer.text.components(separatedBy:"\n").count else{breakpointNotice="Breakpoint position is outside current source.";return}
        let key=Data(buffer.file.path.utf8)
        if intentOwners[key] != buffer.id,!breakpointIntents(buffer.file.path).isEmpty {retainedBreakpointIntents[key]=Array(((retainedBreakpointIntents[key] ?? [])+breakpointIntents(buffer.file.path).map{mark in var item=mark;item["source_hash"]=intentSources[key].map(nativeSourceHash) ?? "unknown";return item}).suffix(4096));breakpoints[buffer.file.path]=[];intentSources.removeValue(forKey:key)}
        intentOwners[key]=buffer.id;breakpointBuffers[key]=NativeDebugBufferRef(buffer)
        if sourceIdentities[key]==nil,sourceNamespace==launchNamespace,let proof=exactUTF8(buffer.file.path,owner) ? launchBoundHash:sourceHashes[key] {_ = admitSourceReading(buffer,path:buffer.file.path,proof:proof)}
        toggle(buffer.file.path,line:line,condition:condition,source:buffer.text,writable:!buffer.dirty && !buffer.saving && !buffer.resourceDeleted)
    }
    func toggle(_ path:String,line:Int,condition:String?=nil,source:String?=nil,writable:Bool=true) {guard line>=0,line<=Int(Int32.max) else{breakpointNotice="Invalid breakpoint coordinate refused; current intentions are retained.";return};let key=Data(path.utf8);let text=source
        if text==nil {var unknown:[String:Any]=["line":line,"source_hash":"unknown"];if let condition{unknown["condition"]=condition};retainedBreakpointIntents[key]=Array(((retainedBreakpointIntents[key] ?? [])+[unknown]).suffix(4096));breakpointNotice="Unbound path-only intention retained. Place it explicitly on current source; no coordinates were sent.";return}
        if let text,line>=text.components(separatedBy:"\n").count {breakpointNotice="Breakpoint position is outside the captured source. Current intentions are retained.";return}
        if let text,let buffer=breakpointBuffers[key]?.buffer,(!exactUTF8(buffer.file.path,path) || !exactUTF8(buffer.text,text)){breakpointNotice="Stale breakpoint source/ownership refused. Current intentions and editor text are retained.";return}
        if let text,!breakpointIntents(path).isEmpty,intentSources[key].map({!exactUTF8($0,text)}) ?? true {let previous=intentSources[key];retainedBreakpointIntents[key]=Array(((retainedBreakpointIntents[key] ?? [])+breakpointIntents(path).map{mark in var retained=mark;retained["source_hash"]=previous.map(nativeSourceHash) ?? "unknown";return retained}).suffix(4096));breakpoints[path]=[];breakpointNotice="Earlier source breakpoint intentions remain readable; place new intentions explicitly on current text."};if let text{intentSources[key]=text}
        let bound=boundSources[key].flatMap{saved in text.map{exactUTF8(saved,$0)}} ?? false
        let index=(breakpoints[path] ?? []).firstIndex{$0["line"] as? Int==line} ?? (bound ? (breakpoints[path] ?? []).firstIndex{$0["display_line"] as? Int==line}:nil)
        var held=breakpoints[path] ?? [];if let index {if let condition{held[index]["condition"]=condition}else{held.remove(at:index)}}else{var mark:[String:Any]=["line":line];if let condition{mark["condition"]=condition};held.append(mark)}
        if let recordedLaunch, exactUTF8(path, owner), active, !recordedLaunch.fence.contains(line) {
            breakpoints[path] = held
            breakpointNotice = "Breakpoint retained for another fence in this message. It will bind only if you launch that fence."
            return
        }
        breakpoints[path]=held;boundSources.removeValue(forKey:key);breakpointRequests.removeValue(forKey:key)
        guard writable,let text,let sessionID,["running","paused"].contains(status) else{breakpointNotice="Breakpoint intention retained; binding requires saved source and a running compatible session.";return}
        if let buffer=breakpointBuffers[key]?.buffer {guard identityMatches(buffer,path:path,confirmed:false),intentOwners[key]==buffer.id,!buffer.dirty,!buffer.saving,exactUTF8(buffer.file.path,path),exactUTF8(buffer.text,text) else{breakpointNotice="Breakpoint intention retained, but current editor ownership/text is not the captured source. No coordinates were sent.";return}}
        guard let original=breakpointBuffers[key]?.buffer,identityMatches(original,path:path,confirmed:false),intentOwners[key]==original.id else{breakpointNotice="This source is not the captured original Buffer identity. Intention retained without runtime effects.";return}
        let captured=exactUTF8(path,owner) ? launchBoundHash:(!ambiguousSources.contains(key) ? sourceHashes[key]:nil)
        guard captured==nativeSourceHash(text) else{breakpointNotice="Breakpoint intention retained for current saved text. This session did not capture those bytes; launch current source before binding positions.";return}
        let id=UUID().uuidString;breakpointRequests[key]=NativeDebugBreakpointRequest(id:id,source:text)
        var request:[String:Any]=["op":exactUTF8(path,owner) ? "breakpoints":"source_breakpoints","session":sessionID,"breakpoints":launchBreakpointIntents(path),"source_hash":nativeSourceHash(text),"request_id":id];if !exactUTF8(path,owner){request["source"]=nativeDebugURI(path)};send(request)
    }
    func step(_ how: String) {
        guard canResume,let sessionID,stackNotice.isEmpty || how=="continue" else { return }
        guard let stop=latestThreadStop,let thread=selectedThread else{return}
        let ticket=UUID().uuidString;movementRequest=(ticket,stackNotice)
        var request: [String: Any] = ["op": "step", "session": sessionID, "how": how,"stop":stop,"thread":thread,"request_id":ticket]
        if stackNotice.isEmpty,let selectedFrame { request["frame"] = selectedFrame }
        retireThreadChoices();status = "running"; pausedLine = nil; variables = []; values = [:]; send(request)
    }
    func canMoveToSource(_ buffer:DocumentBuffer,how:String)->Bool {canInspectFrame && sourceMatches(buffer) && exactUTF8(buffer.file.path,owner) && (capabilities[how=="jump" ? "goto_targets":"run_to"] as? Bool==true)}
    func moveToSource(_ buffer:DocumentBuffer,line:Int,how:String) {guard ["jump","run_to"].contains(how),line>=0,line<buffer.text.components(separatedBy:"\n").count,recordedLaunch?.fence.contains(line) ?? true,canMoveToSource(buffer,how:how),let sessionID,let stop=latestThreadStop,let thread=selectedThread else{message="Runtime target requires the unchanged captured primary source and a supported paused adapter.";return};let ticket=UUID().uuidString;movementRequest=(ticket,stackNotice)
        let request:[String:Any]=["op":how,"session":sessionID,"line":line,"stop":stop,"thread":thread,"request_id":ticket,"source_hash":nativeSourceHash(buffer.text)]
        retireThreadChoices();status="running";pausedLine=nil;variables=[];values=[:];send(request)
    }
    func stop() {
        movementRequest=nil
        guard let sessionID else {
            if status == "starting" || status == "stopping" { stopPending = true; status = "stopping"; message = "Stopping as soon as the adapter acknowledges launch…" }
            return
        }
        retireThreadChoices();status = "stopping"; pausedLine = nil; variables = []; values = [:]; send(["op": "stop", "session": sessionID])
    }
    func isWatched(_ expression:String)->Bool {watches.contains{exactUTF8($0,expression)}}
    func removeWatch(_ expression:String) {retireExpressionRoot(expression);watches.removeAll{exactUTF8($0,expression)};values.removeValue(forKey:expression);evaluations.removeValue(forKey:Data(expression.utf8))}
    func evaluate(_ expression: String, watch: Bool = false) {
        guard canEvaluate,let sessionID,let stop=latestThreadStop,let thread=selectedThread,!expression.isEmpty else{return}
        if watch && !isWatched(expression) {watches.append(expression)}
        retireExpressionRoot(expression);values.removeValue(forKey:expression);evaluationNotice="";let id=UUID().uuidString;evaluations[Data(expression.utf8)]=NativeDebugExpressionRequest(expression:expression,request:NativeDebugEvaluation(id:id,stop:stop,thread:thread,frame:selectedFrame))
        var request:[String:Any]=["op":"eval","session":sessionID,"expression":expression,"context":watch ? "watch":"repl","stop":stop,"thread":thread,"request_id":id]
        if let selectedFrame {request["frame"]=selectedFrame};send(request)
    }
    func select(_ frame:[String:Any],snapshot:UUID?=nil) {
        guard status=="paused",snapshot==nil || snapshot==frameSnapshot,let id=nativeDebugID(frame["id"]),frames.contains(where:{nativeDebugID($0["id"])==id}),threadRequest?.thread==nil,!refreshingPause,stackNotice.isEmpty else{return}
        adapterSources.retire();clearVariableTree();selectedFrame=id;evaluations=[:];evaluationNotice="";frameRequest=nil;frameBusy=false;frameVariablesCurrent=false
        variables = selectedFrame == nativeDebugID(frames.first?["id"]) ? topVariables : []; values = [:]
        if let path=frame["source"] as? String,let hash=frame["source_hash"] as? String{confirmKnownSource(path,hash:hash)}
        else if frame["in_document"] as? Bool==true,let hash=launchBoundHash{confirmKnownSource(owner,hash:hash)}
        readFrameLocals()
        for expression in watches { evaluate(expression, watch: true) }
        if let recordedLaunch { revealRecordedFrame(frame, launch: recordedLaunch) }
        else if let path = frame["source"] as? String, let line = nativeDebugLine(frame["source_line"]) { revealLocation(path:path,line:line) }
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
