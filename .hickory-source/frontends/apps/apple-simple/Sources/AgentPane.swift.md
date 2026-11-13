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
        if let held = documents[id], held.text == text { return held }
        let path = "conversation/\(id).md"
        let buffer = DocumentBuffer(file: FileNode(name: id, path: path, dir: false, generated_by: "conversation"), text: FileText(path: path, content: text, hash: ""))
        documents[id] = buffer
        return buffer
    }
    var sending = false
    var error = ""
    var mutation = 0
    var running: Bool { sending || turns.contains { $0.status == "running" } }
}
</ui:copy>

## Compose native literate messages without file writes

NativeEditor renders each in-memory Markdown buffer immediately and attaches its
inline capabilities independently. History is read-only and has no document ID,
so its examples cannot become execution controls. Keep the draft buffer above the
pane lifecycle. Place subscription controls and Send below it; changing a provider
resets the incompatible model and effort rather than replacing the editor.

<ui:copy id="simple-apple-agent-pane">
struct AgentPane: View {
    let engine: Engine?
    @Bindable var session: AgentSession
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
                            message(session.document("\(turn.id)-user", turn.prompt)).padding(8).background(.quaternary, in: RoundedRectangle(cornerRadius: 12)).padding(.leading, 24)
                            Text("Agent · \(turn.status)").font(.caption).foregroundStyle(.secondary)
                            message(session.document("\(turn.id)-agent", turn.answer ?? turn.error ?? (turn.status == "running" ? "Working…" : "")))
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
        }.task {
            guard let engine else { session.error = "Connect to a workspace server to use the agent."; return }
            while !Task.isCancelled {
                do { let mutation = session.mutation; let history = try await engine.turns(); guard !Task.isCancelled else { return }; if mutation == session.mutation && !session.sending { session.turns = history.turns } }
                catch { if Task.isCancelled { return }; session.error = "\(error.localizedDescription) Check subscription sign-in on the workspace server." }
                do { try await Task.sleep(for: .seconds(1)) } catch { return }
            }
        }
    }
</ui:copy>

Capture the prompt before sending and clear it only if the user has not typed a replacement. An optimistic running record prevents a second send before the next history refresh.

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
    private func message(_ buffer: DocumentBuffer) -> some View {
        NativeEditor(buffer: buffer).id(buffer.id)
            .frame(height: min(420, max(100, CGFloat(buffer.text.components(separatedBy: "\n").count) * 22 + 36)))
    }
    private func send() async {
        guard let engine, !session.running, !session.prompt.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return }
        let submitted = session.prompt
        session.sending = true; session.error = ""; session.mutation += 1
        defer { session.sending = false; session.mutation += 1 }
        do {
            let started = try await engine.send(submitted, parent: session.turns.last?.id, provider: session.provider, model: session.model, effort: session.effort)
            session.turns.append(AgentTurn(id: started.session_id, prompt: submitted, status: "running"))
            if session.prompt == submitted { session.prompt = "" }
        } catch { session.error = "\(error.localizedDescription) The request was not retried. Refresh history before sending again." }
    }
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/apple-simple/Sources/AgentPane.swift"><ui:paste select="#simple-42e0887630-0"/><ui:paste select="#simple-apple-agent-pane"/><ui:paste select="#simple-42e0887630-1"/></ui:file>

</ui:doc>
