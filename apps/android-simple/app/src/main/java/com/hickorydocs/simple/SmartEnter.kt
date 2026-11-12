package com.hickorydocs.simple

import org.json.JSONObject

data class SimpleEnterPlan(val from: Int, val to: Int, val text: String, val caret: Int)
internal class EnterProfile(val json: JSONObject) {
    val id = json.getString("id")
    fun strings(key: String) = json.getJSONArray(key).let { values -> (0 until values.length()).map { values.getString(it) } }
    val names = strings("names"); val comments = strings("comments"); val multiline = strings("multiline")
    val blocks = json.getJSONArray("blocks").let { values -> (0 until values.length()).map { values.getJSONArray(it).let { pair -> require(pair.length() == 2 && pair.getString(0).isNotEmpty() && pair.getString(1).isNotEmpty()); pair.getString(0) to pair.getString(1) } } }
    val quotes = json.getString("quotes"); val tags = json.getBoolean("tags"); val voidTags = strings("voidTags")
    val regexLiterals = json.getBoolean("regexLiterals")
    val foldCase = json.getBoolean("foldCase"); val offside = json.getBoolean("offside")
    private val patterns = mutableMapOf<String, Regex>()
    fun matches(key: String, text: String): Boolean {
        val pattern = json.getString(key)
        if (pattern.isEmpty()) return false
        return patterns.getOrPut(key) { Regex(pattern, if (foldCase) setOf(RegexOption.IGNORE_CASE) else emptySet()) }.containsMatchIn(text)
    }
}
class SimpleEnterRules(data: String) {
    private val registry = JSONObject(data)
    val maxContext = registry.getInt("maxContext"); val maxLine = registry.getInt("maxLine")
    private val profiles = registry.getJSONArray("profiles").let { values -> (0 until values.length()).map { EnterProfile(values.getJSONObject(it)) } }
    internal fun profile(path: String): EnterProfile? {
        val name = path.replace('\\', '/').substringAfterLast('/').lowercase()
        val ext = name.substringAfterLast('.')
        return profiles.firstOrNull { ext in it.names }
    }
}
private fun enterOutdent(prefix: String, unit: String): String {
    if (prefix.endsWith('\t')) return prefix.dropLast(1)
    var result = prefix; var count = 0
    while (count < unit.length && result.endsWith(' ')) { result = result.dropLast(1); count++ }
    return result
}
private val enterExpressionCache = java.util.concurrent.ConcurrentHashMap<String, Regex>()
private fun enterExpression(pattern: String): Regex = enterExpressionCache.computeIfAbsent(pattern) { Regex(it) }
private data class EnterOpening(val token: String, val indent: String, val line: Int)
private data class EnterResult(val indent: String, val split: String = "", val literal: Boolean = false)
private class EnterScanner {
    var block = ""; var quoted = false
    val brackets = mutableListOf<EnterOpening>(); private val keywords = mutableListOf<String>()
    private var logical = ""; private var logicalIndent = ""; private var literal: String? = null; private var singleIndent: String? = null
    fun clean(row: String, profile: EnterProfile): String {
        val code = StringBuilder(); var index = 0; quoted = false
        while (index < row.length) {
            if (block.isNotEmpty()) {
                if (row.startsWith(block, index) && (index == 0 || row[index - 1] != '\\')) { index += block.length; block = "" } else index++
                code.append(' '); continue
            }
            val pair = profile.blocks.firstOrNull { row.startsWith(it.first, index) }
            val multi = profile.multiline.firstOrNull { row.startsWith(it, index) }
            if (pair != null || multi != null) { block = pair?.second ?: multi!!; index += pair?.first?.length ?: multi!!.length; code.append(' '); continue }
            if (profile.comments.any { row.startsWith(it, index) }) break
            if (profile.regexLiterals && row[index] == '/' && enterExpression("(?:^|\\b(?:return|throw|yield|case)|[=(:,!&|?;{}\\[>])\\s*$").containsMatchIn(code)) {
                index++; var characterClass = false; var closed = false
                while (index < row.length) {
                    val next = row[index++]
                    if (next == '\\') { index = minOf(row.length, index + 1); continue }
                    if (next == '[') characterClass = true
                    if (next == ']') characterClass = false
                    if (next == '/' && !characterClass) { closed = true; break }
                }
                if (!closed) { quoted = true; block = "/" }
                code.append('~'); continue
            }
            val char = row[index++]
            if (char in profile.quotes) {
                if (char == '\'' && profile.id == "rust" && !row.substring(index).contains('\'')) { code.append(char); continue }
                var closed = false
                while (index < row.length) {
                    val next = row[index++]
                    if (next == '\\') { index = minOf(row.length, index + 1); continue }
                    if (next == char) { closed = true; break }
                }
                if (!closed) { quoted = true; block = char.toString() }
                code.append('~'); continue
            }
            code.append(char)
        }
        return code.toString().trim()
    }
    fun line(row: String, profile: EnterProfile, number: Int, unit: String): EnterResult {
        val prefix = row.takeWhile { it == ' ' || it == '\t' }
        literal?.let { if (row.isBlank() || prefix.length > it.length) return EnterResult(prefix, literal = true) }
        literal = null
        val wasBlock = block.isNotEmpty(); val code = clean(row, profile); val startDepth = brackets.size
        if (wasBlock || block.isNotEmpty() || quoted || code.isEmpty()) return EnterResult(prefix, literal = true)
        if (startDepth == 0) { logical = code; logicalIndent = prefix }
        var base = prefix
        if (profile.matches("close", code)) base = keywords.removeLastOrNull() ?: enterOutdent(prefix, unit)
        val middle = profile.matches("middle", code)
        if (middle && !profile.offside && keywords.isNotEmpty()) base = keywords.last()
        if (profile.tags) {
            for (match in enterExpression("<(\\/?)([A-Za-z][\\w:.-]*)\\b[^>]*>").findAll(code)) {
                val tag = if (profile.foldCase) match.groupValues[2].lowercase() else match.groupValues[2]
                if (match.groupValues[1].isNotEmpty()) { if (brackets.lastOrNull()?.token == "</$tag>") base = brackets.removeLast().indent }
                else if (!match.value.endsWith("/>") && tag !in profile.voidTags) brackets.add(EnterOpening("</$tag>", base, number))
            }
        } else for (char in code) {
            if (char in "{[(") brackets.add(EnterOpening(char.toString(), base, number))
            val opening = "}])".indexOf(char)
            if (opening >= 0 && brackets.lastOrNull()?.token == "{[("[opening].toString()) base = brackets.removeLast().indent
        }
        brackets.lastOrNull()?.let { top ->
            val split = if (top.token.startsWith("</")) top.token else "}])"["{[(".indexOf(top.token)].toString()
            return EnterResult(if (top.line == number || base.length <= top.indent.length) top.indent + unit else base, split)
        }
        if (profile.matches("next", code)) {
            if (singleIndent == null) singleIndent = base
            return EnterResult(base + unit)
        }
        if (profile.matches("open", code) || middle) {
            if (!middle) keywords.add(base)
            return EnterResult(base + unit)
        }
        if (profile.matches("literal", code)) { literal = base; return EnterResult(base + unit) }
        if (profile.matches("after", logical)) base = enterOutdent(logicalIndent, unit)
        singleIndent?.let { base = it; singleIndent = null }
        return EnterResult(base)
    }
}
fun smartEnter(source: String, from: Int, to: Int, path: String, unit: String, mode: String, rules: SimpleEnterRules?): SimpleEnterPlan {
    val start = if (from == 0) 0 else source.lastIndexOf('\n', from - 1) + 1
    val prefix = source.substring(start, from).takeWhile { it == ' ' || it == '\t' }
    val eol = if (simpleNewlineText(source, from).startsWith("\r\n")) "\r\n" else "\n"
    fun fallback(indent: String) = SimpleEnterPlan(from, to, eol + indent, from + eol.length + indent.length)
    if (mode == "none") return fallback("")
    if (mode == "copy" || rules == null || from > rules.maxContext || from - start > rules.maxLine) return fallback(prefix)
    val rows = source.substring(0, from).split('\n'); val markdown = path.substringAfterLast('.', "").lowercase() in listOf("md", "markdown")
    var profile = rules.profile(path); var scanner = EnterScanner(); var fence = ""; var payload = ""
    var result = EnterResult(prefix, literal = true)
    if (!markdown && profile == null) return fallback(prefix)
    for ((line, raw) in rows.withIndex()) {
        val row = raw.removeSuffix("\r"); val trimmed = row.trim()
        if (row.length > rules.maxLine) return fallback(prefix)
        if (markdown) {
            val marker = enterExpression("^(`{3,}|~{3,})([^ ]*)").find(trimmed)
            if (fence.isNotEmpty()) {
                if (trimmed.startsWith(fence) && enterExpression("^(`+|~+)\\s*$").containsMatchIn(trimmed)) { fence = ""; profile = null; scanner = EnterScanner(); continue }
            } else if (payload.isEmpty() && marker != null) { fence = marker.groupValues[1]; profile = rules.profile(marker.groupValues[2]); scanner = EnterScanner(); continue }
            else {
                val open = enterExpression("^<[\\w.-]+:(file|copy|cut|exec)\\b[^>]*>").find(trimmed)
                if (open != null) {
                    payload = open.groupValues[1]; val hint = enterExpression("\\b(?:path|language|lang)\\s*=\\s*[\"']([^\"']+)[\"']").find(open.value)
                    profile = hint?.let { rules.profile(it.groupValues[1]) }; scanner = EnterScanner(); continue
                }
                if (payload.isNotEmpty() && trimmed.startsWith("</") && trimmed.contains(":$payload>")) { payload = ""; profile = null; scanner = EnterScanner(); continue }
            }
        }
        profile?.let { result = scanner.line(row, it, line, unit) }
    }
    if (profile == null || result.literal) return fallback(prefix)
    val end = source.indexOf('\n', to).let { if (it < 0) source.length else it }
    val suffix = source.substring(to, end).removeSuffix("\r"); val whitespace = suffix.takeWhile { it == ' ' || it == '\t' }.length
    val top = scanner.brackets.lastOrNull()
    if (result.split.isNotEmpty() && (if (profile.foldCase) suffix.trimStart().lowercase() else suffix.trimStart()).startsWith(result.split) && top != null) {
        val openingHere = top.line == rows.size - 1
        val inserted = if (openingHere) eol + result.indent + eol + top.indent else eol + top.indent
        return SimpleEnterPlan(from, to + whitespace, inserted, from + eol.length + if (openingHere) result.indent.length else top.indent.length)
    }
    return fallback(result.indent)
}
