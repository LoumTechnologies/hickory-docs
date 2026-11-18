import XCTest

@testable import HickorySimple

#if os(macOS)
  import AppKit
#endif
@MainActor final class SnippetRegexTests: XCTestCase {
  #if os(macOS)
    private func exercise(templates: Bool) async throws {
      guard let origin = ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_ORIGIN"],
        let directory = ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_WORKSPACE"]
      else { throw XCTSkip("Run the maintained snippet regex fixture") }
      let root = URL(fileURLWithPath: directory)
      XCTAssertEqual(
        try String(
          contentsOf: root.appendingPathComponent(".hickory-snippet-regex-fixture"), encoding: .utf8
        ), "owned-snippet-regex-v3\n")
      let dataURL = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
        .deletingLastPathComponent().deletingLastPathComponent().appendingPathComponent(
          "simple-contract/snippet-regex-live-cases.json")
      let rawCases =
        try JSONSerialization.jsonObject(with: Data(contentsOf: dataURL)) as! [String: Any]
      let cases = rawCases.compactMapValues { $0 as? String }
      let engine = try Engine(address: origin)
      let tree = try await engine.files()
      _ = NSApplication.shared
      func find(_ path: String, _ nodes: [FileNode]) -> FileNode? {
        for node in nodes {
          if node.path == path { return node }
          if let child = find(path, node.children ?? []) { return child }
        }
        return nil
      }
      let events = root.deletingLastPathComponent().appendingPathComponent("provider-events.jsonl")
      func records() throws -> [[String: Any]] {
        guard FileManager.default.fileExists(atPath: events.path) else { return [] }
        return try String(contentsOf: events, encoding: .utf8).split(separator: "\n").map {
          try JSONSerialization.jsonObject(with: Data($0.utf8)) as! [String: Any]
        }
      }
      for path in ["regex.py", "budget.py"] {
        let file = try XCTUnwrap(find(path, tree.tree))
        let saved = try await engine.file(file)
        XCTAssertEqual(saved.content, cases["source"])
        let space = Workspace()
        space.engine = engine
        space.rootFiles = tree.tree
        space.sourceRootAddress = tree.root_path
        space.sourceWorkspaceID = tree.workspace_id
        let buffer = DocumentBuffer(file: file, text: saved)
        space.buffers[path] = buffer
        space.layout.activate(path, in: "document-group")
        let editor = MacEditor(buffer)
        let window = NSWindow(
          contentRect: NSRect(x: 0, y: 0, width: 700, height: 480), styleMask: [.titled],
          backing: .buffered, defer: false)
        window.contentView = editor.scroll
        editor.text.setSelectedRange(NSRange(location: 0, length: 6))
        buffer.selection = editor.text.selectedRange()
        let identity = try XCTUnwrap(buffer.sourceEditorState?()).identity
        let manager = try XCTUnwrap(editor.text.undoManager)
        let scroll = editor.scroll.contentView.bounds.origin
        let geometry = editor.scroll.frame
        var clipboardReads = 0
        let language = NativeLanguage(engine: engine, buffer: buffer)
        buffer.language = language
        language.snippetClipboardReader = {
          clipboardReads += 1
          return "must not be read"
        }
        language.start()
        defer {
          space.sourceTemplates.close()
          language.close()
          window.contentView = nil
        }
        if templates {
          let model = space.sourceTemplates
          model.clipboardReader = {
            clipboardReads += 1
            return "must not be read"
          }
          await model.open(space)
          let entry = try XCTUnwrap(
            model.catalog?.entries.first(where: {
              $0.id == (path == "regex.py" ? "regex.mirrors" : "regex.budget")
            }), model.notice)
          XCTAssertTrue(entry.available, entry.reason ?? model.notice)
          await model.choose(entry)
          XCTAssertNotNil(model.preview, model.notice)
          XCTAssertEqual(buffer.text, saved.content)
          let applied = await model.apply()
          XCTAssertTrue(applied, model.notice)
        } else {
          for _ in 0..<500 {
            if language.status.hasPrefix("LSP") { break }
            try await Task.sleep(for: .milliseconds(10))
          }
          await language.complete()
          for _ in 0..<500 {
            if !language.suggestions.isEmpty { break }
            try await Task.sleep(for: .milliseconds(10))
          }
          let offered = try XCTUnwrap(
            language.suggestions.first(where: { $0.label == "Regex mirrors" }), language.status)
          let accepted = await language.accept(offered)
          XCTAssertTrue(accepted, language.completionNotice)
        }
        let expected = try XCTUnwrap(cases[path == "regex.py" ? "after" : "budget_after"])
        XCTAssertEqual(buffer.text, expected)
        XCTAssertNotNil(buffer.snippets.active)
        XCTAssertEqual(editor.text.selectedRange(), NSRange(location: 0, length: 6))
        XCTAssertEqual(clipboardReads, 0)
        if path == "regex.py" {
          let active = try XCTUnwrap(buffer.snippets.active)
          XCTAssertEqual(active.fields.filter { $0.transform != nil }.count, 5)
          editor.text.insertText("NewNew", replacementRange: editor.text.selectedRange())
          XCTAssertEqual(buffer.text, cases["typed"])
          manager.undo()
          XCTAssertEqual(buffer.text, expected)
          manager.redo()
          XCTAssertEqual(buffer.text, cases["typed"])
          manager.undo()
          editor.text.insertText("😀", replacementRange: editor.text.selectedRange())
          XCTAssertEqual(buffer.text, cases["emoji"])
          manager.undo()
          XCTAssertEqual(buffer.text, expected)
        } else {
          editor.text.insertText(
            String(repeating: "a", count: 100) + "!", replacementRange: editor.text.selectedRange())
          XCTAssertEqual(
            buffer.text, expected, "A refused transform must not leave a primary-only edit")
          XCTAssertEqual(editor.text.selectedRange(), NSRange(location: 0, length: 6))
        }
        manager.undo()
        XCTAssertEqual(buffer.text, saved.content)
        manager.redo()
        XCTAssertEqual(buffer.text, expected)
        XCTAssertEqual(buffer.sourceEditorState?()?.identity, identity)
        XCTAssertEqual(editor.scroll.contentView.bounds.origin, scroll)
        XCTAssertEqual(editor.scroll.frame, geometry)
        XCTAssertTrue(editor.text.undoManager === manager)
        XCTAssertEqual(buffer.base, saved.content)
        let disk = try await engine.file(file)
        XCTAssertEqual(disk.content, saved.content)
        if !templates {
          let observed = try records()
          let uri = root.appendingPathComponent(path).standardizedFileURL.absoluteString
          XCTAssertTrue(observed.contains { $0["initialize_snippets"] as? Bool == true })
          XCTAssertEqual(observed.filter { $0["resolve"] as? String == uri }.count, 1)
        }
      }
    }
    func testLiveRegexProviderAndHistory() async throws { try await exercise(templates: false) }
    func testLiveRegexTemplatesAndHistory() async throws { try await exercise(templates: true) }
  #endif
}
