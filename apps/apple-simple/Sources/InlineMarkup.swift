import Foundation
struct InlineMark: Sendable { var kind: String; var range: NSRange; var level: Int = 0 }
struct InlineCell: Identifiable, Sendable { var line: Int; var column: Int; var range: NSRange; var value: String; var format = "markdown"; var delimiter = ","; var id: Int { range.location } }
struct InlineEquation: Identifiable, Sendable { var line: Int; var range: NSRange; var body: String; var id: Int { range.location } }
struct InlineSnapshot: Sendable { var source: String; var lineStarts: [Int] = [0]; var marks: [InlineMark] = []; var cells: [InlineCell] = []; var equations: [InlineEquation] = [] }
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
func inlineLineAt(_ offset: Int, starts: [Int]) -> Int {
    var low = 0, high = starts.count
    while low < high { let middle = (low + high) / 2; if starts[middle] <= offset { low = middle + 1 } else { high = middle } }
    return max(0, low - 1)
}
func computeInlineMarkup(_ source: String, path: String) -> InlineSnapshot {
    var starts = [0]
    for (index, unit) in source.utf16.enumerated() { if unit == 10 { starts.append(index + 1) } }
    var result = InlineSnapshot(source: source, lineStarts: starts)
    guard ["md", "markdown"].contains((path as NSString).pathExtension.lowercased()), source.utf16.count <= 200_000 else { return result }
    let rows = source.components(separatedBy: "\n")
    let separator = try! NSRegularExpression(pattern: #"^\s*\|?\s*:?-{3,}:?\s*(\|\s*:?-{3,}:?\s*)+\|?\s*$"#)
    let heading = try! NSRegularExpression(pattern: #"^(#{1,6})\s+"#)
    let styles = [("code", #"`[^`\n]+`"#), ("strong", #"\*\*[^*\n]+\*\*"#), ("em", #"(?<!\*)\*[^*\n]+\*(?!\*)"#), ("math", #"(?<!\\)\$([^$\n]+)\$"#)].map { ($0.0, try! NSRegularExpression(pattern: $0.1)) }
    let opening = try! NSRegularExpression(pattern: #"^\s*<[A-Za-z_][\w.-]*:(file|copy|cut|exec|diagram|math|table)\b[^>]*>"#)
    var offset = 0, fence = "", payload = "", table = false
    var equationStart: Int?, equationLine = 0
    var tableStart: Int?, tableLine = 0, delimiter = ","
    for (line, rawRow) in rows.enumerated() {
        defer { offset += rawRow.utf16.count + 1 }
        let row = rawRow.hasSuffix("\r") ? String(rawRow.dropLast()) : rawRow
        if Task.isCancelled { return InlineSnapshot(source: source) }
        let trimmed = row.trimmingCharacters(in: .whitespaces), ns = row as NSString, all = NSRange(location: 0, length: row.utf16.count)
        if !fence.isEmpty { if trimmed.hasPrefix(fence) { fence = "" }; continue }
        if trimmed.hasPrefix("```") || trimmed.hasPrefix("~~~") { fence = String(trimmed.prefix(3)); table = false; continue }
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
