package com.hickorydocs.simple

data class InlineMark(val kind: String, val from: Int, val to: Int, val level: Int = 0)
data class InlineCell(val line: Int, val column: Int, val from: Int, val to: Int, val value: String, val format: String = "markdown", val delimiter: String = ",")
data class InlineEquation(val line: Int, val from: Int, val to: Int, val body: String, val renderedFrom: Int = from, val renderedTo: Int = to, val displayBlock: Boolean = false)
data class InlineDiagram(val line: Int, val from: Int, val to: Int, val body: String, val renderer: String)
data class InlineLink(val from: Int, val to: Int, val href: String, val sourceFrom: Int, val sourceTo: Int)
fun InlineSnapshot.linkAtCaret(currentSource: String, from: Int, to: Int): InlineLink? {
    if (source != currentSource) return null
    return links.firstOrNull { from in it.sourceFrom until it.sourceTo && to <= it.sourceTo }
}
data class WorkspaceLinkTarget(val path: String, val line: Int?)
fun workspaceLinkTarget(href: String, source: String): WorkspaceLinkTarget? {
    val parts = href.split('#', limit = 2)
    val raw = parts.first()
    val path = runCatching { java.net.URLDecoder.decode(raw.replace("+", "%2B"), "UTF-8") }.getOrNull() ?: return null
    if (path.isEmpty() || path.startsWith('/') || path.any { it in ":\\\r\n\u0000" }) return null
    val resolved = source.split('/').dropLast(1).toMutableList()
    for (part in path.split('/')) {
        when (part) {
            ".." -> if (resolved.isEmpty()) return null else resolved.removeAt(resolved.lastIndex)
            "", "." -> Unit
            else -> resolved.add(part)
        }
    }
    if (resolved.isEmpty()) return null
    val line = parts.getOrNull(1)?.takeIf { it.startsWith("L") }?.drop(1)?.toIntOrNull()?.takeIf { it > 0 }?.minus(1)
    return WorkspaceLinkTarget(resolved.joinToString("/"), line)
}
data class InlineSnapshot(val source: String, val lineStarts: List<Int> = listOf(0), val marks: List<InlineMark> = emptyList(), val cells: List<InlineCell> = emptyList(), val equations: List<InlineEquation> = emptyList(), val debug: InlineDebugContext = InlineDebugContext(), val diagrams: List<InlineDiagram> = emptyList(), val gridOverrides: List<InlineTable> = emptyList(), val links: List<InlineLink> = emptyList())
// Keep source offsets authoritative while the field displays a grid-sized placeholder.
data class InlineTable(val from: Int, val to: Int, val rows: List<List<InlineCell>>, val visibleRows: Int? = null) {
    val displayRows: Int get() = visibleRows ?: maxOf(22, minOf(32, rows.size * 2 + 10))
}
fun tablePresentation(snapshot: InlineSnapshot, source: String): InlineSnapshot {
    if (snapshot.source == source) return snapshot
    val old = snapshot.source
    var prefix = 0; var suffix = 0
    while (prefix < minOf(old.length, source.length) && old[prefix] == source[prefix]) prefix++
    while (suffix < minOf(old.length, source.length) - prefix && old[old.length - suffix - 1] == source[source.length - suffix - 1]) suffix++
    val end = old.length - suffix; val delta = source.length - old.length
    val starts = listOf(0) + source.indices.filter { source[it] == '\n' }.map { it + 1 }
    val cells = snapshot.cells.mapNotNull { original ->
        val cell = when {
            original.to <= prefix -> original
            original.from >= end -> original.copy(from = original.from + delta, to = original.to + delta)
            original.from <= prefix && original.to >= end && !source.substring(prefix, source.length - suffix).contains('\n') -> original.copy(to = original.to + delta)
            else -> return@mapNotNull null
        }
        if (cell.from < 0 || cell.to < cell.from || cell.to > source.length) return@mapNotNull null
        val raw = source.substring(cell.from, cell.to)
        val value = if (cell.format == "csv" && raw.startsWith('"') && raw.endsWith('"') && raw.length >= 2) raw.substring(1, raw.length - 1).replace("\"\"", "\"") else raw.replace("\\|", "|").replace("\\\\", "\\")
        cell.copy(line = inlineLineAt(cell.from, starts), value = value)
    }
    val diagrams = snapshot.diagrams.mapNotNull { diagram ->
        val mapped = when { diagram.to <= prefix -> diagram; diagram.from >= end -> diagram.copy(from = diagram.from + delta, to = diagram.to + delta); else -> null }
        mapped?.copy(line = inlineLineAt(mapped.from, starts))
    }
    val changed = old.substring(prefix, end) + source.substring(prefix, source.length - suffix)
    val grids = inlineTables(snapshot).filter { table ->
        val afterNewline = prefix == end && prefix >= table.to && prefix - table.to <= 2 && old.substring(table.to, prefix).all { it == '\r' || it == '\n' }
        prefix >= table.from && (end <= table.to || afterNewline) && end >= table.from && changed.any { char -> char in "\n,|" }
    }
        .map { it.copy(to = it.to + delta) }.filter { it.to > it.from }
    return InlineSnapshot(source, lineStarts = starts, cells = cells, diagrams = diagrams, gridOverrides = grids)
}
fun inlineTables(snapshot: InlineSnapshot): List<InlineTable> {
    val lines = snapshot.cells.filter { it.format == "markdown" }.groupBy { it.line }
    val consumed = mutableSetOf<Int>(); val result = mutableListOf<InlineTable>()
    for (line in lines.keys.sorted()) {
        if (line in consumed || line + 1 >= snapshot.lineStarts.size || lines.containsKey(line + 1)) continue
        val separator = snapshot.source.substring(snapshot.lineStarts[line + 1]).substringBefore('\n').trim()
        if (separator.isEmpty() || '-' !in separator || separator.any { it !in "|:- \t" }) continue
        val rows = mutableListOf(lines.getValue(line)); var last = line + 1; var next = line + 2
        consumed.add(line)
        while (lines.containsKey(next)) { rows.add(lines.getValue(next)); consumed.add(next); last = next; next++ }
        val end = snapshot.source.indexOf('\n', snapshot.lineStarts[last]).let { if (it < 0) snapshot.source.length else it }
        val previous = if (line > 0) snapshot.source.substring(snapshot.lineStarts[line - 1], snapshot.lineStarts[line]).trim() else ""
        val start = if (previous.startsWith("<" + "!-- hickory-table-formats: ") && previous.endsWith(" --" + ">")) snapshot.lineStarts[line - 1] else snapshot.lineStarts[line]
        result.add(InlineTable(start, if (end > 0 && snapshot.source[end - 1] == '\r') end - 1 else end, rows))
    }
    val csv = snapshot.cells.filter { it.format == "csv" }.sortedBy { it.from }
    val groups = mutableListOf<MutableList<InlineCell>>()
    for (cell in csv) {
        val last = groups.lastOrNull()?.lastOrNull()
        if (last != null && snapshot.source.substring(last.to, cell.from).all { it == '\r' || it == '\n' || it in cell.delimiter }) groups.last().add(cell)
        else groups.add(mutableListOf(cell))
    }
    for (cells in groups) {
        val rows = mutableListOf<MutableList<InlineCell>>()
        for (cell in cells) { if (cell.column == 0) rows.add(mutableListOf()); rows.last().add(cell) }
        result.add(InlineTable(cells.first().from, cells.last().to, rows))
    }
    return (result.filter { table -> snapshot.gridOverrides.none { it.from < table.to && table.from < it.to } } + snapshot.gridOverrides).sortedBy { it.from }
}
fun inlineCellText(value: String, format: String = "markdown", delimiter: String = ","): String {
    if (format == "csv") return if (value.contains(delimiter) || value.contains('\n') || value.contains('\r') || value.contains('"')) "\"" + value.replace("\"", "\"\"") + "\"" else value
    return value.replace("\\", "\\\\").replace("|", "\\|").replace("\n", "<br>")
}
fun inlineCells(row: String, line: Int, offset: Int): List<InlineCell> {
    val boundaries = mutableListOf(-1); var codeRun = 0; var index = 0
    while (index < row.length) {
        val char = row[index]
        if (char == '\\' && codeRun == 0) { index += minOf(2, row.length - index); continue }
        if (char == '`') {
            var end = index + 1
            while (end < row.length && row[end] == '`') end++
            val run = end - index
            if (codeRun == 0) { if (row.substring(end).contains("`".repeat(run))) codeRun = run }
            else if (codeRun == run) codeRun = 0
            index = end; continue
        }
        if (char == '|' && codeRun == 0) boundaries.add(index)
        index++
    }
    boundaries.add(row.length); val result = mutableListOf<InlineCell>()
    for (index in 0 until boundaries.size - 1) {
        val from = boundaries[index] + 1; val to = boundaries[index + 1]; val raw = row.substring(from, to)
        if ((index == 0 || index == boundaries.size - 2) && raw.isBlank()) continue
        val start = from + raw.takeWhile { it == ' ' || it == '\t' }.length; val value = raw.trim()
        result.add(InlineCell(line, result.size, offset + start, offset + start + value.length, value.replace("\\|", "|").replace("\\\\", "\\")))
    }
    return result
}
fun inlineCSVCells(body: String, line: Int, offset: Int, delimiter: String): List<InlineCell> {
    if (delimiter.length != 1) return emptyList()
    var index = if (body.startsWith("\r\n")) 2 else if (body.startsWith("\n")) 1 else 0
    var row = line + if (index > 0) 1 else 0; var start = index; var startLine = row; var column = 0; var quoted = false
    val result = mutableListOf<InlineCell>()
    while (index <= body.length) {
        val char = body.getOrNull(index)
        if (char == '"') { if (quoted && body.getOrNull(index + 1) == '"') { index += 2; continue }; quoted = !quoted }
        val newline = !quoted && (char == '\n' || char == '\r')
        if (!quoted && (char == delimiter[0] || newline || index == body.length)) {
            if (start < body.length || char == delimiter[0] || index == body.length && body.getOrNull(index - 1) == delimiter[0]) {
                val raw = body.substring(start, index)
                val value = if (raw.startsWith('"') && raw.endsWith('"') && raw.length >= 2) raw.substring(1, raw.length - 1).replace("\"\"", "\"") else raw
                result.add(InlineCell(startLine, column, offset + start, offset + index, value, "csv", delimiter))
            }
            if (newline) { if (char == '\r' && body.getOrNull(index + 1) == '\n') index++; row++; column = 0 } else column++
            start = index + 1; startLine = row
        } else if (char == '\n') row++
        index++
    }
    return if (quoted) emptyList() else result
}
// Candidate discovery belongs to the background source snapshot, not each draw.
data class InlineDebugContext(
    val code: Map<Int, String> = emptyMap(),
    val launch: Map<Int, String> = emptyMap(),
    val recordedClosing: Map<Int, Int> = emptyMap(),
    val recordedPatches: Set<Int> = emptySet(),
    val declaredPatches: Set<Int> = emptySet(),
    val recordedPatchLines: Map<Int, Int> = emptyMap(),
)
// Patch prefixes describe old and new hunk counts, independent of code indentation.
data class SimplePatchHunk(val header: Int, val end: Int, val changed: List<Int>)
fun simplePatchHunks(rows: List<String>, first: Int, last: Int): List<SimplePatchHunk> {
    val header = Regex("""^@@ -\d+(?:,(\d+))? \+\d+(?:,(\d+))? @@""")
    val result = mutableListOf<SimplePatchHunk>()
    var index = first
    while (index < last) {
        val match = header.find(rows[index])
        if (match == null) { index++; continue }
        var old = match.groups[1]?.value?.let { it.toLongOrNull() ?: Long.MAX_VALUE } ?: 1
        var new = match.groups[2]?.value?.let { it.toLongOrNull() ?: Long.MAX_VALUE } ?: 1
        val changed = mutableListOf<Int>()
        var body = index + 1
        while (body < last) {
            val row = rows[body].removeSuffix("\r")
            when {
                row == "\\ No newline at end of file" -> Unit
                row.startsWith('-') && old > 0 -> { old--; changed.add(body) }
                row.startsWith('+') && new > 0 -> { new--; changed.add(body) }
                row.startsWith(' ') && old > 0 && new > 0 -> { old--; new-- }
                else -> break
            }
            body++
        }
        if (old == 0L && new == 0L && body > index + 1) {
            result.add(SimplePatchHunk(index, body - 1, changed)); index = body
        } else index++
    }
    return result
}
// Recorded cards suggest complete pathless fences and declared patch openers.
// Only the engine can associate a declaration with a verified construction.
fun recordedFenceDebugContext(source: String): InlineDebugContext {
    if (source.length > 200_000) return InlineDebugContext()
    val marker = Regex("""^ {0,3}(`{3,}|~{3,})(.*)$""")
    val languages = listOf(
        "python" to setOf("py", "python", "python3"), "javascript" to setOf("js", "javascript", "node"),
        "typescript" to setOf("ts", "typescript"), "rust" to setOf("rs", "rust"), "c" to setOf("c"),
        "cpp" to setOf("cpp", "c++", "cxx"), "go" to setOf("go", "golang"),
        "csharp" to setOf("cs", "csharp", "c#"), "ruby" to setOf("rb", "ruby"),
    )
    val launch = mutableMapOf<Int, String>()
    val code = mutableMapOf<Int, String>()
    val closing = mutableMapOf<Int, Int>()
    val patches = mutableSetOf<Int>()
    val declared = mutableSetOf<Int>()
    val changed = mutableMapOf<Int, Int>()
    val patchTag = Regex("""^ {0,3}<([\w.-]+):patch(?=[\s/>]|$)""")
    val patchFence = Regex("""^patch\s+.*[\w.-]+\s*=\s*["']""")
    var opening: MatchResult? = null
    var first = 0
    var tag: String? = null
    var tagStart = 0
    val rows = source.split('\n')
    fun address(first: Int, last: Int) {
        for (hunk in simplePatchHunks(rows, first + 1, last)) for (line in hunk.changed) changed[line] = first
    }
    for ((line, row) in rows.withIndex()) {
        val authored = if (line == 0) row.removePrefix("\uFEFF") else row
        val openTag = tag
        if (openTag != null) {
            if (openTag in authored) { address(tagStart, line); tag = null }
            continue
        }
        val declaration = if (opening == null) patchTag.find(authored) else null
        if (declaration != null) {
            declared.add(line)
            val close = "</${declaration.groupValues[1]}:patch>"
            if (close !in authored && !authored.trimEnd().endsWith("/>")) { tag = close; tagStart = line }
            continue
        }
        val match = marker.matchEntire(authored.removeSuffix("\r")) ?: continue
        val held = opening
        if (held == null) { opening = match; first = line; continue }
        val original = held.groupValues[1]
        val close = match.groupValues[1]
        if (original.first() != close.first() || original.length > close.length || match.groupValues[2].isNotBlank()) continue
        opening = null
        val info = held.groupValues[2].trim()
        val hint = info.lowercase()
        val body = (first + 1 until line).filter { rows[it].isNotBlank() }
        if (body.isEmpty()) continue
        if (hint == "diff" || hint == "patch") { patches.add(first); address(first, line); continue }
        if (patchFence.containsMatchIn(info)) { declared.add(first); address(first, line); continue }
        val language = languages.firstOrNull { hint in it.second }?.first ?: continue
        launch[first] = language
        closing[first] = line
        for (codeLine in body) code[codeLine] = language
    }
    return InlineDebugContext(code, launch, closing, patches, declared, changed)
}
fun inlineDebugContext(source: String, path: String): InlineDebugContext {
    if (source.length > 200_000) return InlineDebugContext()
    val supported = listOf("py", "js", "ts", "jsx", "tsx", "rs", "go", "c", "cpp", "java", "cs")
    val rows = source.split('\n'); val code = mutableMapOf<Int, String>(); val launch = mutableMapOf<Int, String>()
    if (path.substringAfterLast('.').lowercase() in supported) {
        launch[0] = path
        rows.forEachIndexed { index, row -> if (row.isNotBlank()) code[index] = path }
        return InlineDebugContext(code, launch)
    }
    if (path.substringAfterLast('.').lowercase() !in listOf("md", "markdown")) return InlineDebugContext()
    var fence = ""; var mode = ""; var target: String? = null; var oldLines = 0; var newLines = 0
    var literalOpening: Int? = null
    val literalLanguages = setOf("python", "py", "python3", "javascript", "js", "node", "typescript", "ts", "rust", "rs", "c", "cpp", "c++", "cxx", "go", "golang", "csharp", "cs", "c#", "ruby", "rb")
    val opening = Regex("""^ {0,3}<[\w.-]+:(file|patch|copy|cut|exec|entry|diagram|math|table)\b[^>]*>""")
    val pathHint = Regex("""\bpath\s*=\s*["']([^"']+)["']""")
    val hunk = Regex("""^@@ -\d+(?:,(\d+))? \+\d+(?:,(\d+))? @@""")
    for ((index, row) in rows.withIndex()) {
        val trimmed = row.trim()
        if (fence.isNotEmpty()) {
            if (trimmed.length >= fence.length && trimmed.all { it == fence[0] }) {
                literalOpening?.let { start ->
                    val body = (start + 1 until index).filter { rows[it].isNotBlank() }
                    if (body.isNotEmpty()) {
                        val address = "fence:$start"
                        launch[start] = address
                        body.forEach { code[it] = address }
                    }
                }
                literalOpening = null; fence = ""; mode = ""; target = null; continue
            }
        } else if (mode.isNotEmpty()) {
            if (trimmed.contains("</") && trimmed.contains(":" + mode + ">")) { mode = ""; target = null; continue }
        } else {
            val marker = Regex("""^ {0,3}(`{3,}|~{3,})""").find(row)
            val declaration: String
            if (marker != null) {
                fence = marker.value.trim(); declaration = row.substring(marker.range.last + 1)
                mode = if (declaration.trim().substringBefore(' ') == "patch") "patch" else "file"
                if (declaration.trim().lowercase() in literalLanguages) literalOpening = index
            }
            else { val match = opening.find(row) ?: continue; mode = match.groupValues[1]; declaration = match.value }
            oldLines = 0; newLines = 0; target = null
            if (mode == "file") {
                val path = pathHint.find(declaration)?.groupValues?.get(1)
                target = path?.takeIf { it.substringAfterLast('.').lowercase() in supported }
                target?.let { launch[index] = it }
            }
            continue
        }
        if (mode == "file" && trimmed.isNotEmpty()) target?.let { code[index] = it }
        if (mode == "patch") {
            if (oldLines > 0 || newLines > 0) {
                if (row.startsWith('+') && newLines > 0) { newLines--; if (row.drop(1).isNotBlank()) target?.let { code[index] = it } }
                else if (row.startsWith('-') && oldLines > 0) oldLines--
                else if (row.startsWith(' ') && oldLines > 0 && newLines > 0) {
                    oldLines--; newLines--
                    if (row.drop(1).isNotBlank()) target?.let { code[index] = it }
                }
            } else if (row.startsWith("+++ ")) {
                val path = row.drop(4).substringBefore('\t').removePrefix("b/")
                target = path.takeIf { it.substringAfterLast('.').lowercase() in supported }
                target?.let { launch[index] = it }
            } else hunk.find(row)?.let { match -> oldLines = match.groupValues[1].ifEmpty { "1" }.toIntOrNull() ?: 0; newLines = match.groupValues[2].ifEmpty { "1" }.toIntOrNull() ?: 0 }
        }
    }
    return InlineDebugContext(code, launch)
}
fun inlineLaunchEnd(source: String, header: Int): Int {
    val rows = source.split('\n')
    val tag = rows.getOrNull(header)?.trim().orEmpty()
    val patch = tag.startsWith("+++ ")
    val file = tag.startsWith('<') && !patch
    for (line in (header + 1)..rows.lastIndex) {
        val text = rows[line].trim()
        if ((file || patch) && text.startsWith("</") && text.contains(if (patch) ":patch>" else ":file>")) return line
        val marker = text.firstOrNull()
        if (!file && text.length >= 3 && (marker == '`' || marker == '~') &&
            (patch || marker == tag.firstOrNull()) && text.all { it == marker }) return line
    }
    return maxOf(header, rows.lastIndex - if (source.endsWith('\n')) 1 else 0)
}
fun inlineDebugPath(source: String, path: String, line: Int, launch: Boolean = false): String? {
    val context = inlineDebugContext(source, path)
    return if (launch) context.launch[line] else context.code[line]
}
fun inlineLineAt(offset: Int, starts: List<Int>): Int { val found = starts.binarySearch(offset); return if (found >= 0) found else maxOf(0, -found - 2) }
fun inlineSyntaxMarks(source: String, path: String): List<InlineMark> {
    val markdown = path.substringAfterLast('.').lowercase() in listOf("md", "markdown")
    val known = setOf("python", "py", "javascript", "js", "typescript", "ts", "jsx", "tsx", "rust", "rs", "go", "c", "cpp", "java", "cs", "kotlin", "kt", "swift", "ruby", "rb", "sh", "bash", "json", "yaml", "yml", "toml", "sql", "css", "html", "xml", "php")
    val keywords = "and as assert async await break case catch class const continue def default del do elif else enum except export extends false False finally fn for from fun func function global if impl import in interface is lambda let match module mut new nil None not null or package pass private public raise return self static struct super switch this throw trait true True try type typeof use val var void when while with yield".split(' ').toSet()
    val open = Regex("""^\s*<[\w.-]+:(file|copy|cut|exec|container|entry|patch|diagram|math|table)\b[^>]*>""")
    val hint = Regex("""\b(language|path)\s*=\s*["']([^"']+)["']""")
    val result = mutableListOf<InlineMark>()
    var fence = ""; var payload = ""; var language = if (markdown) "" else path.substringAfterLast('.').lowercase(); var offset = 0
    for (row in source.split('\n')) {
        val start = offset; offset += row.length + 1; val trimmed = row.trim()
        if (markdown) {
            if (fence.isNotEmpty()) {
                if (trimmed.length >= fence.length && trimmed.all { it == fence[0] }) { fence = ""; language = ""; continue }
            } else if (payload.isNotEmpty()) {
                if (trimmed.contains("</") && trimmed.contains(":" + payload + ">")) { payload = ""; language = ""; continue }
            } else {
                val marker = Regex("""^ {0,3}(`{3,}|~{3,})""").find(row)
                if (marker != null) { fence = marker.value.trim(); language = row.substring(marker.range.last + 1).trim().substringBefore(' ').lowercase(); continue }
                val match = open.find(row)
                if (match != null) {
                    payload = match.groupValues[1]; language = if (payload in listOf("exec", "container")) "sh" else ""
                    for (item in hint.findAll(match.value)) {
                        val value = item.groupValues[2].lowercase()
                        if (item.groupValues[1] == "language") { language = value; break }
                        language = value.substringAfterLast('.', "")
                    }
                    if (trimmed.endsWith("/>") || trimmed.contains("</")) { payload = ""; language = "" }
                }
                continue
            }
        }
        if (language !in known) continue
        val hashComments = language in listOf("python", "py", "sh", "bash", "ruby", "rb", "yaml", "yml", "toml")
        val lexer = Regex((if (hashComments) """#[^\n]*|""" else """//[^\n]*|""") + "\"(?:\\\\.|[^\"\\\\])*\"|'(?:\\\\.|[^'\\\\])*'|`(?:\\\\.|[^`\\\\])*`|\\b[0-9]+(?:\\.[0-9]+)?\\b|\\b[A-Za-z_][A-Za-z_0-9]*\\b")
        for (match in lexer.findAll(row)) {
            val token = match.value
            val kind = when { token.startsWith('#') || token.startsWith("//") -> "comment"; token.startsWith('"') || token.startsWith('\'') || token.startsWith('`') -> "string"; token.first().isDigit() -> "number"; token in keywords -> "keyword"; else -> continue }
            result.add(InlineMark("syntax-" + kind, start + match.range.first, start + match.range.last + 1))
        }
    }
    return result
}
fun inlineDiagrams(source: String): List<InlineDiagram> {
    val open = Regex("""^ {0,3}<[\w.-]+:(file|copy|cut|exec|container|entry|patch|diagram|math|table|expect)\b[^>]*>""")
    val renderer = Regex("""\brenderer\s*=\s*["']([^"']+)["']""")
    val result = mutableListOf<InlineDiagram>()
    var offset = 0; var fence = ""; var payload = ""; var bodyStart = 0; var blockStart = 0; var blockLine = 0; var engine = "mermaid"; var inExec = false; var renderable = false
    for ((line, row) in source.split('\n').withIndex()) {
        val start = offset; offset += row.length + 1; val trimmed = row.trim()
        if (fence.isNotEmpty()) { if (trimmed.length >= fence.length && trimmed.all { it == fence[0] }) fence = ""; continue }
        if (payload.isEmpty()) {
            if (inExec && trimmed.startsWith("</") && trimmed.contains(":exec>")) { inExec = false; continue }
            val marker = Regex("""^ {0,3}(`{3,}|~{3,})""").find(row)
            if (!inExec && marker != null) { fence = marker.value.trim(); continue }
            val match = open.find(row) ?: continue
            payload = match.groupValues[1]
            if (payload == "exec") { inExec = !trimmed.endsWith("/>"); payload = ""; continue }
            renderable = payload == "expect" || payload == "diagram" && !inExec
            blockStart = start; blockLine = line; bodyStart = start + match.range.last + 1
            engine = renderer.find(match.value)?.groupValues?.get(1) ?: "mermaid"
            if (payload == "expect") engine = "terminal"
            if (trimmed.endsWith("/>")) {
                if (payload == "expect") result.add(InlineDiagram(line, start, start + row.length, "", "terminal"))
                payload = ""; continue
            }
        }
        val close = row.indexOf("</")
        if (close >= 0 && row.substring(close).contains(":" + payload + ">")) {
            val bodyEnd = start + close
            if (renderable && bodyEnd >= bodyStart) {
                val body = source.substring(bodyStart, bodyEnd)
                result.add(InlineDiagram(blockLine, blockStart, start + row.length, if (payload == "expect") body.removePrefix("\r\n").removePrefix("\n") else body, engine))
            }
            payload = ""
        }
    }
    return result
}
fun computeInlineMarkup(source: String, path: String, recordedMessage: Boolean = false): InlineSnapshot {
    val starts = mutableListOf(0); source.forEachIndexed { index, char -> if (char == '\n') starts.add(index + 1) }
    val debug = if (recordedMessage) recordedFenceDebugContext(source) else inlineDebugContext(source, path)
    if (path.substringAfterLast('.', "").lowercase() == "csv" && source.length <= 200_000) return InlineSnapshot(source, lineStarts = starts, cells = if (source.isEmpty()) listOf(InlineCell(0, 0, 0, 0, "", "csv")) else inlineCSVCells(source, 0, 0, ","), debug = debug)
    if (path.substringAfterLast('.', "").lowercase() !in listOf("md", "markdown") || source.length > 200_000) return InlineSnapshot(source, lineStarts = starts, debug = debug)
    val rows = source.split('\n'); val marks = mutableListOf<InlineMark>(); val cells = mutableListOf<InlineCell>(); val equations = mutableListOf<InlineEquation>()
    val links = mutableListOf<InlineLink>()
    val linkPattern = Regex("""(?<![!\\])\[([^\]\n]+)\]\(([^\s()]+)\)""")
    val separator = Regex("^\\s*\\|?\\s*:?-{3,}:?\\s*(\\|\\s*:?-{3,}:?\\s*)+\\|?\\s*$")
    val heading = Regex("^(#{1,6})\\s+")
    val styles = listOf("code" to Regex("`[^`\\n]+`"), "strong" to Regex("\\*\\*[^*\\n]+\\*\\*"), "em" to Regex("(?<!\\*)\\*[^*\\n]+\\*(?!\\*)"), "math" to Regex("(?<!\\\\)\\$([^$\\n]+)\\$"))
    val opening = Regex("^\\s*<[A-Za-z_][\\w.-]*:(file|copy|cut|exec|container|diagram|math|table)\\b[^>]*>")
    var offset = 0; var fence = ""; var payload = ""; var table = false; var diffFence = false; var equationStart: Int? = null; var equationLine = 0; var equationOpen = 0; var tableStart: Int? = null; var tableLine = 0; var delimiter = ","
    rows.forEachIndexed { line, rawRow ->
        val row = rawRow.removeSuffix("\r")
        val start = offset; offset += rawRow.length + 1; val trimmed = row.trim()
        if (fence.isNotEmpty()) {
            if (trimmed.startsWith(fence)) { fence = ""; diffFence = false }
            else if (diffFence) marks.add(InlineMark(if (row.startsWith('+')) "diff-add" else if (row.startsWith('-')) "diff-remove" else "code", start, start + row.length))
            return@forEachIndexed
        }
        if (trimmed.startsWith("```") || trimmed.startsWith("~~~")) { fence = trimmed.take(3); diffFence = trimmed.dropWhile { it == trimmed[0] }.trim().substringBefore(' ') in listOf("diff", "patch"); table = false; return@forEachIndexed }
        if (payload.isNotEmpty()) {
            val close = row.indexOf("</")
            if (trimmed.contains(":" + payload + ">") && close >= 0) {
                val from = equationStart
                if (payload == "math" && from != null && start + close >= from) equations.add(InlineEquation(equationLine, from, start + close, source.substring(from, start + close), equationOpen, start + row.length, displayBlock = true))
                val tableFrom = tableStart
                if (payload == "table" && tableFrom != null && start + close >= tableFrom) cells.addAll(inlineCSVCells(source.substring(tableFrom, start + close), tableLine, tableFrom, delimiter))
                payload = ""; equationStart = null; tableStart = null
            }
            return@forEachIndexed
        }
        val opened = opening.find(row)
        if (opened != null) {
            payload = opened.groupValues[1]
            if (payload == "math") { equationStart = start + opened.range.last + 1; equationLine = line; equationOpen = start + opened.range.first }
            if (payload == "table") {
                tableStart = start + opened.range.last + 1; tableLine = line
                delimiter = Regex("delimiter\\s*=\\s*[\"']([^\"']*)[\"']").find(opened.value)?.groupValues?.get(1)?.replace("\\t", "\t") ?: ","
                val closeTable = row.indexOf("</"); val tableFrom = tableStart!!
                if (closeTable >= 0 && start + closeTable >= tableFrom) cells.addAll(inlineCSVCells(source.substring(tableFrom, start + closeTable), tableLine, tableFrom, delimiter))
            }
            val close = row.indexOf("</"); val from = equationStart
            if (payload == "math" && from != null && close >= 0 && start + close >= from) equations.add(InlineEquation(line, from, start + close, source.substring(from, start + close), equationOpen, start + row.length, displayBlock = true))
            if (trimmed.contains("</") || trimmed.endsWith("/>")) { payload = ""; equationStart = null }
            table = false; return@forEachIndexed
        }
        heading.find(row)?.let { marks.add(InlineMark("heading", start, start + row.length, it.groupValues[1].length)) }
        val codeRanges = mutableListOf<IntRange>()
        for ((kind, expression) in styles) for (match in expression.findAll(row)) {
            if (kind != "code" && codeRanges.any { match.range.first <= it.last && match.range.last >= it.first }) continue
            if (kind == "code") codeRanges.add(match.range)
            if (kind == "math") {
                val body = match.groupValues[1]
                if (body.first().isWhitespace() || body.last().isWhitespace() || row.getOrNull(match.range.last + 1)?.isDigit() == true) continue
            }
            marks.add(InlineMark(kind, start + match.range.first, start + match.range.last + 1))
            if (kind == "math") { val body = match.groups[1]!!; equations.add(InlineEquation(line, start + body.range.first, start + body.range.last + 1, body.value, start + match.range.first, start + match.range.last + 1)) }
        }
        for (match in linkPattern.findAll(row)) {
            if (codeRanges.any { match.range.first <= it.last && match.range.last >= it.first }) continue
            val href = match.groupValues[2]
            if (workspaceLinkTarget(href, path) == null) continue
            val label = match.groups[1]!!.range
            links.add(InlineLink(start + label.first, start + label.last + 1, href,
                start + match.range.first, start + match.range.last + 1))
            marks.add(InlineMark("link", start + label.first, start + label.last + 1))
        }
        val isSeparator = separator.matches(row); val nextSeparator = rows.getOrNull(line + 1)?.let { separator.matches(it) } == true
        if (nextSeparator || table && row.contains('|')) { table = true; if (!isSeparator) cells.addAll(inlineCells(row, line, start)) } else if (!isSeparator) table = false
    }
    val patchMarks = debug.code.flatMap { (line, target) ->
        if (rows.getOrNull(line)?.startsWith('+') == true) inlineSyntaxMarks(rows[line].drop(1), target).map { it.copy(from = it.from + starts[line] + 1, to = it.to + starts[line] + 1) } else emptyList()
    }
    return InlineSnapshot(source, starts, marks + inlineSyntaxMarks(source, path) + patchMarks, cells, equations, debug, inlineDiagrams(source), links = links)
}

fun inlineTableAttribute(name: String, table: InlineTable, source: String): String? {
    if (table.rows.firstOrNull()?.firstOrNull()?.format != "csv" || table.from > source.length) return null
    val prefix = source.substring(0, table.from)
    val tag = Regex("""<[\w.-]+:table\b[^>]*>""").findAll(prefix).lastOrNull() ?: return null
    if (prefix.substring(tag.range.last + 1).isNotBlank()) return null
    return Regex("""\b${Regex.escape(name)}\s*=\s*["']([^"']*)["']""").find(tag.value)?.groupValues?.get(1)
}
