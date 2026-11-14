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
