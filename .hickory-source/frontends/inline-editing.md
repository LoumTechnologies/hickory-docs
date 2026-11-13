<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Add inline presentation without replacing source

For maintainers implementing rich source editing on native platforms.

Compute a line-start index and presentation against immutable UTF-16 text. Visible gutters use binary search into the current snapshot instead of repeatedly copying and counting the entire prefix on every scrolled line. Keep Markdown delimiters and original bytes in the editor, skip fenced and Hickory payload bodies, and expose exact cell/equation ranges for guarded contextual edits. Large documents retain plain source while this optional scanner is bounded to 200,000 UTF-16 units.

CSV declarations and standalone CSV files use the same grid as prose tables. Group cells only while the intervening source consists of delimiters and record separators; tags between two declarations keep their grids separate. Preserve each cell’s format and delimiter on commit so commas, quotes and embedded newlines round-trip. Revealing a table records its anchor independently of the other grids.

Markdown prose tables now use those exact cell ranges to form whole-table presentation ranges. Keep the final newline outside the replacement so following prose retains its paragraph. Header separators belong to the replacement, but never become editable data cells. Only cells admitted by the prose or CSV scanner can enter a grid; fenced code and Hickory literal bodies stay source. The Swift and Kotlin range witnesses check CRLF, emoji and escaped pipes.

Captured program bytes and authored expectations share a read-only terminal
screen. The browser binds its existing watching emulator; native previews load
the same locked xterm package from local resources. No channel names, source
locations or prompts are inserted into those bytes. Source metadata remains in
the debugger controller for positioning. An expected terminal represents authored
bytes, not evidence of a successful run. Its explicit source action retains
normal guarded editing and undo. The preview scanner admits expect children of
exec, skips literal files/fragments and fences, and retains exact source ranges.

Diagram previews use the existing isolated native WebView and a local, locked
Mermaid bundle; graph JSON is drawn as SVG using text nodes for labels. The body
is passed as JSON rather than executable script. A monotonically increasing
render ticket rejects obsolete Mermaid results. Native WebViews skip identical
source/renderer updates so typing elsewhere cannot restart an unchanged diagram. Invalid and unsupported diagrams
show their source and a status message. Network navigation remains disabled.
Browser slots include the graph canvas stylesheet inside their shadow root; a stylesheet on the outer document cannot style that canvas. Native editors reserve a fixed preview rectangle and offer explicit source mode;
rendering never resizes the editor or mutates the diagram. Fenced examples and
other Hickory literal bodies do not create live diagrams.

Ordinary diff fences remain review text. Explicit patch fences are project constructions: offer launch beside each supported destination header and pending breakpoints on added code rows. Hunk counts distinguish destination headers from added strings beginning with plus signs. Deleted and context rows do not acquire new-source ownership; the project mapper remains authoritative about whether added bytes survive later patches. Local syntax colors the added code without its diff prefix. Path-bearing ordinary code fences use the same file context as file tags.

While a new parse is pending, map unaffected table cells through the text change.
This is presentation only: keep debugging and equation actions on exact snapshots.
A newline outside a table must not temporarily turn a grid back into source and
change the document height. Drop cells touched by a structural edit; the next
background parse discovers their new structure. Cell-value edits retain their
range and decode the new value, including empty values.

Known fenced-code languages receive local lexical colors in the background,
without assigning a fictitious workspace path or waiting for LSP installation.
The same fallback colors file/copy bodies; semantic server colors remain an
independent capability. Unknown languages retain literal editable source. These
are bounded lexical previews, not full grammar parsing or project diagnostics.

The Swift scanner returns source-backed styles and cell ranges. Escaped pipes and code-span pipes cannot create extra columns; applying a cell edit escapes literal pipes and retains all untouched bytes.

Whole-table structural edits retain a temporary presentation anchor until the background scanner catches up. Old cell rows describe presentation only; the shared spreadsheet writes the exact current table range through the host guard. Keep the Markdown number-format comment inside that replacement and admit a zero-length first cell for an empty standalone CSV. Split HTML comment delimiters in source-language strings so the literate compiler does not treat them as document comments.

<ui:copy id="simple-inline-model-0">
import Foundation
struct InlineMark: Sendable { var kind: String; var range: NSRange; var level: Int = 0 }
struct InlineCell: Identifiable, Sendable { var line: Int; var column: Int; var range: NSRange; var value: String; var format = "markdown"; var delimiter = ","; var id: Int { range.location } }
struct InlineEquation: Identifiable, Sendable {
    var line: Int; var range: NSRange; var body: String
    // Edits target the body; presentation replaces the complete authored syntax.
    var presentationRange: NSRange? = nil
    var displayBlock = false
    var renderedRange: NSRange { presentationRange ?? range }
    var id: Int { range.location }
}
struct InlineDiagram: Identifiable, Sendable { var line: Int; var range: NSRange; var body: String; var renderer: String; var id: Int { range.location } }
struct InlineSnapshot: Sendable { var diagrams: [InlineDiagram] = []; var source: String; var lineStarts: [Int] = [0]; var marks: [InlineMark] = []; var cells: [InlineCell] = []; var equations: [InlineEquation] = []; var debug = InlineDebugContext(); var gridOverrides: [InlineTable] = [] }
// Tables are presentation ranges over the original source, never replacement text.
struct InlineTable: Identifiable, Sendable {
    var range: NSRange; var rows: [[InlineCell]]
    var id: Int { range.location }
    var preferredHeight: CGFloat? = nil
    var height: CGFloat { preferredHeight ?? max(280, min(420, CGFloat(rows.count + 5) * 30)) }
}
func tablePresentation(_ snapshot: InlineSnapshot, source: String) -> InlineSnapshot {
    if snapshot.source == source { return snapshot }
    let old = Array(snapshot.source.utf16), new = Array(source.utf16)
    var prefix = 0, suffix = 0
    while prefix < min(old.count, new.count), old[prefix] == new[prefix] { prefix += 1 }
    while suffix < min(old.count, new.count) - prefix, old[old.count - suffix - 1] == new[new.count - suffix - 1] { suffix += 1 }
    let end = old.count - suffix, delta = new.count - old.count, text = source as NSString
    var result = InlineSnapshot(source: source)
    // Preserve a changed grid's anchor until the background scanner supplies new cells.
    // These old rows grant no write authority; the host edits the exact new range.
    result.gridOverrides = inlineTables(snapshot).compactMap { table in
        let tail = NSMaxRange(table.range)
        let insertionAfterNewline = prefix == end && prefix >= tail && prefix - tail <= 2 && (snapshot.source as NSString).substring(with: NSRange(location: tail, length: prefix - tail)).allSatisfy { $0 == "\r" || $0 == "\n" }
        guard prefix >= table.range.location, (end <= tail || insertionAfterNewline), end >= table.range.location,
              ((snapshot.source as NSString).substring(with: NSRange(location: prefix, length: end - prefix)) + (source as NSString).substring(with: NSRange(location: prefix, length: new.count - suffix - prefix))).contains(where: { "\n,|".contains($0) }) else { return nil }
        var table = table; table.range.length += delta
        return table.range.length > 0 ? table : nil
    }
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
        var start = snapshot.lineStarts[line]
        if line > 0 {
            let previous = source.substring(with: NSRange(location: snapshot.lineStarts[line - 1], length: start - snapshot.lineStarts[line - 1])).trimmingCharacters(in: .whitespacesAndNewlines)
            if previous.hasPrefix("<" + "!-- hickory-table-formats: ") && previous.hasSuffix(" --" + ">") { start = snapshot.lineStarts[line - 1] }
        }
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
    return (result.filter { table in !snapshot.gridOverrides.contains { NSIntersectionRange($0.range, table.range).length > 0 } } + snapshot.gridOverrides).sorted { $0.range.location < $1.range.location }
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
    let open = try! NSRegularExpression(pattern: #"^ {0,3}<[\w.-]+:(file|copy|cut|exec|container|entry|patch|diagram|math|table|expect)\b[^>]*>"#)
    let renderer = try! NSRegularExpression(pattern: #"\brenderer\s*=\s*["']([^"']+)["']"#)
    var result: [InlineDiagram] = [], offset = 0, fence = "", payload = "", bodyStart = 0, blockStart = 0, blockLine = 0, engine = "mermaid", inExec = false, renderable = false
    for (line, row) in source.components(separatedBy: "\n").enumerated() {
        let text = row as NSString, trimmed = row.trimmingCharacters(in: .whitespacesAndNewlines)
        defer { offset += text.length + 1 }
        if !fence.isEmpty { if trimmed.count >= fence.count, trimmed.allSatisfy({ $0 == fence.first! }) { fence = "" }; continue }
        if payload.isEmpty {
            if inExec && trimmed.hasPrefix("</") && trimmed.contains(":exec>") { inExec = false; continue }
            if !inExec, let marker = row.range(of: #"^ {0,3}(`{3,}|~{3,})"#, options: .regularExpression) { fence = String(row[marker]).trimmingCharacters(in: .whitespaces); continue }
            guard let match = open.firstMatch(in: row, range: NSRange(location: 0, length: text.length)) else { continue }
            payload = text.substring(with: match.range(at: 1))
            if payload == "exec" { inExec = !trimmed.hasSuffix("/>"); payload = ""; continue }
            renderable = payload == "expect" || payload == "diagram" && !inExec
            blockStart = offset; blockLine = line; bodyStart = offset + NSMaxRange(match.range)
            engine = renderer.firstMatch(in: row, range: match.range).map { text.substring(with: $0.range(at: 1)) } ?? "mermaid"
            if payload == "expect" { engine = "terminal" }
            if trimmed.hasSuffix("/>") {
                if payload == "expect" { result.append(InlineDiagram(line: line, range: NSRange(location: offset, length: text.length), body: "", renderer: "terminal")) }
                payload = ""; continue
            }
        }
        if let close = row.range(of: "</"), row[close.lowerBound...].contains(":" + payload + ">") {
            let bodyEnd = offset + row[..<close.lowerBound].utf16.count
            if renderable, bodyEnd >= bodyStart {
                var body = (source as NSString).substring(with: NSRange(location: bodyStart, length: bodyEnd - bodyStart))
                if payload == "expect" { body = body.replacingOccurrences(of: "^\\r?\\n", with: "", options: .regularExpression) }
                result.append(InlineDiagram(line: blockLine, range: NSRange(location: blockStart, length: offset + text.length - blockStart), body: body, renderer: engine))
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
        result.cells = source.isEmpty ? [InlineCell(line: 0, column: 0, range: NSRange(location: 0, length: 0), value: "", format: "csv")] : inlineCSVCells(source, line: 0, offset: 0, delimiter: ","); return result
    }
    guard ["md", "markdown"].contains((path as NSString).pathExtension.lowercased()), source.utf16.count <= 200_000 else { return result }
    let rows = source.components(separatedBy: "\n")
    let separator = try! NSRegularExpression(pattern: #"^\s*\|?\s*:?-{3,}:?\s*(\|\s*:?-{3,}:?\s*)+\|?\s*$"#)
    let heading = try! NSRegularExpression(pattern: #"^(#{1,6})\s+"#)
    let styles = [("code", #"`[^`\n]+`"#), ("strong", #"\*\*[^*\n]+\*\*"#), ("em", #"(?<!\*)\*[^*\n]+\*(?!\*)"#), ("math", #"(?<!\\)\$([^$\n]+)\$"#)].map { ($0.0, try! NSRegularExpression(pattern: $0.1)) }
    let opening = try! NSRegularExpression(pattern: #"^\s*<[A-Za-z_][\w.-]*:(file|copy|cut|exec|container|diagram|math|table)\b[^>]*>"#)
    var offset = 0, fence = "", payload = "", table = false, diffFence = false
    var equationStart: Int?, equationLine = 0, equationOpen = 0
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
                    let closeEnd = row[close.lowerBound...].firstIndex(of: ">").map { row[..<row.index(after: $0)].utf16.count } ?? ns.length
                    if end >= start { let range = NSRange(location: start, length: end - start); result.equations.append(InlineEquation(line: equationLine, range: range, body: (source as NSString).substring(with: range), presentationRange: NSRange(location: equationOpen, length: offset + closeEnd - equationOpen), displayBlock: true)) }
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
            if payload == "math" { equationStart = offset + NSMaxRange(match.range); equationLine = line; equationOpen = offset + match.range.location }
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
                let closeEnd = row[close.lowerBound...].firstIndex(of: ">").map { row[..<row.index(after: $0)].utf16.count } ?? ns.length
                if end >= start { let range = NSRange(location: start, length: end - start); result.equations.append(InlineEquation(line: line, range: range, body: (source as NSString).substring(with: range), presentationRange: NSRange(location: equationOpen, length: offset + closeEnd - equationOpen), displayBlock: true)) }
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
                if kind == "math" { let body = match.range(at: 1); result.equations.append(InlineEquation(line: line, range: NSRange(location: offset + body.location, length: body.length), body: ns.substring(with: body), presentationRange: NSRange(location: offset + match.range.location, length: match.range.length))) }
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

// Only a declaration directly before this body supplies table attributes.
func inlineTableAttribute(_ name: String, table: InlineTable, source: String) -> String? {
    guard table.rows.first?.first?.format == "csv", table.range.location <= (source as NSString).length else { return nil }
    let prefix = (source as NSString).substring(to: table.range.location) as NSString
    let tag = try! NSRegularExpression(pattern: #"<[\w.-]+:table\b[^>]*>"#)
    guard let match = tag.matches(in: prefix as String, range: NSRange(location: 0, length: prefix.length)).last,
          prefix.substring(from: NSMaxRange(match.range)).trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return nil }
    let declaration = prefix.substring(with: match.range) as NSString
    let attr = try! NSRegularExpression(pattern: "\\b" + NSRegularExpression.escapedPattern(for: name) + #"\s*=\s*["']([^"']*)["']"#)
    return attr.firstMatch(in: declaration as String, range: NSRange(location: 0, length: declaration.length)).map { declaration.substring(with: $0.range(at: 1)) }
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/InlineMarkup.swift"><ui:paste select="#simple-inline-model-0"/></ui:file>

The Kotlin scanner returns source-backed styles and cell ranges. Escaped pipes and code-span pipes cannot create extra columns; applying a cell edit escapes literal pipes and retains all untouched bytes.

<ui:copy id="simple-inline-model-1">
package com.hickorydocs.simple

data class InlineMark(val kind: String, val from: Int, val to: Int, val level: Int = 0)
data class InlineCell(val line: Int, val column: Int, val from: Int, val to: Int, val value: String, val format: String = "markdown", val delimiter: String = ",")
data class InlineEquation(val line: Int, val from: Int, val to: Int, val body: String)
data class InlineDiagram(val line: Int, val from: Int, val to: Int, val body: String, val renderer: String)
data class InlineSnapshot(val source: String, val lineStarts: List<Int> = listOf(0), val marks: List<InlineMark> = emptyList(), val cells: List<InlineCell> = emptyList(), val equations: List<InlineEquation> = emptyList(), val debug: InlineDebugContext = InlineDebugContext(), val diagrams: List<InlineDiagram> = emptyList(), val gridOverrides: List<InlineTable> = emptyList())
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
    var fence = ""; var mode = ""; var target: String? = null; var oldLines = 0; var newLines = 0
    val opening = Regex("""^ {0,3}<[\w.-]+:(file|patch|copy|cut|exec|entry|diagram|math|table)\b[^>]*>""")
    val pathHint = Regex("""\bpath\s*=\s*["']([^"']+)["']""")
    val hunk = Regex("""^@@ -\d+(?:,(\d+))? \+\d+(?:,(\d+))? @@""")
    for ((index, row) in rows.withIndex()) {
        val trimmed = row.trim()
        if (fence.isNotEmpty()) {
            if (trimmed.length >= fence.length && trimmed.all { it == fence[0] }) { fence = ""; mode = ""; target = null; continue }
        } else if (mode.isNotEmpty()) {
            if (trimmed.contains("</") && trimmed.contains(":" + mode + ">")) { mode = ""; target = null; continue }
        } else {
            val marker = Regex("""^ {0,3}(`{3,}|~{3,})""").find(row)
            val declaration: String
            if (marker != null) { fence = marker.value.trim(); declaration = row.substring(marker.range.last + 1); mode = if (declaration.trim().substringBefore(' ') == "patch") "patch" else "file" }
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
                if (row.startsWith('+')) { newLines--; if (row.drop(1).isNotBlank()) target?.let { code[index] = it } }
                else if (row.startsWith('-')) oldLines--
                else if (row.startsWith(' ')) { oldLines--; newLines-- }
            } else if (row.startsWith("+++ ")) {
                val path = row.drop(4).substringBefore('\t').removePrefix("b/")
                target = path.takeIf { it.substringAfterLast('.').lowercase() in supported }
                target?.let { launch[index] = it }
            } else hunk.find(row)?.let { match -> oldLines = match.groupValues[1].ifEmpty { "1" }.toIntOrNull() ?: 0; newLines = match.groupValues[2].ifEmpty { "1" }.toIntOrNull() ?: 0 }
        }
    }
    return InlineDebugContext(code, launch)
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
fun computeInlineMarkup(source: String, path: String): InlineSnapshot {
    val starts = mutableListOf(0); source.forEachIndexed { index, char -> if (char == '\n') starts.add(index + 1) }
    val debug = inlineDebugContext(source, path)
    if (path.substringAfterLast('.', "").lowercase() == "csv" && source.length <= 200_000) return InlineSnapshot(source, lineStarts = starts, cells = if (source.isEmpty()) listOf(InlineCell(0, 0, 0, 0, "", "csv")) else inlineCSVCells(source, 0, 0, ","), debug = debug)
    if (path.substringAfterLast('.', "").lowercase() !in listOf("md", "markdown") || source.length > 200_000) return InlineSnapshot(source, lineStarts = starts, debug = debug)
    val rows = source.split('\n'); val marks = mutableListOf<InlineMark>(); val cells = mutableListOf<InlineCell>(); val equations = mutableListOf<InlineEquation>()
    val separator = Regex("^\\s*\\|?\\s*:?-{3,}:?\\s*(\\|\\s*:?-{3,}:?\\s*)+\\|?\\s*$")
    val heading = Regex("^(#{1,6})\\s+")
    val styles = listOf("code" to Regex("`[^`\\n]+`"), "strong" to Regex("\\*\\*[^*\\n]+\\*\\*"), "em" to Regex("(?<!\\*)\\*[^*\\n]+\\*(?!\\*)"), "math" to Regex("(?<!\\\\)\\$([^$\\n]+)\\$"))
    val opening = Regex("^\\s*<[A-Za-z_][\\w.-]*:(file|copy|cut|exec|container|diagram|math|table)\\b[^>]*>")
    var offset = 0; var fence = ""; var payload = ""; var table = false; var diffFence = false; var equationStart: Int? = null; var equationLine = 0; var tableStart: Int? = null; var tableLine = 0; var delimiter = ","
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
    val patchMarks = debug.code.flatMap { (line, target) ->
        if (rows.getOrNull(line)?.startsWith('+') == true) inlineSyntaxMarks(rows[line].drop(1), target).map { it.copy(from = it.from + starts[line] + 1, to = it.to + starts[line] + 1) } else emptyList()
    }
    return InlineSnapshot(source, starts, marks + inlineSyntaxMarks(source, path) + patchMarks, cells, equations, debug, inlineDiagrams(source))
}

fun inlineTableAttribute(name: String, table: InlineTable, source: String): String? {
    if (table.rows.firstOrNull()?.firstOrNull()?.format != "csv" || table.from > source.length) return null
    val prefix = source.substring(0, table.from)
    val tag = Regex("""<[\w.-]+:table\b[^>]*>""").findAll(prefix).lastOrNull() ?: return null
    if (prefix.substring(tag.range.last + 1).isNotBlank()) return null
    return Regex("""\b${Regex.escape(name)}\s*=\s*["']([^"']*)["']""").find(tag.value)?.groupValues?.get(1)
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/InlineMarkup.kt"><ui:paste select="#simple-inline-model-1"/></ui:file>


The local math page loads only bundled KaTeX and fonts. Formula strings enter through a JSON argument, never HTML interpolation. KaTeX trust is disabled and the page CSP denies remote resources; malformed or oversized equations retain readable source.

The interactive terminal binding below explicitly enables stdin and the cursor. It sends only input and measured size messages to a native host; transport and session identity stay outside this local page. PTY output arrives as base64 bytes so UTF-8 and ANSI sequences split across messages remain intact in xterm’s parser. The existing recorded-output binding keeps stdin disabled.

<ui:copy id="simple-inline-math-0">
<!doctype html>
<html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'self' 'nonce-hickory-math'; style-src 'self' 'unsafe-inline'; font-src 'self' data:; img-src data:;">
<link rel="stylesheet" href="node_modules/xterm.css"><link rel="stylesheet" href="katex/katex.min.css"><style>html{height:100%;color-scheme:light dark;font:14px system-ui}body{margin:0;height:100%;overflow:hidden}body.terminal-mode{color-scheme:dark;height:100%;margin:0;background:#10151d;color:#e6edf3;overflow:hidden}.terminal-mode #formula{position:static;width:auto;transform:none!important;height:100%;box-sizing:border-box;padding:6px;overflow:hidden}.terminal-mode:not(.interactive-terminal) .xterm-cursor-layer{display:none}#formula{position:absolute;width:max-content;padding:8px;transform-origin:top left;box-sizing:border-box}#status{position:absolute;bottom:0;font-size:11px}#formula .katex-display{margin:0}.inline-math{font-size:13px}.inline-math #formula{padding:1px}.display-math #formula{padding:4px}</style>
</head><body><div id="formula" aria-label="Typeset equation"></div><div id="status" role="status"></div>
<script nonce="hickory-math" src="node_modules/xterm.js"></script><script nonce="hickory-math" src="node_modules/addon-fit.js"></script><script nonce="hickory-math" src="katex/katex.min.js"></script><script nonce="hickory-math" src="node_modules/mermaid.min.js"></script><script nonce="hickory-math">
if (globalThis.hickoryTableMode || location.search === '?table') {
  document.body.className = 'table-mode'; document.getElementById('formula').setAttribute('aria-label', 'Inline table grid');
  const style = document.createElement('style'); style.textContent = '#formula{position:static!important;width:100%!important;padding:0;transform:none!important;box-sizing:border-box}#status{display:none}body{overflow:auto}#formula.native-table-fill{height:100vh}.native-table-fill>.table-panel{height:100%;margin:0;display:flex;flex-direction:column}.native-table-fill .table-panel__scroll{flex:1;min-height:0}'; document.head.append(style);
  const css = document.createElement('link'); css.rel = 'stylesheet'; css.href = 'node_modules/table.css'; document.head.append(css);
  const script = document.createElement('script'); script.src = 'node_modules/table.js'; script.nonce = 'hickory-math'; document.head.append(script);
}
// Transforms do not change layout dimensions, avoiding a ResizeObserver feedback loop.
function fitPreview() {
  if (document.body.classList.contains('notebook-mode') || document.body.classList.contains('terminal-mode') || document.body.classList.contains('table-mode')) return;
  const node = document.getElementById('formula');
  const width = node.offsetWidth, height = node.offsetHeight;
  if (globalThis.mathRevision !== undefined && document.querySelector('.katex')) {
    globalThis.webkit?.messageHandlers?.mathSize?.postMessage({ revision: globalThis.mathRevision, width, height });
  }
  const availableHeight = Math.max(1, innerHeight - document.getElementById('status').offsetHeight);
  const scale = Math.min(1, innerWidth / Math.max(1, width), availableHeight / Math.max(1, height));
  node.style.transform = `scale(${scale})`;
  node.style.left = Math.max(0, (innerWidth - width * scale) / 2) + 'px';
  node.style.top = Math.max(0, (availableHeight - height * scale) / 2) + 'px';
}
const previewResize = new ResizeObserver(fitPreview);
previewResize.observe(document.getElementById('formula'));
previewResize.observe(document.getElementById('status'));
addEventListener('resize', fitPreview);
document.fonts.ready.then(fitPreview);
globalThis.renderMath = function (formula, inline = false, revision = 0) {
  globalThis.mathRevision = revision;
  document.body.classList.remove('terminal-mode', 'inline-math', 'display-math');
  document.body.classList.add(inline ? 'inline-math' : 'display-math');
  const node = document.getElementById('formula'), status = document.getElementById('status');
  node.textContent = formula;
  if (typeof katex === 'undefined') { status.textContent = 'Math engine unavailable. Run npm --prefix apps/web ci and python3 scripts/prepare-simple-resources.py, then rebuild.'; return; }
  if (formula.length > 4000) { status.textContent = 'Equation is too large to typeset here. Its source remains editable.'; return; }
  try { katex.render(formula, node, {throwOnError:false, trust:false, strict:'warn', displayMode:!inline, maxExpand:1000}); status.textContent = ''; }
  catch (_) { node.textContent = formula; status.textContent = 'Check the LaTeX expression.'; }
  fitPreview();
};
// This screen has no input binding and never executes expected/output bytes.
let outputTerminal, outputFit, outputBytes = '', outputResize;
globalThis.renderNotebookOutput = function (source) {
  globalThis.hickoryOutputSource = source;
  if (globalThis.hickoryOutput) { globalThis.hickoryOutput.update(source); return; }
  document.body.className = 'notebook-mode';
  const node = document.getElementById('formula'); node.textContent = source;
  node.style.cssText = 'position:static;width:100%;height:100%;overflow:auto;transform:none;padding:4px';
  if (globalThis.hickoryOutputLoading) return;
  globalThis.hickoryOutputLoading = true;
  const css = document.createElement('link'); css.rel = 'stylesheet'; css.href = 'node_modules/table.css'; document.head.append(css);
  const script = document.createElement('script'); script.src = 'node_modules/table.js'; script.nonce = 'hickory-math';
  script.onerror = () => { document.getElementById('status').textContent = 'Table renderer unavailable; output is shown as text.'; }; document.head.append(script);
};
// Interactive panes share xterm with output previews, but only this explicit
// binding enables stdin. The host owns transport, session lifetime and errors.
let interactiveTerminal;
globalThis.startInteractiveTerminal = function () {
  if (interactiveTerminal) return;
  const node = document.getElementById('formula');
  document.body.className = 'terminal-mode interactive-terminal';
  node.setAttribute('aria-label', 'Terminal screen'); node.replaceChildren();
  const post = message => {
    if (globalThis.webkit?.messageHandlers?.terminal) webkit.messageHandlers.terminal.postMessage(message);
    else if (globalThis.TerminalHost) TerminalHost.postMessage(JSON.stringify(message));
  };
  interactiveTerminal = new Terminal({convertEol:false, cursorBlink:true, fontSize:13, scrollback:5000, theme:{background:'#10151d',foreground:'#e6edf3'}});
  const fit = new FitAddon.FitAddon(); interactiveTerminal.loadAddon(fit); interactiveTerminal.open(node);
  interactiveTerminal.onData(data => post({kind:'input', data}));
  let size = '';
  const measure = () => {
    if (!node.clientWidth || !node.clientHeight) return;
    fit.fit();
    const next = interactiveTerminal.rows + ':' + interactiveTerminal.cols;
    if (size !== next) { size = next; post({kind:'resize',rows:interactiveTerminal.rows,cols:interactiveTerminal.cols}); }
  };
  new ResizeObserver(measure).observe(node); measure();
};
globalThis.writeInteractiveTerminal = function (base64) {
  if (interactiveTerminal) interactiveTerminal.write(Uint8Array.from(atob(base64), c => c.charCodeAt(0)));
};
globalThis.renderTerminal = function (source) {
  const node = document.getElementById('formula'), status = document.getElementById('status');
  document.body.classList.add('terminal-mode'); status.textContent = '';
  if (!globalThis.Terminal) { node.textContent = source; status.textContent = 'Terminal renderer unavailable. Prepare resources and rebuild.'; return; }
  try {
  if (!outputTerminal) {
    node.replaceChildren();
    outputTerminal = new Terminal({ disableStdin:true, convertEol:true, cursorBlink:false, screenReaderMode:true, fontSize:12, fontFamily:'ui-monospace, monospace', rows:6, cols:80, scrollback:5000, theme:{background:'#10151d',foreground:'#e6edf3'} });
    outputFit = new FitAddon.FitAddon(); outputTerminal.loadAddon(outputFit); outputTerminal.open(node);
    outputResize = new ResizeObserver(() => { if (node.clientWidth && node.clientHeight) outputFit.fit(); }); outputResize.observe(node);
  }
  const append = source.startsWith(outputBytes);
  if (!append) outputTerminal.reset();
  outputTerminal.write(append ? source.slice(outputBytes.length) : source, () => { if (node.clientWidth && node.clientHeight) outputFit.fit(); });
  outputBytes = source;
  } catch (error) {
    outputResize?.disconnect(); try { outputTerminal?.dispose(); } catch (_) {}
    outputTerminal = undefined; outputBytes = '';
    node.textContent = source; status.textContent = 'Terminal unavailable: ' + (error.message || 'prepare resources and rebuild');
  }
};
globalThis.renderDiagram = async function (source, renderer) {
  const node = document.getElementById('formula'), status = document.getElementById('status');
  const revision = globalThis.diagramRevision = (globalThis.diagramRevision || 0) + 1;
  node.replaceChildren(); status.textContent = '';
  if (source.length > 200000) { status.textContent = 'Diagram is too large to preview. Edit its source.'; return; }
  try {
    if (renderer === 'graph') {
      const graph = JSON.parse(source), topology = graph.topology || {}, layout = graph.layout || {};
      const nodes = topology.nodes || [], edges = topology.edges || [];
      if (nodes.length > 500 || edges.length > 2000) throw Error('Diagram is too large to preview.');
      const ns = 'http://www.w3.org/2000/svg';
      const element = (name, attrs = {}, text) => { const el = document.createElementNS(ns, name); for (const [key, value] of Object.entries(attrs)) el.setAttribute(key, String(value)); if (text !== undefined) el.textContent = text; return el; };
      const boxes = new Map(nodes.map((n, i) => { const box = layout[n.id] || {}; return [n.id, { x: Number.isFinite(box.x) ? box.x : i % 4 * 170 + 10, y: Number.isFinite(box.y) ? box.y : Math.floor(i / 4) * 90 + 10, w: Number.isFinite(box.w) ? Math.max(30, box.w) : 140, h: Number.isFinite(box.h) ? Math.max(30, box.h) : 50 }]; }));
      const values = [...boxes.values()], minX = Math.min(0, ...values.map(b => b.x)) - 10, minY = Math.min(0, ...values.map(b => b.y)) - 10;
      const svg = element('svg', { viewBox: `${minX} ${minY} ${Math.max(200, ...values.map(b => b.x + b.w)) - minX + 10} ${Math.max(80, ...values.map(b => b.y + b.h)) - minY + 10}`, role: 'img', 'aria-label': 'Diagram preview', width: Math.max(200, ...values.map(b => b.x + b.w)) - minX + 10, height: Math.max(80, ...values.map(b => b.y + b.h)) - minY + 10 });
      const defs = element('defs'), marker = element('marker', { id: 'arrow', markerWidth: 8, markerHeight: 8, refX: 7, refY: 4, orient: 'auto' });
      marker.append(element('path', { d: 'M0,0 L8,4 L0,8 Z', fill: 'currentColor' })); defs.append(marker); svg.append(defs);
      for (const edge of edges) { const a = boxes.get(edge.from), b = boxes.get(edge.to); if (a && b) svg.append(element('line', { x1: a.x + a.w, y1: a.y + a.h / 2, x2: b.x, y2: b.y + b.h / 2, stroke: 'currentColor', 'marker-end': 'url(#arrow)' })); }
      for (const item of nodes) { const b = boxes.get(item.id); svg.append(element('rect', { ...b, width: b.w, height: b.h, rx: 6, fill: 'Canvas', stroke: 'currentColor' })); svg.append(element('text', { x: b.x + b.w / 2, y: b.y + b.h / 2, 'text-anchor': 'middle', 'dominant-baseline': 'middle', fill: 'currentColor' }, item.label || item.id)); }
      node.append(svg);
    } else if (renderer === 'mermaid') {
      if (!globalThis.mermaid) throw Error('Diagram engine unavailable. Prepare native resources and rebuild.');
      mermaid.initialize({ startOnLoad: false, securityLevel: 'strict', theme: 'neutral', maxTextSize: 200000 });
      const result = await mermaid.render('diagram-' + revision, source);
      if (revision === globalThis.diagramRevision) {
        node.innerHTML = result.svg;
        const svg = node.querySelector('svg'), box = svg?.viewBox.baseVal;
        if (box?.width && box?.height) { svg.style.maxWidth = 'none'; svg.setAttribute('width', box.width); svg.setAttribute('height', box.height); }
      }
    } else throw Error('No preview for renderer ' + renderer + '. Edit its source.');
  } catch (error) { if (revision === globalThis.diagramRevision) { node.textContent = source; status.textContent = error.message || 'Check the diagram source.'; } }
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
# Mermaid's UMD bundle is self-contained; native previews never fetch a CDN.
mermaid = ROOT / "apps/web/node_modules/mermaid/dist/mermaid.min.js"
if mermaid.is_file():
    destination = ROOT / "apps/apple-simple/Sources/Resources/node_modules/mermaid.min.js"
    destination.parent.mkdir(parents=True, exist_ok=True)
    data = mermaid.read_bytes()
    if not destination.is_file() or destination.read_bytes() != data:
        with tempfile.NamedTemporaryFile(dir=destination.parent, delete=False) as temporary:
            temporary.write(data)
            staged = temporary.name
        os.chmod(staged, 0o644)
        os.replace(staged, destination)
    print("Prepared local native diagram resources.")
# Reuse the same emulator for program output and authored expectations.
for package, original, name in [("@xterm/xterm", "lib/xterm.js", "xterm.js"), ("@xterm/xterm", "css/xterm.css", "xterm.css"), ("@xterm/addon-fit", "lib/addon-fit.js", "addon-fit.js")]:
    original = ROOT / "apps/web/node_modules" / package / original
    if not original.is_file():
        print("Terminal assets missing. Run npm --prefix apps/web ci, then prepare resources again."); continue
    destination = ROOT / "apps/apple-simple/Sources/Resources/node_modules" / name
    destination.parent.mkdir(parents=True, exist_ok=True)
    data = original.read_bytes()
    if not destination.is_file() or destination.read_bytes() != data:
        with tempfile.NamedTemporaryFile(dir=destination.parent, delete=False) as temporary:
            temporary.write(data); staged = temporary.name
        os.chmod(staged, 0o644); os.replace(staged, destination)

# Bundle the existing spreadsheet locally for WKWebView and Android WebView.
# esbuild is pinned by the web lockfile; no network or second formula engine.
import json
import subprocess
bundle = ROOT / "apps/apple-simple/Sources/Resources/node_modules/table.js"
bundle.parent.mkdir(parents=True, exist_ok=True)
entry = "import { mountNativeTable, mountNativeOutput } from " + json.dumps(str(ROOT / "apps/web/src/components/MarkdownTable.tsx")) + ";\n" + "import " + json.dumps(str(ROOT / "apps/web/src/styles.css")) + ";\n" + "if (document.body.classList.contains('notebook-mode')) { globalThis.hickoryOutput = mountNativeOutput(document.getElementById('formula')); globalThis.hickoryOutput.update(globalThis.hickoryOutputSource || ''); } else globalThis.hickoryTable = mountNativeTable(document.getElementById('formula'), message => { const json = JSON.stringify(message); if (globalThis.webkit?.messageHandlers?.table) globalThis.webkit.messageHandlers.table.postMessage(JSON.parse(json)); else globalThis.HickoryTable?.postMessage(json); });"
subprocess.run([str(ROOT / "apps/web/node_modules/.bin/esbuild"), "--bundle", "--format=iife", "--platform=browser", "--jsx=automatic", "--define:import.meta.env={}", "--log-level=warning", "--outfile=" + str(bundle)], input=entry, text=True, cwd=ROOT / "apps/web", check=True)
print("Prepared shared native spreadsheet resources.")
</ui:copy>

<ui:file path="scripts/prepare-simple-resources.py"><ui:paste select="#simple-inline-math-1"/></ui:file>

Apple equations distinguish body ranges for safe edits from full syntax ranges for presentation. Dollar formulas replace their complete delimiter range within the surrounding sentence; math elements replace their whole declaration, body and closing tag. The local typesetter reports natural dimensions with a render revision. TextKit owns placement, and obsolete measurements cannot change a newer source snapshot. Source remains visible until a usable measurement arrives.

Apple previews keep the editor’s reserved inline slots stable. Measure rendered content at its natural size and shrink only when necessary to fit both dimensions; remeasure when fonts, asynchronous diagrams or the viewport change. Graph SVGs need explicit natural dimensions rather than percentage heights, which otherwise resolve against the WebView and distort sizing. Terminal fitting remains independent.

Apple embeds a small typesetter in the line’s floating editor. Navigation is restricted to the local page, and the LaTeX replacement still passes through source, permission, composition and undo checks.

<ui:copy id="simple-inline-math-2">
import SwiftUI
import WebKit
@MainActor final class MathCoordinator: NSObject, WKNavigationDelegate, WKScriptMessageHandler {
    var source = ""
    var diagramRenderer: String?
    var inline = false
    var onSize: ((CGSize) -> Void)?
    private var revision = 0
    private var displayed: String?
    var ready = false
    func display(_ view: WKWebView) {
        guard ready, let bytes = try? JSONSerialization.data(withJSONObject: source, options: .fragmentsAllowed), let json = String(data: bytes, encoding: .utf8) else { return }
        let key = (diagramRenderer.map { "diagram:" + $0 } ?? "math") + "\0" + String(inline) + "\0" + source
        guard displayed != key else { return }
        displayed = key; revision += 1
        if diagramRenderer == "notebook" { view.evaluateJavaScript("globalThis.renderNotebookOutput(" + json + ")", completionHandler: nil) }
        else if diagramRenderer == "terminal" { view.evaluateJavaScript("globalThis.renderTerminal(" + json + ")", completionHandler: nil) }
        else if let renderer = diagramRenderer, let data = try? JSONSerialization.data(withJSONObject: renderer, options: .fragmentsAllowed), let engine = String(data: data, encoding: .utf8) {
            view.evaluateJavaScript("globalThis.renderDiagram(" + json + "," + engine + ")", completionHandler: nil)
        } else { view.evaluateJavaScript("globalThis.renderMath(" + json + "," + String(inline) + "," + String(revision) + ")", completionHandler: nil) }
    }
    func userContentController(_ userContentController: WKUserContentController, didReceive message: WKScriptMessage) {
        guard message.frameInfo.isMainFrame, diagramRenderer == nil,
              let value = message.body as? [String: Any], value["revision"] as? Int == revision,
              let width = value["width"] as? Double, let height = value["height"] as? Double,
              width.isFinite, height.isFinite, width > 0, height > 0 else { return }
        onSize?(CGSize(width: width, height: height))
    }
    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) { ready = true; displayed = nil; display(webView) }
    func webView(_ webView: WKWebView, decidePolicyFor navigationAction: WKNavigationAction, decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
        let url = navigationAction.request.url
        decisionHandler(url?.isFileURL == true || url?.scheme == "about" ? .allow : .cancel)
    }
}
// Inspect bundles without touching SwiftPM's fatal Bundle.module accessor.
func nativeMathResource(roots: [URL], bundles: [Bundle] = [.main, Bundle(for: MathCoordinator.self)]) -> URL? {
    // Resource-only SwiftPM bundles need not initialize as Foundation Bundles.
    // Inspect their installed directories too, including macOS Contents layout.
    let installed = roots.flatMap { root in
        [root, root.appendingPathComponent("HickorySimple_HickorySimple.bundle"), root.appendingPathComponent("HickorySimple_HickorySimple.resources")]
    } + bundles.flatMap { [$0.bundleURL] + [$0.resourceURL].compactMap { $0 } }
    for root in installed {
        for relative in ["HickoryAssets/Resources/math.html", "Resources/math.html", "Contents/Resources/Resources/math.html", "Contents/Resources/math.html", "math.html"] {
            let url = root.appendingPathComponent(relative)
            if FileManager.default.isReadableFile(atPath: url.path) { return url }
        }
    }
    return nil
}
@MainActor func makeMathView(_ coordinator: MathCoordinator) -> WKWebView {
    let configuration = WKWebViewConfiguration(); configuration.websiteDataStore = .nonPersistent(); configuration.preferences.javaScriptCanOpenWindowsAutomatically = false
    configuration.userContentController.add(coordinator, name: "mathSize")
    let view = WKWebView(frame: .zero, configuration: configuration); view.navigationDelegate = coordinator
    let owner = Bundle(for: MathCoordinator.self)
    let roots = [Bundle.main.resourceURL, Bundle.main.bundleURL, owner.resourceURL, owner.bundleURL,
        URL(fileURLWithPath: CommandLine.arguments[0]).deletingLastPathComponent()].compactMap { $0 }
    let url = nativeMathResource(roots: roots)
    if let url { view.loadFileURL(url, allowingReadAccessTo: url.deletingLastPathComponent()) }
    else { view.loadHTMLString("<body><p>Preview resources are missing. Rebuild the app with its bundled assets; source stays editable.</p><div id='math'></div><script>globalThis.renderMath=globalThis.renderDiagram=globalThis.renderTerminal=s=>document.getElementById('math').textContent=s</script></body>", baseURL: nil) }
    return view
}
#if os(macOS)
struct NativeMathView: NSViewRepresentable {
    var source: String
    var diagramRenderer: String? = nil
    var inline = false
    var onSize: ((CGSize) -> Void)? = nil
    func makeCoordinator() -> MathCoordinator { MathCoordinator() }
    func makeNSView(context: Context) -> WKWebView { context.coordinator.source = source; context.coordinator.diagramRenderer = diagramRenderer; context.coordinator.inline = inline; context.coordinator.onSize = onSize; return makeMathView(context.coordinator) }
    func updateNSView(_ view: WKWebView, context: Context) { context.coordinator.source = source; context.coordinator.diagramRenderer = diagramRenderer; context.coordinator.inline = inline; context.coordinator.onSize = onSize; context.coordinator.display(view) }
}
#else
struct NativeMathView: UIViewRepresentable {
    var source: String
    var diagramRenderer: String? = nil
    var inline = false
    var onSize: ((CGSize) -> Void)? = nil
    func makeCoordinator() -> MathCoordinator { MathCoordinator() }
    func makeUIView(context: Context) -> WKWebView { context.coordinator.source = source; context.coordinator.diagramRenderer = diagramRenderer; context.coordinator.inline = inline; context.coordinator.onSize = onSize; return makeMathView(context.coordinator) }
    func updateUIView(_ view: WKWebView, context: Context) { context.coordinator.source = source; context.coordinator.diagramRenderer = diagramRenderer; context.coordinator.inline = inline; context.coordinator.onSize = onSize; context.coordinator.display(view) }
}
#endif
// One read-only terminal surface for captured bytes and expect bodies.
struct NativeInlineTerminal: View {
    var text: String
    var editSource: (() -> Void)? = nil
    var body: some View {
        VStack(spacing: 0) {
            HStack { Image(systemName: "terminal"); Text(text.contains("\u{1e}HICKORY_DISPLAY ") ? "Output" : "Terminal"); Spacer(); if let editSource { Button("Edit source text", action: editSource).buttonStyle(.plain) } }
                .font(.system(size: 11, design: .monospaced)).padding(6).background(Color(red: 0.11, green: 0.15, blue: 0.2))
            NativeMathView(source: text, diagramRenderer: text.contains("\u{1e}HICKORY_DISPLAY ") ? "notebook" : "terminal")
        }.foregroundStyle(Color(red: 0.9, green: 0.93, blue: 0.95)).background(Color(red: 0.06, green: 0.08, blue: 0.11))
            .clipShape(RoundedRectangle(cornerRadius: 6)).overlay(RoundedRectangle(cornerRadius: 6).stroke(.gray.opacity(0.5))).accessibilityLabel("Output terminal")
    }
}
struct NativeDiagramPreview: View {
    var diagram: InlineDiagram
    var buffer: DocumentBuffer
    var body: some View {
        if diagram.renderer == "terminal" { NativeInlineTerminal(text: diagram.body, editSource: { buffer.sourceTable = diagram.id }) }
        else { VStack(spacing: 0) {
            HStack { Text("Diagram").font(.caption); Spacer(); Button("Edit source text") { buffer.sourceTable = diagram.id }.buttonStyle(.bordered) }.padding(4)
            NativeMathView(source: diagram.body, diagramRenderer: diagram.renderer)
        }.background(.background) }
    }
}
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
@Composable fun NativeMathView(source: String, modifier: Modifier = Modifier, diagramRenderer: String? = null) {
    AndroidView(modifier = modifier, factory = { context ->
        WebView(context).apply {
            settings.javaScriptEnabled = true; settings.allowFileAccess = false; settings.allowContentAccess = false; settings.blockNetworkLoads = true
            webViewClient = object : WebViewClient() {
                override fun onRenderProcessGone(view: WebView, detail: RenderProcessGoneDetail): Boolean { view.destroy(); return true }
                override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest): Boolean = request.url.toString() != "file:///android_asset/math.html"
                override fun onPageFinished(view: WebView, url: String) {
                    if (url == "file:///android_asset/math.html") view.evaluateJavascript(view.tag as? String ?: "", null)
                }
            }
            tag = if (diagramRenderer == "notebook") "globalThis.renderNotebookOutput(" + JSONObject.quote(source) + ")" else if (diagramRenderer == "terminal") "globalThis.renderTerminal(" + JSONObject.quote(source) + ")" else if (diagramRenderer == null) "globalThis.renderMath(" + JSONObject.quote(source) + ")" else "globalThis.renderDiagram(" + JSONObject.quote(source) + "," + JSONObject.quote(diagramRenderer) + ")"; loadUrl("file:///android_asset/math.html")
        }
    }, update = { view ->
        val script = if (diagramRenderer == "notebook") "globalThis.renderNotebookOutput(" + JSONObject.quote(source) + ")" else if (diagramRenderer == "terminal") "globalThis.renderTerminal(" + JSONObject.quote(source) + ")" else if (diagramRenderer == null) "globalThis.renderMath(" + JSONObject.quote(source) + ")" else "globalThis.renderDiagram(" + JSONObject.quote(source) + "," + JSONObject.quote(diagramRenderer) + ")"
        if (view.tag != script) {
            view.tag = script
            view.evaluateJavascript("if(globalThis.renderDiagram){" + script + "}", null)
        }
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
  const [receiptRevision, refreshReceipts] = useState(0);
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
  }, [buffer, buffer.base, buffer.dirty, receiptRevision]);
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
    const context = { simpleTables: true, view, path: buffer.file.path, docId: buffer.file.doc_id, execBlocks: saved.filter((block): block is ExecBlock => block.kind === "exec"),
      diagramBlocks: saved.filter((block): block is DiagramBlock => block.kind === "diagram"), runningCells: new Set<string>(), replaying: [],
      replaceBlockContent: (target: RenderedSlot, text: string, origin?: string) => modules.edit.replaceCapturedBlock(view, target, text, origin ?? "input.table"),
    };
    return <SlotPortal key={slot.key} slot={slot} css={modules.css.default + modules.katexCss.default + modules.graphCss.default + modules.terminalCss.default}>
      {!slot.outputOnly && <button type="button" style={{ border: "1px solid currentColor", borderRadius: 4, padding: "3px 8px", cursor: "pointer" }} onClick={() => {
        if (view.state.doc !== captured) return;
        view.dispatch({ effects: modules.rendered.showBlockSource.of(slot.at), selection: { anchor: Math.min(slot.at, view.state.doc.length) }, scrollIntoView: true }); view.focus();
      }}>Edit source text</button>}
      {slot.kind === "exec" && buffer.file.doc_id && <button type="button" disabled={buffer.dirty} onClick={() => refreshReceipts(value => value + 1)}>Refresh recorded output</button>}
      {modules.elements.elementViews[slot.kind].render(slot, context)}
    </SlotPortal>;
  })}</>;
}
async function load() {
  // Resolve the shared cyclic registry graph before importing its members in parallel.
  const parser = await import("../editor/hickLang");
  await parser.loadHickLang();
  const rendered = await import("../editor/rendered");
  const [prose, math, spans, hick, elements, edit, css, katexCss, graphCss, terminalCss] = await Promise.all([
    import("../editor/wysiwyg"), import("../editor/mathRender"), import("../lib/math"), import("../editor/hickDoc"),
    import("../elements"), import("../editor/replaceCapturedBlock"), import("../styles.css?inline"), import("katex/dist/katex.min.css?inline"), import("@xyflow/react/dist/style.css?inline"), import("@xterm/xterm/css/xterm.css?inline"),
  ]);
  return { prose, rendered, math, spans, hick, elements, edit, css, katexCss, graphCss, terminalCss };
}
</ui:copy>

<ui:file path="apps/web/src/simple/RichInline.tsx"><ui:paste select="#simple-inline-browser-owners"/></ui:file>

</ui:doc>
