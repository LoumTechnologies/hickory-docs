<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Add inline presentation without replacing source

For maintainers implementing rich source editing on native platforms.

Compute a line-start index and presentation against immutable UTF-16 text. Visible gutters use binary search into the current snapshot instead of repeatedly copying and counting the entire prefix on every scrolled line. Keep Markdown delimiters and original bytes in the editor, skip fenced and Hickory payload bodies, and expose exact cell/equation ranges for guarded contextual edits. Large documents retain plain source while this optional scanner is bounded to 200,000 UTF-16 units.

CSV declarations and standalone CSV files use the same grid as prose tables. Group cells only while the intervening source consists of delimiters and record separators; tags between two declarations keep their grids separate. Preserve each cell’s format and delimiter on commit so commas, quotes and embedded newlines round-trip. Revealing a table records its anchor independently of the other grids.

Markdown prose tables now use those exact cell ranges to form whole-table presentation ranges. Keep the final newline outside the replacement so following prose retains its paragraph. Header separators belong to the replacement, but never become editable data cells. Only cells admitted by the prose or CSV scanner can enter a grid; fenced code and Hickory literal bodies stay source. The Swift and Kotlin range witnesses check CRLF, emoji and escaped pipes.

Fenced diff examples retain their literal text and receive addition/removal backgrounds on native platforms. They are review text; launch and breakpoint controls belong to the accompanying executable file block. This does not claim that diff context or deleted lines are independent executable statements.

While a new parse is pending, map unaffected table cells through the text change.
This is presentation only: keep debugging and equation actions on exact snapshots.
A newline outside a table must not temporarily turn a grid back into source and
change the document height. Drop cells touched by a structural edit; the next
background parse discovers their new structure. Cell-value edits retain their
range and decode the new value, including empty values.

The Swift scanner returns source-backed styles and cell ranges. Escaped pipes and code-span pipes cannot create extra columns; applying a cell edit escapes literal pipes and retains all untouched bytes.

<ui:copy id="simple-inline-model-0">
import Foundation
struct InlineMark: Sendable { var kind: String; var range: NSRange; var level: Int = 0 }
struct InlineCell: Identifiable, Sendable { var line: Int; var column: Int; var range: NSRange; var value: String; var format = "markdown"; var delimiter = ","; var id: Int { range.location } }
struct InlineEquation: Identifiable, Sendable { var line: Int; var range: NSRange; var body: String; var id: Int { range.location } }
struct InlineSnapshot: Sendable { var source: String; var lineStarts: [Int] = [0]; var marks: [InlineMark] = []; var cells: [InlineCell] = []; var equations: [InlineEquation] = []; var debug = InlineDebugContext() }
// Tables are presentation ranges over the original source, never replacement text.
struct InlineTable: Identifiable, Sendable {
    var range: NSRange; var rows: [[InlineCell]]
    var id: Int { range.location }
    var height: CGFloat { min(320, CGFloat(rows.count + 2) * 30) }
}
func tablePresentation(_ snapshot: InlineSnapshot, source: String) -> InlineSnapshot {
    if snapshot.source == source { return snapshot }
    let old = Array(snapshot.source.utf16), new = Array(source.utf16)
    var prefix = 0, suffix = 0
    while prefix < min(old.count, new.count), old[prefix] == new[prefix] { prefix += 1 }
    while suffix < min(old.count, new.count) - prefix, old[old.count - suffix - 1] == new[new.count - suffix - 1] { suffix += 1 }
    let end = old.count - suffix, delta = new.count - old.count, text = source as NSString
    var result = InlineSnapshot(source: source)
    for (index, char) in new.enumerated() where char == 10 { result.lineStarts.append(index + 1) }
    result.cells = snapshot.cells.compactMap { cell in
        var cell = cell
        if NSMaxRange(cell.range) <= prefix { }
        else if cell.range.location >= end { cell.range.location += delta }
        else if cell.range.location <= prefix, NSMaxRange(cell.range) >= end,
                !new[prefix..<(new.count - suffix)].contains(10) { cell.range.length += delta }
        else { return nil }
        guard cell.range.location >= 0, cell.range.length >= 0, NSMaxRange(cell.range) <= text.length else { return nil }
        cell.line = inlineLineAt(cell.range.location, starts: result.lineStarts)
        let raw = text.substring(with: cell.range)
        cell.value = cell.format == "csv" && raw.hasPrefix("\"") && raw.hasSuffix("\"") && raw.count >= 2
            ? String(raw.dropFirst().dropLast()).replacingOccurrences(of: "\"\"", with: "\"")
            : raw.replacingOccurrences(of: "\\|", with: "|").replacingOccurrences(of: "\\\\", with: "\\")
        return cell
    }
    return result
}
func inlineTables(_ snapshot: InlineSnapshot) -> [InlineTable] {
    let source = snapshot.source as NSString
    let lines = Dictionary(grouping: snapshot.cells.filter { $0.format == "markdown" }, by: \.line)
    var result: [InlineTable] = [], consumed = Set<Int>()
    for line in lines.keys.sorted() where !consumed.contains(line) {
        guard let header = lines[line], line + 1 < snapshot.lineStarts.count,
              lines[line + 1] == nil else { continue }
        let separator = source.substring(with: source.lineRange(for: NSRange(location: snapshot.lineStarts[line + 1], length: 0))).trimmingCharacters(in: .whitespacesAndNewlines)
        guard !separator.isEmpty, separator.contains("-"), separator.allSatisfy({ "|:- \t".contains($0) }) else { continue }
        var rows = [header], last = line + 1, next = line + 2
        consumed.insert(line)
        while let cells = lines[next] { rows.append(cells); consumed.insert(next); last = next; next += 1 }
        let start = snapshot.lineStarts[line]
        let end = NSMaxRange(source.lineRange(for: NSRange(location: snapshot.lineStarts[last], length: 0)))
        // Retain the final newline outside the replacement so following prose keeps its own paragraph.
        var to = end
        while to > start && [10, 13].contains(Int(source.character(at: to - 1))) { to -= 1 }
        result.append(InlineTable(range: NSRange(location: start, length: to - start), rows: rows))
    }
    let csv = snapshot.cells.filter { $0.format == "csv" }.sorted { $0.range.location < $1.range.location }
    var groups: [[InlineCell]] = []
    for cell in csv {
        if let last = groups.last?.last {
            let gap = source.substring(with: NSRange(location: NSMaxRange(last.range), length: cell.range.location - NSMaxRange(last.range)))
            if gap.allSatisfy({ $0.isNewline || cell.delimiter.contains($0) }) { groups[groups.count - 1].append(cell); continue }
        }
        groups.append([cell])
    }
    for cells in groups {
        guard let first = cells.first, let last = cells.last else { continue }
        var rows: [[InlineCell]] = []
        for cell in cells { if cell.column == 0 { rows.append([]) }; rows[rows.count - 1].append(cell) }
        result.append(InlineTable(range: NSRange(location: first.range.location, length: NSMaxRange(last.range) - first.range.location), rows: rows))
    }
    return result.sorted { $0.range.location < $1.range.location }
}
func inlineCellText(_ value: String, format: String = "markdown", delimiter: String = ",") -> String {
    if format == "csv" { return value.contains(delimiter) || value.contains("\n") || value.contains("\r") || value.contains("\"") ? "\"" + value.replacingOccurrences(of: "\"", with: "\"\"") + "\"" : value }
    return value.replacingOccurrences(of: "\\", with: "\\\\").replacingOccurrences(of: "|", with: "\\|").replacingOccurrences(of: "\n", with: "<br>") }
func inlineCells(_ row: String, line: Int, offset: Int) -> [InlineCell] {
    let text = row as NSString; var boundaries = [-1], codeRun = 0, index = 0
    while index < text.length {
        let char = text.character(at: index)
        if char == 92 && codeRun == 0 { index += min(2, text.length - index); continue }
        if char == 96 {
            var end = index + 1
            while end < text.length && text.character(at: end) == 96 { end += 1 }
            let run = end - index
            if codeRun == 0 {
                // An unmatched delimiter is literal; only paired runs hide pipes.
                let tail = text.substring(from: end)
                if tail.contains(String(repeating: "`", count: run)) { codeRun = run }
            } else if codeRun == run { codeRun = 0 }
            index = end; continue
        }
        if char == 124 && codeRun == 0 { boundaries.append(index) }
        index += 1
    }
    boundaries.append(text.length)
    var result: [InlineCell] = []
    for index in 0..<(boundaries.count - 1) {
        let from = boundaries[index] + 1, to = boundaries[index + 1]
        let raw = text.substring(with: NSRange(location: from, length: to - from))
        if (index == 0 || index == boundaries.count - 2) && raw.trimmingCharacters(in: .whitespaces).isEmpty { continue }
        let leading = raw.prefix { $0 == " " || $0 == "\t" }.utf16.count
        let value = raw.trimmingCharacters(in: .whitespaces)
        result.append(InlineCell(line: line, column: result.count, range: NSRange(location: offset + from + leading, length: value.utf16.count), value: value.replacingOccurrences(of: "\\|", with: "|").replacingOccurrences(of: "\\\\", with: "\\")))
    }
    return result
}
func inlineCSVCells(_ body: String, line: Int, offset: Int, delimiter: String) -> [InlineCell] {
    let text = body as NSString
    guard let separator = delimiter.utf16.first, delimiter.utf16.count == 1 else { return [] }
    var index = body.hasPrefix("\r\n") ? 2 : body.hasPrefix("\n") ? 1 : 0
    var row = line + (index > 0 ? 1 : 0), start = index, startLine = row, column = 0, quoted = false, result: [InlineCell] = []
    while index <= text.length {
        let char: unichar = index < text.length ? text.character(at: index) : 0
        if char == 34 {
            if quoted && index + 1 < text.length && text.character(at: index + 1) == 34 { index += 2; continue }
            quoted.toggle()
        }
        let newline = !quoted && (char == 10 || char == 13)
        if !quoted && (char == separator || newline || index == text.length) {
            if start < text.length || char == separator || index == text.length && index > 0 && text.character(at: index - 1) == separator {
                let raw = text.substring(with: NSRange(location: start, length: index - start))
                let value = raw.hasPrefix("\"") && raw.hasSuffix("\"") && raw.utf16.count >= 2 ? String(raw.dropFirst().dropLast()).replacingOccurrences(of: "\"\"", with: "\"") : raw
                result.append(InlineCell(line: startLine, column: column, range: NSRange(location: offset + start, length: index - start), value: value, format: "csv", delimiter: delimiter))
            }
            if newline { if char == 13 && index + 1 < text.length && text.character(at: index + 1) == 10 { index += 1 }; row += 1; column = 0 } else { column += 1 }
            start = index + 1; startLine = row
        } else if char == 10 { row += 1 }
        index += 1
    }
    return quoted ? [] : result
}
// Candidate discovery runs with the background snapshot, once per source revision.
// The adapter remains authoritative about binding or refusing a pending stop.
struct InlineDebugContext: Sendable { var code: [Int: String] = [:]; var launch: [Int: String] = [:] }
func inlineDebugContext(_ source: String, path: String) -> InlineDebugContext {
    var result = InlineDebugContext()
    guard source.utf16.count <= 200_000 else { return result }
    let supported = ["py", "js", "ts", "jsx", "tsx", "rs", "go", "c", "cpp", "java", "cs"]
    let rows = source.components(separatedBy: "\n")
    if supported.contains((path as NSString).pathExtension.lowercased()) {
        result.launch[0] = path
        for (index, row) in rows.enumerated() where !row.trimmingCharacters(in: .whitespaces).isEmpty { result.code[index] = path }
        return result
    }
    guard ["md", "markdown"].contains((path as NSString).pathExtension.lowercased()) else { return result }
    var fence = "", target: String?
    let open = try! NSRegularExpression(pattern: #"^\s*<[\w.-]+:file\b[^>]*\bpath\s*=\s*["']([^"']+)["'][^>]*>"#)
    for (index, row) in rows.enumerated() {
        let trimmed = row.trimmingCharacters(in: .whitespaces)
        if !fence.isEmpty { if trimmed.hasPrefix(fence) { fence = "" }; continue }
        if target == nil && (trimmed.hasPrefix("```") || trimmed.hasPrefix("~~~")) { fence = String(trimmed.prefix(3)); continue }
        if let match = open.firstMatch(in: row, range: NSRange(location: 0, length: row.utf16.count)) {
            let candidate = (row as NSString).substring(with: match.range(at: 1))
            target = supported.contains((candidate as NSString).pathExtension.lowercased()) ? candidate : nil
            result.launch[index] = target
            continue
        }
        if trimmed.hasPrefix("</"), trimmed.contains(":file>") { target = nil }
        if !trimmed.isEmpty { result.code[index] = target }
    }
    return result
}
func inlineDebugPath(_ source: String, path: String, line: Int, launch: Bool = false) -> String? {
    let context = inlineDebugContext(source, path: path)
    return launch ? context.launch[line] : context.code[line]
}
func inlineLineAt(_ offset: Int, starts: [Int]) -> Int {
    var low = 0, high = starts.count
    while low < high { let middle = (low + high) / 2; if starts[middle] <= offset { low = middle + 1 } else { high = middle } }
    return max(0, low - 1)
}
func computeInlineMarkup(_ source: String, path: String) -> InlineSnapshot {
    var starts = [0]
    for (index, unit) in source.utf16.enumerated() { if unit == 10 { starts.append(index + 1) } }
    var result = InlineSnapshot(source: source, lineStarts: starts, debug: inlineDebugContext(source, path: path))
    if (path as NSString).pathExtension.lowercased() == "csv", source.utf16.count <= 200_000 {
        result.cells = inlineCSVCells(source, line: 0, offset: 0, delimiter: ","); return result
    }
    guard ["md", "markdown"].contains((path as NSString).pathExtension.lowercased()), source.utf16.count <= 200_000 else { return result }
    let rows = source.components(separatedBy: "\n")
    let separator = try! NSRegularExpression(pattern: #"^\s*\|?\s*:?-{3,}:?\s*(\|\s*:?-{3,}:?\s*)+\|?\s*$"#)
    let heading = try! NSRegularExpression(pattern: #"^(#{1,6})\s+"#)
    let styles = [("code", #"`[^`\n]+`"#), ("strong", #"\*\*[^*\n]+\*\*"#), ("em", #"(?<!\*)\*[^*\n]+\*(?!\*)"#), ("math", #"(?<!\\)\$([^$\n]+)\$"#)].map { ($0.0, try! NSRegularExpression(pattern: $0.1)) }
    let opening = try! NSRegularExpression(pattern: #"^\s*<[A-Za-z_][\w.-]*:(file|copy|cut|exec|diagram|math|table)\b[^>]*>"#)
    var offset = 0, fence = "", payload = "", table = false, diffFence = false
    var equationStart: Int?, equationLine = 0
    var tableStart: Int?, tableLine = 0, delimiter = ","
    for (line, rawRow) in rows.enumerated() {
        defer { offset += rawRow.utf16.count + 1 }
        let row = rawRow.hasSuffix("\r") ? String(rawRow.dropLast()) : rawRow
        if Task.isCancelled { return InlineSnapshot(source: source) }
        let trimmed = row.trimmingCharacters(in: .whitespaces), ns = row as NSString, all = NSRange(location: 0, length: row.utf16.count)
        if !fence.isEmpty {
            if trimmed.hasPrefix(fence) { fence = ""; diffFence = false }
            else if diffFence { result.marks.append(InlineMark(kind: row.hasPrefix("+") ? "diff-add" : row.hasPrefix("-") ? "diff-remove" : "code", range: NSRange(location: offset, length: row.utf16.count))) }
            continue
        }
        if trimmed.hasPrefix("```") || trimmed.hasPrefix("~~~") { fence = String(trimmed.prefix(3)); diffFence = trimmed.dropFirst(3).trimmingCharacters(in: .whitespaces) == "diff"; table = false; continue }
        if !payload.isEmpty {
            if trimmed.contains(":" + payload + ">"), let close = row.range(of: "</") {
                if payload == "math", let start = equationStart {
                    let end = offset + row[..<close.lowerBound].utf16.count
                    if end >= start { let range = NSRange(location: start, length: end - start); result.equations.append(InlineEquation(line: equationLine, range: range, body: (source as NSString).substring(with: range))) }
                }
                if payload == "table", let start = tableStart {
                    let end = offset + row[..<close.lowerBound].utf16.count
                    if end >= start { result.cells += inlineCSVCells((source as NSString).substring(with: NSRange(location: start, length: end - start)), line: tableLine, offset: start, delimiter: delimiter) }
                }
                payload = ""; equationStart = nil; tableStart = nil
            }
            continue
        }
        if let match = opening.firstMatch(in: row, range: all) {
            payload = ns.substring(with: match.range(at: 1))
            if payload == "math" { equationStart = offset + NSMaxRange(match.range); equationLine = line }
            if payload == "table" {
                tableStart = offset + NSMaxRange(match.range); tableLine = line; delimiter = ","
                let declaration = ns.substring(with: match.range)
                let pattern = try! NSRegularExpression(pattern: #"delimiter\s*=\s*["']([^"']*)["']"#)
                if let attr = pattern.firstMatch(in: declaration, range: NSRange(location: 0, length: declaration.utf16.count)) { delimiter = (declaration as NSString).substring(with: attr.range(at: 1)).replacingOccurrences(of: "\\t", with: "\t") }
                if let close = row.range(of: "</"), offset + row[..<close.lowerBound].utf16.count >= tableStart! {
                    let end = offset + row[..<close.lowerBound].utf16.count
                    result.cells += inlineCSVCells((source as NSString).substring(with: NSRange(location: tableStart!, length: end - tableStart!)), line: tableLine, offset: tableStart!, delimiter: delimiter)
                }
            }
            if let close = row.range(of: "</"), payload == "math", let start = equationStart {
                let end = offset + row[..<close.lowerBound].utf16.count
                if end >= start { let range = NSRange(location: start, length: end - start); result.equations.append(InlineEquation(line: line, range: range, body: (source as NSString).substring(with: range))) }
            }
            if trimmed.contains("</") || trimmed.hasSuffix("/>") { payload = ""; equationStart = nil }
            table = false; continue
        }
        if let match = heading.firstMatch(in: row, range: all) { result.marks.append(InlineMark(kind: "heading", range: NSRange(location: offset, length: row.utf16.count), level: match.range(at: 1).length)) }
        var codeRanges: [NSRange] = []
        for (kind, expression) in styles {
            for match in expression.matches(in: row, range: all) {
                if kind != "code" && codeRanges.contains(where: { NSIntersectionRange($0, match.range).length > 0 }) { continue }
                if kind == "code" { codeRanges.append(match.range) }
                if kind == "math" {
                    let body = ns.substring(with: match.range(at: 1)), end = NSMaxRange(match.range)
                    if body.first?.isWhitespace == true || body.last?.isWhitespace == true || end < ns.length && (48...57).contains(Int(ns.character(at: end))) { continue }
                }
                result.marks.append(InlineMark(kind: kind, range: NSRange(location: offset + match.range.location, length: match.range.length)))
                if kind == "math" { let body = match.range(at: 1); result.equations.append(InlineEquation(line: line, range: NSRange(location: offset + body.location, length: body.length), body: ns.substring(with: body))) }
            }
        }
        let isSeparator = separator.firstMatch(in: row, range: all) != nil
        let nextSeparator = line + 1 < rows.count && separator.firstMatch(in: rows[line + 1], range: NSRange(location: 0, length: rows[line + 1].utf16.count)) != nil
        if nextSeparator || (table && row.contains("|")) {
            table = true
            if !isSeparator { result.cells += inlineCells(row, line: line, offset: offset) }
        } else if !isSeparator { table = false }
    }
    return result
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/InlineMarkup.swift"><ui:paste select="#simple-inline-model-0"/></ui:file>

The Kotlin scanner returns source-backed styles and cell ranges. Escaped pipes and code-span pipes cannot create extra columns; applying a cell edit escapes literal pipes and retains all untouched bytes.

<ui:copy id="simple-inline-model-1">
package com.hickorydocs.simple

data class InlineMark(val kind: String, val from: Int, val to: Int, val level: Int = 0)
data class InlineCell(val line: Int, val column: Int, val from: Int, val to: Int, val value: String, val format: String = "markdown", val delimiter: String = ",")
data class InlineEquation(val line: Int, val from: Int, val to: Int, val body: String)
data class InlineSnapshot(val source: String, val lineStarts: List<Int> = listOf(0), val marks: List<InlineMark> = emptyList(), val cells: List<InlineCell> = emptyList(), val equations: List<InlineEquation> = emptyList(), val debug: InlineDebugContext = InlineDebugContext())
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
    return InlineSnapshot(source, lineStarts = starts, cells = cells)
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
    return InlineSnapshot(source, starts, marks, cells, equations, debug)
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/InlineMarkup.kt"><ui:paste select="#simple-inline-model-1"/></ui:file>


The local math page loads only bundled KaTeX and fonts. Formula strings enter through a JSON argument, never HTML interpolation. KaTeX trust is disabled and the page CSP denies remote resources; malformed or oversized equations retain readable source.

<ui:copy id="simple-inline-math-0">
<!doctype html>
<html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'self' 'nonce-hickory-math'; style-src 'self' 'unsafe-inline'; font-src 'self' data:;">
<link rel="stylesheet" href="katex/katex.min.css"><style>html{color-scheme:light dark;font:14px system-ui}body{margin:8px}#formula{overflow:auto;padding:8px}#status{font-size:11px}</style>
</head><body><div id="formula" aria-label="Typeset equation"></div><div id="status" role="status"></div>
<script nonce="hickory-math" src="katex/katex.min.js"></script><script nonce="hickory-math">
globalThis.renderMath = function (formula) {
  const node = document.getElementById('formula'), status = document.getElementById('status');
  node.textContent = formula;
  if (typeof katex === 'undefined') { status.textContent = 'Math engine unavailable. Run npm --prefix apps/web ci and python3 scripts/prepare-simple-resources.py, then rebuild.'; return; }
  if (formula.length > 4000) { status.textContent = 'Equation is too large to typeset here. Its source remains editable.'; return; }
  try { katex.render(formula, node, {throwOnError:false, trust:false, strict:'warn', displayMode:true, maxExpand:1000}); status.textContent = ''; }
  catch (_) { node.textContent = formula; status.textContent = 'Check the LaTeX expression.'; }
};
</script></body></html>
</ui:copy>

<ui:file path="apps/apple-simple/Sources/Resources/math.html"><ui:paste select="#simple-inline-math-0"/></ui:file>

The build helper copies assets from the locked frontend installation. Missing optional assets degrade to source rather than preventing native editing. Build outputs stay ignored and are not independent maintained copies.

<ui:copy id="simple-inline-math-1">
"""Copy locked local math assets into the native resource bundle; never download."""
from pathlib import Path
import os
import tempfile

ROOT = Path(__file__).resolve().parents[1]
source = ROOT / "apps/web/node_modules/katex/dist"
target = ROOT / "apps/apple-simple/Sources/Resources/katex"
if not (source / "katex.min.js").is_file():
    print("Native math will keep showing source. Run npm --prefix apps/web ci, then rerun scripts/prepare-simple-resources.py for typesetting.")
else:
    target.mkdir(parents=True, exist_ok=True)
    files = [source / "katex.min.js", source / "katex.min.css", *sorted((source / "fonts").glob("*"))]
    for original in files:
        destination = target / original.relative_to(source)
        data = original.read_bytes()
        if destination.is_file() and destination.read_bytes() == data:
            destination.chmod(0o644)
            continue
        destination.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.NamedTemporaryFile(dir=destination.parent, delete=False) as temporary:
            temporary.write(data)
            staged = temporary.name
        os.chmod(staged, 0o644)
        os.replace(staged, destination)
    print("Prepared local native math resources.")
</ui:copy>

<ui:file path="scripts/prepare-simple-resources.py"><ui:paste select="#simple-inline-math-1"/></ui:file>

Apple embeds a small typesetter in the line’s floating editor. Navigation is restricted to the local page, and the LaTeX replacement still passes through source, permission, composition and undo checks.

<ui:copy id="simple-inline-math-2">
import SwiftUI
import WebKit
@MainActor final class MathCoordinator: NSObject, WKNavigationDelegate {
    var source = ""
    var ready = false
    func display(_ view: WKWebView) {
        guard ready, let bytes = try? JSONSerialization.data(withJSONObject: source, options: .fragmentsAllowed), let json = String(data: bytes, encoding: .utf8) else { return }
        view.evaluateJavaScript("globalThis.renderMath(" + json + ")", completionHandler: nil)
    }
    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) { ready = true; display(webView) }
    func webView(_ webView: WKWebView, decidePolicyFor navigationAction: WKNavigationAction, decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
        let url = navigationAction.request.url
        decisionHandler(url?.isFileURL == true || url?.scheme == "about" ? .allow : .cancel)
    }
}
// Inspect bundles without touching SwiftPM's fatal Bundle.module accessor.
func nativeMathResource(roots: [URL], bundles: [Bundle] = [.main]) -> URL? {
    let candidates = bundles + roots.compactMap { Bundle(url: $0.appendingPathComponent("HickorySimple_HickorySimple.bundle")) }
    return candidates.lazy.compactMap { bundle in
        bundle.url(forResource: "math", withExtension: "html", subdirectory: "Resources") ?? bundle.url(forResource: "math", withExtension: "html")
    }.first
}
@MainActor func makeMathView(_ coordinator: MathCoordinator) -> WKWebView {
    let configuration = WKWebViewConfiguration(); configuration.websiteDataStore = .nonPersistent(); configuration.preferences.javaScriptCanOpenWindowsAutomatically = false
    let view = WKWebView(frame: .zero, configuration: configuration); view.navigationDelegate = coordinator
    let roots = [Bundle.main.resourceURL, Bundle.main.bundleURL,
        URL(fileURLWithPath: CommandLine.arguments[0]).deletingLastPathComponent()].compactMap { $0 }
    let url = nativeMathResource(roots: roots)
    if let url { view.loadFileURL(url, allowingReadAccessTo: url.deletingLastPathComponent()) }
    else { view.loadHTMLString("<body><p>Math resource missing; source stays editable.</p><div id='math'></div><script>globalThis.renderMath=s=>document.getElementById('math').textContent=s</script></body>", baseURL: nil) }
    return view
}
#if os(macOS)
struct NativeMathView: NSViewRepresentable {
    var source: String
    func makeCoordinator() -> MathCoordinator { MathCoordinator() }
    func makeNSView(context: Context) -> WKWebView { context.coordinator.source = source; return makeMathView(context.coordinator) }
    func updateNSView(_ view: WKWebView, context: Context) { context.coordinator.source = source; context.coordinator.display(view) }
}
#else
struct NativeMathView: UIViewRepresentable {
    var source: String
    func makeCoordinator() -> MathCoordinator { MathCoordinator() }
    func makeUIView(context: Context) -> WKWebView { context.coordinator.source = source; return makeMathView(context.coordinator) }
    func updateUIView(_ view: WKWebView, context: Context) { context.coordinator.source = source; context.coordinator.display(view) }
}
#endif
struct InlineEquationControl: View {
    var equation: InlineEquation
    var source: String
    var buffer: DocumentBuffer
    @State private var draft = ""
    @State private var notice = ""
    var body: some View {
        VStack(alignment: .leading) {
            NativeMathView(source: draft).frame(height: 120)
            TextEditor(text: $draft).font(.system(.caption, design: .monospaced)).frame(height: 64).disabled(buffer.readOnly)
            Button("Apply equation") {
                if buffer.applySourceEdits?([NativeSourceEdit(range: equation.range, text: draft)], source) != true { notice = "Equation changed. Reopen this line and try again." }
            }.disabled(buffer.readOnly || draft == equation.body)
            if !notice.isEmpty { Text(notice).font(.caption) }
        }.onAppear { draft = equation.body }
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/InlineMath.swift"><ui:paste select="#simple-inline-math-2"/></ui:file>

Android uses the same local page with network and content access disabled. Source remains in TextFieldState; the WebView receives no native mutation bridge and is destroyed when its contextual control closes.

<ui:copy id="simple-inline-math-3">
package com.hickorydocs.simple

import android.annotation.SuppressLint
import android.webkit.WebView
import android.webkit.WebViewClient
import android.webkit.WebResourceRequest
import android.webkit.RenderProcessGoneDetail
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import org.json.JSONObject

@SuppressLint("SetJavaScriptEnabled")
@Composable fun NativeMathView(source: String, modifier: Modifier = Modifier) {
    AndroidView(modifier = modifier, factory = { context ->
        WebView(context).apply {
            settings.javaScriptEnabled = true; settings.allowFileAccess = false; settings.allowContentAccess = false; settings.blockNetworkLoads = true
            webViewClient = object : WebViewClient() {
                override fun onRenderProcessGone(view: WebView, detail: RenderProcessGoneDetail): Boolean { view.destroy(); return true }
                override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest): Boolean = request.url.toString() != "file:///android_asset/math.html"
                override fun onPageFinished(view: WebView, url: String) {
                    if (url == "file:///android_asset/math.html") view.evaluateJavascript("globalThis.renderMath(" + JSONObject.quote(view.tag as? String ?: "") + ")", null)
                }
            }
            tag = source; loadUrl("file:///android_asset/math.html")
        }
    }, update = { view ->
        view.tag = source
        view.evaluateJavascript("if(globalThis.renderMath)globalThis.renderMath(" + JSONObject.quote(source) + ")", null)
    }, onRelease = { it.stopLoading(); it.destroy() })
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/InlineMath.kt"><ui:paste select="#simple-inline-math-3"/></ui:file>


The browser retains the established rendered-block registry and table/math/exec views. Their React owners live above pane layout, preserving widget-local edits when the cached editor moves. Widgets use a shadow root to reuse the established stylesheet without changing simple-UI chrome. Mutations still use the existing captured-block transaction guard; saved receipts disappear as soon as the buffer is dirty. The optional rendering modules load after readable text mounts.

<ui:copy id="simple-inline-browser-owners">
import { Component, useEffect, useState, useSyncExternalStore } from "react";
import { createPortal } from "react-dom";
import { Buffer, Buffers } from "./buffers";
import { editorForBuffer, SIMPLE_EDITOR_ATTACHED } from "./Document";
import type { EditorView } from "@codemirror/view";
import type { RenderedSlot } from "../editor/rendered";
import type { Block, ExecBlock, DiagramBlock } from "../api/types";
import { request } from "../api/client";

// The owners live above pane layout. Moving a cached editor cannot unmount a
// table's uncommitted cell editor or lose a rendered widget's own local state.
export function RichOwners({ buffers }: { buffers: Buffers }) {
  const [, refresh] = useState(0);
  useEffect(() => { const update = () => refresh(value => value + 1); window.addEventListener(SIMPLE_EDITOR_ATTACHED, update); update(); return () => window.removeEventListener(SIMPLE_EDITOR_ATTACHED, update); }, []);
  return <>{[...buffers.values.values()].map(buffer => {
    const cached = editorForBuffer(buffer);
    return cached && /\.(md|markdown)$/i.test(buffer.file.path) ? <RichBoundary key={buffer.file.path} buffer={buffer} view={cached.view}><RichOwner buffer={buffer} view={cached.view} /></RichBoundary> : null;
  })}</>;
}
class RichBoundary extends Component<{ buffer: Buffer; view: EditorView; children: React.ReactNode }, { failed: boolean }> {
  state = { failed: false };
  static getDerivedStateFromError() { return { failed: true }; }
  componentDidCatch() {
    const cached = editorForBuffer(this.props.buffer);
    if (cached?.view === this.props.view) { cached.richAttached = false; cached.view.dispatch({ effects: cached.prose.reconfigure([]) }); }
    this.props.buffer.error = "Inline rendering unavailable. Continue editing source or reload to retry."; this.props.buffer.publish();
  }
  render() { return this.state.failed ? null : this.props.children; }
}
function SlotPortal({ slot, children, css }: { slot: RenderedSlot; children: React.ReactNode; css: string }) {
  const [root] = useState(() => slot.el.shadowRoot ?? slot.el.attachShadow({ mode: "open" }));
  return createPortal(<><style>{css.replaceAll(":root", ":host")}</style>{children}</>, root);
}
function RichOwner({ buffer, view }: { buffer: Buffer; view: EditorView }) {
  useSyncExternalStore(buffer.subscribe, buffer.snapshot);
  const [slots, setSlots] = useState<RenderedSlot[]>([]);
  const [modules, setModules] = useState<Awaited<ReturnType<typeof load>> | null>(null);
  const [blocks, setBlocks] = useState<{ source: string; blocks: Block[] }>({ source: "", blocks: [] });
  useEffect(() => {
    if (buffer.text.length > 200_000) return;
    let disposed = false, off: (() => void) | undefined, stopWaiting: (() => void) | undefined;
    void load().then(libs => {
      const install = () => {
      if (disposed || !view.dom.isConnected) return;
      const cached = editorForBuffer(buffer); if (!cached || cached.view !== view) return;
      const registry = new libs.rendered.RenderedRegistry();
      off = registry.subscribe(() => setSlots(registry.list()));
      cached.richAttached = true;
      const scroll = view.scrollSnapshot();
      view.dispatch({ effects: cached.prose.reconfigure([
        libs.prose.wysiwyg(new libs.prose.EnvRegistry()), libs.rendered.renderedBlocks(registry),
        libs.math.renderedMath(state => libs.spans.mathSpans(state.doc.toString(), libs.hick.verbatimRanges(libs.prose.structureOf(state).blocks))),
      ]) });
      const initial = libs.elements.renderableBlocks(libs.prose.structureOf(view.state), view.state.doc.toString()).filter(block => !libs.elements.slotKindOf(block)?.startsWith("session-") && !(buffer.dirty && view.state.selection.ranges.some(range => range.from >= block.from && range.to <= block.to)));
      view.dispatch({ effects: [libs.rendered.setRenderedBlocks.of(initial.map(block => block.from)), scroll] });
      setSlots(registry.list()); setModules(libs);
      };
      if (view.dom.isConnected) install();
      else {
        const attached = () => { if (view.dom.isConnected) { stopWaiting?.(); install(); } };
        window.addEventListener(SIMPLE_EDITOR_ATTACHED, attached);
        stopWaiting = () => window.removeEventListener(SIMPLE_EDITOR_ATTACHED, attached);
      }
    }).catch(error => { buffer.error = "Inline rendering unavailable: " + (error instanceof Error ? error.message : String(error)); buffer.publish(); });
    return () => { disposed = true; stopWaiting?.(); off?.(); const cached = editorForBuffer(buffer); if (cached?.view === view) { cached.richAttached = false; view.dispatch({ effects: cached.prose.reconfigure([]) }); } };
  }, [buffer, view, buffer.text.length > 200_000]);
  useEffect(() => {
    if (typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver(() => view.requestMeasure()); slots.forEach(slot => observer.observe(slot.el)); return () => observer.disconnect();
  }, [view, slots]);
  useEffect(() => {
    if (!buffer.file.doc_id || buffer.dirty) return;
    const source = buffer.text, id = buffer.file.doc_id; let cancelled = false;
    void crypto.subtle.digest("SHA-256", new TextEncoder().encode(source)).then(async bytes => {
      const expected = [...new Uint8Array(bytes)].map(byte => byte.toString(16).padStart(2, "0")).join("");
      const result = await request<{ document: string; source_hash: string; blocks: Block[] }>("GET", `/api/docs/${id}/render`);
      if (!cancelled && buffer.text === source && !buffer.dirty && result.document === id && result.source_hash.replace(/^sha256:/, "") === expected) setBlocks({ source, blocks: result.blocks });
    }).catch(() => { /* Saved receipts are optional, never a reason to hide source. */ });
    return () => { cancelled = true; };
  }, [buffer, buffer.base, buffer.dirty]);
  if (!modules) return null;
  const saved: Block[] = [];
  if (blocks.source === buffer.text && !buffer.dirty) {
    const bytes = new TextEncoder().encode(blocks.source), decoder = new TextDecoder("utf-8", { fatal: true });
    for (const block of blocks.blocks) {
      if (!Array.isArray(block.span) || block.span[0] < 0 || block.span[1] < block.span[0] || block.span[1] > bytes.length) continue;
      try { saved.push({ ...block, span: [decoder.decode(bytes.slice(0, block.span[0])).length, decoder.decode(bytes.slice(0, block.span[1])).length] }); }
      catch { /* Invalid UTF-8 boundaries cannot identify a source widget. */ }
    }
  }
  const tables = modules.elements.renderableBlocks(modules.prose.structureOf(view.state), view.state.doc.toString()).filter(block => modules.elements.slotKindOf(block) === "table");
  const sourceTables = tables.filter(block => !slots.some(slot => slot.at === block.from));
  return <>{sourceTables.length > 0 && view.dom.parentElement && createPortal(<button type="button" className="simple-render-tables" onClick={() => {
    view.dispatch({ effects: modules.rendered.setRenderedBlocks.of([...new Set([...slots.map(slot => slot.at), ...tables.map(block => block.from)])]) });
  }}>Edit tables as grids</button>, view.dom.parentElement)}{slots.map(slot => {
    const captured = view.state.doc;
    const context = { view, path: buffer.file.path, docId: buffer.file.doc_id, execBlocks: saved.filter((block): block is ExecBlock => block.kind === "exec"),
      diagramBlocks: saved.filter((block): block is DiagramBlock => block.kind === "diagram"), runningCells: new Set<string>(), replaying: [],
      replaceBlockContent: (target: RenderedSlot, text: string, origin?: string) => modules.edit.replaceCapturedBlock(view, target, text, origin ?? "input.table"),
    };
    return <SlotPortal key={slot.key} slot={slot} css={modules.css.default + modules.katexCss.default}>
      {!slot.outputOnly && <button type="button" style={{ border: "1px solid currentColor", borderRadius: 4, padding: "3px 8px", cursor: "pointer" }} onClick={() => {
        if (view.state.doc !== captured) return;
        view.dispatch({ effects: modules.rendered.showBlockSource.of(slot.at), selection: { anchor: Math.min(slot.at, view.state.doc.length) }, scrollIntoView: true }); view.focus();
      }}>Edit source text</button>}
      {modules.elements.elementViews[slot.kind].render(slot, context)}
    </SlotPortal>;
  })}</>;
}
async function load() {
  // Resolve the shared cyclic registry graph before importing its members in parallel.
  const parser = await import("../editor/hickLang");
  await parser.loadHickLang();
  const rendered = await import("../editor/rendered");
  const [prose, math, spans, hick, elements, edit, css, katexCss] = await Promise.all([
    import("../editor/wysiwyg"), import("../editor/mathRender"), import("../lib/math"), import("../editor/hickDoc"),
    import("../elements"), import("../editor/replaceCapturedBlock"), import("../styles.css?inline"), import("katex/dist/katex.min.css?inline"),
  ]);
  return { prose, rendered, math, spans, hick, elements, edit, css, katexCss };
}
</ui:copy>

<ui:file path="apps/web/src/simple/RichInline.tsx"><ui:paste select="#simple-inline-browser-owners"/></ui:file>

</ui:doc>
