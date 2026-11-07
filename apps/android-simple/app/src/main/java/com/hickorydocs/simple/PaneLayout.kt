package com.hickorydocs.simple

data class PaneNode(
    val id: String, val tabs: List<String> = emptyList(), val active: String? = null,
    val axis: String? = null, val first: String? = null, val second: String? = null, val ratio: Double = 0.5
)
data class PaneLayout(val nodes: Map<String, PaneNode> = mapOf(
    "root" to PaneNode("root", axis = "horizontal", first = "files-group", second = "document-group", ratio = 0.22),
    "files-group" to PaneNode("files-group", tabs = listOf("files"), active = "files"),
    "document-group" to PaneNode("document-group")
)) {
    fun groups(id: String = "root"): List<PaneNode> {
        val node = nodes[id] ?: return emptyList()
        return if (node.first != null && node.second != null) groups(node.first) + groups(node.second) else listOf(node)
    }
    fun activate(tab: String, group: String): PaneLayout {
        val node = nodes[group] ?: return this
        if (node.axis != null) return this
        return copy(nodes = nodes + (group to node.copy(tabs = if (tab in node.tabs) node.tabs else node.tabs + tab, active = tab)))
    }
    fun remove(tab: String): PaneLayout = copy(nodes = nodes.mapValues { (_, node) ->
        val tabs = node.tabs.filter { it != tab }
        node.copy(tabs = tabs, active = if (node.active == tab) tabs.lastOrNull() else node.active)
    })
    fun move(tab: String, target: String): PaneLayout {
        if (groups().none { it.id == target } || groups().none { tab in it.tabs }) return this
        return remove(tab).activate(tab, target)
    }
    fun split(group: String, axis: String, newID: String): PaneLayout {
        val node = nodes[group] ?: return this
        if (node.axis != null) return this
        val retained = "retained-$newID"
        return copy(nodes = nodes + mapOf(
            retained to node.copy(id = retained), newID to PaneNode(newID),
            group to PaneNode(group, axis = axis, first = retained, second = newID)
        ))
    }
    fun resize(id: String, ratio: Double): PaneLayout {
        val node = nodes[id] ?: return this
        if (node.axis == null || !ratio.isFinite()) return this
        return copy(nodes = nodes + (id to node.copy(ratio = ratio.coerceIn(0.1, 0.9))))
    }
}
