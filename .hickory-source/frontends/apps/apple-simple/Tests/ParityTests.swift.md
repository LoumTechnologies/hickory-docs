<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Execute the shared pane scenarios in Swift

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

The language-cache witnesses retain colors on unchanged Unicode text, exercise a real AppKit edit/undo and the refresh task with a closed channel, and reject stale completion writes. They do not prove live-provider or mobile gesture behavior.

The gutter witness lays out real AppKit text, including wrapped text and empty final fragments, and checks the logical rows used to draw numbers. It does not verify UIKit or Compose rendering.

These cases consume the same JSON as the browser tests. They check user-visible tab membership and active-tab validity independently of view framework and internal split identifiers.

The table replacement witnesses admit only prose cells, retain CRLF and Unicode source ranges, and inspect AppKit’s actual grid frame and following text geometry. The scanner callback must also refresh the measured line rail without a scroll event. Source reveal and native undo keep the same text storage. The anchor witness drives native backward/forward Delete at the table’s boundaries and in its cells, then checks that source mode and ordinary prose deletion still work. These are in-process geometry checks; they do not establish device gestures or screen-reader acceptance.

The indentation witness checks four spaces, two spaces, literal tabs, whitespace-only fallback and two successive Tab presses after partial indentation. These assertions protect alignment without needing a language server.

<ui:copy id="simple-7d7b2a4b09-0">
// Contract: .hickory-source/frontends/contract.md
import XCTest
#if os(macOS)
import AppKit
import SwiftUI
import WebKit
#endif
@testable import HickorySimple

final class ParityTests: XCTestCase {
    func testDetectedIndentationAndPartialTabStops() {
        // Contract: .hickory-source/frontends/contract.md
        XCTAssertEqual(simpleIndent("root\n    one\n        two\n"), "    ")
        XCTAssertEqual(simpleIndent("root\n  one\n    two\n"), "  ")
        XCTAssertEqual(simpleIndent("root\n\tone\n\t\ttwo\n"), "\t")
        XCTAssertEqual(simpleIndent("\n   \n"), "    ")
        let source = "root\n    child\n "
        XCTAssertEqual(simpleTabText(source, at: source.utf16.count), "   ")
        XCTAssertEqual(simpleTabText(source + "   ", at: source.utf16.count + 3), "    ")
        let block = "one\ntwo"
        let edits = simpleTabEdits(block, range: NSRange(location: 0, length: block.utf16.count))
        XCTAssertEqual(edits.map(\.text), ["    ", "    "])
        XCTAssertTrue(edits.allSatisfy { $0.range.length == 0 })
    }

    func testInlineMarkupKeepsSourceCoordinatesAndLiteralPayloads() {
        let source = "# Heading\r\n**bold** and `**literal**`\r\n| Name | Value |\r\n| --- | --- |\r\n| 😀 | a\\|b |\r\n```md\r\n# raw\r\n```\r\n<h:math>\r\nx^2\r\n</h:math>\r\n"
        let result = computeInlineMarkup(source, path: "source.md")
        XCTAssertEqual(result.source, source)
        XCTAssertEqual(inlineLineAt((source as NSString).range(of: "😀").location, starts: result.lineStarts), 4)
        XCTAssertEqual(result.marks.filter { $0.kind == "heading" }.count, 1)
        XCTAssertEqual(result.marks.filter { $0.kind == "strong" }.count, 1)
        XCTAssertEqual(result.cells.map(\.value), ["Name", "Value", "😀", "a|b"])
        XCTAssertEqual((source as NSString).substring(with: result.cells[2].range), "😀")
        XCTAssertFalse(result.cells.contains { $0.value.contains("\r") })
        XCTAssertEqual(result.equations.count, 1)
        XCTAssertEqual((source as NSString).substring(with: result.equations[0].range), "\r\nx^2\r\n")
        XCTAssertTrue(computeInlineMarkup("$5 and $6", path: "money.md").equations.isEmpty)
        XCTAssertTrue(computeInlineMarkup(source, path: "code.py").marks.isEmpty)
        XCTAssertEqual(inlineCellText("a|b"), "a\\|b")
        let csv = "\nName,Value\r\n\"a,b\",\"two\nlines\"\r\nlast,"
        let cells = inlineCSVCells(csv, line: 0, offset: 10, delimiter: ",")
        XCTAssertEqual(cells.map(\.value), ["Name", "Value", "a,b", "two\nlines", "last", ""])
        XCTAssertEqual(inlineCells("| ``a|`b`` | c |", line: 0, offset: 0).count, 2)
        XCTAssertEqual(cells.last?.line, 4)
        XCTAssertEqual(inlineCellText("a,\"b", format: "csv"), "\"a,\"\"b\"")
        XCTAssertTrue(inlineCSVCells("\"unterminated", line: 0, offset: 0, delimiter: ",").isEmpty)

    }
    func testGridCaretEntryUsesVisualColumnAndWholeGraphemes() {
        // Contract: .hickory-source/frontends/contract.md
        let measure: (String) -> CGFloat = { CGFloat($0.utf16.count * 5) }
        XCTAssertEqual(gridCaretOffset("a😀é", request: GridCaretRequest(x: 14), measure: measure), 3)
        XCTAssertEqual(gridCaretOffset("a😀é", request: GridCaretRequest(x: 100), measure: measure), 5)
        XCTAssertEqual(gridCaretOffset("", request: GridCaretRequest(x: 100), measure: measure), 0)
        XCTAssertEqual(gridCaretOffset("previous", request: GridCaretRequest(edge: -1), measure: measure), 8)
        XCTAssertEqual(gridCaretOffset("next", request: GridCaretRequest(edge: 1), measure: measure), 0)
    }
    #if os(macOS)
    @MainActor func testGridHorizontalKeysMoveWithinTextBeforeLeavingCell() async throws {
        // Contract: .hickory-source/frontends/contract.md
        var exits: [Int] = []
        let field = GridCellField(text: .constant("abcd"), request: nil, readOnly: false, onFocus: {}, onCommit: {}, onArrow: { _, dc, _ in exits.append(dc) })
        let host = NSHostingView(rootView: field)
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 200, height: 40), styleMask: [.titled], backing: .buffered, defer: false)
        window.contentView = host; host.layoutSubtreeIfNeeded(); defer { window.orderOut(nil) }
        func find(_ view: NSView) -> NSTextField? { (view as? NSTextField) ?? view.subviews.lazy.compactMap { find($0) }.first }
        for _ in 0..<20 { if find(host) != nil { break }; try await Task.sleep(for: .milliseconds(10)) }
        let control = try XCTUnwrap(find(host)); XCTAssertTrue(window.makeFirstResponder(control))
        let editor = try XCTUnwrap(control.currentEditor() as? NSTextView)
        editor.setSelectedRange(NSRange(location: 2, length: 0))
        editor.doCommand(by: NSSelectorFromString("moveLeft:"))
        XCTAssertEqual(editor.selectedRange(), NSRange(location: 1, length: 0)); XCTAssertTrue(exits.isEmpty)
        editor.doCommand(by: NSSelectorFromString("moveRight:"))
        XCTAssertEqual(editor.selectedRange(), NSRange(location: 2, length: 0)); XCTAssertTrue(exits.isEmpty)
        editor.setSelectedRange(NSRange(location: 1, length: 2))
        editor.doCommand(by: NSSelectorFromString("moveLeft:"))
        XCTAssertEqual(editor.selectedRange(), NSRange(location: 1, length: 0)); XCTAssertTrue(exits.isEmpty)
        editor.setSelectedRange(NSRange(location: 0, length: 0))
        editor.doCommand(by: NSSelectorFromString("moveLeft:")); XCTAssertEqual(exits, [-1])
        editor.setSelectedRange(NSRange(location: 4, length: 0))
        editor.doCommand(by: NSSelectorFromString("moveRight:")); XCTAssertEqual(exits, [-1, 1])
    }
    @MainActor func testGridArrowsTransferNativeCaretBetweenCells() async throws {
        // Contract: .hickory-source/frontends/contract.md
        let source = "| WWWW | z |\n| --- | --- |\n| iiiiiii | target |"
        let file = FileNode(name: "grid.md", path: "grid.md", dir: false)
        let buffer = DocumentBuffer(file: file, text: FileText(path: file.path, content: source, hash: "base"))
        buffer.inlineMarkup = computeInlineMarkup(source, path: file.path)
        let table = try XCTUnwrap(inlineTables(buffer.inlineMarkup).first)
        let host = NSHostingView(rootView: NativeTableGrid(table: table, source: source, buffer: buffer))
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 400, height: 180), styleMask: [.titled], backing: .buffered, defer: false)
        window.contentView = host; host.layoutSubtreeIfNeeded(); defer { window.orderOut(nil) }
        func find(_ view: NSView, _ value: String) -> NSTextField? {
            if let field = view as? NSTextField, field.stringValue == value { return field }
            return view.subviews.lazy.compactMap { find($0, value) }.first
        }
        for _ in 0..<20 { if find(host, "WWWW") != nil { break }; try await Task.sleep(for: .milliseconds(10)) }
        let first = try XCTUnwrap(find(host, "WWWW"))
        XCTAssertTrue(window.makeFirstResponder(first))
        let editor = try XCTUnwrap(first.currentEditor() as? NSTextView)
        editor.setSelectedRange(NSRange(location: 1, length: 0))
        let delegate = try XCTUnwrap(first.delegate as? GridCellField.Coordinator)
        XCTAssertTrue(delegate.control(first, textView: editor, doCommandBy: NSSelectorFromString("moveDown:")))
        for _ in 0..<20 { if (window.firstResponder as? NSTextView)?.string == "iiiiiii" { break }; try await Task.sleep(for: .milliseconds(10)) }
        let destination = try XCTUnwrap(window.firstResponder as? NSTextView)
        let font = NSFont.systemFont(ofSize: 13)
        let x = ("W" as NSString).size(withAttributes: [.font: font]).width
        let expected = gridCaretOffset("iiiiiii", request: GridCaretRequest(x: x)) { ($0 as NSString).size(withAttributes: [.font: font]).width }
        XCTAssertEqual(destination.string, "iiiiiii")
        XCTAssertEqual(destination.selectedRange(), NSRange(location: expected, length: 0))
        let second = try XCTUnwrap(find(host, "iiiiiii"))
        let secondDelegate = try XCTUnwrap(second.delegate as? GridCellField.Coordinator)
        XCTAssertFalse(secondDelegate.control(second, textView: destination, doCommandBy: NSSelectorFromString("moveRight:")))
        destination.setSelectedRange(NSRange(location: 7, length: 0))
        XCTAssertTrue(secondDelegate.control(second, textView: destination, doCommandBy: NSSelectorFromString("moveRight:")))
        for _ in 0..<20 { if (window.firstResponder as? NSTextView)?.string == "target" { break }; try await Task.sleep(for: .milliseconds(10)) }
        XCTAssertEqual((window.firstResponder as? NSTextView)?.string, "target")
        XCTAssertEqual((window.firstResponder as? NSTextView)?.selectedRange(), NSRange(location: 0, length: 0))
    }
    @MainActor func testGridFieldArrowEntryCollapsesNativeSelection() async throws {
        // Contract: .hickory-source/frontends/contract.md
        let request = GridCaretRequest(edge: -1)
        let field = GridCellField(text: .constant("Café 😀"), request: request, readOnly: false, onFocus: {}, onCommit: {}, onArrow: { _, _, _ in })
        let host = NSHostingView(rootView: field)
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 200, height: 40), styleMask: [.titled], backing: .buffered, defer: false)
        window.contentView = host; host.layoutSubtreeIfNeeded(); defer { window.orderOut(nil) }
        for _ in 0..<20 {
            if let editor = window.firstResponder as? NSTextView, editor.string == "Café 😀" {
                XCTAssertEqual(editor.selectedRange(), NSRange(location: 7, length: 0)); return
            }
            try await Task.sleep(for: .milliseconds(10))
        }
        XCTFail("The destination cell did not receive a collapsed native caret")
    }
    #endif
    func testDiffPresentationKeepsPatchTextLiteral() {
        // Contract: .hickory-source/frontends/contract.md
        let source = "```diff\n-old **text**\n+new **text**\n```"
        let snapshot = computeInlineMarkup(source, path: "workbook.md")
        XCTAssertEqual(snapshot.marks.map(\.kind), ["diff-remove", "diff-add"])
        XCTAssertEqual(snapshot.source, source)
        XCTAssertTrue(snapshot.debug.code.isEmpty)
    }
    func testMissingMathBundleDoesNotTrap() {
        // Contract: .hickory-source/frontends/contract.md
        XCTAssertNil(nativeMathResource(roots: [URL(fileURLWithPath: "/missing/" + UUID().uuidString)], bundles: []))
    }
    func testCSVGridsAndContextualCodeLines() {
        // Contract: .hickory-source/frontends/contract.md
        let csv = "Name,Note\r\nCafé,\"two\nlines\"\r\nlast,"
        let grid = inlineTables(computeInlineMarkup(csv, path: "stock.csv"))
        XCTAssertEqual(grid.count, 1)
        XCTAssertEqual(grid.first?.rows.map { $0.map(\.value) }, [["Name", "Note"], ["Café", "two\nlines"], ["last", ""]])
        let source = "| A | B |\n| --- | --- |\n| 1 | 2 |\n\n<h:table path=\"stock.csv\">\nName,Note\nPen,Blue\n</h:table>\n<h:file path=\"coupon.py\" language=\"python\">\ndef coupon():\n    return 1\n</h:file>\nProse"
        XCTAssertEqual(inlineTables(computeInlineMarkup(source, path: "workbook.md")).count, 2)
        XCTAssertNil(inlineDebugPath(source, path: "workbook.md", line: 12))
        XCTAssertNil(inlineDebugPath(source, path: "workbook.md", line: 0))
        XCTAssertEqual(inlineDebugPath(source, path: "workbook.md", line: 8, launch: true), "coupon.py")
        XCTAssertEqual(inlineDebugPath(source, path: "workbook.md", line: 10), "coupon.py")
        XCTAssertNil(inlineDebugPath("```xml\n<h:file path=\"fake.py\">\nprint(1)\n</h:file>\n```", path: "workbook.md", line: 2))
    }
    func testMarkdownTableRangesRetainCrLfUnicodeAndLiteralCode() {
        let source = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">\r\nBefore 😀\r\n| A | B |\r\n| --- | ---: |\r\n| café | a\\|b |\r\nAfter\r\n```md\r\n| X | Y |\r\n| --- | --- |\r\n```\r\n</h:doc>"
        let tables = inlineTables(computeInlineMarkup(source, path: "workbook.md"))
        XCTAssertEqual(tables.count, 1)
        XCTAssertEqual(tables[0].rows.map { $0.map(\.value) }, [["A", "B"], ["café", "a|b"]])
        XCTAssertEqual((source as NSString).substring(with: tables[0].range), "| A | B |\r\n| --- | ---: |\r\n| café | a\\|b |")
        XCTAssertEqual(tableColumnName(26), "AA")
    }
    #if os(macOS)
    @MainActor func testCSVGridSourceRevealLeavesOtherTablesMounted() throws {
        // Contract: .hickory-source/frontends/contract.md
        let source = "| A | B |\n| --- | --- |\n| 1 | 2 |\n\n<h:table path=\"stock.csv\">\nName,Note\nPen,Blue\n</h:table>"
        let file = FileNode(name: "workbook.md", path: "workbook.md", dir: false)
        let buffer = DocumentBuffer(file: file, text: FileText(path: file.path, content: source, hash: "base"))
        let editor = MacEditor(buffer)
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 700, height: 600), styleMask: [.titled], backing: .buffered, defer: false)
        window.contentView = editor.scroll; defer { window.orderOut(nil) }
        buffer.inlineMarkup = computeInlineMarkup(source, path: file.path); buffer.renderMarkup?()
        let tables = inlineTables(buffer.inlineMarkup)
        XCTAssertEqual(editor.text.subviews.filter { $0 is NSHostingView<NativeTableGrid> }.count, 2)
        buffer.sourceTable = tables.first?.id
        XCTAssertEqual(editor.text.subviews.filter { $0 is NSHostingView<NativeTableGrid> }.count, 1)
        XCTAssertEqual(editor.text.string, source)
        buffer.sourceTable = nil
        XCTAssertEqual(editor.text.subviews.filter { $0 is NSHostingView<NativeTableGrid> }.count, 2)
        let field = NSTextField(string: "cell draft"); editor.scroll.addSubview(field)
        XCTAssertTrue(window.makeFirstResponder(field))
        let focused = try XCTUnwrap(window.firstResponder)
        XCTAssertFalse(focused === editor.text)
        let cell = try XCTUnwrap(tables.last?.rows.last?.last)
        XCTAssertTrue(buffer.applyTableEdit?(NativeSourceEdit(range: cell.range, text: inlineCellText("a,b", format: cell.format, delimiter: cell.delimiter)), source) == true)
        XCTAssertTrue(window.firstResponder === focused, "A grid commit must not move keyboard focus into hidden source")
        XCTAssertTrue(buffer.text.contains("Pen,\"a,b\""))
        editor.text.undoManager?.undo(); XCTAssertEqual(buffer.text, source)
    }
    @MainActor func testInlineTableAnchorRejectsDeleteWithoutRevealingSource() throws {
        let source = "Before\n\n| A | B |\n| --- | --- |\n| café 😀 | 2 |\n\nAfter"
        let file = FileNode(name: "grid.md", path: "grid.md", dir: false)
        let buffer = DocumentBuffer(file: file, text: FileText(path: file.path, content: source, hash: "base"))
        let editor = MacEditor(buffer)
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 600, height: 500), styleMask: [.titled], backing: .buffered, defer: false)
        window.contentView = editor.scroll; window.makeFirstResponder(editor.text)
        buffer.inlineMarkup = computeInlineMarkup(source, path: file.path); buffer.renderMarkup?()
        let table = try XCTUnwrap(inlineTables(buffer.inlineMarkup).first)
        let host = try XCTUnwrap(editor.text.subviews.first { $0 is NSHostingView<NativeTableGrid> })
        for offset in [table.range.location, table.rows[1][1].range.location, NSMaxRange(table.range)] {
            editor.text.setSelectedRange(NSRange(location: offset, length: 0))
            XCTAssertTrue(editor.text.tableCaretIsHidden)
            editor.text.deleteBackward(nil); editor.text.deleteBackward(nil); editor.text.deleteForward(nil)
            XCTAssertTrue(editor.textView(editor.text, doCommandBy: NSSelectorFromString("insertTab:")))
            XCTAssertTrue(editor.textView(editor.text, doCommandBy: NSSelectorFromString("insertNewline:")))
            XCTAssertEqual(editor.text.string, source)
            XCTAssertEqual(buffer.text, source)
            XCTAssertFalse(buffer.tableSourceVisible)
            XCTAssertTrue(host.superview === editor.text)
        }
        // A selection crossing the hidden source must not silently remove data.
        editor.text.setSelectedRange(table.range); editor.text.deleteBackward(nil)
        XCTAssertEqual(buffer.text, source)
        XCTAssertFalse(buffer.tableSourceVisible)
        // Source mode restores ordinary caret and deletion, without changing bytes itself.
        buffer.tableSourceVisible = true
        let cell = table.rows[1][1]
        editor.text.setSelectedRange(NSRange(location: NSMaxRange(cell.range), length: 0))
        XCTAssertFalse(editor.text.tableCaretIsHidden)
        editor.text.deleteBackward(nil)
        XCTAssertEqual(buffer.text, source.replacingOccurrences(of: "| 2 |", with: "|  |"))
        editor.text.undoManager?.undo(); XCTAssertEqual(buffer.text, source)
        buffer.tableSourceVisible = false
        editor.text.setSelectedRange(NSRange(location: 1, length: 0))
        XCTAssertFalse(editor.text.tableCaretIsHidden)
        editor.text.deleteBackward(nil)
        XCTAssertEqual(buffer.text, String(source.dropFirst()))
    }
    @MainActor func testInlineGridReplacesTableLinesAndRetainsSourceUndo() throws {
        let source = "Before\n\n| A | B |\n| --- | --- |\n| café 😀 | 2 |\n\nAfter"
        let file = FileNode(name: "grid.md", path: "grid.md", dir: false)
        let buffer = DocumentBuffer(file: file, text: FileText(path: file.path, content: source, hash: "base"))
        let editor = MacEditor(buffer)
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 600, height: 500), styleMask: [.titled], backing: .buffered, defer: false)
        window.contentView = editor.scroll
        buffer.inlineMarkup = computeInlineMarkup(source, path: file.path); buffer.renderMarkup?()
        XCTAssertFalse(buffer.visibleDebugLines.contains { $0.line == 3 || $0.line == 4 }, "The scanner callback must retire source-row rail entries without a scroll")
        let host = try XCTUnwrap(editor.text.subviews.first { $0 is NSHostingView<NativeTableGrid> })
        XCTAssertEqual(editor.text.string, source)
        let table = try XCTUnwrap(inlineTables(buffer.inlineMarkup).first)
        XCTAssertEqual(host.frame.height, table.height)
        let manager = try XCTUnwrap(editor.text.layoutManager), container = try XCTUnwrap(editor.text.textContainer)
        let after = (source as NSString).range(of: "After")
        let afterRect = manager.boundingRect(forGlyphRange: manager.glyphRange(forCharacterRange: after, actualCharacterRange: nil), in: container)
        XCTAssertGreaterThanOrEqual(afterRect.minY + editor.text.textContainerOrigin.y, host.frame.maxY)
        // The source rows have no separate visual height beneath the grid.
        XCTAssertLessThan(afterRect.minY + editor.text.textContainerOrigin.y - host.frame.maxY, 50)
        buffer.tableSourceVisible = true
        XCTAssertFalse(editor.text.subviews.contains { $0 is NSHostingView<NativeTableGrid> })
        XCTAssertEqual(editor.text.string, source)
        buffer.tableSourceVisible = false
        let cell = table.rows[1][1]
        let viewport = editor.scroll.contentView.bounds.origin
        XCTAssertTrue(buffer.applyTableEdit?(NativeSourceEdit(range: cell.range, text: "3"), source) == true)
        XCTAssertEqual(editor.scroll.contentView.bounds.origin, viewport)
        XCTAssertTrue(editor.text.undoManager?.canUndo == true)
        XCTAssertEqual(buffer.text, source.replacingOccurrences(of: "| 2 |", with: "| 3 |"))
        // Undo must still work after the background scan has hidden the edited rows again.
        buffer.inlineMarkup = computeInlineMarkup(buffer.text, path: file.path); buffer.renderMarkup?()
        editor.text.undoManager?.undo()
        XCTAssertEqual(buffer.text, source)
        XCTAssertFalse(editor.applySourceEdits([NativeSourceEdit(range: cell.range, text: "4")], source: source + "stale"))
        buffer.inlineMarkup = computeInlineMarkup(source, path: file.path); buffer.renderMarkup?()
        XCTAssertTrue(editor.text.subviews.contains { $0 is NSHostingView<NativeTableGrid> })
    }
    #endif
    func testRecordedOutputDecodesCachedTextAndExitEvents() throws {
        let bytes = Data(#"{"document":"demo","source_hash":"hash","blocks":[{"kind":"exec","span":[0,4],"recorded_output":"saved stdout","transcript":[{"kind":"exit","code":0}]}]}"#.utf8)
        let page = try JSONDecoder().decode(InlineReceiptPage.self, from: bytes)
        XCTAssertEqual(page.blocks.first?.recorded_output, "saved stdout")
        XCTAssertNil(page.blocks.first?.transcript?.first?.data)
    }
    #if os(macOS)
    @MainActor func testLanguageCacheKeepsAppKitColorsSelectionAndUndoWhileRefreshing() throws {
        let original = "def f():\n    return 1\n"
        let file = FileNode(name: "a.py", path: "a.py", dir: false)
        let buffer = DocumentBuffer(file: file, text: FileText(path: file.path, content: original, hash: "base"))
        let service = NativeLanguage(engine: try Engine(address: "http://fixture.invalid"), buffer: buffer)
        buffer.language = service; service.tokenSource = original
        service.tokens = [NativeSemanticToken(range: NSRange(location: 0, length: 3), kind: "keyword")]
        let editor = MacEditor(buffer)
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 600, height: 400), styleMask: [.titled], backing: .buffered, defer: false)
        window.contentView = editor.scroll; window.makeFirstResponder(editor.text)
        defer { service.close(); window.orderOut(nil) }
        XCTAssertTrue(editor.applySourceEdits([NativeSourceEdit(range: NSRange(location: 0, length: 0), text: "# 😀\n")], source: original))
        XCTAssertEqual(editor.text.string, "# 😀\n" + original)
        let selection = editor.text.selectedRange()
        buffer.renderMarkup?()
        XCTAssertEqual(editor.text.selectedRange(), selection)
        XCTAssertEqual(editor.text.textStorage?.attribute(.foregroundColor, at: 5, effectiveRange: nil) as? NSColor, NSColor.systemPurple)
        editor.text.undoManager?.undo()
        XCTAssertEqual(editor.text.string, original)
        XCTAssertEqual(editor.text.textStorage?.attribute(.foregroundColor, at: 0, effectiveRange: nil) as? NSColor, NSColor.systemPurple)
    }
    // Cached display is provisional; writes still require the exact snapshot.
    @MainActor func testLanguageCacheSurvivesEditsAndRejectsStaleCompletion() async throws {
        let original = "def f():\n    value = 1\n"
        let file = FileNode(name: "a.py", path: "a.py", dir: false)
        let buffer = DocumentBuffer(file: file, text: FileText(path: file.path, content: original, hash: "base"))
        let service = NativeLanguage(engine: try Engine(address: "http://fixture.invalid"), buffer: buffer)
        buffer.language = service
        service.tokens = [NativeSemanticToken(range: NSRange(location: 0, length: 3), kind: "keyword"), NativeSemanticToken(range: NSRange(location: 13, length: 5), kind: "variable")]
        service.tokenSource = original; service.hover = "value: int"; service.signature = "f()"
        service.diagnosticSource = original
        let item = NativeCompletion(id: 1, label: "value", detail: "", range: NSRange(location: 13, length: 5), insert: "value", source: original, selection: buffer.selection)
        service.suggestions = [item]
        defer { service.close() }
        var writes = 0
        buffer.applyLanguageCompletion = { _, _, _, _ in writes += 1; return true }
        buffer.text = "# 😀\n" + original
        XCTAssertEqual(service.tokens.map(\.range.location), [5, 18])
        XCTAssertEqual(service.hover, "value: int"); XCTAssertEqual(service.signature, "f()")
        XCTAssertEqual(service.suggestions.count, 1)
        service.accept(item); XCTAssertEqual(writes, 0)
        // Let the real refresh task run with a closed channel: failure retains display.
        try await Task.sleep(for: .milliseconds(450))
        XCTAssertEqual(service.tokens.map(\.range.location), [5, 18])
        XCTAssertEqual(service.hover, "value: int"); XCTAssertEqual(service.suggestions.count, 1)
        // Undo/repeated edits rebase from the last displayed source, not the last server source.
        buffer.text = original
        XCTAssertEqual(service.tokens.map(\.range.location), [0, 13])
        XCTAssertEqual(rebaseNativeTokens(service.tokens, from: original, to: "def f():\n  other = 1\n").map(\.range.location), [0])
        XCTAssertEqual(rebaseNativeTokens(service.tokens, from: original, to: "def f():\n    valxue = 1\n").last?.range, NSRange(location: 13, length: 6))
        XCTAssertTrue(rebaseNativeTokens(service.tokens, from: original, to: "").isEmpty)
    }
    #endif
    func testLanguageDiagnosticPositionsRequireCurrentVersion() {
        let item: [String: Any] = ["message": "error", "range": ["start": ["line": 0, "character": 3], "end": ["line": 0, "character": 4]]]
        XCTAssertNil(decodeNativeDiagnostics(["diagnostics": [item]], source: "😀 x", expectedVersion: 2).first?.range)
        XCTAssertNil(decodeNativeDiagnostics(["version": 1, "diagnostics": [item]], source: "😀 x", expectedVersion: 2).first?.range)
        XCTAssertEqual(decodeNativeDiagnostics(["version": 2, "diagnostics": [item]], source: "😀 x", expectedVersion: 2).first?.range, NSRange(location: 3, length: 1))
        XCTAssertEqual(nativeMarkupText([["language": "python", "value": "def x()"], "detail"]), "def x()\n\ndetail")
    }
    func testNativeSearchUnicodeCapturesAndRefusals() {
        let request = NativeSearchRequest(source: "😀 NAME=12\nname=34\n", query: #"name=(\d+)"#, replacement: "$1 $$ $&", caseSensitive: false, regex: true)
        let result = nativeSearch(request)
        XCTAssertTrue(result.error.isEmpty)
        XCTAssertEqual(result.edits.map(\.range.location), [3, 11])
        XCTAssertEqual(result.edits.map(\.text), ["12 $ NAME=12", "34 $ name=34"])
        var invalid = request; invalid.query = "["; XCTAssertFalse(nativeSearch(invalid).error.isEmpty)
        var zero = request; zero.query = "^"; XCTAssertTrue(nativeSearch(zero).edits.isEmpty)
        var literal = request; literal.regex = false; literal.query = #"\n"#; XCTAssertTrue(nativeSearch(literal).edits.isEmpty)
        var many = request; many.regex = false; many.query = "a"; many.source = String(repeating: "a", count: 10_001)
        XCTAssertFalse(nativeSearch(many).error.isEmpty); XCTAssertTrue(nativeSearch(many).edits.isEmpty)
        XCTAssertFalse(validSourceEdits([NativeSourceEdit(range: NSRange(location: 4, length: 20), text: "")], length: 5))
    }
    #if os(macOS)
    @MainActor func testNativeEquationTypesetter() async throws {
        guard ProcessInfo.processInfo.environment["HICKORY_NATIVE_WEBVIEW_TEST"] == "1" else { throw XCTSkip("Enable the focused native WebKit witness explicitly") }
        let coordinator = MathCoordinator(); coordinator.source = "x^2 + 1"
        let view = makeMathView(coordinator)
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 360, height: 150), styleMask: [.titled], backing: .buffered, defer: false)
        window.contentView = view; defer { view.stopLoading(); window.orderOut(nil) }
        for _ in 0..<100 { if coordinator.ready { break }; try await Task.sleep(for: .milliseconds(100)) }
        XCTAssertTrue(coordinator.ready, "Local math page did not load")
        let rendered = try await view.evaluateJavaScript("Boolean(document.querySelector('.katex'))") as? Bool
        XCTAssertEqual(rendered, true)
        coordinator.source = #"\href{javascript:alert(1)}{click}"#; coordinator.display(view)
        let links = try await view.evaluateJavaScript("document.querySelectorAll('a').length") as? Int
        XCTAssertEqual(links, 0)
    }
    @MainActor func testInlinePresentationAndCellEditPreserveNativeState() {
        let source = "# Heading\n\n| A | B |\n| --- | --- |\n| x | y |\n"
        let file = FileNode(name: "styled.md", path: "styled.md", dir: false)
        let buffer = DocumentBuffer(file: file, text: FileText(path: file.path, content: source, hash: "base"))
        let editor = MacEditor(buffer)
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 600, height: 400), styleMask: [.titled], backing: .buffered, defer: false)
        window.contentView = editor.scroll; window.makeFirstResponder(editor.text); defer { window.orderOut(nil) }
        editor.text.setSelectedRange(NSRange(location: 2, length: 0))
        buffer.inlineMarkup = computeInlineMarkup(source, path: file.path); buffer.renderMarkup?()
        XCTAssertEqual(editor.text.string, source); XCTAssertEqual(editor.text.selectedRange().location, 2)
        XCTAssertGreaterThan((editor.text.textStorage?.attribute(.font, at: 2, effectiveRange: nil) as? NSFont)?.pointSize ?? 0, 13)
        let cell = buffer.inlineMarkup.cells.last!
        XCTAssertTrue(editor.applySourceEdits([NativeSourceEdit(range: cell.range, text: inlineCellText("a|b"))], source: source))
        XCTAssertEqual(buffer.text, source.replacingOccurrences(of: "| x | y |", with: "| x | a\\|b |"))
        editor.text.undoManager?.undo(); XCTAssertEqual(buffer.text, source)
    }
    @MainActor func testMultipleSelectionsTypeAndUndoAtomically() {
        let source = "one one"
        let file = FileNode(name: "multi.txt", path: "multi.txt", dir: false)
        let buffer = DocumentBuffer(file: file, text: FileText(path: file.path, content: source, hash: "base"))
        let editor = MacEditor(buffer)
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 600, height: 400), styleMask: [.titled], backing: .buffered, defer: false)
        window.contentView = editor.scroll; window.makeFirstResponder(editor.text); defer { window.orderOut(nil) }
        XCTAssertTrue(buffer.selectMultiple([NSRange(location: 0, length: 3), NSRange(location: 4, length: 3)], source: source))
        XCTAssertFalse(editor.textView(editor.text, shouldChangeTextIn: NSRange(location: 0, length: 3), replacementString: "two"))
        XCTAssertEqual(buffer.text, "two two"); XCTAssertEqual(buffer.multiRanges.map(\.location), [3, 7])
        editor.text.undoManager?.undo(); XCTAssertEqual(buffer.text, source)
        XCTAssertFalse(buffer.selectMultiple([NSRange(location: 0, length: 3), NSRange(location: 1, length: 3)], source: source))
    }
    @MainActor func testNativeSearchReplacementIsOneUndoAndRejectsStaleSource() {
        let source = "name=12\nname=34\n"
        let file = FileNode(name: "search.txt", path: "search.txt", dir: false)
        let buffer = DocumentBuffer(file: file, text: FileText(path: file.path, content: source, hash: "base"))
        let editor = MacEditor(buffer)
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 600, height: 400), styleMask: [.titled], backing: .buffered, defer: false)
        window.contentView = editor.scroll; window.makeFirstResponder(editor.text)
        defer { window.orderOut(nil) }
        let result = nativeSearch(NativeSearchRequest(source: source, query: #"name=(\d+)"#, replacement: "$1", caseSensitive: true, regex: true))
        XCTAssertTrue(editor.applySourceEdits(result.edits, source: source)); XCTAssertEqual(buffer.text, "12\n34\n")
        XCTAssertFalse(editor.applySourceEdits(result.edits, source: source))
        XCTAssertNotNil(editor.text.undoManager); XCTAssertTrue(editor.text.undoManager?.canUndo == true)
        editor.text.undoManager?.undo()
        XCTAssertEqual(editor.text.string, source); XCTAssertEqual(buffer.text, source)
        editor.toggleFold(0)
        XCTAssertTrue(editor.selectSourceRange(NSRange(location: 8, length: 4), source: source))
        XCTAssertEqual(editor.text.selectedRange(), NSRange(location: 8, length: 4))
    }
    #endif
    struct Fixture: Decodable { var cases: [Scenario]; var edgeCases: [EdgeCase] }
    struct EdgeCase: Decodable { var x: Double; var y: Double; var edge: String? }
    struct Scenario: Decodable { var axes: [String]?; var name: String; var steps: [Step]; var expected: [[String]] }
    struct Step: Decodable { var op: String; var group: Int?; var tab: String?; var axis: String?; var id: String?; var before: String?; var edge: String?; var target: String? }
    #if os(macOS)
    @MainActor func testFoldingPreservesSourceAndProtectsHiddenEdits() {
        let source = "func main() {\n    first()\n    second()\n}\n"
        let file = FileNode(name: "fold.swift", path: "fold.swift", dir: false)
        let buffer = DocumentBuffer(file: file, text: FileText(path: file.path, content: source, hash: "base"))
        let editor = MacEditor(buffer)
        let before = buffer.visibleDebugLines.first { $0.line == 3 }!.y
        editor.toggleFold(0)
        XCTAssertEqual(editor.text.string, source); XCTAssertEqual(buffer.text, source); XCTAssertFalse(buffer.dirty)
        XCTAssertEqual(buffer.foldedLines, [0])
        XCTAssertFalse(buffer.visibleDebugLines.contains { $0.line == 1 || $0.line == 2 })
        XCTAssertLessThan(buffer.visibleDebugLines.first { $0.line == 3 }!.y, before)
        editor.toggleFold(0)
        XCTAssertEqual(buffer.visibleDebugLines.first { $0.line == 3 }!.y, before, accuracy: 0.1)
        XCTAssertTrue(buffer.foldedLines.isEmpty)
        editor.toggleFold(0)
        XCTAssertFalse(editor.textView(editor.text, shouldChangeTextIn: NSRange(location: source.utf16.count / 2, length: 1), replacementString: ""))
        XCTAssertTrue(buffer.foldedLines.isEmpty); XCTAssertEqual(editor.text.string, source)
    }
    func testIndentationFoldOffsets() {
        let source = "😀 header\n\tchild\n        nested\n\nnext\n"
        let ranges = simpleFoldRanges(source)
        XCTAssertEqual(ranges.map(\.line), [0, 1])
        XCTAssertEqual(ranges.map { (source as NSString).substring(with: $0.range) }, ["\n\tchild\n        nested", "\n        nested"])
        XCTAssertTrue(simpleFoldRanges("one\ntwo\n").isEmpty)
    }
    @MainActor func testLogicalLineGutterUsesNativeWrappedLayout() {
        let file = FileNode(name: "numbers.txt", path: "numbers.txt", dir: false)
        let buffer = DocumentBuffer(file: file, text: FileText(path: file.path, content: "", hash: "base"))
        let editor = MacEditor(buffer)
        editor.text.textContainer?.containerSize = NSSize(width: 120, height: CGFloat.greatestFiniteMagnitude)
        for source in ["", "one\ntwo\n", String(repeating: "word ", count: 30) + "\nlast\n"] {
            editor.text.string = source
            editor.measureDebugLines()
            XCTAssertEqual(buffer.visibleDebugLines.map(\.line), Array(0...source.filter { $0 == "\n" }.count))
            XCTAssertTrue(zip(buffer.visibleDebugLines, buffer.visibleDebugLines.dropFirst()).allSatisfy { $0.y < $1.y })
        }
        // Wrapping adds visual height without adding logical numbers.
        XCTAssertGreaterThan(buffer.visibleDebugLines[1].y - buffer.visibleDebugLines[0].y, buffer.visibleDebugLines[0].height * 2)
    }
    #endif
    func testSharedLayoutScenarios() throws {
        let apps = URL(fileURLWithPath: #filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        let data = try Data(contentsOf: apps.appendingPathComponent("simple-contract/layout-cases.json"))
        let fixture = try JSONDecoder().decode(Fixture.self, from: data)
        for point in fixture.edgeCases { XCTAssertEqual(dropEdge(point.x, point.y), point.edge) }
        for scenario in fixture.cases {
            var layout = PaneLayout()
            for step in scenario.steps {
                let group = step.target ?? step.group.map { layout.groups()[$0].id } ?? ""
                switch step.op {
                case "open": layout.activate(step.tab!, in: group)
                case "move": layout.move(step.tab!, to: group, before: step.before)
                case "dock": layout.dock(step.tab!, to: group, edge: step.edge!, newID: step.id!)
                case "remove": layout.remove(step.tab!)
                case "split": _ = layout.split(group, axis: step.axis!, newID: step.id!)
                default: XCTFail("Unknown shared operation: \(step.op)")
                }
            }
            XCTAssertEqual(layout.groups().map(\.tabs), scenario.expected, scenario.name)
            func axes(_ id: String) -> [String] {
                guard let node = layout.nodes[id], let axis = node.axis else { return [] }
                return [axis] + axes(node.first!) + axes(node.second!)
            }
            if let expected = scenario.axes { XCTAssertEqual(axes(layout.rootID), expected, scenario.name) }
            for group in layout.groups() { XCTAssertTrue(group.active == nil || group.tabs.contains(group.active!), scenario.name) }
        }
    }
    func testNativeSemanticRangesUseUTF16AndRejectCrossLineTokens() {
        let tokens = decodeNativeTokens([0, 3, 3, 0, 0, 1, 0, 6, 1, 0], legend: ["keyword", "variable"], source: "😀 let\nlength")
        XCTAssertEqual(tokens.map(\.range), [NSRange(location: 3, length: 3), NSRange(location: 7, length: 6)])
        XCTAssertEqual(tokens.map(\.kind), ["keyword", "variable"])
        XCTAssertTrue(decodeNativeTokens([0, 3, 20, 0, 0], legend: ["keyword"], source: "😀 let").isEmpty)
        XCTAssertEqual(languageURI("a space/100%#?.md"), "hick:///a%20space/100%25%23%3F.md")
    }
    @MainActor func testNativeCompletionRejectsChangedSourceOrSelection() throws {
        let file = FileNode(name: "a.py", path: "a.py", dir: false)
        let buffer = DocumentBuffer(file: file, text: FileText(path: "a.py", content: "cou", hash: "base"))
        let service = NativeLanguage(engine: try Engine(address: "http://fixture.invalid"), buffer: buffer)
        var edits = 0
        buffer.applyLanguageCompletion = { _, _, _, _ in edits += 1; return true }
        let item = NativeCompletion(id: 1, label: "counter", detail: "", range: NSRange(location: 0, length: 3), insert: "counter", source: "cou", selection: NSRange(location: 3, length: 0))
        buffer.selection = NSRange(location: 2, length: 0); service.accept(item); XCTAssertEqual(edits, 0)
        buffer.selection = item.selection; buffer.text = "changed"; service.accept(item); XCTAssertEqual(edits, 0)
        buffer.text = item.source; service.accept(item); XCTAssertEqual(edits, 1)
    }
    @MainActor func testRealNativeLanguageChannel() async throws {
        guard let origin = ProcessInfo.processInfo.environment["HICKORY_NATIVE_LSP_ORIGIN"] else { throw XCTSkip("Set HICKORY_NATIVE_LSP_ORIGIN for the focused real-server witness") }
        let file = FileNode(name: "native-language-check.py", path: "native-language-check.py", dir: false)
        let buffer = DocumentBuffer(file: file, text: FileText(path: file.path, content: "counter = 1\nprint(counter)\n", hash: "base"))
        let service = NativeLanguage(engine: try Engine(address: origin), buffer: buffer); buffer.language = service
        service.start(); defer { service.close() }
        for _ in 0..<100 { if !service.tokens.isEmpty { break }; try await Task.sleep(for: .milliseconds(100)) }
        XCTAssertFalse(service.tokens.isEmpty, service.status)
        buffer.selection = NSRange(location: 2, length: 0); await service.inspect()
        XCTAssertFalse(service.hover.isEmpty, "The real Python hover should describe counter")
        buffer.text = "counter = 1\ncou"; buffer.selection = NSRange(location: buffer.text.utf16.count, length: 0)
        await service.complete()
        XCTAssertTrue(service.suggestions.contains { $0.label == "counter" }, service.status)
    }
    @MainActor func testRealNativeLiterateLanguageChannel() async throws {
        guard let origin = ProcessInfo.processInfo.environment["HICKORY_NATIVE_LSP_ORIGIN"] else { throw XCTSkip("Set HICKORY_NATIVE_LSP_ORIGIN for the focused real-server witness") }
        let file = FileNode(name: "native-literate-check.md", path: "native-literate-check.md", dir: false)
        let source = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">\n<h:file path=\"probe.py\" language=\"python\">\ncounter = 1\nprint(counter)\n</h:file>\n</h:doc>\n"
        let buffer = DocumentBuffer(file: file, text: FileText(path: file.path, content: source, hash: "base"))
        let service = NativeLanguage(engine: try Engine(address: origin), buffer: buffer); buffer.language = service
        service.start(); defer { service.close() }
        for _ in 0..<100 { if !service.tokens.isEmpty { break }; try await Task.sleep(for: .milliseconds(100)) }
        XCTAssertTrue(service.tokens.contains { (source as NSString).substring(with: $0.range) == "counter" }, service.status)
        let expression = (source as NSString).range(of: "print(counter)")
        buffer.selection = NSRange(location: expression.location + "print(counter".utf16.count, length: 0)
        await service.complete(); XCTAssertTrue(service.suggestions.contains { $0.label == "counter" }, service.status)
    }
    func testInvalidMoveRetainsTab() {
        var layout = PaneLayout(); layout.activate("notes.md", in: "document-group")
        layout.move("notes.md", to: "missing")
        XCTAssertEqual(layout.nodes["document-group"]?.tabs, ["notes.md"])
    }
    func testResizeBounds() {
        var layout = PaneLayout(); layout.resize("root", ratio: 10); XCTAssertEqual(layout.nodes["root"]?.ratio, 0.9)
        layout.resize("root", ratio: -10); XCTAssertEqual(layout.nodes["root"]?.ratio, 0.1)
    }
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/apple-simple/Tests/ParityTests.swift"><ui:paste select="#simple-7d7b2a4b09-0"/></ui:file>

</ui:doc>
