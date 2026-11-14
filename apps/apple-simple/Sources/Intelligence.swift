import Foundation

enum SemanticQuery: String, CaseIterable, Identifiable {
    case definition, references, typeDefinition, implementation, declaration, outline, symbols, callers, callees
    var id: String { rawValue }
    var label: String {
        switch self {
        case .definition: return "Go to definition"
        case .references: return "Find usages"
        case .typeDefinition: return "Go to type definition"
        case .implementation: return "Find implementations"
        case .declaration: return "Go to declaration"
        case .outline: return "File outline"
        case .symbols: return "Search workspace symbols"
        case .callers: return "Find callers"
        case .callees: return "Find callees"
        }
    }
    var method: String {
        switch self {
        case .outline: return "textDocument/documentSymbol"
        case .symbols: return "workspace/symbol"
        case .callers, .callees: return "textDocument/prepareCallHierarchy"
        default: return "textDocument/" + rawValue
        }
    }
}
struct SemanticLocation: Identifiable, Equatable {
    var uri: String; var line: Int; var character: Int; var name: String? = nil
    var id: String { "\(uri):\(line):\(character)" }
    var path: String? {
        guard uri.hasPrefix("hick:///"), let value = URLComponents(string: uri), value.scheme == "hick", value.host?.isEmpty != false,
              value.query == nil, value.fragment == nil,
              let decoded = value.percentEncodedPath.removingPercentEncoding, decoded.hasPrefix("/") else { return nil }
        let path = String(decoded.dropFirst()), parts = path.split(separator: "/", omittingEmptySubsequences: false)
        guard !path.isEmpty, !path.contains(":"), !path.contains("\\"), !path.unicodeScalars.contains(where: { $0.value < 32 }),
              parts.allSatisfy({ !$0.isEmpty && $0 != "." && $0 != ".." }) else { return nil }
        return path
    }
    var label: String { (name.map { $0 + " · " } ?? "") + "\(path ?? uri):\(line + 1):\(character + 1)" }
}
func semanticLocations(_ result: Any, defaultURI: String? = nil) -> [SemanticLocation] {
    var seen = Set<String>(), locations: [SemanticLocation] = []
    func visit(_ value: Any, _ depth: Int) {
        guard depth <= 64, locations.count < 10000 else { return }
        if let items = value as? [Any] { for item in items { visit(item, depth + 1) }; return }
        guard let item = value as? [String: Any] else { return }
        if let call = item["from"] ?? item["to"] { visit(call, depth + 1); return }
        let target = item["location"] as? [String: Any] ?? item
        let uri = (target["uri"] ?? target["targetUri"]) as? String ?? (item["name"] is String && item["location"] == nil ? defaultURI : nil)
        if let uri, let range = (target["selectionRange"] ?? target["range"] ?? target["targetSelectionRange"] ?? target["targetRange"]) as? [String: Any],
           let start = range["start"] as? [String: Any], let line = start["line"] as? Int, line >= 0,
           let character = start["character"] as? Int, character >= 0 {
            let location = SemanticLocation(uri: uri, line: line, character: character, name: item["name"] as? String)
            if seen.insert(location.id).inserted { locations.append(location) }
        }
        if let children = item["children"] { visit(children, depth + 1) }
    }
    visit(result, 0); return locations
}
struct NavigationTrail {
    private(set) var entries: [SemanticLocation] = []
    private(set) var index = -1
    var canBack: Bool { index > 0 }
    var canForward: Bool { index >= 0 && index + 1 < entries.count }
    mutating func record(from: SemanticLocation?, to: SemanticLocation) {
        entries = Array(entries.prefix(index + 1))
        if let from, entries.last?.id != from.id { entries.append(from) }
        if entries.last?.id != to.id { entries.append(to) }
        entries = Array(entries.suffix(256)); index = entries.count - 1
    }
    mutating func step(_ direction: Int, current: SemanticLocation?) -> SemanticLocation? {
        let next = index + direction
        guard (direction == -1 || direction == 1), entries.indices.contains(next) else { return nil }
        if let current, entries.indices.contains(index), entries[index].uri == current.uri { entries[index] = current }
        index = next; return entries[next]
    }
}
struct WorkspaceProblem: Identifiable {
    var id: String; var path: String; var message: String; var severity: Int
    var location: SemanticLocation?; var source: String?; var current: Bool
}
func nextWorkspaceProblem(_ problems: [WorkspaceProblem], current: SemanticLocation?, direction: Int) -> WorkspaceProblem? {
    guard direction == -1 || direction == 1 else { return nil }
    let located = problems.filter { $0.current && $0.location != nil }.sorted {
        let a = $0.location!, b = $1.location!; return (a.uri, a.line, a.character) < (b.uri, b.line, b.character)
    }
    guard !located.isEmpty else { return nil }
    if direction == 1 {
        return current.flatMap { point in located.first { let at = $0.location!; return (at.uri, at.line, at.character) > (point.uri, point.line, point.character) } } ?? located.first
    }
    return current.flatMap { point in located.last { let at = $0.location!; return (at.uri, at.line, at.character) < (point.uri, point.line, point.character) } } ?? located.last
}
struct WorkspaceSearchHit: Identifiable {
    var path: String; var line: Int; var text: String; var column: Int; var sourceHash: String?; var generatedBy: String?
    var id: String { "\(path):\(line):\(column)" }
    var character: Int? { searchCharacter(text, byteColumn: column) }
}
struct WorkspaceSearchResult { var hits: [WorkspaceSearchHit]; var truncated: Bool }
func searchCharacter(_ line: String, byteColumn: Int) -> Int? {
    guard byteColumn >= 0 else { return nil }
    var bytes = 0, units = 0
    for scalar in line.unicodeScalars {
        if bytes == byteColumn { return units }
        bytes += String(scalar).utf8.count; units += scalar.value > 0xffff ? 2 : 1
    }
    return bytes == byteColumn ? units : nil
}
