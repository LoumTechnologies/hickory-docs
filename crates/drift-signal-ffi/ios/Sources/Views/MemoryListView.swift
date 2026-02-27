import SwiftUI

// MARK: - VSLICE: DriftSignal @Views

struct MemoryListView: View {
    let engine: FfiDriftSignalEngine
    @State private var memories: [FfiMemory] = []
    @State private var searchQuery = ""
    @State private var searchResults: [FfiMemorySearchResult] = []

    var body: some View {
        NavigationStack {
            List {
                if !searchQuery.isEmpty {
                    Section("Search Results") {
                        ForEach(searchResults, id: \.id) { result in
                            VStack(alignment: .leading) {
                                Text(result.key)
                                    .font(.headline)
                                Text(result.content)
                                    .font(.body)
                                    .foregroundColor(.secondary)
                            }
                        }
                    }
                }

                Section("All Memories") {
                    ForEach(memories, id: \.id) { mem in
                        VStack(alignment: .leading) {
                            Text(mem.key)
                                .font(.headline)
                            Text(mem.content)
                                .font(.body)
                                .foregroundColor(.secondary)
                            Text(mem.createdAt)
                                .font(.caption2)
                                .foregroundColor(.tertiary)
                        }
                    }
                    .onDelete(perform: deleteMemory)
                }
            }
            .navigationTitle("Memory")
            .searchable(text: $searchQuery)
            .onChange(of: searchQuery) { _ in performSearch() }
            .onAppear(perform: loadMemories)
        }
    }

    private func loadMemories() {
        do {
            memories = try engine.listMemories()
        } catch {
            print("Failed to load memories: \(error)")
        }
    }

    private func performSearch() {
        guard !searchQuery.isEmpty else {
            searchResults = []
            return
        }
        do {
            searchResults = try engine.searchMemories(query: searchQuery)
        } catch {
            print("Search failed: \(error)")
        }
    }

    private func deleteMemory(at offsets: IndexSet) {
        for index in offsets {
            let mem = memories[index]
            do {
                try engine.deleteMemory(id: mem.id)
            } catch {
                print("Failed to delete memory: \(error)")
            }
        }
        loadMemories()
    }
}
