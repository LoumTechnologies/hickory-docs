<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Match the browser workspace conversation

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Agent state belongs to its pane and never drives document rendering. Poll the existing durable history independently, cancelling the loop when the pane disappears. This first slice shows completed answers rather than token streaming.

<ui:copy id="simple-42e0887630-0">
import SwiftUI
import Observation

@MainActor @Observable final class AgentSession {
    var turns: [AgentTurn] = []
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
</ui:copy>

## Compose native literate messages without file writes

The native literate editor renders each in-memory Markdown buffer immediately.
Use the same gutter for ordinary recorded code fences, with their immutable
Agent turn and message-side identity supplied separately from the buffer path.
Running assistant text and error text have no recorded answer authority. The
composer keeps its own buffer and editing history across capability changes.
On iPhone, the keyboard can consume the conversation viewport. Once Send is
acknowledged, release the composer's native focus only when clearing the same
submitted draft revision. A newer draft keeps its focus. If the user already
focused another control, the composer leaves that focus alone.

Opening Run sends one request tied to the archived side, source hash, and
opening line. A receipt can feed the gutter terminal only while the mounted
message and workspace remain current. Closing the popover retires its reads;
a request already admitted by the engine may continue to run.

An attributed or XML patch points to a declared construction operation.
Inspect reads that operation's before and after outputs from the completed
capture. It checks the exact recorded bytes, namespace and line again before
showing the comparison in the gutter popover. Those output bytes carry no
Apply or Save authority.

<ui:copy id="simple-apple-agent-pane">
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
                            Text("Agent · \(turn.status)").font(.caption).foregroundStyle(.secondary)
                            message(session.document("\(turn.id)-agent", turn.answer ?? turn.error ?? (turn.status == "running" ? "Working…" : "")), turn: turn.id, side: .assistant)
                        }.frame(maxWidth: .infinity, alignment: .leading)
                    }
                }.padding(10)
            }.accessibilityLabel("Conversation")
            if !session.error.isEmpty { Text(session.error).font(.caption).foregroundStyle(.red).padding(6).textSelection(.enabled) }
            Divider()
            VStack(alignment: .leading, spacing: 8) {
                Text("Your message · literate document").font(.caption).foregroundStyle(.secondary)
                NativeEditor(buffer: session.draft).id(session.draft.id).frame(height: 140).accessibilityLabel("Message")
                ViewThatFits(in: .horizontal) {
                    HStack { choices }
                    VStack(alignment: .leading) { choices }
                }
                HStack {
                Button("Send") { Task { await send() } }.disabled(session.running || session.prompt.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || engine == nil)
                Button("Stop") { Task { do { try await engine?.stop() } catch { session.error = "\(error.localizedDescription) Refresh history and try Stop again." } } }.disabled(!session.running)
                }
                Text("Sign in on the workspace server with \(session.provider == "claude-subscription" ? "claude auth login" : session.provider == "grok-subscription" ? "grok login" : "codex login"). Uses your subscription; no API fallback.").font(.caption2).foregroundStyle(.secondary)
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
</ui:copy>

The workspace polls only while Agent is visible or owns a running turn. This
keeps capture answers available when the reader switches to source. Each reply
requires the same workspace and turn plus the engine’s current capture ID.

Capture the prompt before sending and clear it only if the user has not typed a replacement. An optimistic running record prevents a second send before the next history refresh.

The iPhone inspector is owned by the Agent pane instead of its scrolling message row. When the keyboard takes vertical space, the selected recorded source keeps its inspector. If the recorded source or workspace closes, retire that presentation.

<ui:copy id="simple-42e0887630-1">
    @ViewBuilder private var choices: some View {
        Picker("Provider", selection: $session.provider) {
            Text("ChatGPT subscription").tag("codex-subscription")
            Text("Claude Code subscription").tag("claude-subscription")
            Text("Grok subscription").tag("grok-subscription")
        }.onChange(of: session.provider) { _, _ in session.model = ""; session.effort = "" }.disabled(session.running)
        TextField("Model · provider default", text: $session.model).textFieldStyle(.roundedBorder).disabled(session.running)
        Picker("Effort", selection: $session.effort) {
            Text("Default").tag("")
            ForEach(["low", "medium", "high", "xhigh", "max"], id: \.self) { Text($0).tag($0) }
        }.disabled(session.running || session.provider == "grok-subscription")
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
        defer { session.sending = false; session.mutation += 1 }
        do {
            var context = try session.captureContext?()
            context?["live"] = true
            let started = try await engine.send(submitted, parent: session.turns.last?.id, provider: session.provider, model: session.model, effort: session.effort, context: context)
            guard workspaceCurrent() else { return }
            session.liveTurn = context == nil ? nil : started.session_id
            session.turns.append(AgentTurn(id: started.session_id, prompt: submitted, status: "running"))
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
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

## Attach a review thread to the recorded patch line

Take the line number and literal quote from the retained message. The thread
stores its complete source hash and message identity alongside the comment.
Choosing a different row changes the discussion target. Commenting leaves
the message and workspace source untouched.

<ui:copy id="apple-recorded-patch-comment">
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
                    .disabled(busy || history == nil || !anchor.current() || bodyText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                Button("Refresh comments") { Task { await refresh() } }.disabled(busy)
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
</ui:copy>

<ui:file path="apps/apple-simple/Sources/AgentPane.swift"><ui:paste select="#simple-42e0887630-0"/><ui:paste select="#simple-apple-agent-pane"/><ui:paste select="#simple-42e0887630-1"/><ui:paste select="#apple-recorded-patch-comment"/></ui:file>

</ui:doc>
