<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Give Swift the same split and tab operations as the browser

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

The tree stores tab identities, never text views. Branches have two child identities and a clamped proportion. Leaves remain valid when empty, making them stable drop targets.

<ui:copy id="simple-bcf63acd46-0">
import Foundation

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
    var nodes: [String: PaneNode] = [
        "root": PaneNode(id: "root", axis: "horizontal", first: "files-group", second: "document-group", ratio: 0.22),
        "files-group": PaneNode(id: "files-group", tabs: ["files"], active: "files"),
        "document-group": PaneNode(id: "document-group")
    ]
    func groups(_ id: String = "root") -> [PaneNode] {
        guard let node = nodes[id] else { return [] }
        if let first = node.first, let second = node.second { return groups(first) + groups(second) }
        return [node]
    }
    mutating func activate(_ tab: String, in group: String) {
        guard var node = nodes[group], node.axis == nil else { return }
        if !node.tabs.contains(tab) { node.tabs.append(tab) }
        node.active = tab; nodes[group] = node
    }
    mutating func remove(_ tab: String) {
        for group in groups() where group.tabs.contains(tab) {
            var changed = group
            changed.tabs.removeAll { $0 == tab }
            if changed.active == tab { changed.active = changed.tabs.last }
            nodes[group.id] = changed
        }
    }
</ui:copy>

Validate a move before removing membership. Splitting replaces the leaf position with a branch and moves its content to a new leaf; document identity is unchanged.

<ui:copy id="simple-bcf63acd46-1">
    mutating func move(_ tab: String, to target: String) {
        guard groups().contains(where: { $0.id == target }), groups().contains(where: { $0.tabs.contains(tab) }) else { return }
        remove(tab); activate(tab, in: target)
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
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/apple-simple/Sources/Layout.swift"><ui:paste select="#simple-bcf63acd46-0"/><ui:paste select="#simple-bcf63acd46-1"/></ui:file>

</ui:doc>
