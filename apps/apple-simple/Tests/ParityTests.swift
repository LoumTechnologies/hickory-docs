// Contract: .hickory-source/frontends/contract.md
import XCTest
#if os(macOS)
import AppKit
#endif
@testable import HickorySimple

final class ParityTests: XCTestCase {
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
