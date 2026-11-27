<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Match the browser workspace conversation

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Retain the same message buffer when the host updates its text. Native editor
caches then retain selection and viewport; the independent draft is untouched.
The host owns the replacement baseline, so an answer update never becomes a
local unsaved edit.

The writable composer also uses LanguageEditor, forwarding its explicit Send
callback into the native input surface. Keep its draft buffer identity so adding
the document gutters does not replace undo or focus ownership.

Every answer uses LanguageEditor, including plain Markdown and provisional text.
The generated conversation buffer keeps the reading immutable. Recorded fence
actions still require session ownership of an accepted exact answer, so showing
a provisional construction does not authorize execution or patch application.

Agent state belongs to its pane and never drives document rendering. Poll durable
history independently, cancelling the loop when the pane disappears.

The run channel provides immediate progress and usage. The history read still
confirms the recorded answer before fence actions can use it. Frames that arrive
before Send returns stay provisional until the server identifies the turn.

Put the operation and its target in the live status and in the collapsed tool
rows. Read only the path fields of known tools. For example, a pending edit
can identify the source page without showing the replacement or moving the
reader out of the conversation.

Send and edit refresh keep the originating editor snapshot on the host. An explicit
empty reading projection keeps that snapshot out of model input. The composer tells
users how to supply an excerpt when they want the model to read one.

The activity channel is not authority for writes. Read pending previews from the
workspace host, and send a decision tied to the running turn and both exact byte
hashes. If another reviewer revises the proposal while the sheet is open, neither
its old Approve nor its old Reject button can decide the new proposal.

The desktop patterns we take from OpenCode are small: keep the composer
separate from turn state, make Stop and pending review visible, and hide
secondary actions until asked for. A branch change or explicit jump scrolls
to the latest message; activity updates don't take the reader's scroll position.
Keep Hickory's native recorded editors and their revision-bound action tools.

Each conversation tab retains its own selected ancestry, composer, settings and
message editors. History and branch actions create another tab rather than
mutating an existing reading. Background observers continue for retained tabs;
the workspace server still owns run exclusion. Stop and token counts use only
the selected ancestry, and recorded usage replaces provisional usage once.
Keep settings and usage at this pane's bottom, where two split conversations
can report independently. Branch actions follow their particular message.
The jump control floats over the scroll view it moves, without a second title.

Keep editing in the bottom composer, including an assistant-answer copy. Saving that answer records a new human-edited branch without inference or inherited verification. Reopening the same recorded conversation selects its existing tab and preserves drafts. Activity symbols distinguish working from waiting for input; horizontal tab reveal is scoped to the tab strip.

<ui:copy id="simple-42e0887630-0">
import SwiftUI
import Observation

struct NativePendingAgentReview: Decodable, Identifiable {
    let id: String
    let name: String
    let path: String?
    let oldText: String
    let newText: String
    let editor: Bool
    let status: String

    func matches(_ other: Self) -> Bool {
        id == other.id && !editor && !other.editor && status == "pending" && other.status == "pending"
            && exactUTF8(oldText, other.oldText) && exactUTF8(newText, other.newText)
    }
}

@MainActor @Observable final class AgentSession {
    private static let pythonPath = "/api/docs/workspace/agent/python"
    private struct ReviewState: Decodable {
        struct Edits: Decodable { let captureRequest: String?; let changes: [NativePendingAgentReview] }
        let turn: String?
        let edits: Edits?
    }
    private(set) var pendingReviews: [NativePendingAgentReview] = []
    private(set) var reviewTurn: String?
    private(set) var reviewBusy = false
    private(set) var reviewError = ""
    private(set) var reviewReadError = ""
    struct BranchEdit { let turn: String; let user: Bool; let source: String; let request: String }
    var branchEdit: BranchEdit?
    var openedTip: String?
    var conversationKey: String?
    func sameConversation(_ id: String) -> Bool {
        guard branchEdit == nil else { return false }
        let selected = turns.first { $0.id == tip }, requested = turns.first { $0.id == id }
        return openedTip == id || tip == id || ((selected?.session ?? conversationKey) != nil && (selected?.session ?? conversationKey) == requested?.session)
    }
    func working(_ id: String? = nil) -> Bool {
        if id == nil && branchEdit != nil { return sending }
        if id == nil && sending { return true }
        let selected = turns.first { $0.id == (id ?? tip) }
        return turns.contains { $0.status == "running" && ($0.id == selected?.id || selected?.session != nil && $0.session == selected?.session) && !(reviewTurn == $0.id && !pendingReviews.isEmpty) }
    }
    var turns: [AgentTurn] = []
    struct Selection { let id: String?; var userFork: Bool? = nil }
    var selection: Selection?
    var tip: String? { if let selection { return selection.id }; return turns.last?.id }
    var title: String {
        let rows = transcript
        let session = rows.last?.turn.session
        return rows.first { $0.turn.session == session }?.turn.historyLabel ?? "New conversation"
    }
    var historyLoaded = false
    var messageForks = false
    var historyError = ""
    var historyReload = 0
    var transcript: [NativeAgentMessagePair] {
        var id = tip, userOnly = selection?.userFork == true
        var seen = Set<String>(), rows: [NativeAgentMessagePair] = []
        let byID = Dictionary(turns.map { ($0.id, $0) }, uniquingKeysWith: { first, _ in first })
        while let current = id, seen.insert(current).inserted, let turn = byID[current] {
            rows.append(.init(turn: turn, userOnly: userOnly))
            id = turn.parent_id
            userOnly = turn.parent_is_user == true
        }
        return Array(rows.reversed())
    }
    func select(_ id: String?, fork: Bool? = nil) {
        guard !running else { return }
        guard fork == nil || messageForks else { return }
        guard prompt.isEmpty else { error = "Send or clear your current draft before switching conversations."; return }
        selection = Selection(id: id, userFork: fork)
        error = ""
    }
    func resetHistory() {
        turns = []; selection = nil; historyLoaded = false; messageForks = false; historyError = ""; liveTurn = nil; sending = false; mutation += 1; earlyEvents.removeAll()
        pendingReviews = []; reviewTurn = nil; reviewBusy = false; reviewError = ""; reviewReadError = ""
        activity = [:]; stopPending = false; totals = nil
        documents.removeAll()
    }
    var totals: AgentTotals?
    var activity: [String: NativeAgentActivity] = [:]
    @ObservationIgnored private var earlyEvents: [NativeAgentEvent] = []
    var stopPending = false
    var canStop: Bool { transcript.contains { !$0.userOnly && $0.turn.status == "running" } }

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
            let tool = NativeAgentActivity.Tool(name: name, data: message.event.data)
            value.phase = tool.label
            value.tools.append(tool)
        case "tool_finished":
            if let index = value.tools.lastIndex(where: { $0.name == message.event.name && $0.ok == nil }) {
                value.tools[index].ok = message.event.ok
                value.tools[index].result = message.event.text.map { String($0.prefix(262144)) }
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
        if let held = documents[id] {
            if !exactUTF8(held.text, text) {
                held.base = text
                held.text = text
            }
            return held
        }
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
    func liveAgentContext() throws -> [String: Any]? {
        guard var context = try captureContext?() else { return nil }
        context["live"] = true
        context["projection"] = ["version": 1, "documents": [], "observations": []] as [String: Any]
        return context
    }
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
                    if branchEdit == nil, let key = turns.first(where: { $0.id == tip })?.session { conversationKey = key }
                    historyLoaded = true
                    messageForks = history.message_forks == true
                    historyError = ""
                    totals = history.totals
                    if !canStop { stopPending = false }
                    do { try await observeReviews(engine, mutation: mutation, current: current) }
                    catch {
                        if !Task.isCancelled, current(), mutation == self.mutation {
                                reviewReadError = "Cannot refresh agent review: \(error.localizedDescription)"
                        }
                    }
                }
            } catch {
                guard !Task.isCancelled, current() else { return }
                self.historyError = "Cannot read agent history: \(error.localizedDescription)"
            }
            do { try await Task.sleep(for: .seconds(1)) } catch { return }
        }
    }

    private func observeReviews(_ engine: Engine, mutation: Int, current: () -> Bool) async throws {
        guard let running = turns.last, running.status == "running", running.backend == "python" || running.id == liveTurn else {
            liveTurn = nil
            pendingReviews = []
            reviewTurn = nil
            reviewReadError = ""
            return
        }
        let state: ReviewState = try await engine.request("GET", Self.pythonPath, query: [URLQueryItem(name: "view", value: "pending")])
        guard !Task.isCancelled, current(), mutation == self.mutation else { return }
        reviewReadError = ""
        reviewTurn = state.turn == running.id ? state.turn : nil
        pendingReviews = reviewTurn == nil ? [] : (state.edits?.changes ?? []).filter { !$0.editor && $0.status == "pending" }
        guard reviewTurn == liveTurn, liveTurn != nil, let capture = state.edits?.captureRequest,
              let context = try liveAgentContext() else { return }
        let _: ReviewState = try await engine.request("POST", Self.pythonPath + "/review",
            body: ["capture_request": capture, "context": context])
    }

    func canDecide(_ review: NativePendingAgentReview, turn: String) -> Bool {
        !reviewBusy && !stopPending && reviewTurn == turn && tip == turn
            && pendingReviews.contains { $0.matches(review) }
    }

    func decide(_ review: NativePendingAgentReview, turn: String, accepted: Bool, engine: Engine?, current: () -> Bool) async -> Bool {
        guard let engine, current(), canDecide(review, turn: turn) else { return false }
        reviewBusy = true; reviewError = ""; mutation += 1
        defer { if current() { reviewBusy = false; mutation += 1 } }
        do {
            struct Decision: Decodable { let received: Bool }
            let result: Decision = try await engine.request("POST", Self.pythonPath + "/review-exact", body: [
                "turn": turn, "id": review.id, "accepted": accepted,
                "before": nativeSourceHash(review.oldText), "after": nativeSourceHash(review.newText),
            ])
            guard current(), result.received else { return false }
            pendingReviews.removeAll { $0.matches(review) }
            return true
        } catch {
            if current() { reviewError = error.localizedDescription }
            return false
        }
    }
}
</ui:copy>

## Compose native literate messages without file writes

The native literate editor renders each in-memory Markdown buffer immediately.
Use the same gutter for ordinary recorded code fences, with their immutable
Agent turn and message-side identity supplied separately from the buffer path.
Running assistant text and error text have no recorded answer authority. The
composer keeps its own buffer and editing history across capability changes.
The composer owns Shift+Enter so marked text stays with the input method.
Send also checks the native composition state because a button can invoke
the same operation. A late acknowledgment leaves an active composition alone.
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

Use the already guarded observer for the sidebar, rather than writing session
state from a second task. Child rows disclose their descendants; choosing a row
only loads its ancestry and never starts an agent.

Agent History loads workspace flags independently of the conversation
observer. The row menu changes the session's organization. A filtered parent
can leave an independently retained branch at the top level with its origin
label intact.

<ui:copy id="simple-apple-agent-history">
struct NativeConversationObserver: View {
    let session: AgentSession
    let engine: Engine?
    let epoch: UUID
    let current: () -> Bool
    var body: some View {
        Color.clear.frame(width: 0, height: 0)
            .task(id: [epoch.uuidString, String(session.historyReload)]) {
                guard let engine else { session.historyError = "Connect to a workspace server to use the agent."; return }
                await session.observe(engine, current: current)
            }
            .task(id: epoch) {
                guard let engine else { return }
                await session.observeActivity(engine, current: current)
            }
    }
}

struct NativeAgentMessagePair: Identifiable {
    let turn: AgentTurn
    let userOnly: Bool
    var id: String { turn.id }
}

private struct NativeConversationFlags: Decodable {
    let favorite: Bool
    let archived: Bool
}

private enum NativeConversationHistoryView: String, CaseIterable {
    case history = "History"
    case favorites = "Favorites"
    case archived = "Archived"
}

struct NativeAgentHistoryPane: View {
    private static let historyPath = "/api/docs/workspace/agent/history"
    var session: AgentSession
    var engine: Engine? = nil
    var current: () -> Bool = { true }
    var open: (String?) -> Void
    @State private var view = NativeConversationHistoryView.history
    @State private var flags: [String: NativeConversationFlags]?
    @State private var organizationError = ""
    @State private var busy = false
    @State private var reload = 0

    var body: some View {
        let visible = session.turns.filter { turn in
            let state = flagsFor(turn)
            if view == .archived { return state?.archived == true }
            return state?.archived != true && (view == .history || state?.favorite == true)
        }
        let visibleById = Dictionary(visible.map { ($0.id, $0) }, uniquingKeysWith: { first, _ in first })
        let newest = Dictionary(grouping: visible, by: historyIdentity).compactMapValues { turns in
            turns.compactMap(messageTime).max()
        }
        var children = Dictionary(grouping: visible) { turn -> String? in
            guard let id = turn.parent_id, let source = visibleById[id] else { return nil }
            if let parentSession = source.session, let session = turn.session, parentSession != session { return nil }
            return id
        }
        let roots = (children.removeValue(forKey: nil) ?? []).sorted { a, b in
            let aFavorite = flagsFor(a)?.favorite == true, bFavorite = flagsFor(b)?.favorite == true
            if view == .history && aFavorite != bFavorite { return aFavorite }
            return (newest[historyIdentity(a)] ?? .min) > (newest[historyIdentity(b)] ?? .min)
        }
        VStack(spacing: 0) {
            ScrollView {
                VStack(alignment: .leading, spacing: 4) {
                    HStack {
                        Button { open(nil) } label: { Image(systemName: "square.and.pencil").frame(width: 26, height: 22) }
                            .buttonStyle(.bordered).accessibilityLabel("New conversation").help("New conversation")
                        if flags != nil {
                            Picker("Conversation history view", selection: $view) {
                                ForEach(NativeConversationHistoryView.allCases, id: \.self) { Text($0.rawValue).tag($0) }
                            }.pickerStyle(.menu).labelsHidden()
                        }
                    }
                    if !session.historyError.isEmpty { Text(session.historyError).foregroundStyle(.red) }
                    if !organizationError.isEmpty {
                        Text(organizationError).foregroundStyle(.red)
                    }
                    if visible.isEmpty && session.historyError.isEmpty {
                        Text(emptyMessage).foregroundStyle(.secondary)
                    }
                    rows(roots, children: children, seen: [])
                }.padding(6).frame(maxWidth: .infinity, alignment: .leading)
            }.accessibilityLabel("Agent History")
            PaneStatusBar {
                Text(historyStatus(visible))
                Spacer(minLength: 0)
                if !session.historyError.isEmpty || !organizationError.isEmpty {
                    Button("Retry") {
                        if !session.historyError.isEmpty { session.historyReload += 1 }
                        if !organizationError.isEmpty { reload += 1 }
                    }.accessibilityLabel("Retry Agent History").layoutPriority(1)
                }
            }
        }.task(id: reload) { await readFlags() }
    }

    private func historyIdentity(_ turn: AgentTurn) -> String {
        turn.session.flatMap { $0.isEmpty ? nil : $0 } ?? turn.id
    }

    private func messageTime(_ turn: AgentTurn) -> Int64? {
        if let milliseconds = turn.last_message_at_ms { return milliseconds }
        guard let created = turn.created_at else { return nil }
        let date = (try? Date(created, strategy: .iso8601))
            ?? (try? Date(created, strategy: Date.ISO8601FormatStyle(includingFractionalSeconds: true)))
        return date.map { Int64($0.timeIntervalSince1970 * 1000) }
    }

    private func historyStatus(_ visible: [AgentTurn]) -> String {
        if !session.historyError.isEmpty || !organizationError.isEmpty { return "History unavailable" }
        guard session.historyLoaded, flags != nil else { return "Loading history…" }
        let count = Set(visible.map(historyIdentity)).count
        let noun = view == .favorites ? "favorite" : view == .archived ? "archived conversation" : "conversation"
        return "\(count) \(noun)\(count == 1 ? "" : "s")"
    }

    private var emptyMessage: String {
        guard session.historyLoaded else { return "Loading conversations…" }
        switch view {
        case .history: return "No conversations yet."
        case .favorites: return "No favorite conversations."
        case .archived: return "No archived conversations."
        }
    }

    private func flagsFor(_ turn: AgentTurn) -> NativeConversationFlags? {
        turn.session.flatMap { flags?[$0] }
    }

    @MainActor private func readFlags() async {
        guard let engine, current() else { return }
        busy = true
        organizationError = ""
        defer { if current() && !Task.isCancelled { busy = false } }
        do {
            let saved: [String: NativeConversationFlags] = try await engine.request("GET", Self.historyPath)
            if current() && !Task.isCancelled { flags = saved }
        } catch {
            if current() && !Task.isCancelled { organizationError = "Cannot read conversation organization: \(error.localizedDescription)" }
        }
    }

    @MainActor private func update(_ turn: AgentTurn, field: String, enabled: Bool) {
        guard let engine, current(), !busy, organizationError.isEmpty else { return }
        busy = true
        Task {
            defer { if current() { busy = false } }
            do {
                let saved: [String: NativeConversationFlags] = try await engine.request("POST", Self.historyPath, body: [
                    "turn": turn.id, field: enabled,
                ])
                if current() { flags = saved }
            } catch {
                if current() { organizationError = "Cannot confirm the change: \(error.localizedDescription)" }
            }
        }
    }

    @ViewBuilder private func rows(_ turns: [AgentTurn], children: [String?: [AgentTurn]], seen: Set<String>) -> some View {
        ForEach(turns.filter { !seen.contains($0.id) }) { turn in
            let descendants = children[turn.id] ?? []
            if descendants.isEmpty {
                row(turn)
            } else {
                DisclosureGroup {
                    AnyView(rows(descendants, children: children, seen: seen.union([turn.id]))).padding(.leading, 8)
                } label: {
                    row(turn)
                }
            }
        }
    }

    private func row(_ turn: AgentTurn) -> some View {
        let title = turn.historyLabel
        let source = session.turns.first { $0.id == turn.parent_id }
        let fork = source?.session != nil && turn.session != nil && source?.session != turn.session
        let label = fork ? "Branch from \(turn.parent_is_user == true ? "user" : "agent") · \(title)" : title
        let state = flagsFor(turn)
        let favorite = state?.favorite == true, archived = state?.archived == true
        return HStack {
            Button { open(turn.id) } label: {
                Label(label, systemImage: session.working(turn.id) ? "arrow.triangle.2.circlepath" : "bubble.left")
            }.buttonStyle(.plain)
            Spacer(minLength: 0)
            if favorite { Image(systemName: "star.fill").accessibilityLabel("Favorite") }
            if flags != nil && turn.session?.isEmpty == false {
                Menu {
                    Button(favorite ? "Remove favorite" : "Favorite") { update(turn, field: "favorite", enabled: !favorite) }
                    Button(archived ? "Restore" : "Archive") { update(turn, field: "archived", enabled: !archived) }
                } label: { Image(systemName: "ellipsis") }
                    .menuStyle(.borderlessButton)
                    .fixedSize()
                    .accessibilityLabel("More actions for \(title)")
                    .disabled(busy || !organizationError.isEmpty)
            }
        }
    }
}
</ui:copy>

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

func formattedNativeToolJSON(_ source: String) -> String {
    guard source.utf8.count <= 262144, (try? JSONSerialization.jsonObject(with: Data(source.utf8), options: .fragmentsAllowed)) != nil else { return source }
    let chars = Array(source)
    var output = "", depth = 0, quoted = false, escaped = false
    func newline() { output += "\n" + String(repeating: "  ", count: max(0, depth)) }
    for index in chars.indices {
        let char = chars[index]
        if quoted { output.append(char); if escaped { escaped = false } else if char == "\\" { escaped = true } else if char == "\"" { quoted = false }; continue }
        if char.isWhitespace { continue }
        if char == "\"" { quoted = true; output.append(char) }
        else if char == "{" || char == "[" {
            output.append(char)
            let next = chars.dropFirst(index + 1).first { !$0.isWhitespace }
            if next != (char == "{" ? "}" : "]") { depth += 1; if depth > 64 { return source }; newline() }
        } else if char == "}" || char == "]" {
            let previous = chars.prefix(index).last { !$0.isWhitespace }
            if previous != (char == "}" ? "{" : "[") { depth -= 1; newline() }
            output.append(char)
        } else if char == "," { output.append(char); newline() }
        else if char == ":" { output += ": " }
        else { output.append(char) }
    }
    return output
}

struct NativeAgentActivity {
    struct Tool {
        let name: String
        let label: String
        let input: String?
        var ok: Bool?
        var result: String?

        init(name: String, data: String?) {
            self.name = name
            self.input = data
            let operation: String
            let fields: [String]
            switch name {
            case "read_file": (operation, fields) = ("Read file", ["path"])
            case "read_doc": (operation, fields) = ("Read source", ["upstream", "doc"])
            case "read_output": (operation, fields) = ("Read output", ["path"])
            case "edit_doc": (operation, fields) = ("Propose source edit", ["upstream", "doc"])
            case "edit_output": (operation, fields) = ("Propose output edit", ["path"])
            case "create_doc": (operation, fields) = ("Propose new document", ["path"])
            case "search": (operation, fields) = ("Search workspace", [])
            case "changeset_status": (operation, fields) = ("Inspect changes", [])
            case "submit_changeset": (operation, fields) = ("Submit changeset", [])
            default: (operation, fields) = (name, [])
            }
            guard !fields.isEmpty, let data, data.utf8.count <= 65_536,
                  let call = try? JSONSerialization.jsonObject(with: Data(data.utf8)) as? [String: Any],
                  let arguments = call["arguments"] as? [String: Any],
                  let target = fields.compactMap({ arguments[$0] as? String }).first(where: { !$0.isEmpty })
            else {
                label = operation
                return
            }
            let line = target.split(whereSeparator: \.isNewline).joined(separator: " ")
            label = line.isEmpty ? operation : operation + " · " + String(line.prefix(140))
        }
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
    private struct APIProvider: Decodable, Identifiable {
        let id: String
        let label: String
        let configured: Bool
    }
    @State private var apiProviders: [APIProvider] = []
    @State private var open = false
    @State private var custom = false
    private var tokens: Int? {
        let known = session.transcript.filter { !$0.userOnly }.compactMap { row in
            row.turn.status == "running" ? session.activity[row.turn.id]?.tokens : row.turn.usage?.tokens
        }
        return known.isEmpty ? (session.transcript.isEmpty && !session.sending ? 0 : nil) : known.reduce(0, +)
    }
    private var models: [String] {
        switch session.provider {
        case "claude-subscription": ["sonnet", "opus", "haiku"]
        case "grok-subscription": ["grok-build"]
        case "codex-subscription": codexModels.isEmpty ? ["gpt-6.1-sol"] : codexModels
        default: []
        }
    }
    private var providerIcon: String {
        switch session.provider {
        case "claude-subscription": "asterisk"
        case "grok-subscription": "diamond"
        case "codex-subscription": "command"
        default: "key"
        }
    }
    private var efforts: [String] {
        switch session.provider {
        case "claude-subscription": ["low", "medium", "high", "xhigh", "max"]
        case "grok-subscription": []
        case "codex-subscription": ["low", "medium", "high", "xhigh", "max"]
        default: []
        }
    }
    private var login: String {
        switch session.provider {
        case "claude-subscription": "claude auth login"
        case "grok-subscription": "grok login"
        case "codex-subscription": "codex login"
        default: ""
        }
    }

    var body: some View {
        Button {
            custom = !session.model.isEmpty && !models.contains(session.model)
            open.toggle()
        } label: {
            HStack(spacing: 4) {
                Image(systemName: providerIcon)
                Text(session.model.isEmpty ? (models.first ?? apiProviders.first { $0.id == session.provider }?.label ?? session.provider) : session.model).lineLimit(1)
                if !efforts.isEmpty {
                    Image(systemName: "speedometer")
                    Text(session.effort.isEmpty ? "Default" : session.effort)
                }
            }
        }.accessibilityLabel("Agent settings")
                .popover(isPresented: $open, arrowEdge: .bottom) {
                VStack(alignment: .leading, spacing: 10) {
                    Picker("Provider", selection: $session.provider) {
                        Text("Codex subscription").tag("codex-subscription")
                        Text("Claude subscription").tag("claude-subscription")
                        Text("Grok subscription").tag("grok-subscription")
                        ForEach(apiProviders.filter { $0.configured || $0.id == session.provider }) { provider in
                            Text("\(provider.label) API · billed separately").tag(provider.id)
                        }
                    }.onChange(of: session.provider) { _, _ in session.model = ""; session.effort = ""; custom = false }
                    Picker("Model", selection: Binding(get: { custom ? "custom" : session.model }, set: { value in
                        custom = value == "custom"
                        if !custom { session.model = value }
                    })) {
                        Text(models.first.map { "Provider default (\($0))" } ?? "Provider default").tag("")
                        ForEach(models, id: \.self) { Text($0).tag($0) }
                        Text("Custom model…").tag("custom")
                    }
                    if custom { TextField("Custom model", text: $session.model).textFieldStyle(.roundedBorder) }
                    Picker("Effort", selection: $session.effort) {
                        Text(efforts.isEmpty ? "Managed by provider" : "Default").tag("")
                        ForEach(efforts, id: \.self) { Text($0).tag($0) }
                    }.disabled(reason: efforts.isEmpty ? "The selected model does not expose effort settings." : nil)
                    Text(login.isEmpty ? "Uses the API key configured on the workspace host. This is not subscription billing."
                         : "Sign in on the workspace host with \(login). Configured API providers can be selected above.")
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
             .task(id: open) {
                 guard open, let engine else { return }
                 struct Listing: Decodable { let providers: [APIProvider] }
                 if let listing: Listing = try? await engine.request("GET", "/api/settings/keys"), !Task.isCancelled {
                     apiProviders = listing.providers
                 }
             }
        if let tokens {
            Text("\(tokens.formatted()) tokens").lineLimit(1).help("Reported usage for this conversation; unavailable turn usage is excluded")
        } else {
            Text("Tokens unavailable").lineLimit(1)
        }
    }
}

struct AgentPane: View {
    let engine: Engine?
    @Bindable var session: AgentSession
    var newConversation: ((String?, Bool?) -> Void)? = nil
    var debugger: NativeDebugger? = nil
    var workspaceAddress: String? = nil
    var workspaceID: String? = nil
    var reviewPatch: ((DocumentBuffer, NativeRecordedMessageSource, Int) -> Void)? = nil
    var openProjectProposals: (() -> Void)? = nil
    var workspaceCurrent: @MainActor () -> Bool = { true }
    private struct OpenReview: Identifiable {
        let turn: String
        let change: NativePendingAgentReview
        var id: String { change.id }
    }
    @State private var openReview: OpenReview?
    private enum ConversationAnchor: Hashable { case latest }
    #if os(iOS)
    @State private var recordedInspection: NativeDebugInspector?
    #endif
    var body: some View {
        VStack(spacing: 0) {
            ScrollViewReader { scroll in
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 12) {
                    if session.transcript.isEmpty {
                        Text("What shall we work on?").font(.title2).padding(.top, 24)
                        Text("Ask a question or describe a change. Paste any source you want the agent to see.").foregroundStyle(.secondary)
                    }
                    ForEach(session.transcript) { row in
                        let turn = row.turn
                        VStack(alignment: .leading, spacing: 5) {
                            HStack {
                                Text("You").font(.caption).foregroundStyle(.secondary)
                                Spacer()

                            }
                            message(session.document("\(turn.id)-user", turn.prompt), turn: turn.id, side: .user).padding(8).background(.quaternary, in: RoundedRectangle(cornerRadius: 12)).padding(.leading, 24)
                            branchAction(turn, user: true)
                            if !row.userOnly {
                            let activity = session.activity[turn.id]
                            Text(turn.model.map { "Agent · \($0)" } ?? "Agent").font(.caption).foregroundStyle(.secondary)
                            if let tools = activity?.tools, !tools.isEmpty {
                                DisclosureGroup("Tool calls (\(tools.count))") {
                                    ForEach(tools.indices, id: \.self) { index in
                                        let tool = tools[index]
                                        let status = tool.ok.map { $0 ? "Completed" : "Failed" }
                                            ?? (turn.status == "running" ? "Running" : "Outcome unavailable")
                                        DisclosureGroup {
                                            if let input = tool.input { Text("Input").font(.caption.bold()); Text(formattedNativeToolJSON(input)).font(.caption.monospaced()).textSelection(.enabled) }
                                            if let result = tool.result { Text("Result").font(.caption.bold()); Text(formattedNativeToolJSON(result)).font(.caption.monospaced()).textSelection(.enabled) }
                                        } label: {
                                            Text(tool.label + " · " + status).lineLimit(1).truncationMode(.middle)
                                        }
                                    }
                                }.font(.caption)
                                if let openProjectProposals, tools.contains(where: { $0.name == "propose_workspace_changes" && $0.ok == true }) {
                                    Button("Review proposed project changes", action: openProjectProposals)
                                        .font(.callout)
                                }
                            }
                            if turn.status == "running" {
                                HStack {
                                    ProgressView().controlSize(.small)
                                    Text(session.stopPending ? "Stopping…" : session.reviewTurn == turn.id && !session.pendingReviews.isEmpty ? "Waiting for your review" : activity?.phase ?? "Starting…")
                                        .lineLimit(1).truncationMode(.middle)
                                }.font(.caption).foregroundStyle(.secondary)
                            }
                            if let answer = turn.answer ?? turn.error ?? activity?.answer {
                                message(session.document("\(turn.id)-agent", answer), turn: turn.id, side: .assistant)
                                if turn.answer != nil { branchAction(turn, user: false) }
                            }
                            }
                        }.frame(maxWidth: .infinity, alignment: .leading)
                    }
                    Color.clear.frame(height: 1).id(ConversationAnchor.latest).accessibilityHidden(true)
                }.padding(10)
            }.accessibilityLabel("Conversation")
             .overlay(alignment: .bottomTrailing) {
                 if !session.transcript.isEmpty {
                     Button { scroll.scrollTo(ConversationAnchor.latest, anchor: .bottom) } label: {
                         Label("Latest message", systemImage: "arrow.down.to.line")
                     }.accessibilityLabel("Jump to latest message").help("Jump to latest message").padding(8)
                 }
             }
             .onChange(of: session.tip, initial: true) { _, _ in
                 scroll.scrollTo(ConversationAnchor.latest, anchor: .bottom)
             }
            }
            if let user = session.selection?.userFork { Text("New branch from \(user ? "user" : "assistant") message. \(session.branchEdit?.user == false ? "Save the edited answer to create it." : "Send to start it.")").font(.caption) }
            if let turn = session.reviewTurn, turn == session.tip {
                ForEach(session.pendingReviews) { change in
                    Button {
                        openReview = OpenReview(turn: turn, change: change)
                    } label: {
                        Label("Review \(change.path ?? change.name)", systemImage: "doc.badge.ellipsis").lineLimit(1)
                    }.frame(maxWidth: .infinity, alignment: .leading).padding(.horizontal, 12).padding(.vertical, 4)
                }
            }
            if let error = [session.error, session.reviewError, session.reviewReadError, session.historyError].first(where: { !$0.isEmpty }) {
                Text(error).font(.caption).foregroundStyle(.red).padding(6).textSelection(.enabled)
            }
            Divider()
            VStack(alignment: .leading, spacing: 4) {
                if let edit = session.branchEdit { Text("Edit \(edit.user ? "user" : "assistant") message · The original stays unchanged.").font(.caption) }
                LanguageEditor(buffer: session.draft, debugger: nil, submitMessage: { Task { await send() } }).id(session.draft.id).frame(height: 92).accessibilityLabel("Message")
                HStack {
                    Spacer()
                    if session.canStop {
                        Button { Task { await session.stop(engine) } } label: {
                            Image(systemName: "stop.fill").frame(width: 24, height: 24)
                        }.accessibilityLabel("Stop").disabled(reason: session.stopPending ? "A stop request is in progress." : nil)
                    } else {
                        Button { Task { await send() } } label: {
                            Image(systemName: session.branchEdit?.user == false ? "checkmark" : "arrow.up").frame(width: 24, height: 24)
                        }.accessibilityLabel(session.branchEdit?.user == false ? "Save assistant branch" : "Send")
                            .help(session.branchEdit?.user == false ? "Save assistant branch (⇧↩)" : "Send (⇧↩)")
                            .disabled(reason: sendDisabled)
                    }
                }.buttonStyle(.borderedProminent).buttonBorderShape(.circle)
            }.padding(12).background(.quaternary, in: RoundedRectangle(cornerRadius: 14))
            PaneStatusBar {
                NativeAgentStatus(session: session, engine: engine)
                Spacer()
                Button { if let newConversation { newConversation(nil, nil) } else { session.select(nil) } } label: {
                    Image(systemName: "square.and.pencil")
                }.accessibilityLabel("New conversation").help("New conversation").layoutPriority(1)
            }
        }
        .sheet(item: $openReview) { proposal in
            VStack(alignment: .leading, spacing: 12) {
                Text(proposal.change.path ?? proposal.change.name).font(.headline)
                Text("Compare the saved source with the proposed replacement. Approval lets the host recheck and attempt the edit; it does not save an editor draft or verify the result.")
                    .font(.caption).foregroundStyle(.secondary)
                ScrollView {
                    VStack(alignment: .leading, spacing: 12) {
                        reviewText("Before", proposal.change.oldText)
                        reviewText("Proposed", proposal.change.newText)
                    }.frame(maxWidth: .infinity, alignment: .leading)
                }
                if !session.reviewError.isEmpty {
                    Text(session.reviewError).font(.caption).foregroundStyle(.red).textSelection(.enabled)
                }
                HStack {
                    Button("Close") { openReview = nil }.keyboardShortcut(.cancelAction)
                    Spacer()
                    Group {
                        Button("Reject") { decide(proposal, accepted: false) }
                        Button("Approve") { decide(proposal, accepted: true) }.buttonStyle(.borderedProminent)
                    }.disabled(reason: !session.canDecide(proposal.change, turn: proposal.turn) ? "This preview is no longer actionable. Close and read the current review." : nil)
                }
            }.padding().frame(idealWidth: 680, minHeight: 360, idealHeight: 500)
        }
        #if os(iOS)
        .sheet(item: $recordedInspection) { inspection in
            inspection.onChange(of: inspection.sourceCurrent, initial: true) { _, current in
                if !current { recordedInspection = nil }
            }
        }
        #endif
    }
    private func reviewText(_ title: String, _ text: String) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title).font(.subheadline.bold())
            if text.isEmpty {
                Text("Empty").font(.caption).foregroundStyle(.secondary)
            } else {
                Text(verbatim: text).font(.system(.body, design: .monospaced)).textSelection(.enabled)
            }
        }
    }
    private func decide(_ review: OpenReview, accepted: Bool) {
        Task {
            if await session.decide(review.change, turn: review.turn, accepted: accepted, engine: engine, current: workspaceCurrent),
               openReview?.id == review.id { openReview = nil }
        }
    }
    @ViewBuilder private func branchAction(_ turn: AgentTurn, user: Bool) -> some View {
        if session.messageForks && (turn.backend == nil || turn.backend == "python") {
            HStack {
                Spacer()
                Menu {
                    Button(user ? "Branch from user message" : "Branch from assistant message") {
                        if let newConversation { newConversation(turn.id, user) } else { session.select(turn.id, fork: user) }
                    }
                } label: { Image(systemName: "ellipsis.circle") }
                .buttonStyle(.plain).fixedSize().accessibilityLabel(user ? "User message actions" : "Assistant message actions")
            }
        }
    }
    private var sendDisabled: String? {
        if engine == nil { return "Connect to a workspace server first." }
        if !session.historyLoaded { return "Wait for conversation history to load." }
        if session.sending || session.branchEdit?.user != false && session.running { return "Wait for the current agent turn to finish." }
        if session.branchEdit?.user != false && session.prompt.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty { return "Enter a message first." }
        return nil
    }
</ui:copy>

The workspace polls only while Agent is visible or owns a running turn. This
keeps capture answers available when the reader switches to source. Each reply
requires the same workspace and turn plus the engine’s current capture ID.

Capture the prompt before sending and clear it only if the user has not typed a replacement. An optimistic running record prevents a second send before the next history refresh.

The iPhone inspector is owned by the Agent pane instead of its scrolling message row. When the keyboard takes vertical space, the selected recorded source keeps its inspector. If the recorded source or workspace closes, retire that presentation.

Sort each conversation by its newest message, keeping favorites first in
History. Cross-session branches sort independently while same-session replies
stay under their parent. The history strip counts each displayed session once. A retry restarts the history read or flags request
without changing the open conversation or its activity subscription.

<ui:copy id="simple-42e0887630-1">
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
        guard let engine, workspaceCurrent(), sendDisabled == nil,
              session.draft.sourceEditorState?()?.composing != true else { return }
        if let edit = session.branchEdit, !edit.user {
            let text = session.prompt
            session.sending = true; session.mutation += 1
            defer { session.sending = false; session.mutation += 1 }
            do {
                let turn: AgentTurn = try await engine.request("POST", "/api/docs/workspace/agent/branch", body: ["turn": edit.turn, "source_hash": nativeSourceHash(edit.source), "answer": text, "request_id": edit.request])
                guard workspaceCurrent() else { return }
                session.turns.removeAll { $0.id == turn.id }; session.turns.append(turn); session.selection = .init(id: turn.id); session.openedTip = turn.id; session.conversationKey = turn.session
                if exactUTF8(session.prompt, text) { session.prompt = ""; session.branchEdit = nil }
                else { session.branchEdit = .init(turn: turn.id, user: false, source: text, request: UUID().uuidString.replacingOccurrences(of: "-", with: "").lowercased()) }
            } catch { if workspaceCurrent() { session.error = error.localizedDescription + " Your edited answer is retained. Refresh history before retrying." } }
            return
        }
        let submitted = session.prompt
        let parent = session.tip, fork = session.selection?.userFork
        if let backend = session.turns.first(where: { $0.id == parent })?.backend, backend != "python" {
            session.error = "This conversation uses a different agent. Start a new conversation to send with Hickory Python."
            return
        }
        guard fork == nil || session.messageForks else {
            session.error = "This workspace server does not support message branches. Update the server before sending."
            return
        }
        let submittedDraft = session.draft.sourceEditIdentity
        session.sending = true; session.error = ""; session.mutation += 1
        defer { if workspaceCurrent() { session.replayEarlyEvents(for: ""); session.sending = false; session.mutation += 1 } }
        do {
            let context = try session.liveAgentContext()
            let started = try await engine.send(submitted, parent: parent, provider: session.provider, model: session.model, effort: session.effort, context: context, fork: fork)
            guard workspaceCurrent() else { return }
            session.liveTurn = context == nil ? nil : started.session_id
            session.conversationKey = fork == nil ? session.conversationKey ?? session.turns.first(where: { $0.id == parent })?.session : nil
            session.selection = .init(id: started.session_id); session.openedTip = started.session_id; session.branchEdit = nil
            session.turns.append(AgentTurn(id: started.session_id, prompt: submitted, status: "running", parent_id: parent, parent_is_user: fork))
            session.replayEarlyEvents(for: started.session_id)
            if session.draft.sourceEditIdentity == submittedDraft && exactUTF8(session.prompt, submitted)
                && session.draft.sourceEditorState?()?.composing != true {
                session.prompt = ""
                #if os(iOS)
                session.draft.releaseSourceFocus?()
                #endif
            }
        } catch { if workspaceCurrent() { session.error = "\(error.localizedDescription) The request was not retried. Refresh history before sending again." } }
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
</ui:copy>

<ui:file path="apps/apple-simple/Sources/AgentPane.swift"><ui:paste select="#simple-42e0887630-0"/><ui:paste select="#simple-apple-agent-history"/><ui:paste select="#simple-apple-agent-pane"/><ui:paste select="#simple-42e0887630-1"/><ui:paste select="#apple-recorded-patch-comment"/></ui:file>

</ui:doc>
