<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Implement the shared pane contract in Kotlin

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

A pane tree contains tab identities and proportions only. Returning a new layout makes structural changes observable without routing document keystrokes through the shell.

Docking validates source, target, direction and fresh IDs before removing anything. It retains the destination under a new identity and inserts the moved tab on the requested side of an equal split. Empty groups disappear after insertion, and each surviving sibling fills the freed rectangle. One empty root remains only when all tabs are closed; explicit Split can create a temporary empty destination. The outer twenty percent selects an edge, with left/right owning corners and the center joining existing tabs.

Use an optional destination tab identity to mean insert-before, and no anchor to mean append. Check the source and anchor before removing anything; resolve the anchor index in the remaining list. This keeps forward moves correct and obsolete or self-drops harmless without touching retained editor state.

<ui:copy id="simple-5953f13779-0">
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
    fun remove(tab: String): PaneLayout = if (groups().any { tab in it.tabs }) detach(tab).let { removed -> if (tab != "files" && groups().any { "files" in it.tabs } && groups().filter { "files" !in it.tabs }.all { group -> group.tabs.all { it == tab } }) removed else removed.collapse() } else this
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
</ui:copy>

Splits retain the content under a new leaf identity. Invalid moves and non-finite resize values leave the tree unchanged; proportions reserve space for both children.

<ui:copy id="simple-5953f13779-1">
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
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/PaneLayout.kt"><ui:paste select="#simple-5953f13779-0"/><ui:paste select="#simple-5953f13779-1"/></ui:file>

</ui:doc>
