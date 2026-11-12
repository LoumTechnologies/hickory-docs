import Foundation
struct InlineMark: Sendable { var kind: String; var range: NSRange; var level: Int = 0 }
struct InlineCell: Identifiable, Sendable { var line: Int; var column: Int; var range: NSRange; var value: String; var format = "markdown"; var delimiter = ","; var id: Int { range.location } }
struct InlineEquation: Identifiable, Sendable { var line: Int; var range: NSRange; var body: String; var id: Int { range.location } }
struct InlineDiagram: Identifiable, Sendable { var line: Int; var range: NSRange; var body: String; var renderer: String; var id: Int { range.location } }
struct InlineSnapshot: Sendable { var diagrams: [InlineDiagram] = []; var source: String; var lineStarts: [Int] = [0]; var marks: [InlineMark] = []; var cells: [InlineCell] = []; var equations: [InlineEquation] = []; var debug = InlineDebugContext() }
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
    result.diagrams = snapshot.diagrams.compactMap { diagram in
        var diagram = diagram
        if NSMaxRange(diagram.range) <= prefix { }
        else if diagram.range.location >= end { diagram.range.location += delta }
        else { return nil }
        diagram.line = inlineLineAt(diagram.range.location, starts: result.lineStarts)
        return diagram
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
    var fence = "", mode = "", target: String?, oldLines = 0, newLines = 0
    let opening = try! NSRegularExpression(pattern: #"^ {0,3}<[\w.-]+:(file|patch|copy|cut|exec|entry|diagram|math|table)\b[^>]*>"#)
    let pathHint = try! NSRegularExpression(pattern: #"\bpath\s*=\s*["']([^"']+)["']"#)
    let hunk = try! NSRegularExpression(pattern: #"^@@ -\d+(?:,(\d+))? \+\d+(?:,(\d+))? @@"#)
    for (index, row) in rows.enumerated() {
        let trimmed = row.trimmingCharacters(in: .whitespacesAndNewlines), text = row as NSString
        if !fence.isEmpty {
            if trimmed.count >= fence.count, trimmed.allSatisfy({ $0 == fence.first! }) { fence = ""; mode = ""; target = nil; continue }
        } else if !mode.isEmpty {
            if trimmed.contains("</"), trimmed.contains(":" + mode + ">") { mode = ""; target = nil; continue }
        } else {
            var declaration = ""
            if let marker = row.range(of: #"^ {0,3}(`{3,}|~{3,})"#, options: .regularExpression) {
                fence = String(row[marker]).trimmingCharacters(in: .whitespaces)
                declaration = String(row[marker.upperBound...]); mode = declaration.trimmingCharacters(in: .whitespaces).components(separatedBy: " ")[0] == "patch" ? "patch" : "file"
            } else if let match = opening.firstMatch(in: row, range: NSRange(location: 0, length: text.length)) {
                mode = text.substring(with: match.range(at: 1)); declaration = text.substring(with: match.range)
            } else { continue }
            oldLines = 0; newLines = 0; target = nil
            if mode == "file", let match = pathHint.firstMatch(in: declaration, range: NSRange(location: 0, length: declaration.utf16.count)) {
                let path = (declaration as NSString).substring(with: match.range(at: 1))
                target = supported.contains((path as NSString).pathExtension.lowercased()) ? path : nil
                result.launch[index] = target
            }
            continue
        }
        if mode == "file", !trimmed.isEmpty { result.code[index] = target }
        if mode == "patch" {
            if oldLines > 0 || newLines > 0 {
                if row.hasPrefix("+") { newLines -= 1; if !row.dropFirst().trimmingCharacters(in: .whitespaces).isEmpty { result.code[index] = target } }
                else if row.hasPrefix("-") { oldLines -= 1 }
                else if row.hasPrefix(" ") { oldLines -= 1; newLines -= 1 }
            } else if row.hasPrefix("+++ ") {
                let raw = String(row.dropFirst(4)).components(separatedBy: "\t")[0]
                let path = raw.hasPrefix("b/") ? String(raw.dropFirst(2)) : raw
                target = supported.contains((path as NSString).pathExtension.lowercased()) ? path : nil
                result.launch[index] = target
            } else if let match = hunk.firstMatch(in: row, range: NSRange(location: 0, length: text.length)) {
                oldLines = match.range(at: 1).location == NSNotFound ? 1 : Int(text.substring(with: match.range(at: 1))) ?? 0
                newLines = match.range(at: 2).location == NSNotFound ? 1 : Int(text.substring(with: match.range(at: 2))) ?? 0
            }
        }
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
// Local lexical colors need only the language hint, never an invented file URI.
// Container bodies are shell programs, just like exec bodies; their names do not imply file ownership.
func inlineSyntaxMarks(_ source: String, path: String) -> [InlineMark] {
    let markdown = ["md", "markdown"].contains((path as NSString).pathExtension.lowercased())
    let known = Set(["python", "py", "javascript", "js", "typescript", "ts", "jsx", "tsx", "rust", "rs", "go", "c", "cpp", "java", "cs", "kotlin", "kt", "swift", "ruby", "rb", "sh", "bash", "json", "yaml", "yml", "toml", "sql", "css", "html", "xml", "php"])
    let keywords = Set("and as assert async await break case catch class const continue def default del do elif else enum except export extends false False finally fn for from fun func function global if impl import in interface is lambda let match module mut new nil None not null or package pass private public raise return self static struct super switch this throw trait true True try type typeof use val var void when while with yield".split(separator: " ").map(String.init))
    let open = try! NSRegularExpression(pattern: #"^\s*<[\w.-]+:(file|copy|cut|exec|container|entry|patch|diagram|math|table)\b[^>]*>"#)
    let hint = try! NSRegularExpression(pattern: #"\b(language|path)\s*=\s*["']([^"']+)["']"#)
    var result: [InlineMark] = [], fence = "", payload = "", language = markdown ? "" : (path as NSString).pathExtension.lowercased(), offset = 0
    for row in source.components(separatedBy: "\n") {
        let text = row as NSString, trimmed = row.trimmingCharacters(in: .whitespacesAndNewlines)
        defer { offset += text.length + 1 }
        if markdown {
            if !fence.isEmpty {
                if trimmed.count >= fence.count, trimmed.allSatisfy({ $0 == fence.first! }) { fence = ""; language = ""; continue }
            } else if !payload.isEmpty {
                if trimmed.contains("</"), trimmed.contains(":" + payload + ">") { payload = ""; language = ""; continue }
            } else {
                if let marker = row.range(of: #"^ {0,3}(`{3,}|~{3,})"#, options: .regularExpression) {
                    fence = String(row[marker]).trimmingCharacters(in: .whitespaces)
                    language = String(row[marker.upperBound...]).split(whereSeparator: { $0.isWhitespace }).first.map(String.init)?.lowercased() ?? ""; continue
                }
                if let match = open.firstMatch(in: row, range: NSRange(location: 0, length: text.length)) {
                    payload = text.substring(with: match.range(at: 1)); language = ["exec", "container"].contains(payload) ? "sh" : ""
                    let hints = hint.matches(in: row, range: match.range)
                    for item in hints {
                        let value = text.substring(with: item.range(at: 2)).lowercased()
                        if text.substring(with: item.range(at: 1)) == "language" { language = value; break }
                        language = (value as NSString).pathExtension
                    }
                    if trimmed.hasSuffix("/>") || trimmed.contains("</") { payload = ""; language = "" }
                    continue
                }
                continue
            }
        }
        guard known.contains(language) else { continue }
        let hashComments = ["python", "py", "sh", "bash", "ruby", "rb", "yaml", "yml", "toml"].contains(language)
        let pattern = (hashComments ? #"#[^\n]*|"# : #"//[^\n]*|"#) + #""(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|`(?:\\.|[^`\\])*`|\b[0-9]+(?:\.[0-9]+)?\b|\b[A-Za-z_][A-Za-z_0-9]*\b"#
        let lexer = try! NSRegularExpression(pattern: pattern)
        for match in lexer.matches(in: row, range: NSRange(location: 0, length: text.length)) {
            let token = text.substring(with: match.range), kind: String
            if token.hasPrefix("#") || token.hasPrefix("//") { kind = "comment" }
            else if token.hasPrefix("\"") || token.hasPrefix("'") || token.hasPrefix("`") { kind = "string" }
            else if token.first?.isNumber == true { kind = "number" }
            else if keywords.contains(token) { kind = "keyword" }
            else { continue }
            result.append(InlineMark(kind: "syntax-" + kind, range: NSRange(location: offset + match.range.location, length: match.range.length)))
        }
    }
    return result
}
func inlineDiagrams(_ source: String) -> [InlineDiagram] {
    let open = try! NSRegularExpression(pattern: #"^ {0,3}<[\w.-]+:(file|copy|cut|exec|container|entry|patch|diagram|math|table)\b[^>]*>"#)
    let renderer = try! NSRegularExpression(pattern: #"\brenderer\s*=\s*["']([^"']+)["']"#)
    var result: [InlineDiagram] = [], offset = 0, fence = "", payload = "", bodyStart = 0, blockStart = 0, blockLine = 0, engine = "mermaid"
    for (line, row) in source.components(separatedBy: "\n").enumerated() {
        let text = row as NSString, trimmed = row.trimmingCharacters(in: .whitespacesAndNewlines)
        defer { offset += text.length + 1 }
        if !fence.isEmpty { if trimmed.count >= fence.count, trimmed.allSatisfy({ $0 == fence.first! }) { fence = "" }; continue }
        if payload.isEmpty {
            if let marker = row.range(of: #"^ {0,3}(`{3,}|~{3,})"#, options: .regularExpression) { fence = String(row[marker]).trimmingCharacters(in: .whitespaces); continue }
            guard let match = open.firstMatch(in: row, range: NSRange(location: 0, length: text.length)) else { continue }
            payload = text.substring(with: match.range(at: 1)); blockStart = offset; blockLine = line; bodyStart = offset + NSMaxRange(match.range)
            engine = renderer.firstMatch(in: row, range: match.range).map { text.substring(with: $0.range(at: 1)) } ?? "mermaid"
            if trimmed.hasSuffix("/>") { payload = ""; continue }
        }
        if let close = row.range(of: "</"), row[close.lowerBound...].contains(":" + payload + ">") {
            let bodyEnd = offset + row[..<close.lowerBound].utf16.count
            if payload == "diagram", bodyEnd >= bodyStart {
                result.append(InlineDiagram(line: blockLine, range: NSRange(location: blockStart, length: offset + text.length - blockStart), body: (source as NSString).substring(with: NSRange(location: bodyStart, length: bodyEnd - bodyStart)), renderer: engine))
            }
            payload = ""
        }
    }
    return result
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
        if trimmed.hasPrefix("```") || trimmed.hasPrefix("~~~") { fence = String(trimmed.prefix(3)); diffFence = ["diff", "patch"].contains(String(trimmed.drop(while: { $0 == trimmed.first! })).trimmingCharacters(in: .whitespaces).components(separatedBy: " ")[0]); table = false; continue }
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
    result.marks += inlineSyntaxMarks(source, path: path)
    for (line, target) in result.debug.code where rows.indices.contains(line) && rows[line].hasPrefix("+") {
        let start = result.lineStarts[line] + 1
        result.marks += inlineSyntaxMarks(String(rows[line].dropFirst()), path: target).map { var mark = $0; mark.range.location += start; return mark }
    }
    result.diagrams = inlineDiagrams(source)
    return result
}
