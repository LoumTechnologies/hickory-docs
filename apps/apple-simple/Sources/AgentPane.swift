import SwiftUI
import Observation

@MainActor @Observable final class AgentSession {
    var turns: [AgentTurn] = []
    var totals: AgentTotals?
    var activity: [String: NativeAgentActivity] = [:]
    @ObservationIgnored private var earlyEvents: [NativeAgentEvent] = []
    var stopPending = false
    var canStop: Bool { turns.contains { $0.status == "running" } }

    func receive(_ message: NativeAgentEvent) {
        guard message.exec_id == "agent" else { return }
        guard turns.contains(where: { $0.id == message.run_id && $0.status == "running" }) else {
            if sending && earlyEvents.count < 128 { earlyEvents.append(message) }
            return
        }
        var value = activity[message.run_id] ?? NativeAgentActivity()
        switch message.event.kind {
        case "thinking": value.phase = "Waiting for the model…"
        case "token":
            guard value.phase != "Receiving model output…" else { return }
            value.phase = "Receiving model output…"
        case "response_complete": value.answer = message.event.text
        case "turn_usage": value.tokens = message.event.total_usage?.tokens ?? 0
        case "tool_started":
            guard let name = message.event.name else { return }
            value.phase = name
            value.tools.append(.init(name: name))
        case "tool_finished":
            if let index = value.tools.lastIndex(where: { $0.name == message.event.name && $0.ok == nil }) {
                value.tools[index].ok = message.event.ok
                value.tools[index].result = message.event.text.map { String($0.prefix(4000)) }
            }
            value.phase = "Preparing the next step…"
        default: return
        }
        activity[message.run_id] = value
    }

    func replayEarlyEvents(for turn: String) {
        let events = earlyEvents
        earlyEvents.removeAll()
        for event in events where event.run_id == turn { receive(event) }
    }

    func observeActivity(_ engine: Engine, current: () -> Bool) async {
        var url = URLComponents(url: engine.origin, resolvingAgainstBaseURL: false)!
        url.scheme = engine.origin.scheme == "https" ? "wss" : "ws"
        url.path = "/api/ws"
        url.queryItems = [URLQueryItem(name: "doc", value: "workspace")]
        while !Task.isCancelled && current() {
            let socket = engine.session.webSocketTask(with: url.url!)
            socket.resume()
            await withTaskCancellationHandler {
                while !Task.isCancelled && current(), let frame = try? await socket.receive() {
                    guard !Task.isCancelled, current() else { break }
                    if case .data(let data) = frame, data.first == 1,
                       let event = try? JSONDecoder().decode(NativeAgentEvent.self, from: Data(data.dropFirst())) {
                        receive(event)
                    }
                }
            } onCancel: { socket.cancel(with: .goingAway, reason: nil) }
            socket.cancel(with: .goingAway, reason: nil)
            do { try await Task.sleep(for: .seconds(2)) } catch { return }
        }
    }

    func stop(_ engine: Engine?) async {
        guard let engine, canStop, !stopPending else { return }
        stopPending = true
        do { try await engine.stop() }
        catch {
            self.error = "\(error.localizedDescription) Refresh history and try Stop again."
            stopPending = false
        }
    }
    var provider = "codex-subscription"
    var model = ""
    var effort = ""
    let draft = DocumentBuffer(file: FileNode(name: "Message", path: "conversation/draft.md", dir: false), text: FileText(path: "conversation/draft.md", content: "", hash: ""))
    var prompt: String { get { draft.text } set { draft.text = newValue } }
    @ObservationIgnored private var documents: [String: DocumentBuffer] = [:]
    func document(_ id: String, _ text: String) -> DocumentBuffer {
        if let held = documents[id], exactUTF8(held.text,text) { return held }
        let path = "hick-agent-message:///\(id).md"
        let buffer = DocumentBuffer(file: FileNode(name: id, path: path, dir: false, generated_by: "conversation"), text: FileText(path: path, content: text, hash: ""))
        documents[id] = buffer
        return buffer
    }
    func owns(_ buffer: DocumentBuffer, turn id: String, side: RecordedMessageSide) -> Bool {
        guard let turn = turns.first(where: { exactUTF8($0.id, id) }) else { return false }
        let text = side == .user ? turn.prompt : (turn.status == "running" ? nil : turn.answer)
        return text.map { exactUTF8($0, buffer.text) } == true && documents["\(id)-\(side.bufferSuffix)"] === buffer
    }
    var captureContext:(()throws->[String:Any])?
    var liveTurn: String?
    var sending = false
    var error = ""
    var mutation = 0
    var running: Bool { sending || turns.contains { $0.status == "running" } }

    func observe(_ engine: Engine, current: () -> Bool) async {
        while !Task.isCancelled && current() {
            do {
                let mutation = self.mutation
                let history = try await engine.turns()
                guard !Task.isCancelled, current() else { return }
                if mutation == self.mutation && !sending {
                    turns = history.turns
                    totals = history.totals
                    if !canStop { stopPending = false }
                    try await answerCapture(engine, mutation: mutation, current: current)
                }
            } catch {
                guard !Task.isCancelled, current() else { return }
                self.error = error.localizedDescription
            }
            do { try await Task.sleep(for: .seconds(1)) } catch { return }
        }
    }

    private func answerCapture(_ engine: Engine, mutation: Int, current: () -> Bool) async throws {
        guard let turn = liveTurn else { return }
        guard turns.contains(where: { $0.id == turn && $0.status == "running" }) else {
            liveTurn = nil
            return
        }
        struct CaptureState: Decodable {
            struct Edits: Decodable { let captureRequest: String? }
            let turn: String?
            let edits: Edits?
        }
        let state: CaptureState = try await engine.request("GET", "/api/docs/workspace/agent/python")
        guard !Task.isCancelled, current(), mutation == self.mutation, turn == liveTurn, state.turn == turn,
              let capture = state.edits?.captureRequest, var context = try captureContext?() else { return }
        context["live"] = true
        let _: CaptureState = try await engine.request("POST", "/api/docs/workspace/agent/python/review",
            body: ["capture_request": capture, "context": context])
    }
}
struct NativeRecordedRunTerminal: Decodable {
    var id: String
    var title: String
    var state: String
    var exit_code: Int?
}

private struct NativeRecordedRunReceipt: Decodable {
    var version: Int
    var terminal: NativeRecordedRunTerminal
    var source_hash: String
    var opening_line: Int
    var workspace: String
    var turn: String
    var side: String
}

@MainActor @Observable final class NativeRecordedFenceRun {
    private(set) var line: Int?
    private(set) var terminal: NativeRecordedRunTerminal?
    private(set) var notice = ""
    private(set) var output = ""
    @ObservationIgnored private var ticket = UUID()
    @ObservationIgnored private var ownsSource: (() -> Bool)?

    func close() {
        ticket = UUID()
        ownsSource = nil
        line = nil
        terminal = nil
        notice = ""
        output = ""
    }

    func start(_ buffer: DocumentBuffer, source: NativeRecordedMessageSource, fence: RecordedDebugFence, engine: Engine) async {
        guard line == nil, let root = source.workspace, !root.isEmpty, let endpoint = source.actionRoot,
              source.current(), buffer.readOnly, !buffer.dirty, !buffer.saving, !buffer.resourceDeleted,
              exactUTF8(buffer.inlineMarkup.source, buffer.text), buffer.inlineMarkup.debug.recorded[fence.opening] == fence else { return }
        let text = buffer.text, path = buffer.file.path, hash = nativeSourceHash(text), request = UUID()
        ticket = request
        line = fence.opening
        notice = "Starting recorded fence…"
        ownsSource = { [weak buffer] in
            guard let buffer else { return false }
            return source.current() && buffer.readOnly && !buffer.dirty && !buffer.saving && !buffer.resourceDeleted
                && exactUTF8(path, buffer.file.path) && exactUTF8(text, buffer.text)
                && exactUTF8(buffer.inlineMarkup.source, text) && buffer.inlineMarkup.debug.recorded[fence.opening] == fence
        }
        do {
            let value: NativeRecordedRunReceipt = try await engine.request("POST", endpoint + "/fence-run", body: [
                "side": source.side.rawValue, "source_hash": hash, "opening_line": fence.opening,
            ])
            guard current(request) else { return }
            guard value.version == 1, value.opening_line == fence.opening, exactUTF8(value.source_hash, hash),
                  exactUTF8(value.workspace, root), exactUTF8(value.turn, source.turn), exactUTF8(value.side, source.side.rawValue),
                  value.terminal.id.range(of: "^term-[0-9a-f]{32}$", options: .regularExpression) != nil else {
                notice = "The engine did not confirm the selected fence and terminal. Check the current conversation before running again."
                return
            }
            terminal = value.terminal
            notice = "Reading terminal output…"
        } catch {
            guard current(request) else { return }
            notice = "\(error.localizedDescription) The Run request was not retried."
        }
    }

    private func current(_ request: UUID) -> Bool {
        guard !Task.isCancelled, ticket == request else { return false }
        guard ownsSource?() == true else { close(); return false }
        return true
    }

    func refresh(_ engine: Engine) async -> Bool {
        let request = ticket
        guard let id = terminal?.id, current(request) else { return false }
        struct Terminals: Decodable { var sessions: [NativeRecordedRunTerminal] }
        do {
            async let listing: Terminals = engine.request("GET", "/api/terminals")
            async let screen: TerminalPage = engine.request("GET", "/api/terminals/" + id + "/output", query: [
                URLQueryItem(name: "view", value: "screen"), URLQueryItem(name: "offset", value: "0"),
                URLQueryItem(name: "limit", value: "16384"),
            ])
            let (terminals, page) = try await (listing, screen)
            guard current(request) else { return false }
            guard let summary = terminals.sessions.first(where: { exactUTF8($0.id, id) }), page.bytes.count <= 16384 else {
                notice = "The matching terminal output is unavailable."
                return false
            }
            terminal = summary
            output = String(decoding: page.bytes, as: UTF8.self)
            notice = page.next == nil ? "" : "Output exceeds this terminal preview."
            return summary.exit_code == nil
        } catch {
            if current(request) { notice = "\(error.localizedDescription) Terminal output could not be read." }
            return false
        }
    }
}

struct NativeRecordedConstructionEntry: Decodable {
    let kind: String
    let executable: Bool
    let bytes: Int
    let sha256: String
    let text: String?
    let text_status: String

    var valid: Bool {
        guard ["file", "symlink"].contains(kind), bytes >= 0, nativeConstructionDigest(sha256) else { return false }
        if let text {
            return text_status == "exact" && text.utf8.count == bytes && nativeSourceHash(text) == sha256
        }
        return ["over_limit", "non_utf8"].contains(text_status)
    }
}

struct NativeRecordedConstructionChange: Decodable, Identifiable {
    let path: String
    let before: NativeRecordedConstructionEntry?
    let after: NativeRecordedConstructionEntry?
    var id: Data { Data(path.utf8) }

    var valid: Bool {
        !path.isEmpty && path.utf8.count <= 1024 && (before != nil || after != nil)
            && (before?.valid ?? true) && (after?.valid ?? true)
    }
}

struct NativeRecordedConstructionReceipt: Decodable {
    let version: Int
    let read_only: Bool
    let turn: String
    let side: String
    let source_hash: String
    let opening_line: Int
    let workspace: String
    let workspace_id: String
    let entry: String
    let document: String
    let operation_id: String
    let opening_byte: Int
    let completed_fingerprint: String
    let changes: [NativeRecordedConstructionChange]

    func matches(_ source: NativeRecordedMessageSource, text: String, hash: String, line: Int) -> Bool {
        let bytes = Array(text.utf8)
        return version == 1 && read_only && exactUTF8(turn, source.turn) && exactUTF8(side, source.side.rawValue)
            && exactUTF8(source_hash, hash) && opening_line == line
            && exactUTF8(workspace, source.workspace) && exactUTF8(workspace_id, source.workspaceID)
            && [entry, document, operation_id].allSatisfy { !$0.isEmpty && $0.utf8.count <= 1024 }
            && bytes.indices.contains(opening_byte) && bytes.prefix(opening_byte).filter { $0 == 10 }.count == line
            && nativeConstructionDigest(completed_fingerprint) && (1...128).contains(changes.count)
            && Set(changes.map(\.id)).count == changes.count && changes.allSatisfy(\.valid)
    }
}

private func nativeConstructionDigest(_ text: String) -> Bool {
    text.utf8.count == 64 && text.utf8.allSatisfy { (48...57).contains($0) || (97...102).contains($0) }
}

@MainActor @Observable final class NativeRecordedConstruction {
    private(set) var line: Int?
    private(set) var receipt: NativeRecordedConstructionReceipt?
    private(set) var notice = ""
    private var ticket = UUID()

    func close() {
        ticket = UUID()
        line = nil
        receipt = nil
        notice = ""
    }

    func inspect(_ buffer: DocumentBuffer, source: NativeRecordedMessageSource, line opening: Int, engine: Engine) async {
        guard source.workspace?.isEmpty == false, source.workspaceID?.isEmpty == false,
              let endpoint = source.actionRoot else { return }
        let text = buffer.text, path = buffer.file.path, revision = buffer.sourceRevision
        func current() -> Bool {
            source.current() && buffer.readOnly && !buffer.dirty && !buffer.saving && !buffer.resourceDeleted
                && revision == buffer.sourceRevision && exactUTF8(buffer.file.path, path)
                && exactUTF8(buffer.text, text) && exactUTF8(buffer.inlineMarkup.source, text)
                && buffer.inlineMarkup.debug.recordedConstructions.contains(opening)
        }
        guard current() else { return }
        let request = UUID(), hash = nativeSourceHash(text)
        ticket = request
        line = opening
        receipt = nil
        notice = "Reading the captured construction…"
        do {
            let response: NativeRecordedConstructionReceipt = try await engine.request("POST", endpoint + "/fence-construction", body: [
                "side": source.side.rawValue, "source_hash": hash, "opening_line": opening,
            ])
            guard !Task.isCancelled, ticket == request else { return }
            guard current() else { close(); return }
            guard response.matches(source, text: text, hash: hash, line: opening) else {
                notice = "The engine could not confirm this recorded construction. Open its current patch and try again."
                return
            }
            receipt = response
            notice = ""
        } catch {
            guard !Task.isCancelled, ticket == request else { return }
            guard current() else { close(); return }
            notice = error.localizedDescription
        }
    }
}

struct NativeAgentActivity {
    struct Tool {
        let name: String
        var ok: Bool?
        var result: String?
    }
    var phase = "Starting…"
    var answer: String?
    var tools: [Tool] = []
    var tokens = 0
}

struct NativeAgentStatus: View {
    @Bindable var session: AgentSession
    let engine: Engine?
    @State private var codexModels = [String]()
    @State private var open = false
    @State private var custom = false
    private var tokens: Int {
        let live = session.turns.filter { $0.status == "running" }.reduce(0) { $0 + (session.activity[$1.id]?.tokens ?? 0) }
        return (session.totals?.tokens ?? 0) + live
    }
    private var models: [String] {
        switch session.provider {
        case "claude-subscription": ["sonnet", "opus", "haiku"]
        case "grok-subscription": ["grok-build"]
        default: codexModels.isEmpty ? ["gpt-6.1-sol"] : codexModels
        }
    }
    private var providerIcon: String {
        switch session.provider {
        case "claude-subscription": "asterisk"
        case "grok-subscription": "diamond"
        default: "command"
        }
    }
    private var efforts: [String] {
        switch session.provider {
        case "claude-subscription": ["low", "medium", "high", "xhigh", "max"]
        case "grok-subscription": []
        default: ["low", "medium", "high", "xhigh", "max"]
        }
    }
    private var login: String {
        switch session.provider {
        case "claude-subscription": "claude auth login"
        case "grok-subscription": "grok login"
        default: "codex login"
        }
    }

    var body: some View {
        Button {
            custom = !session.model.isEmpty && !models.contains(session.model)
            open.toggle()
        } label: {
            HStack(spacing: 4) {
                Image(systemName: providerIcon)
                Text(session.model.isEmpty ? models[0] : session.model).lineLimit(1)
                Image(systemName: "speedometer")
                Text(session.effort.isEmpty ? "Default" : session.effort)
            }
        }.accessibilityLabel("Agent settings")
            .popover(isPresented: $open, arrowEdge: .bottom) {
                VStack(alignment: .leading, spacing: 10) {
                    Picker("Provider", selection: $session.provider) {
                        Text("Codex subscription").tag("codex-subscription")
                        Text("Claude subscription").tag("claude-subscription")
                        Text("Grok subscription").tag("grok-subscription")
                    }.onChange(of: session.provider) { _, _ in session.model = ""; session.effort = ""; custom = false }
                    Picker("Model", selection: Binding(get: { custom ? "custom" : session.model }, set: { value in
                        custom = value == "custom"
                        if !custom { session.model = value }
                    })) {
                        Text("Provider default (\(models[0]))").tag("")
                        ForEach(models, id: \.self) { Text($0).tag($0) }
                        Text("Custom model…").tag("custom")
                    }
                    if custom { TextField("Custom model", text: $session.model).textFieldStyle(.roundedBorder) }
                    Picker("Effort", selection: $session.effort) {
                        Text(efforts.isEmpty ? "Managed by provider" : "Default").tag("")
                        ForEach(efforts, id: \.self) { Text($0).tag($0) }
                    }.disabled(reason: efforts.isEmpty ? "The selected model does not expose effort settings." : nil)
                    Text("Model presets depend on subscription access. Sign in on the workspace server with \(login).")
                        .font(.caption).foregroundStyle(.secondary)
                }.disabled(reason: session.running ? "Wait for the current agent turn to finish." : nil).padding(12).frame(idealWidth: 320)
                 .task(id: session.provider) {
                     guard session.provider == "codex-subscription", let engine else { return }
                     struct Catalog: Decodable { let models: [String] }
                     if let catalog: Catalog = try? await engine.request("GET", "/api/agents/codex-subscription/models"),
                        !Task.isCancelled, !catalog.models.isEmpty {
                         codexModels = catalog.models
                         if catalog.models.contains(session.model) { custom = false }
                     }
                 }
                 #if os(iOS)
                 .presentationCompactAdaptation(.popover)
                 #endif
            }
        if tokens > 0 {
            Text("\(tokens.formatted()) tokens").lineLimit(1)
            if !session.running, let totals = session.totals, totals.subscription != true, let usd = totals.usd {
                Text(usd.formatted(.currency(code: "USD").precision(.fractionLength(4))))
            } else {
                Text("Cost unavailable").lineLimit(1)
            }
        } else {
            Text("Tokens unavailable").lineLimit(1)
        }
    }
}

private struct NativeAgentProse: View {
    let text: String

    var body: some View {
        VStack(alignment: .leading, spacing: 5) {
            let rows = text.components(separatedBy: .newlines)
            ForEach(rows.indices, id: \.self) { index in
                let row = rows[index]
                let level = row.prefix { $0 == "#" }.count
                let heading = (1...6).contains(level) && row.dropFirst(level).first == " "
                let content = heading ? String(row.dropFirst(level + 1)) : row
                Text((try? AttributedString(markdown: content, options: .init(interpretedSyntax: .inlineOnlyPreservingWhitespace))) ?? AttributedString(content))
                    .font(heading ? (level == 1 ? .title2 : .headline) : .body)
                    .accessibilityAddTraits(heading ? .isHeader : [])
            }
        }.textSelection(.enabled)
         .environment(\.openURL, OpenURLAction { _ in .discarded })
    }
}

struct AgentPane: View {
    let engine: Engine?
    @Bindable var session: AgentSession
    var debugger: NativeDebugger? = nil
    var workspaceAddress: String? = nil
    var workspaceID: String? = nil
    var reviewPatch: ((DocumentBuffer, NativeRecordedMessageSource, Int) -> Void)? = nil
    var workspaceCurrent: @MainActor () -> Bool = { true }
    #if os(iOS)
    @State private var recordedInspection: NativeDebugInspector?
    #endif
    var body: some View {
        VStack(spacing: 0) {
            HStack { Text("Workspace agent").bold(); Spacer(); Text("Hickory · Python only").font(.caption).foregroundStyle(.secondary) }.padding(12)
            Divider()
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 12) {
                    if session.turns.isEmpty {
                        Text("What shall we work on?").font(.title2).padding(.top, 24)
                        Text("Write a literate document below. Code and explanations belong together.").foregroundStyle(.secondary)
                    }
                    ForEach(session.turns) { turn in
                        VStack(alignment: .leading, spacing: 5) {
                            Text("You").font(.caption).foregroundStyle(.secondary)
                            message(session.document("\(turn.id)-user", turn.prompt), turn: turn.id, side: .user).padding(8).background(.quaternary, in: RoundedRectangle(cornerRadius: 12)).padding(.leading, 24)
                            let activity = session.activity[turn.id]
                            Text(turn.model.map { "Agent · \($0)" } ?? "Agent").font(.caption).foregroundStyle(.secondary)
                            if let tools = activity?.tools, !tools.isEmpty {
                                DisclosureGroup("Tool calls (\(tools.count))") {
                                    ForEach(tools.indices, id: \.self) { index in
                                        let tool = tools[index]
                                        DisclosureGroup(tool.name + " · " + (tool.ok.map { $0 ? "Completed" : "Failed" } ?? (turn.status == "running" ? "Running" : "Outcome unavailable"))) {
                                            if let result = tool.result { Text(result).font(.caption.monospaced()).textSelection(.enabled) }
                                        }
                                    }
                                }.font(.caption)
                            }
                            if turn.status == "running" {
                                HStack {
                                    ProgressView().controlSize(.small)
                                    Text(session.stopPending ? "Stopping…" : activity?.phase ?? "Starting…")
                                }.font(.caption).foregroundStyle(.secondary)
                            }
                            if let answer = turn.answer ?? turn.error ?? activity?.answer {
                                if turn.status != "running" && (answer.contains("```") || answer.contains("~~~") || answer.contains("xmlns:")) {
                                    message(session.document("\(turn.id)-agent", answer), turn: turn.id, side: .assistant)
                                } else {
                                    NativeAgentProse(text: answer)
                                }
                            }
                        }.frame(maxWidth: .infinity, alignment: .leading)
                    }
                }.padding(10)
            }.accessibilityLabel("Conversation")
            if !session.error.isEmpty { Text(session.error).font(.caption).foregroundStyle(.red).padding(6).textSelection(.enabled) }
            Divider()
            VStack(alignment: .leading, spacing: 8) {
                Text("Your message · literate document").font(.caption).foregroundStyle(.secondary)
                NativeEditor(buffer: session.draft).id(session.draft.id).frame(height: 140).accessibilityLabel("Message")
                HStack {
                    Spacer()
                    if session.canStop {
                        Button { Task { await session.stop(engine) } } label: {
                            Image(systemName: "stop.fill").frame(width: 24, height: 24)
                        }.accessibilityLabel("Stop").disabled(reason: session.stopPending ? "A stop request is in progress." : nil)
                    } else {
                        Button { Task { await send() } } label: {
                            Image(systemName: "arrow.up").frame(width: 24, height: 24)
                        }.accessibilityLabel("Send")
                            .disabled(reason: session.running ? "Wait for the current agent turn to finish." : session.prompt.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty ? "Enter a message first." : engine == nil ? "Connect to a workspace server first." : nil)
                    }
                }.buttonStyle(.borderedProminent).buttonBorderShape(.circle)
            }.padding(12).background(.quaternary, in: RoundedRectangle(cornerRadius: 14))
        }
        #if os(iOS)
        .sheet(item: $recordedInspection) { inspection in
            inspection.onChange(of: inspection.sourceCurrent, initial: true) { _, current in
                if !current { recordedInspection = nil }
            }
        }
        #endif
    }
    private var presentRecordedInspection: ((NativeDebugInspector) -> Void)? {
        #if os(iOS)
        return { inspection in
            guard inspection.sourceCurrent else { return }
            recordedInspection = inspection
        }
        #else
        return nil
        #endif
    }
    private func message(_ buffer: DocumentBuffer, turn: String, side: RecordedMessageSide) -> some View {
        let source = NativeRecordedMessageSource(turn: turn, side: side, workspace: workspaceAddress, workspaceID: workspaceID) { [weak session, weak buffer] in
            guard let session, let buffer, workspaceCurrent() else { return false }
            return session.owns(buffer, turn: turn, side: side)
        }
        return LanguageEditor(buffer: buffer, debugger: debugger, recordedDebug: source, recordedRunEngine: engine, recordedPatchReview: reviewPatch.map { review in
            { line in review(buffer, source, line) }
        }, presentRecordedInspection: presentRecordedInspection).id(buffer.id)
            .frame(height: min(420, max(100, CGFloat(buffer.text.components(separatedBy: "\n").count) * 22 + 36)))
            .accessibilityElement(children: .contain)
            .accessibilityLabel(side == .user ? "Recorded user message" : "Recorded assistant message")
    }
    private func send() async {
        guard let engine, !session.running, !session.prompt.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return }
        let submitted = session.prompt
        #if os(iOS)
        let submittedDraft = session.draft.sourceEditIdentity
        #endif
        session.sending = true; session.error = ""; session.mutation += 1
        defer { session.replayEarlyEvents(for: ""); session.sending = false; session.mutation += 1 }
        do {
            var context = try session.captureContext?()
            context?["live"] = true
            let started = try await engine.send(submitted, parent: session.turns.last?.id, provider: session.provider, model: session.model, effort: session.effort, context: context)
            guard workspaceCurrent() else { return }
            session.liveTurn = context == nil ? nil : started.session_id
            session.turns.append(AgentTurn(id: started.session_id, prompt: submitted, status: "running"))
            session.replayEarlyEvents(for: started.session_id)
            #if os(iOS)
            if session.draft.sourceEditIdentity == submittedDraft && exactUTF8(session.prompt, submitted) {
                session.prompt = ""
                session.draft.releaseSourceFocus?()
            }
            #else
            if session.prompt == submitted { session.prompt = "" }
            #endif
        } catch { session.error = "\(error.localizedDescription) The request was not retried. Refresh history before sending again." }
    }
}
struct NativePatchCommentAnchor {
    var path: String
    var hash: String
    var line: Int
    var opening: Int
    var turn: String
    var quote: String
    var current: @MainActor () -> Bool

    var target: [String: Any] {
        ["kind": "patch-line", "path": path, "quote": quote, "source_hash": hash, "lines": [line, line],
         "relationship": ["turn": turn, "side": "assistant", "patch_opening_line": opening]]
    }
}

struct NativePatchCommentView: View {
    let anchor: NativePatchCommentAnchor
    let engine: Engine

    private struct Target: Decodable {
        var kind: String
        var path: String
        var quote: String
        var source_hash: String
        var lines: [Int]
    }
    private struct Event: Decodable, Identifiable {
        var id: String
        var thread: String
        var target: Target?
        var body: String
        var actor: String
    }
    private struct History: Decodable {
        var scope: String
        var events: [Event]
    }

    @State private var history: History?
    @State private var selectedThread: String?
    @State private var bodyText = ""
    @State private var notice = ""
    @State private var busy = false

    private var threads: [Event] {
        history?.events.filter { event in
            guard let target = event.target else { return false }
            return target.kind == "patch-line" && exactUTF8(target.path, anchor.path)
                && target.source_hash == anchor.hash && target.lines == [anchor.line, anchor.line]
                && exactUTF8(target.quote, anchor.quote)
        } ?? []
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Patch line \(anchor.line)").font(.headline)
            Text(anchor.quote).font(.caption.monospaced()).textSelection(.enabled)
            if !threads.isEmpty {
                Picker("Review thread", selection: $selectedThread) {
                    ForEach(threads) { thread in Text(String(thread.body.prefix(50))).tag(Optional(thread.thread)) }
                    Text("New thread").tag(String?.none)
                }
                if let selectedThread {
                    ScrollView {
                        VStack(alignment: .leading, spacing: 6) {
                            ForEach(history?.events.filter { $0.thread == selectedThread } ?? []) { event in
                                Text(event.body).textSelection(.enabled)
                            }
                        }.frame(maxWidth: .infinity, alignment: .leading)
                    }.frame(maxHeight: 160)
                }
            }
            TextField(selectedThread == nil ? "Write a comment" : "Write a reply", text: $bodyText, axis: .vertical)
                .textFieldStyle(.roundedBorder).lineLimit(2...5)
            HStack {
                Button(selectedThread == nil ? "Save comment" : "Save reply") { Task { await save() } }
                    .disabled(reason: busy ? "An operation is already in progress." : history == nil ? "Wait for the comment history to load." : !anchor.current() ? "Refresh comments against the current source." : bodyText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty ? "Enter a comment first." : nil)
                Button("Refresh comments") { Task { await refresh() } }.disabled(reason: busy ? "An operation is already in progress." : nil)
            }
            if !notice.isEmpty { Text(notice).font(.caption).textSelection(.enabled) }
        }.padding(12).frame(idealWidth: 350).task { await refresh() }
    }

    private func load() async throws {
        let value: History = try await engine.request("GET", "/api/reviews")
        guard !Task.isCancelled, anchor.current() else { return }
        history = value
    }

    private func refresh() async {
        guard !busy, anchor.current() else { return }
        busy = true; notice = ""
        defer { busy = false }
        do {
            try await load()
            if selectedThread == nil || !threads.contains(where: { $0.thread == selectedThread }) {
                selectedThread = threads.first?.thread
            }
        } catch { notice = error.localizedDescription }
    }

    private func save() async {
        guard !busy, let history, anchor.current() else { return }
        let comment = bodyText, thread = selectedThread
        busy = true; notice = ""
        defer { busy = false }
        do {
            var request: [String: Any] = ["scope": history.scope, "action": "comment", "body": comment]
            if let thread { request["thread"] = thread } else { request["target"] = anchor.target }
            let saved: Event = try await engine.request("POST", "/api/reviews", body: request)
            guard anchor.current() else { return }
            if exactUTF8(bodyText, comment) { bodyText = "" }
            selectedThread = saved.thread
            try await load()
        } catch {
            self.history = nil
            notice = error.localizedDescription + " Refresh comments before retrying."
        }
    }
}
