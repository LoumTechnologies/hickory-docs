<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Match the browser workspace conversation

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Agent state belongs to its pane and never drives document rendering. Poll the existing durable history independently, cancelling the loop when the pane disappears. This first slice shows completed answers rather than token streaming.

<ui:copy id="simple-42e0887630-0">
import SwiftUI
import Observation

@MainActor @Observable final class AgentSession {
    var turns: [AgentTurn] = []
    var prompt = ""
    var sending = false
    var error = ""
    var mutation = 0
    var running: Bool { sending || turns.contains { $0.status == "running" } }
}
struct AgentPane: View {
    let engine: Engine?
    @Bindable var session: AgentSession
    var body: some View {
        VStack(spacing: 0) {
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 12) {
                    ForEach(session.turns) { turn in
                        VStack(alignment: .leading, spacing: 5) {
                            Text("You").bold(); Text(turn.prompt).textSelection(.enabled)
                            Text("Agent · \(turn.status)").bold()
                            Text(turn.answer ?? turn.error ?? (turn.status == "running" ? "Working…" : "")).textSelection(.enabled)
                        }.frame(maxWidth: .infinity, alignment: .leading)
                    }
                }.padding(10)
            }.accessibilityLabel("Conversation")
            if !session.error.isEmpty { Text(session.error).font(.caption).foregroundStyle(.red).padding(6).textSelection(.enabled) }
            Divider()
            HStack {
                TextField("Message the workspace agent", text: $session.prompt, axis: .vertical).accessibilityLabel("Message").lineLimit(1...5)
                Button("Send") { Task { await send() } }.disabled(session.running || session.prompt.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || engine == nil)
                Button("Stop") { Task { do { try await engine?.stop() } catch { session.error = "\(error.localizedDescription) Refresh history and try Stop again." } } }.disabled(!session.running)
            }.padding(6)
        }.task {
            guard let engine else { session.error = "Connect to a workspace server to use the agent."; return }
            while !Task.isCancelled {
                do { let mutation = session.mutation; let history = try await engine.turns(); guard !Task.isCancelled else { return }; if mutation == session.mutation && !session.sending { session.turns = history.turns } }
                catch { if Task.isCancelled { return }; session.error = "\(error.localizedDescription) Check agent configuration in the existing frontend." }
                do { try await Task.sleep(for: .seconds(1)) } catch { return }
            }
        }
    }
</ui:copy>

Capture the prompt before sending and clear it only if the user has not typed a replacement. An optimistic running record prevents a second send before the next history refresh.

<ui:copy id="simple-42e0887630-1">
    private func send() async {
        guard let engine, !session.running, !session.prompt.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return }
        let submitted = session.prompt
        session.sending = true; session.error = ""; session.mutation += 1
        defer { session.sending = false; session.mutation += 1 }
        do {
            let started = try await engine.send(submitted, parent: session.turns.last?.id)
            session.turns.append(AgentTurn(id: started.session_id, prompt: submitted, status: "running"))
            if session.prompt == submitted { session.prompt = "" }
        } catch { session.error = "\(error.localizedDescription) The request was not retried. Refresh history before sending again." }
    }
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/apple-simple/Sources/AgentPane.swift"><ui:paste select="#simple-42e0887630-0"/><ui:paste select="#simple-42e0887630-1"/></ui:file>

</ui:doc>
