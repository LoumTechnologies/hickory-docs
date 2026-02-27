import SwiftUI

// MARK: - VSLICE: DriftSignal @Views

struct TaskListView: View {
    let engine: FfiDriftSignalEngine
    @State private var tasks: [FfiTask] = []
    @State private var showingCreate = false
    @State private var newDescription = ""

    var body: some View {
        NavigationStack {
            List {
                ForEach(tasks, id: \.id) { task in
                    HStack {
                        Button(action: { toggleTask(task) }) {
                            Image(systemName: task.status == "done" ? "checkmark.circle.fill" : "circle")
                                .foregroundColor(task.status == "done" ? .green : .secondary)
                        }
                        .buttonStyle(.plain)

                        VStack(alignment: .leading) {
                            Text(task.description)
                                .strikethrough(task.status == "done")
                            if let due = task.dueDate {
                                Text(due)
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                        }
                    }
                }
                .onDelete(perform: deleteTask)
            }
            .navigationTitle("Tasks")
            .toolbar {
                ToolbarItem(placement: .primaryAction) {
                    Button(action: { showingCreate = true }) {
                        Image(systemName: "plus")
                    }
                }
            }
            .alert("New Task", isPresented: $showingCreate) {
                TextField("Description", text: $newDescription)
                Button("Create") { createTask() }
                Button("Cancel", role: .cancel) {}
            }
            .onAppear(perform: loadTasks)
        }
    }

    private func loadTasks() {
        do {
            tasks = try engine.listTasks()
        } catch {
            print("Failed to load tasks: \(error)")
        }
    }

    private func createTask() {
        guard !newDescription.isEmpty else { return }
        do {
            _ = try engine.createTask(description: newDescription, conversationId: nil)
            newDescription = ""
            loadTasks()
        } catch {
            print("Failed to create task: \(error)")
        }
    }

    private func toggleTask(_ task: FfiTask) {
        do {
            if task.status != "done" {
                try engine.completeTask(taskId: task.id)
            }
            loadTasks()
        } catch {
            print("Failed to toggle task: \(error)")
        }
    }

    private func deleteTask(at offsets: IndexSet) {
        for index in offsets {
            let task = tasks[index]
            do {
                try engine.deleteTask(taskId: task.id)
            } catch {
                print("Failed to delete task: \(error)")
            }
        }
        loadTasks()
    }
}
