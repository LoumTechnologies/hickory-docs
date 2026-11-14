import Foundation
import Observation
struct AnalysisKnowledge: Identifiable {
    var id: String { path }; var path: String; var source: String?; var locations: [SemanticLocation]; var state: String; var notice: String
}
@MainActor @Observable final class WorkspaceAnalysis {
    var running = false; var stopping = false; var completed = 0
    var scope = workspaceAnalysisScope([]); var knowledge: [String:AnalysisKnowledge] = [:]
    func stop() { if running { stopping = true } }
    func run(_ files: [FileNode], load: @escaping (FileNode) async throws -> DocumentBuffer,
             outline: @escaping (FileNode,String) async throws -> [SemanticLocation]) async -> Bool {
        guard !running else { return false }
        scope = workspaceAnalysisScope(files); running = true; stopping = false; completed = 0; knowledge = [:]
        var cursor = 0
        @MainActor func worker() async {
            while !stopping && !Task.isCancelled && cursor < scope.files.count {
                let file = scope.files[cursor]; cursor += 1; var source: String?
                do {
                    let buffer = try await load(file); source = buffer.text
                    let locations = try await outline(file,buffer.text)
                    let current = buffer.text == source
                    knowledge[file.path] = AnalysisKnowledge(path:file.path,source:source,locations:current ? locations : [],state:current ? "ready":"stale",notice:current ? "Provider outline returned; diagnostics arrive independently.":"Source changed during analysis. Run again.")
                } catch { knowledge[file.path] = AnalysisKnowledge(path:file.path,source:source,locations:[],state:"unavailable",notice:error.localizedDescription) }
                completed += 1
            }
        }
        let first=Task {await worker()},second=Task {await worker()}
        _ = await (first.value,second.value); running = false; stopping = false; return true
    }
    func current(_ path: String, source: String) -> AnalysisKnowledge? {
        guard let item = knowledge[path],item.state == "ready",item.source == source else { return nil }; return item
    }
}
