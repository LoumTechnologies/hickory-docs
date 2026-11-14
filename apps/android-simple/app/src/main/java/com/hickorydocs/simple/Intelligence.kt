package com.hickorydocs.simple

import org.json.JSONObject
import org.json.JSONArray
import java.net.URI

enum class SemanticQuery(val method: String, val label: String) {
    Definition("textDocument/definition", "Go to definition"),
    References("textDocument/references", "Find usages"),
    TypeDefinition("textDocument/typeDefinition", "Go to type definition"),
    Implementation("textDocument/implementation", "Find implementations"),
    Declaration("textDocument/declaration", "Go to declaration"),
    Outline("textDocument/documentSymbol", "File outline"),
    Symbols("workspace/symbol", "Search workspace symbols"),
    Callers("textDocument/prepareCallHierarchy", "Find callers"),
    Callees("textDocument/prepareCallHierarchy", "Find callees")
}
data class SemanticLocation(val uri: String, val line: Int, val character: Int, val name: String? = null) {
    val path: String? get() = runCatching {
        val value = URI(uri)
        if (!uri.startsWith("hick:///") || value.scheme != "hick" || !value.authority.isNullOrEmpty() || value.query != null || value.fragment != null) return null
        val decoded = value.path ?: return null
        if (!decoded.startsWith("/")) return null
        val path = decoded.drop(1)
        if (path.isEmpty() || path.any { it == ':' || it == '\\' || it.code < 32 } || path.split('/').any { it.isEmpty() || it == "." || it == ".." }) null else path
    }.getOrNull()
    val label: String get() = (name?.let { "$it · " } ?: "") + "${path ?: uri}:${line + 1}:${character + 1}"
}
fun semanticLocations(result: Any, defaultURI: String? = null): List<SemanticLocation> {
    val locations = linkedMapOf<String, SemanticLocation>()
    fun visit(value: Any?, depth: Int) {
        if (depth > 64 || locations.size >= 10000) return
        if (value is JSONArray) { for (index in 0 until value.length()) visit(value.opt(index), depth + 1); return }
        val item = value as? JSONObject ?: return
        if (item.has("from") || item.has("to")) { visit(item.opt("from") ?: item.opt("to"), depth + 1); return }
        val target = item.optJSONObject("location") ?: item
        val uri = target.optString("uri").ifEmpty { target.optString("targetUri") }.ifEmpty { if (item.has("name") && !item.has("location")) defaultURI.orEmpty() else "" }
        val range = target.optJSONObject("selectionRange") ?: target.optJSONObject("range") ?: target.optJSONObject("targetSelectionRange") ?: target.optJSONObject("targetRange")
        val start = range?.optJSONObject("start")
        val line = start?.optInt("line", -1) ?: -1; val character = start?.optInt("character", -1) ?: -1
        if (uri.isNotEmpty() && line >= 0 && character >= 0) {
            val location = SemanticLocation(uri, line, character, item.optString("name").takeIf { it.isNotEmpty() })
            locations.putIfAbsent("$uri:$line:$character", location)
        }
        visit(item.opt("children"), depth + 1)
    }
    visit(result, 0); return locations.values.toList()
}
data class NavigationTrail(val entries: List<SemanticLocation> = emptyList(), val index: Int = -1) {
    val canBack get() = index > 0
    val canForward get() = index >= 0 && index + 1 < entries.size
    private fun same(a: SemanticLocation?, b: SemanticLocation) = a?.uri == b.uri && a.line == b.line && a.character == b.character
    fun record(from: SemanticLocation?, to: SemanticLocation): NavigationTrail {
        val values = entries.take(index + 1).toMutableList()
        if (from != null && !same(values.lastOrNull(), from)) values.add(from)
        if (!same(values.lastOrNull(), to)) values.add(to)
        val retained = values.takeLast(256); return NavigationTrail(retained, retained.lastIndex)
    }
    fun step(direction: Int, current: SemanticLocation?): Pair<NavigationTrail, SemanticLocation?> {
        val next = index + direction
        if (direction !in listOf(-1, 1) || next !in entries.indices) return this to null
        val values = entries.toMutableList()
        if (current != null && index in entries.indices && entries[index].uri == current.uri) values[index] = current
        return NavigationTrail(values, next) to values[next]
    }
}
data class WorkspaceProblem(val id: String, val path: String, val message: String, val severity: Int, val location: SemanticLocation?, val source: String?, val current: Boolean)
fun nextWorkspaceProblem(problems: List<WorkspaceProblem>, current: SemanticLocation?, direction: Int): WorkspaceProblem? {
    if (direction !in listOf(-1, 1)) return null
    val order = compareBy<SemanticLocation>({ it.uri }, { it.line }, { it.character })
    val located = problems.filter { it.current && it.location != null }.sortedWith { a, b -> order.compare(a.location!!, b.location!!) }
    if (located.isEmpty()) return null
    if (direction == 1) return current?.let { point -> located.firstOrNull { order.compare(it.location!!, point) > 0 } } ?: located.first()
    return current?.let { point -> located.lastOrNull { order.compare(it.location!!, point) < 0 } } ?: located.last()
}
data class WorkspaceSearchHit(val path: String, val line: Int, val text: String, val column: Int, val sourceHash: String?, val generatedBy: String?) {
    val character: Int? get() = searchCharacter(text, column)
}
data class WorkspaceSearchResult(val hits: List<WorkspaceSearchHit>, val truncated: Boolean)
fun searchCharacter(line: String, byteColumn: Int): Int? {
    if (byteColumn < 0) return null
    var bytes = 0; var units = 0
    while (units < line.length) {
        if (bytes == byteColumn) return units
        val scalar = line.codePointAt(units)
        bytes += when { scalar <= 0x7f -> 1; scalar <= 0x7ff -> 2; scalar <= 0xffff -> 3; else -> 4 }
        units += Character.charCount(scalar)
    }
    return if (bytes == byteColumn) units else null
}
