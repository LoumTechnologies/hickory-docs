package com.hickorydocs.simple

fun dropEdge(x: Double, y: Double): String? {
    if (!x.isFinite() || !y.isFinite() || x !in 0.0..1.0 || y !in 0.0..1.0) return null
    return if (x < .2) "left" else if (x > .8) "right" else if (y < .2) "top" else if (y > .8) "bottom" else null
}
data class PaneNode(
    val id: String, val tabs: List<String> = emptyList(), val active: String? = null,
    val axis: String? = null, val first: String? = null, val second: String? = null, val ratio: Double = 0.5
)
data class PaneLayout(val rootID: String = "root", val nodes: Map<String, PaneNode> = mapOf(
    "root" to PaneNode("root", axis = "horizontal", first = "files-group", second = "document-group", ratio = 0.22),
    "files-group" to PaneNode("files-group", tabs = listOf("files"), active = "files"),
    "document-group" to PaneNode("document-group")
)) {
    fun groups(id: String = rootID): List<PaneNode> {
        val node = nodes[id] ?: return emptyList()
        return if (node.first != null && node.second != null) groups(node.first) + groups(node.second) else listOf(node)
    }
    fun activate(tab: String, group: String): PaneLayout {
        val node = nodes[group] ?: return this
        if (node.axis != null) return this
        return copy(nodes = nodes + (group to node.copy(tabs = if (tab in node.tabs) node.tabs else node.tabs + tab, active = tab)))
    }
    // Compact after insertion, never between detach and insert: the destination
    // may be the source itself. Keep the survivor's identity and ratio.
    private fun collapse(): PaneLayout {
        val kept = mutableMapOf<String, PaneNode>()
        fun live(id: String): String? {
            val node = nodes.getValue(id)
            if (node.first == null || node.second == null) {
                if (node.tabs.isEmpty()) return null
                kept[id] = node; return id
            }
            val a = live(node.first); val b = live(node.second)
            if (a != null && b != null) { kept[id] = node.copy(first = a, second = b); return id }
            return a ?: b
        }
        val root = live(rootID) ?: rootID.also { kept[it] = PaneNode(it) }
        return copy(rootID = root, nodes = kept)
    }
    fun remove(tab: String): PaneLayout = if (groups().any { tab in it.tabs }) detach(tab).collapse() else this
    private fun detach(tab: String): PaneLayout = copy(nodes = nodes.mapValues { (_, node) ->
        val tabs = node.tabs.filter { it != tab }
        node.copy(tabs = tabs, active = if (node.active == tab) tabs.lastOrNull() else node.active)
    })
    fun move(tab: String, target: String, before: String? = null): PaneLayout {
        val destination = groups().find { it.id == target } ?: return this
        if (groups().none { tab in it.tabs } || (before != null && before !in destination.tabs) || before == tab) return this
        val removed = detach(tab)
        val node = removed.nodes.getValue(target)
        val tabs = node.tabs.toMutableList()
        tabs.add(if (before == null) tabs.size else tabs.indexOf(before), tab)
        return removed.copy(nodes = removed.nodes + (target to node.copy(tabs = tabs, active = tab))).collapse()
    }
    fun dock(tab: String, target: String, edge: String, newID: String): PaneLayout {
        val retained = "retained-$newID"
        if (edge !in listOf("left", "right", "top", "bottom") || newID.isEmpty() || newID in nodes || retained in nodes ||
            groups().none { it.id == target } || groups().none { tab in it.tabs }) return this
        val removed = detach(tab)
        val old = removed.nodes.getValue(target)
        val leading = edge == "left" || edge == "top"
        return removed.copy(nodes = removed.nodes + mapOf(
            retained to old.copy(id = retained), newID to PaneNode(newID, tabs = listOf(tab), active = tab),
            target to PaneNode(target, axis = if (edge == "left" || edge == "right") "horizontal" else "vertical",
                first = if (leading) newID else retained, second = if (leading) retained else newID)
        )).collapse()
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
