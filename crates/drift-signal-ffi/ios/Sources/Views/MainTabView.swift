import SwiftUI

// MARK: - VSLICE: DriftSignal @Views

struct MainTabView: View {
    let engine: FfiDriftSignalEngine

    var body: some View {
        TabView {
            ChatListView(engine: engine)
                .tabItem {
                    Label("Chat", systemImage: "bubble.left.and.bubble.right")
                }

            MemoryListView(engine: engine)
                .tabItem {
                    Label("Memory", systemImage: "brain")
                }

            TaskListView(engine: engine)
                .tabItem {
                    Label("Tasks", systemImage: "checklist")
                }
        }
    }
}
