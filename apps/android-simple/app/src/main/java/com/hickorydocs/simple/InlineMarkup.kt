package com.hickorydocs.simple

data class InlineMark(val kind: String, val from: Int, val to: Int, val level: Int = 0)
data class InlineCell(val line: Int, val column: Int, val from: Int, val to: Int, val value: String, val format: String = "markdown", val delimiter: String = ",")
data class InlineEquation(val line: Int, val from: Int, val to: Int, val body: String)
data class InlineDiagram(val line: Int, val from: Int, val to: Int, val body: String, val renderer: String)
data class InlineSnapshot(val source: String, val lineStarts: List<Int> = listOf(0), val marks: List<InlineMark> = emptyList(), val cells: List<InlineCell> = emptyList(), val equations: List<InlineEquation> = emptyList(), val debug: InlineDebugContext = InlineDebugContext(), val diagrams: List<InlineDiagram> = emptyList())
// Keep source offsets authoritative while the field displays a grid-sized placeholder.
data class InlineTable(val from: Int, val to: Int, val rows: List<List<InlineCell>>) {
    val displayRows: Int get() = minOf(10, rows.size + 2)
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
    return InlineSnapshot(source, lineStarts = starts, cells = cells, diagrams = diagrams)
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
        result.add(InlineTable(snapshot.lineStarts[line], if (end > 0 && snapshot.source[end - 1] == '\r') end - 1 else end, rows))
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
    return result.sortedBy { it.from }
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
data class InlineDebugContext(val code: Map<Int, String> = emptyMap(), val launch: Map<Int, String> = emptyMap())
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
    var fence = ""; var target: String? = null
    val open = Regex("""^\s*<[\w.-]+:file\b[^>]*\bpath\s*=\s*["']([^"']+)["'][^>]*>""")
    for ((index, row) in rows.withIndex()) {
        val trimmed = row.trim()
        if (fence.isNotEmpty()) { if (trimmed.startsWith(fence)) fence = ""; continue }
        if (target == null && (trimmed.startsWith("```") || trimmed.startsWith("~~~"))) { fence = trimmed.take(3); continue }
        val match = open.find(row)
        if (match != null) {
            target = match.groupValues[1].takeIf { it.substringAfterLast('.').lowercase() in supported }
            target?.let { launch[index] = it }; continue
        }
        if (trimmed.startsWith("</") && trimmed.contains(":file>")) target = null
        if (trimmed.isNotEmpty()) target?.let { code[index] = it }
    }
    return InlineDebugContext(code, launch)
}
fun inlineDebugPath(source: String, path: String, line: Int, launch: Boolean = false): String? {
    val context = inlineDebugContext(source, path)
    return if (launch) context.launch[line] else context.code[line]
}
fun inlineLineAt(offset: Int, starts: List<Int>): Int { val found = starts.binarySearch(offset); return if (found >= 0) found else maxOf(0, -found - 2) }
fun inlineDiagrams(source: String): List<InlineDiagram> {
    val open = Regex("""^ {0,3}<[\w.-]+:(file|copy|cut|exec|entry|patch|diagram|math|table)\b[^>]*>""")
    val renderer = Regex("""\brenderer\s*=\s*["']([^"']+)["']""")
    val result = mutableListOf<InlineDiagram>()
    var offset = 0; var fence = ""; var payload = ""; var bodyStart = 0; var blockStart = 0; var blockLine = 0; var engine = "mermaid"
    for ((line, row) in source.split('\n').withIndex()) {
        val start = offset; offset += row.length + 1; val trimmed = row.trim()
        if (fence.isNotEmpty()) { if (trimmed.length >= fence.length && trimmed.all { it == fence[0] }) fence = ""; continue }
        if (payload.isEmpty()) {
            val marker = Regex("""^ {0,3}(`{3,}|~{3,})""").find(row)
            if (marker != null) { fence = marker.value.trim(); continue }
            val match = open.find(row) ?: continue
            payload = match.groupValues[1]; blockStart = start; blockLine = line; bodyStart = start + match.range.last + 1
            engine = renderer.find(match.value)?.groupValues?.get(1) ?: "mermaid"
            if (trimmed.endsWith("/>")) { payload = ""; continue }
        }
        val close = row.indexOf("</")
        if (close >= 0 && row.substring(close).contains(":" + payload + ">")) {
            val bodyEnd = start + close
            if (payload == "diagram" && bodyEnd >= bodyStart) result.add(InlineDiagram(blockLine, blockStart, start + row.length, source.substring(bodyStart, bodyEnd), engine))
            payload = ""
        }
    }
    return result
}
fun computeInlineMarkup(source: String, path: String): InlineSnapshot {
    val starts = mutableListOf(0); source.forEachIndexed { index, char -> if (char == '\n') starts.add(index + 1) }
    val debug = inlineDebugContext(source, path)
    if (path.substringAfterLast('.', "").lowercase() == "csv" && source.length <= 200_000) return InlineSnapshot(source, lineStarts = starts, cells = inlineCSVCells(source, 0, 0, ","), debug = debug)
    if (path.substringAfterLast('.', "").lowercase() !in listOf("md", "markdown") || source.length > 200_000) return InlineSnapshot(source, lineStarts = starts, debug = debug)
    val rows = source.split('\n'); val marks = mutableListOf<InlineMark>(); val cells = mutableListOf<InlineCell>(); val equations = mutableListOf<InlineEquation>()
    val separator = Regex("^\\s*\\|?\\s*:?-{3,}:?\\s*(\\|\\s*:?-{3,}:?\\s*)+\\|?\\s*$")
    val heading = Regex("^(#{1,6})\\s+")
    val styles = listOf("code" to Regex("`[^`\\n]+`"), "strong" to Regex("\\*\\*[^*\\n]+\\*\\*"), "em" to Regex("(?<!\\*)\\*[^*\\n]+\\*(?!\\*)"), "math" to Regex("(?<!\\\\)\\$([^$\\n]+)\\$"))
    val opening = Regex("^\\s*<[A-Za-z_][\\w.-]*:(file|copy|cut|exec|diagram|math|table)\\b[^>]*>")
    var offset = 0; var fence = ""; var payload = ""; var table = false; var diffFence = false; var equationStart: Int? = null; var equationLine = 0; var tableStart: Int? = null; var tableLine = 0; var delimiter = ","
    rows.forEachIndexed { line, rawRow ->
        val row = rawRow.removeSuffix("\r")
        val start = offset; offset += rawRow.length + 1; val trimmed = row.trim()
        if (fence.isNotEmpty()) {
            if (trimmed.startsWith(fence)) { fence = ""; diffFence = false }
            else if (diffFence) marks.add(InlineMark(if (row.startsWith('+')) "diff-add" else if (row.startsWith('-')) "diff-remove" else "code", start, start + row.length))
            return@forEachIndexed
        }
        if (trimmed.startsWith("```") || trimmed.startsWith("~~~")) { fence = trimmed.take(3); diffFence = trimmed.drop(3).trim() == "diff"; table = false; return@forEachIndexed }
        if (payload.isNotEmpty()) {
            val close = row.indexOf("</")
            if (trimmed.contains(":" + payload + ">") && close >= 0) {
                val from = equationStart
                if (payload == "math" && from != null && start + close >= from) equations.add(InlineEquation(equationLine, from, start + close, source.substring(from, start + close)))
                val tableFrom = tableStart
                if (payload == "table" && tableFrom != null && start + close >= tableFrom) cells.addAll(inlineCSVCells(source.substring(tableFrom, start + close), tableLine, tableFrom, delimiter))
                payload = ""; equationStart = null; tableStart = null
            }
            return@forEachIndexed
        }
        val opened = opening.find(row)
        if (opened != null) {
            payload = opened.groupValues[1]
            if (payload == "math") { equationStart = start + opened.range.last + 1; equationLine = line }
            if (payload == "table") {
                tableStart = start + opened.range.last + 1; tableLine = line
                delimiter = Regex("delimiter\\s*=\\s*[\"']([^\"']*)[\"']").find(opened.value)?.groupValues?.get(1)?.replace("\\t", "\t") ?: ","
                val closeTable = row.indexOf("</"); val tableFrom = tableStart!!
                if (closeTable >= 0 && start + closeTable >= tableFrom) cells.addAll(inlineCSVCells(source.substring(tableFrom, start + closeTable), tableLine, tableFrom, delimiter))
            }
            val close = row.indexOf("</"); val from = equationStart
            if (payload == "math" && from != null && close >= 0 && start + close >= from) equations.add(InlineEquation(line, from, start + close, source.substring(from, start + close)))
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
            if (kind == "math") { val body = match.groups[1]!!; equations.add(InlineEquation(line, start + body.range.first, start + body.range.last + 1, body.value)) }
        }
        val isSeparator = separator.matches(row); val nextSeparator = rows.getOrNull(line + 1)?.let { separator.matches(it) } == true
        if (nextSeparator || table && row.contains('|')) { table = true; if (!isSeparator) cells.addAll(inlineCells(row, line, start)) } else if (!isSeparator) table = false
    }
    return InlineSnapshot(source, starts, marks, cells, equations, debug, inlineDiagrams(source))
}
