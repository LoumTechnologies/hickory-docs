import Foundation

func dropEdge(_ x: Double, _ y: Double) -> String? {
    guard x.isFinite, y.isFinite, (0...1).contains(x), (0...1).contains(y) else { return nil }
    return x < 0.2 ? "left" : x > 0.8 ? "right" : y < 0.2 ? "top" : y > 0.8 ? "bottom" : nil
}
struct PaneNode: Identifiable, Equatable {
    var id: String
    var tabs: [String] = []
    var active: String? = nil
    var axis: String? = nil
    var first: String? = nil
    var second: String? = nil
    var ratio: Double = 0.5
}
struct PaneLayout {
    var rootID = "root"
    var nodes: [String: PaneNode] = [
        "root": PaneNode(id: "root", axis: "horizontal", first: "files-group", second: "document-group", ratio: 0.22),
        "files-group": PaneNode(id: "files-group", tabs: ["files"], active: "files"),
        "document-group": PaneNode(id: "document-group")
    ]
    func groups(_ id: String? = nil) -> [PaneNode] {
        guard let node = nodes[id ?? rootID] else { return [] }
        if let first = node.first, let second = node.second { return groups(first) + groups(second) }
        return [node]
    }
    mutating func activate(_ tab: String, in group: String) {
        guard var node = nodes[group], node.axis == nil else { return }
        if !node.tabs.contains(tab) { node.tabs.append(tab) }
        node.active = tab; nodes[group] = node
    }
    // Preserve node identities when a sibling takes over the freed rectangle.
    // Sweep unreachable nodes too, so retired drop targets cannot be reused.
    private mutating func collapse() {
        func live(_ id: String) -> String? {
            guard let node = nodes[id] else { return nil }
            guard let first = node.first, let second = node.second else { return node.tabs.isEmpty ? nil : id }
            let a = live(first), b = live(second)
            if let a, let b { nodes[id]?.first = a; nodes[id]?.second = b; return id }
            return a ?? b
        }
        if let surviving = live(rootID) { rootID = surviving }
        else { nodes[rootID] = PaneNode(id: rootID) }
        var kept: [String: PaneNode] = [:]
        func retain(_ id: String) {
            guard let node = nodes[id] else { return }
            kept[id] = node
            if let a = node.first, let b = node.second { retain(a); retain(b) }
        }
        retain(rootID); nodes = kept
    }
    mutating func remove(_ tab: String) {
        guard groups().contains(where: { $0.tabs.contains(tab) }) else { return }
        let keepDocument = tab != "files" && groups().contains { $0.tabs.contains("files") } && groups().filter { !$0.tabs.contains("files") }.allSatisfy { $0.tabs.allSatisfy { $0 == tab } }
        detach(tab); if !keepDocument { collapse() }
    }
    private mutating func detach(_ tab: String) {
        for group in groups() where group.tabs.contains(tab) {
            var changed = group
            changed.tabs.removeAll { $0 == tab }
            if changed.active == tab { changed.active = changed.tabs.last }
            nodes[group.id] = changed
        }
    }
    mutating func move(_ tab: String, to target: String, before: String? = nil) {
        guard let destination = groups().first(where: { $0.id == target }),
              groups().contains(where: { $0.tabs.contains(tab) }),
              before == nil || destination.tabs.contains(before!), before != tab else { return }
        detach(tab)
        guard var node = nodes[target] else { return }
        let index = before.flatMap { node.tabs.firstIndex(of: $0) } ?? node.tabs.count
        node.tabs.insert(tab, at: index); node.active = tab; nodes[target] = node
        collapse()
    }
    @discardableResult mutating func dock(_ tab: String, to target: String, edge: String, newID: String) -> String? {
        let retained = "retained-" + newID
        guard ["left", "right", "top", "bottom"].contains(edge), !newID.isEmpty,
              nodes[newID] == nil, nodes[retained] == nil,
              groups().contains(where: { $0.id == target }), groups().contains(where: { $0.tabs.contains(tab) }) else { return nil }
        detach(tab)
        var old = nodes[target]!; old.id = retained; nodes[retained] = old
        nodes[newID] = PaneNode(id: newID, tabs: [tab], active: tab)
        let leading = edge == "left" || edge == "top"
        nodes[target] = PaneNode(id: target, axis: edge == "left" || edge == "right" ? "horizontal" : "vertical",
                                 first: leading ? newID : retained, second: leading ? retained : newID)
        collapse()
        return newID
    }
    mutating func split(_ group: String, axis: String, newID: String) -> String? {
        guard var old = nodes[group], old.axis == nil else { return nil }
        let retained = "retained-" + newID
        old.id = retained
        nodes[retained] = old
        nodes[newID] = PaneNode(id: newID)
        nodes[group] = PaneNode(id: group, axis: axis, first: retained, second: newID)
        return retained
    }
    mutating func resize(_ id: String, ratio: Double) {
        guard ratio.isFinite, nodes[id]?.axis != nil else { return }
        nodes[id]?.ratio = min(0.9, max(0.1, ratio))
    }
}
