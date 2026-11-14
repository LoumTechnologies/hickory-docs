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
