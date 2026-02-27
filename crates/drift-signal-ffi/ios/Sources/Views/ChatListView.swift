import SwiftUI

// MARK: - VSLICE: DriftSignal @Views

struct ChatListView: View {
    let engine: FfiDriftSignalEngine
    @State private var conversations: [FfiConversation] = []
    @State private var newTitle = ""

    var body: some View {
        NavigationStack {
            List {
                ForEach(conversations, id: \.id) { conv in
                    NavigationLink(conv.title) {
                        ChatDetailView(engine: engine, conversation: conv)
                    }
                }
            }
            .navigationTitle("Conversations")
            .toolbar {
                ToolbarItem(placement: .primaryAction) {
                    Button(action: createConversation) {
                        Image(systemName: "plus")
                    }
                }
            }
            .onAppear(perform: loadConversations)
        }
    }

    private func loadConversations() {
        do {
            conversations = try engine.listConversations()
        } catch {
            print("Failed to load conversations: \(error)")
        }
    }

    private func createConversation() {
        do {
            let id = try engine.createConversation(title: "New Conversation")
            loadConversations()
        } catch {
            print("Failed to create conversation: \(error)")
        }
    }
}

struct ChatDetailView: View {
    let engine: FfiDriftSignalEngine
    let conversation: FfiConversation
    @State private var messages: [FfiMessage] = []
    @State private var draft = ""

    var body: some View {
        VStack {
            List(messages, id: \.id) { msg in
                VStack(alignment: msg.role == "user" ? .trailing : .leading) {
                    Text(msg.role)
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Text(msg.content)
                }
                .frame(maxWidth: .infinity, alignment: msg.role == "user" ? .trailing : .leading)
            }

            HStack {
                TextField("Message...", text: $draft)
                    .textFieldStyle(.roundedBorder)
                Button("Send") {
                    sendMessage()
                }
                .disabled(draft.isEmpty)
            }
            .padding()
        }
        .navigationTitle(conversation.title)
        .onAppear(perform: loadMessages)
    }

    private func loadMessages() {
        do {
            messages = try engine.listMessages(conversationId: conversation.id)
        } catch {
            print("Failed to load messages: \(error)")
        }
    }

    private func sendMessage() {
        let text = draft
        draft = ""
        do {
            _ = try engine.sendMessage(conversationId: conversation.id, role: "user", content: text)
            loadMessages()
        } catch {
            print("Failed to send message: \(error)")
        }
    }
}
