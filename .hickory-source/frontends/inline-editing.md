<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Add inline presentation without replacing source

For maintainers implementing rich source editing on native platforms.

Compute a line-start index and presentation against immutable UTF-16 text. Visible gutters use binary search into the current snapshot instead of repeatedly copying and counting the entire prefix on every scrolled line. Keep Markdown delimiters and original bytes in the editor, skip fenced and Hickory payload bodies, and expose exact cell/equation ranges for guarded contextual edits. The same bounded scan supplies local syntax colors for standalone source files and embedded code. Standalone files skip Markdown presentation rules. Large files retain readable text while the optional scanner is bounded to 200,000 UTF-16 units.

For native Markdown links the label remains the touch target and the whole
authored link is the caret target. Both refer to the same workspace destination.
Keep their ranges in the same UTF-16 snapshot so an action can reject obsolete
markup when the editor already contains a newer draft.

Ordinary executable-language fences in a recorded message need an opening-line
action and breakpoint targets in their body. Unified diff examples need their
own review action. The scan publishes a candidate only after matching its
closing delimiter. These positions become usable only when a retained message
supplies its turn and exact source to the engine.

CSV declarations and standalone CSV files use the same grid as prose tables. Group cells only while the intervening source consists of delimiters and record separators; tags between two declarations keep their grids separate. Preserve each cell’s format and delimiter on commit so commas, quotes and embedded newlines round-trip. Revealing a table records its anchor independently of the other grids.

Markdown prose tables now use those exact cell ranges to form whole-table presentation ranges. Keep the final newline outside the replacement so following prose retains its paragraph. Header separators belong to the replacement, but never become editable data cells. Only cells admitted by the prose or CSV scanner can enter a grid; fenced code and Hickory literal bodies stay source. The Swift and Kotlin range witnesses check CRLF, emoji and escaped pipes.

The shared table surface renders workspace links in Markdown cell text. A
link button owns its pointer and keyboard gestures. The cell still opens the
original Markdown for editing. The native bridge sends only the link and
table revision, and the host checks that the table still matches its source
before handing navigation to the workspace.

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
is passed as JSON rather than executable script. Like the browser, the shared
Apple and Android graph renderer reads authored top-level nodes and edges as
well as scene topology nested under `topology`. Both forms keep layout at the
top level. Check scene shape and size before drawing. Empty, invalid and
oversize diagrams report what stopped the preview, with the source action on
the diagram gutter. Include the message in the native size receipt so it fits
even when there is no SVG to measure. A monotonically increasing
render ticket rejects obsolete Mermaid results. Native WebViews skip identical
source/renderer updates so typing elsewhere cannot restart an unchanged diagram.
Network navigation remains disabled.
Browser slots include the graph canvas stylesheet inside their shadow root; a stylesheet on the outer document cannot style that canvas. Native editors reserve the measured natural content height, shrinking wide diagrams to the available width. Measurement changes preserve the visible source anchor and never mutate diagram bytes. Source mode is reversible through the same right-gutter icon. Fenced examples and
other Hickory literal bodies do not create live diagrams.

Ordinary diff fences remain review text. Explicit patch fences are project constructions: offer launch beside each supported destination header and pending breakpoints on nonempty destination rows. Hunk counts distinguish destination headers from added strings beginning with plus signs. A context breakpoint aliases the validated destination line while retaining the pre-existing source owner. The debugger admits the alias only while the destination still matches the completed patch, including its provenance. Local syntax colors the added code without its diff prefix. Path-bearing ordinary code fences use the same file context as file tags.

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

The Apple scan records each quote prefix and each list marker at its source
offset. Adjacent quoted rows share an opening offset. TextKit can reveal them
as a unit and remove just those prefixes from a copied selection.

<ui:copy id="simple-inline-model-0">
import Foundation
// Diagram links are paths relative to the document, bounded by the workspace root.
func diagramFilePath(_ href: String, relativeTo source: String) -> String? {
    guard !href.isEmpty, !href.contains(":"), !href.hasPrefix("/"), !href.contains("\\"), !href.contains("\n") else { return nil }
    guard let path = href.removingPercentEncoding,
          !path.contains(where: { $0.isNewline || $0 == "\0" }) else { return nil }
    guard !path.contains(":"), !path.hasPrefix("/"), !path.contains("\\") else { return nil }
    var parts = source.split(separator: "/").dropLast().map(String.init)
    for part in path.split(separator: "/") {
        if part == ".." { guard !parts.isEmpty else { return nil }; parts.removeLast() }
        else if part != "." { parts.append(String(part)) }
    }
    return parts.isEmpty ? nil : parts.joined(separator: "/")
}
func workspaceLinkTarget(_ href: String, relativeTo source: String) -> (path: String, line: Int?)? {
    let parts = href.split(separator: "#", maxSplits: 1, omittingEmptySubsequences: false)
    guard let first = parts.first, let path = diagramFilePath(String(first), relativeTo: source) else { return nil }
    let line = parts.count == 2 && parts[1].hasPrefix("L") ? Int(parts[1].dropFirst()) : nil
    return (path, line.flatMap { $0 > 0 ? $0 - 1 : nil })
}
struct InlineLink: Equatable, Sendable {
    var range: NSRange
    var sourceRange: NSRange
    var href: String

    static func == (lhs: Self, rhs: Self) -> Bool {
        lhs.range == rhs.range && lhs.sourceRange == rhs.sourceRange && exactUTF8(lhs.href, rhs.href)
    }
}
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
// Typeset glyphs have no one-to-one TeX position. Map across the visible body
// and snap to a whole grapheme so the next native edit can start safely.
func equationSourceCaret(_ equation: InlineEquation, fraction: CGFloat) -> Int {
    let leading = equation.body.prefix(while: \.isWhitespace).utf16.count
    let body = equation.body.trimmingCharacters(in: .whitespacesAndNewlines)
    let boundaries = body.indices.map { $0.utf16Offset(in: body) } + [body.utf16.count]
    let target = Int((CGFloat(body.utf16.count) * min(1, max(0, fraction))).rounded())
    let nearest = boundaries.min { abs($0 - target) < abs($1 - target) } ?? 0
    return equation.range.location + min(equation.range.length, leading + nearest)
}
struct InlineDiagram: Identifiable, Sendable { var line: Int; var range: NSRange; var body: String; var renderer: String; var id: Int { range.location } }
struct MarkdownMarker: Sendable {
    var range: NSRange
    var label: String
}

struct MarkdownBlockLine: Sendable {
    var range: NSRange
    var quoteStart: Int?
    var quotePrefix: NSRange?
    var markers: [MarkdownMarker]
}

extension InlineSnapshot {
    func markdownGlyphs(carets: [Int]) -> [Int: UInt16] {
        let active = blockLines.filter { line in
            carets.contains { $0 >= line.range.location && $0 <= NSMaxRange(line.range) }
        }
        let activeRows = Set(active.map { $0.range.location })
        let activeQuotes = Set(active.compactMap(\.quoteStart))
        var glyphs: [Int: UInt16] = [:]
        for line in blockLines where !activeRows.contains(line.range.location) {
            if let quote = line.quoteStart, activeQuotes.contains(quote) { continue }
            for marker in line.markers {
                let label = Array(marker.label.utf16)
                for index in 0..<marker.range.length {
                    // A zero entry suppresses a glyph without moving any source offset.
                    glyphs[marker.range.location + index] = index < label.count ? label[index] : 0
                }
            }
        }
        return glyphs
    }

    func quoteContents(_ selection: NSRange) -> String? {
        let text = source as NSString, end = NSMaxRange(selection)
        guard selection.length > 0, end <= text.length,
              let first = blockLines.first(where: { selection.location >= $0.range.location && selection.location <= NSMaxRange($0.range) }),
              let quote = first.quoteStart,
              let last = blockLines.last(where: { $0.quoteStart == quote }),
              end <= NSMaxRange(last.range) + 1 else { return nil }
        var parts: [String] = [], offset = selection.location
        for line in blockLines where line.quoteStart == quote {
            guard let prefix = line.quotePrefix, NSMaxRange(prefix) > offset, prefix.location < end else { continue }
            let stop = max(offset, prefix.location)
            parts.append(text.substring(with: NSRange(location: offset, length: stop - offset)))
            offset = min(end, NSMaxRange(prefix))
        }
        guard offset > selection.location else { return nil }
        parts.append(text.substring(with: NSRange(location: offset, length: end - offset)))
        return parts.joined()
    }
}

struct InlineSnapshot: Sendable { var blockLines: [MarkdownBlockLine] = []; var diagrams: [InlineDiagram] = []; var source: String; var lineStarts: [Int] = [0]; var marks: [InlineMark] = []; var cells: [InlineCell] = []; var equations: [InlineEquation] = []; var debug = InlineDebugContext(); var gridOverrides: [InlineTable] = []; var links: [InlineLink] = [] }
// Tables are presentation ranges over the original source, never replacement text.
struct InlineTable: Identifiable, Sendable {
    var range: NSRange; var rows: [[InlineCell]]
    var id: Int { range.location }
    var preferredHeight: CGFloat? = nil
    var height: CGFloat { preferredHeight ?? min(420, CGFloat(rows.count + 1) * 30 + 2) }
}
func tablePresentation(_ snapshot: InlineSnapshot, source: String) -> InlineSnapshot {
    if exactUTF8(snapshot.source,source) { return snapshot }
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
struct RecordedDebugFence: Sendable, Equatable {
    var opening: Int
    var closing: Int
    var language: String
    func contains(_ line: Int) -> Bool { opening < line && line < closing }
}
struct InlineDebugContext: Sendable {
    var code: [Int: String] = [:]
    var launch: [Int: String] = [:]
    var recorded: [Int: RecordedDebugFence] = [:]
    var recordedPatches: Set<Int> = []
    var recordedPatchLines: [Int: Int] = [:]
    var recordedConstructions: Set<Int> = []
}
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
    var recorded: (opening: Int, language: String)?
    var declaredPatch: Int?, declaredClose: String?
    var patchLines: [Int] = []
    let standalone = Set(["py", "python", "python3", "js", "javascript", "node", "ts", "typescript", "rs", "rust", "c", "cpp", "c++", "cxx", "go", "golang", "cs", "csharp", "c#", "rb", "ruby"])
    let patchLanguages = Set(["diff", "patch"])
    let opening = try! NSRegularExpression(pattern: #"^ {0,3}<[\w.-]+:(file|patch|copy|cut|exec|entry|diagram|math|table)\b[^>]*>"#)
    let pathHint = try! NSRegularExpression(pattern: #"\bpath\s*=\s*["']([^"']+)["']"#)
    let hunk = try! NSRegularExpression(pattern: #"^@@ -\d+(?:,(\d+))? \+\d+(?:,(\d+))? @@"#)
    for (index, row) in rows.enumerated() {
        let trimmed = row.trimmingCharacters(in: .whitespacesAndNewlines), text = row as NSString
        if !fence.isEmpty {
            if trimmed.count >= fence.count, trimmed.allSatisfy({ $0 == fence.first! }) {
                if let recorded, index > recorded.opening + 1,
                   row.prefix(while: { $0 == " " }).count <= 3,
                   rows[(recorded.opening + 1)..<index].contains(where: { !$0.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }) {
                    if patchLanguages.contains(recorded.language) {
                        result.recordedPatches.insert(recorded.opening)
                        for line in patchLines { result.recordedPatchLines[line] = recorded.opening }
                    }
                    else {
                        result.recorded[recorded.opening] = RecordedDebugFence(opening: recorded.opening, closing: index, language: recorded.language)
                        let address = "fence:\(recorded.opening)"
                        result.launch[recorded.opening] = address
                        for line in (recorded.opening + 1)..<index where !rows[line].trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                            result.code[line] = address
                        }
                    }
                }
                if let declaredPatch, index > declaredPatch + 1,
                   row.prefix(while: { $0 == " " }).count <= 3,
                   rows[(declaredPatch + 1)..<index].contains(where: { !$0.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }) {
                    result.recordedConstructions.insert(declaredPatch)
                    for line in patchLines { result.recordedPatchLines[line] = declaredPatch }
                }
                fence = ""; mode = ""; target = nil; recorded = nil; declaredPatch = nil; continue
            }
        } else if !mode.isEmpty {
            if trimmed.contains("</"), trimmed.contains(":" + mode + ">") {
                if let declaredPatch, let declaredClose, trimmed.contains(declaredClose),
                   index > declaredPatch + 1,
                   rows[(declaredPatch + 1)..<index].contains(where: { !$0.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }) {
                    result.recordedConstructions.insert(declaredPatch)
                    for line in patchLines { result.recordedPatchLines[line] = declaredPatch }
                }
                mode = ""; target = nil; declaredPatch = nil; declaredClose = nil; continue
            }
        } else {
            var declaration = ""
            if let marker = row.range(of: #"^ {0,3}(`{3,}|~{3,})"#, options: .regularExpression) {
                fence = String(row[marker]).trimmingCharacters(in: .whitespaces)
                declaration = String(row[marker.upperBound...]); mode = declaration.trimmingCharacters(in: .whitespaces).components(separatedBy: " ")[0] == "patch" ? "patch" : "file"
                let language = declaration.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
                if standalone.contains(language) || patchLanguages.contains(language) { recorded = (index, language) }
                if language == "diff" { mode = "diff" }
                let info = declaration.trimmingCharacters(in: .whitespacesAndNewlines)
                if info != "patch", info.split(whereSeparator: { $0.isWhitespace }).first == "patch" { declaredPatch = index }
            } else if let match = opening.firstMatch(in: row, range: NSRange(location: 0, length: text.length)) {
                mode = text.substring(with: match.range(at: 1)); declaration = text.substring(with: match.range)
                if mode == "patch", !declaration.hasSuffix("/>"),
                   let tag = declaration.trimmingCharacters(in: .whitespaces).dropFirst().split(separator: ":", maxSplits: 1).first {
                    declaredPatch = index
                    declaredClose = "</\(tag):patch>"
                }
            } else { continue }
            oldLines = 0; newLines = 0; target = nil; patchLines = []
            if mode == "file", let match = pathHint.firstMatch(in: declaration, range: NSRange(location: 0, length: declaration.utf16.count)) {
                let path = (declaration as NSString).substring(with: match.range(at: 1))
                target = supported.contains((path as NSString).pathExtension.lowercased()) ? path : nil
                result.launch[index] = target
            }
            continue
        }
        if mode == "file", !trimmed.isEmpty { result.code[index] = target }
        if mode == "patch" || mode == "diff" {
            if oldLines > 0 || newLines > 0 {
                if row.hasPrefix("+"), newLines > 0 {
                    newLines -= 1; patchLines.append(index)
                    if mode == "patch", !row.dropFirst().trimmingCharacters(in: .whitespaces).isEmpty { result.code[index] = target }
                } else if row.hasPrefix("-"), oldLines > 0 { oldLines -= 1; patchLines.append(index) }
                else if row.hasPrefix(" "), oldLines > 0, newLines > 0 {
                    oldLines -= 1; newLines -= 1
                    if mode == "patch", !row.dropFirst().trimmingCharacters(in: .whitespacesAndNewlines).isEmpty { result.code[index] = target }
                }
            } else if row.hasPrefix("+++ ") {
                let raw = String(row.dropFirst(4)).components(separatedBy: "\t")[0]
                let path = raw.hasPrefix("b/") ? String(raw.dropFirst(2)) : raw
                target = supported.contains((path as NSString).pathExtension.lowercased()) ? path : nil
                if mode == "patch" { result.launch[index] = target }
            } else if let match = hunk.firstMatch(in: row, range: NSRange(location: 0, length: text.length)) {
                oldLines = match.range(at: 1).location == NSNotFound ? 1 : Int(text.substring(with: match.range(at: 1))) ?? 0
                newLines = match.range(at: 2).location == NSNotFound ? 1 : Int(text.substring(with: match.range(at: 2))) ?? 0
            }
        }
    }
    return result
}
func inlineLaunchEnd(_ source: String, header: Int) -> Int {
    let rows = source.components(separatedBy: "\n")
    guard rows.indices.contains(header) else { return 0 }
    let tag = rows[header].trimmingCharacters(in: .whitespaces)
    let patch = tag.hasPrefix("+++ ")
    let file = tag.first == "<" && !patch
    let marker = tag.first
    for line in (header + 1)..<rows.count {
        let text = rows[line].trimmingCharacters(in: .whitespacesAndNewlines)
        if (file || patch) && text.hasPrefix("</") && text.contains(patch ? ":patch>" : ":file>") { return line }
        if !file && text.count >= 3 && (patch ? (text.first == "`" || text.first == "~") : text.first == marker) &&
            (text.first == "`" || text.first == "~") && text.allSatisfy({ $0 == text.first }) { return line }
    }
    return max(header, rows.count - (source.hasSuffix("\n") ? 2 : 1))
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
    guard source.utf16.count <= 200_000 else { return result }
    guard ["md", "markdown"].contains((path as NSString).pathExtension.lowercased()) else {
        result.marks = inlineSyntaxMarks(source, path: path)
        return result
    }
    let rows = source.components(separatedBy: "\n")
    let separator = try! NSRegularExpression(pattern: #"^\s*\|?\s*:?-{3,}:?\s*(\|\s*:?-{3,}:?\s*)+\|?\s*$"#)
    let heading = try! NSRegularExpression(pattern: #"^(#{1,6})\s+"#)
    let links = try! NSRegularExpression(pattern: #"(?<![!\\])\[([^\]\n]+)\]\(([^\s()]+)\)"#)
    let styles = [("code", #"`[^`\n]+`"#), ("strong", #"\*\*[^*\n]+\*\*"#), ("em", #"(?<!\*)\*[^*\n]+\*(?!\*)"#), ("math", #"(?<!\\)\$([^$\n]+)\$"#)].map { ($0.0, try! NSRegularExpression(pattern: $0.1)) }
    let opening = try! NSRegularExpression(pattern: #"^\s*<[A-Za-z_][\w.-]*:(file|copy|cut|exec|container|diagram|math|table|entry|patch|expect)\b[^>]*>"#)
    var offset = 0, fence = "", payload = "", table = false, diffFence = false
    var equationStart: Int?, equationLine = 0, equationOpen = 0
    var tableStart: Int?, tableLine = 0, delimiter = ","
    let quoteMarker = try! NSRegularExpression(pattern: #"^ {0,3}(?:>[ \t]?)+"#)
    let listMarker = try! NSRegularExpression(pattern: #"^( *)([-*+]|[0-9]{1,9}[.)])(?=[ \t]|$)[ \t]*"#)
    let taskMarker = try! NSRegularExpression(pattern: #"^\[([ xX])\](?=[ \t]|$)"#)
    let ruleMarker = try! NSRegularExpression(pattern: #"^ {0,3}(?:(?:-[ \t]*){3,}|(?:\*[ \t]*){3,}|(?:_[ \t]*){3,})$"#)
    let frontMatterEnd = rows.first?.trimmingCharacters(in: CharacterSet(charactersIn: "\u{feff}\r")) == "---"
        ? rows.dropFirst().firstIndex { ["---", "..."].contains($0.trimmingCharacters(in: .whitespacesAndNewlines)) } : nil
    let quoteFenceMarker = try! NSRegularExpression(pattern: #"^ {0,3}(`{3,}|~{3,})(.*)$"#)
    var quoteFence = ""
    var quoteRow = -2, quoteStart: Int?, listIndent: Int?, listRow = -2
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
        if frontMatterEnd.map({ line > $0 }) ?? true {
            var content = row, base = offset, prefix: NSRange?, markers: [MarkdownMarker] = []
            if let match = quoteMarker.firstMatch(in: row, range: all) {
                let raw = ns.substring(with: match.range)
                let depth = raw.filter { $0 == ">" }.count
                prefix = NSRange(location: offset, length: match.range.length)
                markers.append(MarkdownMarker(range: prefix!, label: String(repeating: "│", count: depth) + (depth < match.range.length ? " " : "")))
                if quoteRow != line - 1 { quoteStart = offset; quoteFence = "" }
                quoteRow = line
                content = ns.substring(from: match.range.length)
                base += match.range.length
            }
            let contentNS = content as NSString, contentRange = NSRange(location: 0, length: contentNS.length)
            let inQuoteFence = prefix != nil && !quoteFence.isEmpty
            let quotedFence = prefix == nil ? nil : quoteFenceMarker.firstMatch(in: content, range: contentRange)
            if let quotedFence {
                let delimiter = contentNS.substring(with: quotedFence.range(at: 1))
                let tail = contentNS.substring(with: quotedFence.range(at: 2))
                if !inQuoteFence { quoteFence = delimiter }
                else if delimiter.hasPrefix(quoteFence), tail.trimmingCharacters(in: .whitespaces).isEmpty { quoteFence = "" }
            }
            if !inQuoteFence, quotedFence == nil, let match = listMarker.firstMatch(in: content, range: contentRange),
               match.range(at: 1).length <= (listRow == line - 1 ? (listIndent ?? -1) + 4 : 3),
               ruleMarker.firstMatch(in: content, range: contentRange) == nil {
                listIndent = match.range(at: 1).length; listRow = line
                let marker = match.range(at: 2), ordered = marker.length > 1
                let boxOffset = match.range.length
                let tail = contentNS.substring(from: boxOffset)
                let task = taskMarker.firstMatch(in: tail, range: NSRange(location: 0, length: tail.utf16.count))
                let label = ordered ? String(contentNS.substring(with: marker).dropLast()) + "." : task == nil ? "•" : ""
                let markerEnd = task != nil && !ordered ? boxOffset : NSMaxRange(marker)
                markers.append(MarkdownMarker(range: NSRange(location: base + marker.location, length: markerEnd - marker.location), label: label))
                if let task {
                    let checked = (tail as NSString).substring(with: task.range(at: 1)) != " "
                    markers.append(MarkdownMarker(range: NSRange(location: base + boxOffset, length: 3), label: checked ? "☑" : "☐"))
                }
            }
            if !markers.isEmpty {
                result.blockLines.append(MarkdownBlockLine(range: NSRange(location: offset, length: rawRow.utf16.count),
                    quoteStart: prefix == nil ? nil : quoteStart, quotePrefix: prefix, markers: markers))
            }
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
        for match in links.matches(in: row, range: all) where !codeRanges.contains(where: { NSIntersectionRange($0, match.range).length > 0 }) {
            let label = match.range(at: 1), href = ns.substring(with: match.range(at: 2))
            guard workspaceLinkTarget(href, relativeTo: path) != nil else { continue }
            let range = NSRange(location: offset + label.location, length: label.length)
            let syntax = NSRange(location: offset + match.range.location, length: match.range.length)
            result.links.append(InlineLink(range: range, sourceRange: syntax, href: href))
            result.marks.append(InlineMark(kind: "link", range: range))
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

The Android Agent card suggests ordinary complete fences and declared patches
by their opening lines. A fenced example can quote a patch tag without offering
an action for it. The engine rereads the recorded message before supplying launch
authority or establishing a captured construction.

Android also records quote and list prefixes in the source snapshot. Compose
uses its output transformation for the markers and copies from that visual
text. Quote rules are drawn separately so they contribute no clipboard text.

<ui:copy id="simple-inline-model-1">
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
data class MarkdownMarker(val from: Int, val to: Int, val label: String)
data class MarkdownBlockLine(
    val from: Int, val to: Int, val quoteStart: Int?, val quoteDepth: Int, val markers: List<MarkdownMarker>,
)

fun InlineSnapshot.renderedBlockLines(caret: Int?): List<MarkdownBlockLine> {
    val active = caret?.let { offset -> blockLines.firstOrNull { offset in it.from..it.to } }
    return blockLines.filter { it != active && (active?.quoteStart == null || it.quoteStart != active.quoteStart) }
}

data class InlineSnapshot(val source: String, val lineStarts: List<Int> = listOf(0), val marks: List<InlineMark> = emptyList(), val cells: List<InlineCell> = emptyList(), val equations: List<InlineEquation> = emptyList(), val debug: InlineDebugContext = InlineDebugContext(), val diagrams: List<InlineDiagram> = emptyList(), val gridOverrides: List<InlineTable> = emptyList(), val links: List<InlineLink> = emptyList(),
    val blockLines: List<MarkdownBlockLine> = emptyList())
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
    val opening = Regex("^\\s*<[A-Za-z_][\\w.-]*:(file|copy|cut|exec|container|diagram|math|table|entry|patch|expect)\\b[^>]*>")
    var offset = 0; var fence = ""; var payload = ""; var table = false; var diffFence = false; var equationStart: Int? = null; var equationLine = 0; var equationOpen = 0; var tableStart: Int? = null; var tableLine = 0; var delimiter = ","
    val quoteMarker = Regex("""^ {0,3}(?:>[ \t]?)+""")
    val listMarker = Regex("""^( *)([-*+]|[0-9]{1,9}[.)])(?=[ \t]|$)[ \t]*""")
    val taskMarker = Regex("""^\[([ xX])\](?=[ \t]|$)""")
    val ruleMarker = Regex("""^ {0,3}(?:(?:-[ \t]*){3,}|(?:\*[ \t]*){3,}|(?:_[ \t]*){3,})$""")
    val frontMatterEnd = if (rows.first().trim('\uFEFF', '\r') == "---")
        (1 until rows.size).firstOrNull { rows[it].trim() in listOf("---", "...") } ?: -1 else -1
    val blockLines = mutableListOf<MarkdownBlockLine>()
    val quoteFenceMarker = Regex("^ {0,3}(`{3,}|~{3,})(.*)$")
    var quoteFence = ""
    var quoteRow = -2; var quoteStart = 0; var listRow = -2; var listIndent = 0
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
        if (line > frontMatterEnd) {
            val quote = quoteMarker.find(row)
            val markers = mutableListOf<MarkdownMarker>()
            val prefixLength = quote?.value?.length ?: 0
            val depth = quote?.value?.count { it == '>' } ?: 0
            if (quote != null) {
                if (quoteRow != line - 1) { quoteStart = start; quoteFence = "" }
                quoteRow = line
                markers.add(MarkdownMarker(start, start + prefixLength, ""))
            }
            val content = row.substring(prefixLength)
            val inQuoteFence = quote != null && quoteFence.isNotEmpty()
            val quotedFence = if (quote == null) null else quoteFenceMarker.matchEntire(content)
            if (quotedFence != null) {
                val (delimiter, tail) = quotedFence.destructured
                if (!inQuoteFence) quoteFence = delimiter
                else if (delimiter.startsWith(quoteFence) && tail.isBlank()) quoteFence = ""
            }
            val list = if (inQuoteFence || quotedFence != null) null else listMarker.find(content)
            if (list != null && list.groupValues[1].length <= (if (listRow == line - 1) listIndent + 4 else 3) &&
                !ruleMarker.matches(content)) {
                listIndent = list.groupValues[1].length; listRow = line
                val base = start + prefixLength
                val marker = list.groupValues[2]
                val boxOffset = list.value.length
                val task = taskMarker.find(content.substring(boxOffset))
                val ordered = marker.length > 1
                val label = if (ordered) marker.dropLast(1) + "." else if (task == null) "•" else ""
                val markerEnd = if (task != null && !ordered) boxOffset else listIndent + marker.length
                markers.add(MarkdownMarker(base + listIndent, base + markerEnd, label))
                if (task != null) markers.add(MarkdownMarker(base + boxOffset, base + boxOffset + 3,
                    if (task.groupValues[1] == " ") "☐" else "☑"))
            }
            if (markers.isNotEmpty()) blockLines.add(MarkdownBlockLine(start, start + rawRow.length,
                if (quote == null) null else quoteStart, depth, markers))
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
    return InlineSnapshot(source, starts, marks + inlineSyntaxMarks(source, path) + patchMarks, cells, equations, debug, inlineDiagrams(source), links = links, blockLines = blockLines)
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

Measure the natural diagram bounds through the revision-checked math channel.
SVG size reports must describe content before its fitting transform, otherwise a
host resize can feed back into an ever-shrinking preview. Keep Mermaid strict;
bind quoted node file links after rendering and send only paths to the host.
The host resolves them against the containing document and rejects external
schemes and paths escaping the workspace. Double-click is the opening gesture.

Native WebViews paint math and tables from semantic system colors. Pass the
SwiftUI host appearance to each preview so a newly mounted WebKit process can
choose its palette before the first render. Update that palette independently
of the source and keep the same WebView on a live appearance change. Fit prose
tables to their viewport while preserving enough width per column to read words.
At narrower sizes the existing grid scroller keeps cells reachable. Recorded
terminal output updates its emulator palette in place. The interactive terminal
keeps its own palette.

Fit read-only output to the WebView's measured viewport. An Android placement
can start with a zero-height layout viewport and later receive a nonzero
`innerHeight`. Using that measured height before fitting the rows keeps the
first recorded line visible when the Run popover opens.

<ui:copy id="simple-inline-math-0">
<!doctype html>
<html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'self' 'nonce-hickory-math'; style-src 'self' 'unsafe-inline'; font-src 'self' data:; img-src data:;">
<link rel="stylesheet" href="node_modules/xterm.css"><link rel="stylesheet" href="katex/katex.min.css"><style>html{height:100%;color-scheme:light dark;font:14px system-ui}body{margin:0;height:100%;overflow:hidden}body.terminal-mode{color-scheme:dark;height:100%;margin:0;background:#10151d;color:#e6edf3;overflow:hidden}.terminal-mode #formula{position:static;width:auto;transform:none!important;height:100%;box-sizing:border-box;padding:6px;overflow:hidden}.terminal-mode:not(.interactive-terminal) .xterm-cursor-layer{display:none}#formula{position:absolute;width:max-content;padding:8px;transform-origin:top left;box-sizing:border-box}#status{position:absolute;bottom:0;font-size:11px}#formula .katex-display{margin:0}.inline-math{font-size:13px}.inline-math #formula{padding:1px}.display-math #formula{padding:4px}</style>
<style>
.diagram-mode g[role="link"]:focus > rect { stroke-width: 3; }
.diagram-mode #status:not(:empty) { inset-inline: 0; top: 0; bottom: auto; max-height: 100%; box-sizing: border-box; padding: 4px 8px; font-size: inherit; overflow: auto; overflow-wrap: anywhere; }
html:root {
  color-scheme: light dark;
  color: CanvasText; background: Canvas;
  --bg: Canvas; --fg: CanvasText; --fg-muted: GrayText; --muted: GrayText;
  --bg-raised: Canvas; --bg-sunken: color-mix(in srgb, CanvasText 5%, Canvas);
  --border: color-mix(in srgb, CanvasText 22%, Canvas);
  --accent: Highlight; --accent-2: light-dark(#176b73, #71d1d9);
  --select-bg: color-mix(in srgb, Highlight 22%, Canvas);
}
html[data-appearance="light"] { color-scheme: light; }
html[data-appearance="dark"] { color-scheme: dark; }
.table-mode .table-panel__formula:disabled { opacity: 1; }
.table-mode .table-panel__formula::placeholder {
  color: GrayText; -webkit-text-fill-color: currentColor; opacity: 1;
}
.table-mode .table-panel--fit-prose .table-panel__grid {
  min-width: calc(2.25rem + var(--native-table-columns, 1) * 5.5rem);
}
body.terminal-mode:not(.interactive-terminal) { color-scheme: inherit; background: light-dark(#f6f7f8, #1c1c1e); color: CanvasText; }
body.terminal-mode:not(.interactive-terminal) #formula { height: var(--output-terminal-height, 100%); }
</style>
</head><body><div id="formula" aria-label="Typeset equation"></div><div id="status" role="status"></div>
<script nonce="hickory-math" src="node_modules/xterm.js"></script><script nonce="hickory-math" src="node_modules/addon-fit.js"></script><script nonce="hickory-math" src="katex/katex.min.js"></script><script nonce="hickory-math" src="node_modules/mermaid.min.js"></script><script nonce="hickory-math">
if (globalThis.hickoryTableMode || location.search === '?table') {
  document.body.className = 'table-mode'; document.getElementById('formula').setAttribute('aria-label', 'Inline table grid');
  const style = document.createElement('style'); style.textContent = '#formula{position:static!important;width:100%!important;padding:0;transform:none!important;box-sizing:border-box}#status{display:none}body{overflow:auto}#formula.native-table-fill{height:100vh}.native-table-fill>.table-panel{height:100%;margin:0;display:flex;flex-direction:column}.native-table-fill .table-panel__scroll{flex:1;min-height:0}'; document.head.append(style);
  const css = document.createElement('link'); css.rel = 'stylesheet'; css.href = 'node_modules/table.css'; document.head.append(css);
  const script = document.createElement('script'); script.src = 'node_modules/table.js'; script.nonce = 'hickory-math';
  script.onerror = () => globalThis.webkit?.messageHandlers?.table?.postMessage({ kind: 'unavailable' });
  document.head.append(script);
}
// Transforms do not change layout dimensions, avoiding a ResizeObserver feedback loop.
function followDiagramLink(href, revision) {
  if (globalThis.webkit?.messageHandlers?.diagramLink) {
    globalThis.webkit.messageHandlers.diagramLink.postMessage({ href, revision });
  } else if (globalThis.hickoryAndroidDiagram && typeof href === 'string') {
    location.href = 'hickory-diagram://navigate/' + encodeURIComponent(href) + '?revision=' + revision;
  }
}
function fitPreview() {
  if (document.body.classList.contains('notebook-mode') || document.body.classList.contains('terminal-mode') || document.body.classList.contains('table-mode')) return;
  const node = document.getElementById('formula');
  const width = node.offsetWidth, height = node.offsetHeight;
  const statusHeight = document.getElementById('status').offsetHeight;
  const diagram = document.body.classList.contains('diagram-mode');
  if (globalThis.mathRevision !== undefined && (document.querySelector('.katex') || diagram)) {
    globalThis.webkit?.messageHandlers?.mathSize?.postMessage({ revision: globalThis.mathRevision, width, height: height + (diagram ? statusHeight : 0) });
  }
  const availableHeight = Math.max(1, innerHeight - statusHeight);
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
const preferredAppearance = matchMedia('(prefers-color-scheme: dark)');
const darkAppearance = () => (document.documentElement.dataset.appearance ?? (preferredAppearance.matches ? 'dark' : 'light')) === 'dark';
const outputTheme = () => ({
  background: darkAppearance() ? '#1c1c1e' : '#f6f7f8',
  foreground: darkAppearance() ? '#f2f2f7' : '#242527'
});
let displayedMermaid;
function updateAppearance() {
  if (outputTerminal) outputTerminal.options.theme = outputTheme();
  if (displayedMermaid) globalThis.renderDiagram(...displayedMermaid);
}
preferredAppearance.addEventListener('change', () => {
  if (!document.documentElement.dataset.appearance) updateAppearance();
});
globalThis.setPreviewAppearance = dark => {
  const appearance = dark ? 'dark' : 'light';
  if (document.documentElement.dataset.appearance === appearance) return;
  document.documentElement.dataset.appearance = appearance;
  updateAppearance();
};
// This screen has no input binding and never executes expected/output bytes.
let outputTerminal, outputFit, outputBytes = '', outputResize;
function fitOutputViewport() {
  if (!document.body.matches('.terminal-mode:not(.interactive-terminal)')) return;
  const node = document.getElementById('formula');
  if (!innerHeight || !node.clientWidth) return;
  node.style.setProperty('--output-terminal-height', innerHeight + 'px');
  outputFit?.fit();
}
addEventListener('resize', fitOutputViewport);
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
// Source positions are host metadata. A process can only supply text.
globalThis.renderSourceOutput = (source, revision) => {
  const { text, links } = JSON.parse(source);
  if (/[\x00-\x08\x0b\x0c\x0e-\x1f\x7f]|\r(?!\n)/.test(text)) {
    globalThis.renderTerminal(text);
    return;
  }
  document.body.className = 'terminal-mode';
  const root = document.getElementById('formula');
  outputResize?.disconnect();
  outputTerminal?.dispose();
  outputTerminal = outputFit = undefined;
  outputBytes = '';
  fitOutputViewport();
  root.style.overflow = 'auto';
  const lines = document.createElement('pre');
  lines.style.cssText = 'margin:0;font:12px ui-monospace,monospace;white-space:pre-wrap';
  const rows = text.split(/\r?\n/);
  for (const [i, row] of rows.entries()) {
    if (row && Object.hasOwn(links, row)) {
      const line = links[row];
      const button = document.createElement('button');
      button.type = 'button'; button.textContent = row; button.title = 'Go to source line ' + (line + 1);
      button.style.cssText = 'font:inherit;color:inherit;background:none;border:0;padding:0;text-decoration:underline;cursor:pointer;white-space:inherit;text-align:left';
      button.addEventListener('click', () => followDiagramLink(String(line), revision));
      lines.append(button);
    } else lines.append(row);
    if (i < rows.length - 1) lines.append('\n');
  }
  root.replaceChildren(lines);
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
  node.setAttribute('aria-label', 'Terminal output');
  document.body.classList.add('terminal-mode'); status.textContent = '';
  fitOutputViewport();
  if (!globalThis.Terminal) { node.textContent = source; status.textContent = 'Terminal renderer unavailable. Prepare resources and rebuild.'; return; }
  try {
  if (!outputTerminal) {
    node.replaceChildren();
    outputTerminal = new Terminal({ disableStdin:true, convertEol:true, cursorBlink:false, screenReaderMode:true, fontSize:12, fontFamily:'ui-monospace, monospace', rows:6, cols:80, scrollback:5000, minimumContrastRatio:4.5, theme:outputTheme() });
    outputFit = new FitAddon.FitAddon(); outputTerminal.loadAddon(outputFit); outputTerminal.open(node);
    outputResize = new ResizeObserver(fitOutputViewport); outputResize.observe(node);
  }
  const append = source.startsWith(outputBytes);
  if (!append) outputTerminal.reset();
  outputTerminal.write(append ? source.slice(outputBytes.length) : source, fitOutputViewport);
  outputBytes = source;
  } catch (error) {
    outputResize?.disconnect(); try { outputTerminal?.dispose(); } catch (_) {}
    outputTerminal = outputFit = undefined; outputBytes = '';
    node.textContent = source; status.textContent = 'Terminal unavailable: ' + (error.message || 'prepare resources and rebuild');
  }
};
globalThis.renderDiagram = async function (source, renderer, sizeRevision = 0) {
  displayedMermaid = renderer === 'mermaid' ? [source, renderer, sizeRevision] : undefined;
  globalThis.mathRevision = sizeRevision;
  document.body.className = 'diagram-mode';
  const node = document.getElementById('formula'), status = document.getElementById('status');
  const revision = globalThis.diagramRevision = (globalThis.diagramRevision || 0) + 1;
  status.textContent = '';
  try {
    if (source.length > 200000) throw Error('Diagram is too large to preview.');
    if (!source.trim()) throw Error('Diagram source is empty.');
    if (renderer === 'graph') {
      const graph = JSON.parse(source);
      if (!graph || typeof graph !== 'object' || Array.isArray(graph)) throw Error('Graph source must be a JSON object.');
      const layout = graph.layout ?? {};
      if (typeof layout !== 'object' || Array.isArray(layout)) throw Error('Graph layout must be a JSON object.');
      if (Object.values(layout).some(box => !box || !Number.isFinite(box.x) || !Number.isFinite(box.y) ||
        (box.w != null && !Number.isFinite(box.w)) || (box.h != null && !Number.isFinite(box.h)))) throw Error('Graph layout needs numeric positions and sizes.');
      if (graph.topology != null && (typeof graph.topology !== 'object' || Array.isArray(graph.topology))) throw Error('Graph topology must be a JSON object.');
      const nodes = graph.topology?.nodes ?? graph.nodes ?? [];
      const edges = graph.topology?.edges ?? graph.edges ?? [];
      if (!Array.isArray(nodes) || !Array.isArray(edges)) throw Error('Graph nodes and edges must be arrays.');
      if (nodes.length > 500 || edges.length > 2000) throw Error('Diagram is too large to preview.');
      if (nodes.some(item => !item || typeof item.id !== 'string' || !item.id.trim() ||
        [item.label, item.href].some(value => value != null && typeof value !== 'string'))) throw Error('Graph nodes need a text ID and text for any label or destination.');
      if (new Set(nodes.map(item => item.id)).size !== nodes.length) throw Error('Graph node IDs must be unique.');
      if (edges.some(edge => !edge || typeof edge.from !== 'string' || typeof edge.to !== 'string' ||
        [edge.id, edge.label, edge.fromSide, edge.toSide].some(value => value != null && typeof value !== 'string'))) throw Error('Graph edges need text from and to IDs and text for any ID, label or connector.');
      if (!nodes.length) throw Error('Diagram has no nodes.');
      const ns = 'http://www.w3.org/2000/svg';
      const element = (name, attrs = {}, text) => { const el = document.createElementNS(ns, name); for (const [key, value] of Object.entries(attrs)) el.setAttribute(key, String(value)); if (text !== undefined) el.textContent = text; return el; };
      const boxes = new Map(nodes.map((n, i) => { const box = layout[n.id] || {}; return [n.id, { x: Number.isFinite(box.x) ? box.x : i % 4 * 170 + 10, y: Number.isFinite(box.y) ? box.y : Math.floor(i / 4) * 90 + 10, w: Number.isFinite(box.w) ? Math.max(30, box.w) : 140, h: Number.isFinite(box.h) ? Math.max(30, box.h) : 50 }]; }));
      const values = [...boxes.values()];
      const minX = values.length ? Math.min(...values.map(b => b.x)) - 10 : 0, minY = values.length ? Math.min(...values.map(b => b.y)) - 10 : 0;
      const width = values.length ? Math.max(...values.map(b => b.x + b.w)) - minX + 10 : 24;
      const height = values.length ? Math.max(...values.map(b => b.y + b.h)) - minY + 10 : 24;
      const hasLink = item => typeof item.href === 'string' && item.href.trim() !== '';
      const svg = element('svg', { viewBox: `${minX} ${minY} ${width} ${height}`, role: nodes.some(hasLink) ? 'group' : 'img', 'aria-label': 'Diagram preview', width, height });
      const defs = element('defs'), marker = element('marker', { id: 'arrow', markerWidth: 8, markerHeight: 8, refX: 7, refY: 4, orient: 'auto' });
      marker.append(element('path', { d: 'M0,0 L8,4 L0,8 Z', fill: 'currentColor' })); defs.append(marker); svg.append(defs);
      for (const edge of edges) { const a = boxes.get(edge.from), b = boxes.get(edge.to); if (a && b) svg.append(element('line', { x1: a.x + a.w, y1: a.y + a.h / 2, x2: b.x, y2: b.y + b.h / 2, stroke: 'currentColor', 'marker-end': 'url(#arrow)' })); }
      for (const item of nodes) {
        const b = boxes.get(item.id), group = element('g');
        group.append(element('rect', { ...b, width: b.w, height: b.h, rx: 6, fill: 'Canvas', stroke: 'currentColor' }));
        const label = element('text', { x: b.x + b.w / 2, y: b.y + b.h / 2, 'text-anchor': 'middle', 'dominant-baseline': 'middle', fill: 'currentColor' }, item.label || item.id);
        group.append(label);
        if (hasLink(item)) {
          group.style.cursor = 'pointer';
          group.setAttribute('role', 'link'); group.setAttribute('tabindex', '0');
          group.setAttribute('aria-label', 'Open ' + (item.label || item.id));
          label.setAttribute('text-decoration', 'underline');
          const follow = () => followDiagramLink(item.href, sizeRevision);
          group.addEventListener('click', follow);
          group.addEventListener('keydown', event => { if (event.key === 'Enter') { event.preventDefault(); follow(); } });
        } else group.prepend(element('title', {}, 'No workspace destination'));
        svg.append(group);
      }
      node.replaceChildren(svg);
    } else if (renderer === 'mermaid') {
      if (!globalThis.mermaid) throw Error('Diagram engine unavailable. Prepare native resources and rebuild.');
      mermaid.initialize({ startOnLoad: false, securityLevel: 'strict', theme: darkAppearance() ? 'dark' : 'neutral', maxTextSize: 200000 });
      const result = await mermaid.render('diagram-' + revision, source);
      if (revision === globalThis.diagramRevision) {
        node.innerHTML = result.svg;
        // Strict Mermaid disables authored handlers. Bind only quoted file links,
        // after rendering, without enabling arbitrary callback execution.
        for (const match of source.matchAll(/(?:^|[;\n])\s*click\s+([\w-]+)\s+(?:href\s+)?"([^"]+)"/g)) {
          const group = [...node.querySelectorAll('g.node')].find(el => el.id.match(/(?:^|-)flowchart-(.+)-\d+$/)?.[1] === match[1]);
          if (group) { group.style.cursor = 'pointer'; group.addEventListener('click', event => event.preventDefault()); group.addEventListener('dblclick', () => followDiagramLink(match[2], sizeRevision)); }
        }
        const svg = node.querySelector('svg'), box = svg?.viewBox?.baseVal;
        if (box?.width && box?.height) { svg.style.maxWidth = 'none'; svg.setAttribute('width', box.width); svg.setAttribute('height', box.height); }
      }
    } else throw Error('No preview for renderer ' + renderer + '.');
  } catch (error) {
    if (revision === globalThis.diagramRevision) {
      node.replaceChildren();
      status.textContent = (error.message || 'Could not preview this diagram.') + ' Use the diagram gutter to edit its source.';
    }
  }
  if (revision === globalThis.diagramRevision) fitPreview();
};

</script></body></html>
</ui:copy>

<ui:file path="apps/apple-simple/Sources/Resources/math.html"><ui:paste select="#simple-inline-math-0"/></ui:file>

The build helper copies assets from the locked frontend installation. Missing optional assets degrade to source rather than preventing native editing. Build outputs stay ignored and are not independent maintained copies.

SwiftPM copies the whole resource tree when esbuild rewrites a preview file.
Build the script and stylesheet in a temporary directory and publish changed
bytes atomically. A second run with identical inputs should keep the modification
times of all four generated preview files. A web source change still reaches the
bundle through esbuild's normal dependency resolution.

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


def bundle_preview(entry, filename, *options):
    web = ROOT / "apps/web"
    resources = ROOT / "apps/apple-simple/Sources/Resources/node_modules"
    resources.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as scratch:
        bundle = Path(scratch) / filename
        subprocess.run([
            str(web / "node_modules/.bin/esbuild"), "--bundle", "--format=iife",
            "--platform=browser", "--jsx=automatic", "--define:import.meta.env={}",
            "--log-level=warning", "--outfile=" + str(bundle), *options,
        ], input=entry, text=True, cwd=web, check=True)
        for output in Path(scratch).iterdir():
            destination = resources / output.name
            data = output.read_bytes()
            if destination.is_file() and destination.read_bytes() == data:
                continue
            with tempfile.NamedTemporaryFile(dir=resources, delete=False) as temporary:
                temporary.write(data)
                staged = temporary.name
            os.chmod(staged, 0o644)
            os.replace(staged, destination)


entry = "import { mountNativeTable, mountNativeOutput } from " + json.dumps(str(ROOT / "apps/web/src/components/MarkdownTable.tsx")) + ";\n" + "import " + json.dumps(str(ROOT / "apps/web/src/styles.css")) + ";\n" + "if (document.body.classList.contains('notebook-mode')) { globalThis.hickoryOutput = mountNativeOutput(document.getElementById('formula')); globalThis.hickoryOutput.update(globalThis.hickoryOutputSource || ''); } else globalThis.hickoryTable = mountNativeTable(document.getElementById('formula'), message => { const json = JSON.stringify(message); if (globalThis.webkit?.messageHandlers?.table) globalThis.webkit.messageHandlers.table.postMessage(JSON.parse(json)); else globalThis.HickoryTable?.postMessage(json); });"
bundle_preview(entry, "table.js")
print("Prepared shared native spreadsheet resources.")

# Bundle the same optional coverage renderer for both native hosts.
entry = "import { mountNativeCoverage } from " + json.dumps(str(ROOT / "apps/web/src/coverage/NativeCoverage.tsx")) + ";\n" + "globalThis.hickoryCoverage = mountNativeCoverage(document.getElementById('surface'), message => { const json = JSON.stringify(message); if (globalThis.webkit?.messageHandlers?.coverage) globalThis.webkit.messageHandlers.coverage.postMessage(JSON.parse(json)); else globalThis.HickoryCoverage?.postMessage(json); });"
bundle_preview(entry, "coverage.js")
print("Prepared shared native coverage resources.")

# Bundle the authoring form with the same parser the browser uses. Embedded
# WASM bytes let the native page stay local with network access disabled.
entry = (
    "import { mountNativeElements } from " + json.dumps(str(ROOT / "apps/web/src/components/ElementAuthoring.tsx")) + ";\n"
    "globalThis.hickoryElements = mountNativeElements(document.getElementById('elements'), message => {"
    "if (globalThis.webkit?.messageHandlers?.elements) globalThis.webkit.messageHandlers.elements.postMessage(message); "
    "else globalThis.HickoryElements?.postMessage(JSON.stringify(message)); });"
)
bundle_preview(entry, "element-authoring.js", "--loader:.wasm=binary", "--minify",
               "--define:process.env.NODE_ENV=\"production\"", "--define:import.meta.url=undefined")
print("Prepared shared native element authoring resources.")
</ui:copy>

<ui:file path="scripts/prepare-simple-resources.py"><ui:paste select="#simple-inline-math-1"/></ui:file>

Apple equations distinguish body ranges for safe edits from full syntax ranges for presentation. Dollar formulas replace their complete delimiter range within the surrounding sentence; math elements replace their whole declaration, body and closing tag. The local typesetter reports natural dimensions with a render revision. TextKit owns placement, and obsolete measurements cannot change a newer source snapshot. Source remains visible until a usable measurement arrives.

Apple previews keep the editor’s reserved inline slots stable. Measure rendered content at its natural size and shrink only when necessary to fit both dimensions; remeasure when fonts, asynchronous diagrams or the viewport change. Graph SVGs need explicit natural dimensions rather than percentage heights, which otherwise resolve against the WebView and distort sizing. Terminal fitting remains independent.

Apple embeds a small typesetter in the line’s floating editor. Navigation is restricted to the local page, and the LaTeX replacement still passes through source, permission, composition and undo checks.

The Swift package installs the preview page and its local terminal renderer next to the test bundle. Resolve that exact resource package when the preview is hosted by XCTest so the same renderer is available there.

<ui:copy id="simple-inline-math-2">
import SwiftUI
import WebKit
@MainActor final class MathCoordinator: NSObject, WKNavigationDelegate, WKScriptMessageHandler {
    var source = ""
    var diagramRenderer: String?
    var inline = false
    var onSize: ((CGSize) -> Void)?
    var onLink: ((String) -> Void)?
    private var revision = 0
    private var displayed: String?
    var darkAppearance = false
    private var appliedAppearance: Bool?
    var ready = false
    func display(_ view: WKWebView) {
        guard ready else { return }
        if appliedAppearance != darkAppearance {
            appliedAppearance = darkAppearance
            view.evaluateJavaScript("globalThis.setPreviewAppearance?.(\(darkAppearance))", completionHandler: nil)
        }
        guard let bytes = try? JSONSerialization.data(withJSONObject: source, options: .fragmentsAllowed), let json = String(data: bytes, encoding: .utf8) else { return }
        let key = (diagramRenderer.map { "diagram:" + $0 } ?? "math") + "\0" + String(inline) + "\0" + source
        guard displayed != key else { return }
        displayed = key; revision += 1
        if diagramRenderer == "notebook" { view.evaluateJavaScript("globalThis.renderNotebookOutput(" + json + ")", completionHandler: nil) }
        else if diagramRenderer == "terminal" { view.evaluateJavaScript("globalThis.renderTerminal(" + json + ")", completionHandler: nil) }
        else if diagramRenderer == "source-output" { view.evaluateJavaScript("globalThis.renderSourceOutput(" + json + "," + String(revision) + ")", completionHandler: nil) }
        else if let renderer = diagramRenderer, let data = try? JSONSerialization.data(withJSONObject: renderer, options: .fragmentsAllowed), let engine = String(data: data, encoding: .utf8) {
            view.evaluateJavaScript("globalThis.renderDiagram(" + json + "," + engine + "," + String(revision) + ")", completionHandler: nil)
        } else { view.evaluateJavaScript("globalThis.renderMath(" + json + "," + String(inline) + "," + String(revision) + ")", completionHandler: nil) }
    }
    func userContentController(_ userContentController: WKUserContentController, didReceive message: WKScriptMessage) {
        guard message.frameInfo.isMainFrame else { return }
        if message.name == "diagramLink" {
            guard let link = message.body as? [String: Any], link["revision"] as? Int == revision, let href = link["href"] as? String else { return }
            onLink?(href); return
        }
        guard
              let value = message.body as? [String: Any], value["revision"] as? Int == revision,
              let width = value["width"] as? Double, let height = value["height"] as? Double,
              width.isFinite, height.isFinite, width > 0, height > 0 else { return }
        onSize?(CGSize(width: width, height: height))
    }
    func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
        ready = true
        displayed = nil
        appliedAppearance = nil
        display(webView)
    }
    func webView(_ webView: WKWebView, decidePolicyFor navigationAction: WKNavigationAction, decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
        let url = navigationAction.request.url
        decisionHandler(url?.isFileURL == true || url?.scheme == "about" ? .allow : .cancel)
    }
}
// Inspect bundles without touching SwiftPM's fatal Bundle.module accessor.
func nativeMathResource(roots: [URL], bundles: [Bundle] = [.main, Bundle(for: MathCoordinator.self)], requireSpreadsheet: Bool = false) -> URL? {
    // Resource-only SwiftPM bundles need not initialize as Foundation Bundles.
    // Inspect their installed directories too, including macOS Contents layout.
    let installed = roots.flatMap { root in
        [root, root.appendingPathComponent("HickorySimple_HickorySimple.bundle"), root.appendingPathComponent("HickorySimple_HickorySimple.resources")]
    } + bundles.flatMap { [$0.bundleURL] + [$0.resourceURL].compactMap { $0 } }
    #if SWIFT_PACKAGE
    let testResources = bundles.filter { $0.bundleURL.pathExtension == "xctest" }.map {
        $0.bundleURL.deletingLastPathComponent().appendingPathComponent("HickorySimple_HickorySimple.bundle")
    }
    #else
    let testResources: [URL] = []
    #endif
    for root in installed + testResources {
        for relative in ["HickoryAssets/Resources/math.html", "Resources/math.html", "Contents/Resources/Resources/math.html", "Contents/Resources/math.html", "math.html"] {
            let url = root.appendingPathComponent(relative)
            guard FileManager.default.isReadableFile(atPath: url.path) else { continue }
            if requireSpreadsheet {
                let table = url.deletingLastPathComponent().appendingPathComponent("node_modules/table.js")
                guard FileManager.default.isReadableFile(atPath: table.path) else { continue }
            }
            return url
        }
    }
    return nil
}
@MainActor func makeMathView(_ coordinator: MathCoordinator) -> WKWebView {
    let configuration = WKWebViewConfiguration(); configuration.websiteDataStore = .nonPersistent(); configuration.preferences.javaScriptCanOpenWindowsAutomatically = false
    configuration.userContentController.add(coordinator, name: "mathSize")
    configuration.userContentController.add(coordinator, name: "diagramLink")
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
    var onLink: ((String) -> Void)? = nil
    @Environment(\.colorScheme) private var colorScheme

    func makeCoordinator() -> MathCoordinator { MathCoordinator() }
    func makeNSView(context: Context) -> WKWebView { context.coordinator.source = source; context.coordinator.diagramRenderer = diagramRenderer; context.coordinator.inline = inline; context.coordinator.darkAppearance = colorScheme == .dark; context.coordinator.onSize = onSize; context.coordinator.onLink = onLink; return makeMathView(context.coordinator) }
    func updateNSView(_ view: WKWebView, context: Context) { context.coordinator.source = source; context.coordinator.diagramRenderer = diagramRenderer; context.coordinator.inline = inline; context.coordinator.darkAppearance = colorScheme == .dark; context.coordinator.onSize = onSize; context.coordinator.onLink = onLink; context.coordinator.display(view) }
}
#else
struct NativeMathView: UIViewRepresentable {
    var source: String
    var diagramRenderer: String? = nil
    var inline = false
    var onSize: ((CGSize) -> Void)? = nil
    var onLink: ((String) -> Void)? = nil
    @Environment(\.colorScheme) private var colorScheme

    func makeCoordinator() -> MathCoordinator { MathCoordinator() }
    func makeUIView(context: Context) -> WKWebView { context.coordinator.source = source; context.coordinator.diagramRenderer = diagramRenderer; context.coordinator.inline = inline; context.coordinator.darkAppearance = colorScheme == .dark; context.coordinator.onSize = onSize; context.coordinator.onLink = onLink; return makeMathView(context.coordinator) }
    func updateUIView(_ view: WKWebView, context: Context) { context.coordinator.source = source; context.coordinator.diagramRenderer = diagramRenderer; context.coordinator.inline = inline; context.coordinator.darkAppearance = colorScheme == .dark; context.coordinator.onSize = onSize; context.coordinator.onLink = onLink; context.coordinator.display(view) }
}
#endif
// One read-only terminal surface for captured bytes and expect bodies.
struct NativeInlineTerminal: View {
    var text: String
    var editSource: (() -> Void)? = nil
    var sourceLinks: [String: Int] = [:]
    var onSource: ((Int) -> Void)? = nil
    var body: some View {
        VStack(spacing: 0) {
            HStack { Image(systemName: "terminal"); Text(text.contains("\u{1e}HICKORY_DISPLAY ") ? "Output" : "Terminal"); Spacer(); if let editSource { Button("Edit source text", action: editSource).buttonStyle(.plain) } }
                .font(.system(size: 11, design: .monospaced)).padding(6).background(.primary.opacity(0.06))
            if !sourceLinks.isEmpty, let json = try? JSONSerialization.data(withJSONObject: ["text": text, "links": sourceLinks]) {
                NativeMathView(source: String(decoding: json, as: UTF8.self), diagramRenderer: "source-output", onLink: { href in
                    if let line = Int(href), sourceLinks.values.contains(line) { onSource?(line) }
                })
            } else { NativeMathView(source: text, diagramRenderer: text.contains("\u{1e}HICKORY_DISPLAY ") ? "notebook" : "terminal") }
        }.foregroundStyle(.primary).background(.background)
            .clipShape(RoundedRectangle(cornerRadius: 6)).overlay(RoundedRectangle(cornerRadius: 6).stroke(.secondary.opacity(0.35)))
            .accessibilityElement(children: .contain).accessibilityLabel("Output terminal")
    }
}
struct NativeDiagramPreview: View {
    var diagram: InlineDiagram
    var buffer: DocumentBuffer
    @State private var architecture: String?
    @State private var architectureNotice = "Reading the code model…"
    @State private var architectureRefresh = 0
    private var current: Bool {
        exactUTF8(buffer.text, buffer.inlineMarkup.source) &&
            buffer.inlineMarkup.diagrams.contains { $0.id == diagram.id && $0.body == diagram.body && $0.renderer == diagram.renderer }
    }
    var body: some View {
        if diagram.renderer == "terminal" { NativeInlineTerminal(text: diagram.body, editSource: { buffer.sourceTable = diagram.id }) }
        else if diagram.renderer == "code-model" {
            ZStack(alignment: .topTrailing) {
                if let architecture { preview(architecture, renderer: "graph") }
                else { Text(architectureNotice).font(.caption).frame(maxWidth: .infinity, maxHeight: .infinity) }
                Button("Refresh diagram", systemImage: "arrow.clockwise") { architectureRefresh += 1 }.labelStyle(.iconOnly)
            }.task(id: "\(architectureRefresh):\(diagram.body)") {
                let path = buffer.file.path
                var revision: String?
                architecture = nil
                architectureNotice = "Reading the code model…"
                repeat {
                    do {
                        guard let engine = buffer.receiptEngine else {
                            architectureNotice = "Reconnect to the workspace and refresh."
                            return
                        }
                        var body: [String: Any] = ["uri": nativeDebugURI(path), "script": diagram.body]
                        if let revision { body["revision"] = revision }
                        let result: NativeTableJSON = try await engine.request("POST", "/api/diagram/model", body: body)
                        guard !Task.isCancelled, current, exactUTF8(path, buffer.file.path) else { return }
                        if let update = result.object as? [String: Any], let scene = update["scene"] as? [String: Any] {
                            architecture = String(decoding: try JSONSerialization.data(withJSONObject: scene, options: [.sortedKeys]), as: UTF8.self)
                            revision = update["revision"] as? String
                        }
                    } catch {
                        guard !Task.isCancelled, current else { return }
                        architectureNotice = error.localizedDescription
                        architecture = nil
                        revision = nil
                    }
                    try? await Task.sleep(for: .milliseconds(2500))
                } while !Task.isCancelled && current
            }
        } else { preview(diagram.body, renderer: diagram.renderer) }
    }

    private func preview(_ source: String, renderer: String) -> some View {
        NativeMathView(source: source, diagramRenderer: renderer, onSize: { size in
            guard current, buffer.diagramSizes[diagram.id] != size else { return }
            buffer.diagramSizes[diagram.id] = size; buffer.renderMarkup?()
        }, onLink: { href in if current { buffer.openDiagramFile?(href) } }).background(.background)
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
            TextEditor(text: $draft).font(.system(.caption, design: .monospaced)).frame(height: 64).disabled(reason: buffer.readOnly ? "This source is read-only." : nil)
            Button("Apply equation") {
                if buffer.applySourceEdits?([NativeSourceEdit(range: equation.range, text: draft)], source) != true { notice = "Equation changed. Reopen this line and try again." }
            }.disabled(reason: buffer.readOnly ? "This source is read-only." : draft == equation.body ? "Edit the equation before applying it." : nil)
            if !notice.isEmpty { Text(notice).font(.caption) }
        }.onAppear { draft = equation.body }
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/InlineMath.swift"><ui:paste select="#simple-inline-math-2"/></ui:file>

Android uses the same local page with network and content access disabled. Source remains in TextFieldState; the WebView receives no native mutation bridge and is destroyed when its contextual control closes.

The WebView keeps its active revision and script together in its ordinary
view tag. Android reserves keyed tags for application resource IDs, so a
framework ID can throw while the view is mounting, including when a recorded
fence opens its terminal. Both the navigation and completion callbacks check
the active preview before exposing a result.

<ui:copy id="simple-inline-math-3">
package com.hickorydocs.simple

import android.annotation.SuppressLint
import android.webkit.WebView
import android.webkit.WebViewClient
import android.webkit.WebResourceRequest
import android.webkit.RenderProcessGoneDetail
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import org.json.JSONObject

private data class MathPreview(val revision: Int, val script: String)

private fun mathPreviewScript(source: String, renderer: String?, revision: Int): String {
    val escaped = JSONObject.quote(source)
    return when (renderer) {
        "notebook" -> "globalThis.renderNotebookOutput($escaped)"
        "terminal" -> "globalThis.renderTerminal($escaped)"
        "source-output" -> "globalThis.renderSourceOutput($escaped,$revision)"
        null -> "globalThis.renderMath($escaped)"
        else -> "globalThis.renderDiagram($escaped,${JSONObject.quote(renderer)},$revision)"
    }
}

@SuppressLint("SetJavaScriptEnabled")
@Composable fun NativeMathView(source: String, modifier: Modifier = Modifier, diagramRenderer: String? = null, onReady: (() -> Unit)? = null, onLink: ((String) -> Unit)? = null) {
    val ready = rememberUpdatedState(onReady)
    val activateLink = rememberUpdatedState(onLink)
    val dark = rememberUpdatedState(isSystemInDarkTheme())
    AndroidView(modifier = modifier, factory = { context ->
        WebView(context).apply {
            settings.javaScriptEnabled = true; settings.allowFileAccess = false; settings.allowContentAccess = false; settings.blockNetworkLoads = true
            webViewClient = object : WebViewClient() {
                override fun onRenderProcessGone(view: WebView, detail: RenderProcessGoneDetail): Boolean { view.destroy(); return true }
                override fun shouldOverrideUrlLoading(view: WebView, request: WebResourceRequest): Boolean {
                    val url = request.url
                    val preview = view.tag as? MathPreview
                    if (preview != null && request.isForMainFrame && url.scheme == "hickory-diagram" && url.host == "navigate" &&
                        url.getQueryParameter("revision") == preview.revision.toString()) {
                        url.pathSegments.singleOrNull()?.let { activateLink.value?.invoke(it) }
                    }
                    return url.toString() != "file:///android_asset/math.html"
                }
                override fun onPageFinished(view: WebView, url: String) {
                    if (url == "file:///android_asset/math.html") {
                        val preview = view.tag as? MathPreview ?: return
                        view.evaluateJavascript("globalThis.hickoryAndroidDiagram=true;globalThis.setPreviewAppearance(${dark.value});" + preview.script + ";!!document.querySelector('.katex')") {
                            if (it == "true" && view.tag === preview) ready.value?.invoke()
                        }
                    }
                }
            }
            tag = MathPreview(1, mathPreviewScript(source, diagramRenderer, 1))
            loadUrl("file:///android_asset/math.html")
        }
    }, update = { view ->
        view.evaluateJavascript("globalThis.setPreviewAppearance?.(${dark.value})", null)
        val current = view.tag as? MathPreview ?: return@AndroidView
        if (current.script != mathPreviewScript(source, diagramRenderer, current.revision)) {
            val revision = current.revision + 1
            val preview = MathPreview(revision, mathPreviewScript(source, diagramRenderer, revision))
            view.tag = preview
            view.evaluateJavascript("if(globalThis.renderDiagram){" + preview.script + "};!!document.querySelector('.katex')") {
                if (it == "true" && view.tag === preview) ready.value?.invoke()
            }
        }
    }, onRelease = { it.stopLoading(); it.destroy() })
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/InlineMath.kt"><ui:paste select="#simple-inline-math-3"/></ui:file>


The browser retains the established rendered-block registry and table/math/exec views. Their React owners live above pane layout, preserving widget-local edits when the cached editor moves. Widgets use a shadow root to reuse the established stylesheet without changing simple-UI chrome. Mutations still use the existing captured-block transaction guard; saved receipts disappear as soon as the buffer is dirty. The optional rendering modules load after readable text mounts.

Embedded widgets reuse the full editor’s structural CSS, but must not adopt its
default dark palette. Override its host tokens with system colors after the
shared styles. Inheriting the surrounding color scheme lets live system-theme
changes restyle controls, tables, math and graph canvases without remounting a
widget or discarding a cell draft. Pair semantic colors for light and dark
grounds so status and diff text remain legible in either mode.

The right rail belongs to source positions, independently of mounted preview
widgets. Derive its controls from current renderable blocks so revealing source
does not destroy the only control capable of restoring the preview. Both
operations change decorations, preserving the editor, undo and unsaved text.
Equation modes can explicitly override caret-driven source reveal in either
direction. The existing rail placement/overflow mechanism owns icon geometry.

Inline rendering reports a failed asset through the workspace status in View or
the Agent heading. The retained editor keeps its viewport when this optional
capability fails. Only the renderer that still owns the retained view can
publish its failure. The focused `--delayed-capabilities-only` and
`--agent-only` browser replays abort the stylesheet after editing and check
that the failure leaves the editor, history and draft in place.

In the published Simple Browser reader, the host supplies a URL resolver. Its
links navigate in the browser. Live editors still resolve links against the
workspace, with the same source-position checks.

<ui:copy id="simple-inline-browser-owners">
import { Component, useEffect, useState, useSyncExternalStore } from "react";
import { createPortal } from "react-dom";
import { Buffer, Buffers } from "./buffers";
import { editorForBuffer, SIMPLE_EDITOR_ATTACHED } from "./Document";
import { EditorView } from "@codemirror/view";
import { DebugRailGroup, debugRailElement } from "../debug/DebugRail";
import type { RenderedSlot } from "../editor/rendered";
import type { Block, ExecBlock, DiagramBlock } from "../api/types";
import { request } from "../api/client";
import { openLocation, workspaceLinkTarget } from "../lib/revealLine";

const inlineUnavailable = "Inline rendering unavailable. Continue editing source or reload to retry.";

// The owners live above pane layout. Moving a cached editor cannot unmount a
// table's uncommitted cell editor or lose a rendered widget's own local state.
type BrowserLink = (target: string) => string | null;

export function RichOwners({ buffers, recorded = false, browserLink }: {
  buffers: Buffers; recorded?: boolean; browserLink?: BrowserLink;
}) {
  const [, refresh] = useState(0);
  useEffect(() => { const update = () => refresh(value => value + 1); window.addEventListener(SIMPLE_EDITOR_ATTACHED, update); update(); return () => window.removeEventListener(SIMPLE_EDITOR_ATTACHED, update); }, []);
  return <>{[...buffers.values.values()].map(buffer => {
    const cached = editorForBuffer(buffer);
    return cached && /\.(md|markdown)$/i.test(buffer.file.path) ? <RichBoundary key={buffer.file.path} buffer={buffer} view={cached.view}><RichOwner buffer={buffer} view={cached.view} keepPatchSource={recorded && buffer.readonly} browserLink={browserLink} /></RichBoundary> : null;
  })}</>;
}
class RichBoundary extends Component<{ buffer: Buffer; view: EditorView; children: React.ReactNode }, { failed: boolean }> {
  state = { failed: false };
  static getDerivedStateFromError() { return { failed: true }; }
  componentDidCatch() {
    const cached = editorForBuffer(this.props.buffer);
    if (cached?.view !== this.props.view) return;
    cached.richAttached = false;
    cached.view.dispatch({ effects: [cached.prose.reconfigure([]), cached.view.scrollSnapshot()] });
    this.props.buffer.setInlineRenderNotice(inlineUnavailable);
  }
  render() { return this.state.failed ? null : this.props.children; }
}
function SlotPortal({ slot, children, css }: { slot: RenderedSlot; children: React.ReactNode; css: string }) {
  const [root] = useState(() => slot.el.shadowRoot ?? slot.el.attachShadow({ mode: "open" }));
  return createPortal(<><style>{css.replaceAll(":root", ":host") + `
:host {
  color-scheme: inherit; color: CanvasText;
  --bg: Canvas; --fg: CanvasText; --fg-muted: color-mix(in srgb, CanvasText 65%, Canvas); --muted: var(--fg-muted);
  --bg-raised: Canvas; --bg-sunken: color-mix(in srgb, CanvasText 5%, Canvas);
  --border: color-mix(in srgb, CanvasText 22%, Canvas);
  --accent: Highlight; --accent-2: light-dark(#176b73, #71d1d9);
  --ok: light-dark(#267341, #8cdaa2); --fail: light-dark(#b42330, #ff9696);
  --danger: var(--fail); --warn: light-dark(#855b0a, #ecc07c); --stale: var(--warn);
  --ok-bg: color-mix(in srgb, var(--ok) 10%, Canvas);
  --fail-bg: color-mix(in srgb, var(--fail) 10%, Canvas);
  --stale-bg: color-mix(in srgb, var(--warn) 10%, Canvas);
  --neutral-bg: var(--bg-sunken);
  --inline-terminal-bg: var(--bg-sunken);
  --inline-terminal-fg: CanvasText;
  --inline-terminal-heading: color-mix(in srgb, CanvasText 8%, Canvas);
  --inline-terminal-border: var(--border);
  --inline-terminal-scheme: light dark;
  --diff-del: var(--fail); --diff-ins: var(--ok);
  --select-bg: color-mix(in srgb, Highlight 22%, Canvas);
  --search-hit: color-mix(in srgb, var(--warn) 20%, Canvas);
  --search-hit-on: color-mix(in srgb, var(--warn) 40%, Canvas);
  --tok-keyword: light-dark(#7637a0, #d4a5ff);
  --tok-function: light-dark(#145e9c, #83c7ff);
  --tok-string: var(--ok); --tok-comment: GrayText;
  --tok-literal: var(--warn); --tok-type: var(--accent-2);
  --tok-property: var(--tok-function); --tok-tag: var(--tok-keyword);
  --tok-meta: GrayText; --tok-operator: GrayText;
  --tok-parameter: var(--tok-type); --tok-invalid: var(--fail);
}
`}</style>{children}</>, root);
}
function RichOwner({ buffer, view, keepPatchSource, browserLink }: {
  buffer: Buffer; view: EditorView; keepPatchSource: boolean; browserLink?: BrowserLink;
}) {
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
        EditorView.updateListener.of(update => { if (update.transactions.length) queueMicrotask(() => { if (!disposed) setSlots(registry.list()); }); }),
        libs.math.renderedMath(state => simpleMathSpans(libs, state)),
        browserLink ? libs.publicLinks.publishedProseLinks(browserLink) : libs.links.mdLinks({
          docPath: () => buffer.file.path,
          skip: state => [...libs.hick.verbatimRanges(libs.prose.structureOf(state).blocks),
            ...libs.markdown.fencedCodeRanges(state.doc.toString())].sort(([a], [b]) => a - b),
          onFollow: href => {
            const target = workspaceLinkTarget(href, buffer.file.path);
            if (!target || !view.dom.isConnected) return { status: "unavailable", reason: "Choose a link to a file in this workspace." };
            openLocation(target.path, target.line);
            return { status: "completed", value: { navigation_requested: true } };
          },
        }),
      ]) });
      const initial = libs.elements.renderableBlocks(libs.prose.structureOf(view.state), view.state.doc.toString()).filter(block =>
        block.name !== "math" && block.name !== "patch" && !libs.elements.slotKindOf(block)?.startsWith("session-")
        && !(buffer.dirty && view.state.selection.ranges.some(range => range.from >= block.from && range.to <= block.to)));
      view.dispatch({ effects: [libs.rendered.setRenderedBlocks.of(initial.map(block => block.from)), scroll] });
      buffer.setInlineRenderNotice("");
      setSlots(registry.list()); setModules(libs);
      };
      if (view.dom.isConnected) install();
      else {
        const attached = () => { if (view.dom.isConnected) { stopWaiting?.(); install(); } };
        window.addEventListener(SIMPLE_EDITOR_ATTACHED, attached);
        stopWaiting = () => window.removeEventListener(SIMPLE_EDITOR_ATTACHED, attached);
      }
    }).catch(() => {
      if (!disposed && editorForBuffer(buffer)?.view === view) buffer.setInlineRenderNotice(inlineUnavailable);
    });
    return () => { disposed = true; stopWaiting?.(); off?.(); const cached = editorForBuffer(buffer); if (cached?.view === view) { cached.richAttached = false; view.dispatch({ effects: cached.prose.reconfigure([]) }); } };
  }, [buffer, view, keepPatchSource, browserLink, buffer.text.length > 200_000]);
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
      if (!cancelled && buffer.file.doc_id === id && buffer.text === source && !buffer.dirty && result.document === id && result.source_hash.replace(/^sha256:/, "") === expected) setBlocks({ source, blocks: result.blocks });
    }).catch(() => { /* Saved receipts are optional, never a reason to hide source. */ });
    return () => { cancelled = true; };
  }, [buffer, buffer.file.doc_id, buffer.base, buffer.dirty, receiptRevision]);
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
  const controls = modules.elements.renderableBlocks(modules.prose.structureOf(view.state), view.state.doc.toString()).filter(block =>
    block.name !== "math" && !(keepPatchSource && block.name === "patch") && !modules.elements.slotKindOf(block)?.startsWith("session-"));
  const math = simpleMathSpans(modules, view.state);
  const rail = debugRailElement(view);
  return <>{rail && math.map(span => createPortal(<DebugRailGroup key={`math:${span.from}`} view={view} at={span.from}>
    <button type="button" className="debug-icon" aria-label="Toggle equation source text" onClick={() => {
      const mode = view.state.field(modules.math.mathSourceModes).get(span.from);
      const touched = view.state.selection.ranges.some(range => view.state.doc.lineAt(range.from).number <= view.state.doc.lineAt(span.to).number && view.state.doc.lineAt(range.to).number >= view.state.doc.lineAt(span.from).number);
      view.dispatch({ effects: modules.math.setMathSource.of({ at: span.from, source: !(mode ?? touched) }) });
    }}>‹/›</button>
  </DebugRailGroup>, rail))}{rail && controls.map(block => createPortal(<DebugRailGroup key={block.from} view={view} at={block.from}>
    <button type="button" className="debug-icon" aria-label={modules.rendered.isRendered(view.state, block.from) ? "Edit source text" : "Render embedded content"} aria-pressed={!modules.rendered.isRendered(view.state, block.from)} onClick={() => {
      const effects = modules.rendered.isRendered(view.state, block.from) ? modules.rendered.showBlockSource.of(block.from) : modules.rendered.renderBlock.of(block.from);
      view.dispatch({ effects });
    }}>‹/›</button>
  </DebugRailGroup>, rail))}{slots.map(slot => {
    const context = { simpleTables: true, systemAppearance: true, view, path: buffer.file.path, docId: buffer.file.doc_id, execBlocks: saved.filter((block): block is ExecBlock => block.kind === "exec"),
      diagramBlocks: saved.filter((block): block is DiagramBlock => block.kind === "diagram"), runningCells: new Set<string>(), replaying: [],
      replaceBlockContent: (target: RenderedSlot, text: string, origin?: string) => modules.edit.replaceCapturedBlock(view, target, text, origin ?? "input.table"),
    };
    return <SlotPortal key={slot.key} slot={slot} css={modules.css.default + modules.katexCss.default + modules.graphCss.default + modules.terminalCss.default}>
      {slot.kind === "exec" && buffer.file.doc_id && <button type="button" disabled={buffer.dirty} title={buffer.dirty ? "Save the source before refreshing the recorded output." : undefined} onClick={() => refreshReceipts(value => value + 1)}>Refresh recorded output</button>}
      {modules.elements.elementViews[slot.kind].render(slot, context)}
    </SlotPortal>;
  })}</>;
}
function simpleMathSpans(libs: Awaited<ReturnType<typeof load>>, state: import("@codemirror/state").EditorState) {
  const { blocks } = libs.prose.structureOf(state);
  return [...libs.spans.mathSpans(state.doc.toString(), libs.hick.verbatimRanges(blocks)),
    ...blocks.filter(block => block.name === "math" && block.close).map(block => ({
      from: block.from, to: block.to, display: true, source: state.doc.sliceString(block.contentFrom, block.contentTo),
    }))].sort((a, b) => a.from - b.from);
}
async function load() {
  // Resolve the shared cyclic registry graph before importing its members in parallel.
  const parser = await import("../editor/hickLang");
  await parser.loadHickLang();
  const rendered = await import("../editor/rendered");
  const [prose, math, spans, hick, elements, edit, css, katexCss, graphCss, terminalCss, links, markdown, publicLinks] = await Promise.all([
    import("../editor/wysiwyg"), import("../editor/mathRender"), import("../lib/math"), import("../editor/hickDoc"),
    import("../elements"), import("../editor/replaceCapturedBlock"), import("../styles.css?inline"), import("katex/dist/katex.min.css?inline"), import("@xyflow/react/dist/style.css?inline"), import("@xterm/xterm/css/xterm.css?inline"), import("../editor/mdLinks"), import("../editor/markdownStyling"), import("../embed/sourceLinks"),
  ]);
  return { prose, rendered, math, spans, hick, elements, edit, css, katexCss, graphCss, terminalCss, links, markdown, publicLinks };
}
</ui:copy>

<ui:file path="apps/web/src/simple/RichInline.tsx"><ui:paste select="#simple-inline-browser-owners"/></ui:file>

</ui:doc>
