import XCTest
#if os(macOS)
import AppKit
#endif
@testable import HickorySimple
@MainActor final class FencedSourceReadingTests:XCTestCase {
 #if os(macOS)
 func testLiveFencedSourceLanguageReadings()async throws {
  guard let origin=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_ORIGIN"],let directory=ProcessInfo.processInfo.environment["HICKORY_NATIVE_TEMPLATE_WORKSPACE"] else{throw XCTSkip("Run test-fenced-source-readings.py --host apple")}
  let root=URL(fileURLWithPath:directory);XCTAssertEqual(try String(contentsOf:root.appendingPathComponent(".hickory-native-template-fixture"),encoding:.utf8),"owned-native-templates-v1\n")
  let fixture=try XCTUnwrap(JSONSerialization.jsonObject(with:Data(contentsOf:root.deletingLastPathComponent().appendingPathComponent("fixture.json"))) as? [String:Any]),rows=try XCTUnwrap(fixture["rows"] as? [[String:Any]])
  let engine=try Engine(address:origin),tree=try await engine.files();_ = NSApplication.shared
  func find(_ nodes:[FileNode])->FileNode?{for node in nodes{if node.path=="notes.md"{return node};if let result=find(node.children ?? []){return result}};return nil}
  let file=try XCTUnwrap(find(tree.tree)),saved=try await engine.file(file),buffer=DocumentBuffer(file:file,text:saved),editor=MacEditor(buffer)
  let window=NSWindow(contentRect:NSRect(x:0,y:0,width:640,height:420),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll
  let language=NativeLanguage(engine:engine,buffer:buffer);buffer.language=language;language.start();defer{language.close();window.contentView=nil}
  let identity=try XCTUnwrap(buffer.sourceEditorState?()).identity,manager=try XCTUnwrap(editor.text.undoManager),scroll=editor.scroll.contentView.bounds.origin
  for _ in 0..<500{if language.status.hasPrefix("LSP"){break};try await Task.sleep(for:.milliseconds(10))};XCTAssertTrue(language.status.hasPrefix("LSP"),language.status)
  for row in rows{
   let query=try XCTUnwrap(row["query"] as? [String:Int]),definition=try XCTUnwrap(row["definition"] as? [String:Int]),offset=try XCTUnwrap(languageOffset(buffer.text,query))
   editor.text.setSelectedRange(NSRange(location:offset,length:0));buffer.selection=editor.text.selectedRange()
   let expectedHover=definition["line"]==4 ? "def greet":"function greet"
   for _ in 0..<200{await language.inspect();if language.hover.contains(expectedHover){break};try await Task.sleep(for:.milliseconds(50))};XCTAssertTrue(language.hover.contains(expectedHover),language.hover)
   await language.navigate(.definition)
   let target=try XCTUnwrap(language.locations.first(where:{$0.path=="notes.md"}),language.locationStatus)
   XCTAssertEqual(target.line,definition["line"]);XCTAssertEqual(target.character,definition["character"])
   language.open(target);let destination=try XCTUnwrap(languageOffset(buffer.text,definition));XCTAssertEqual(buffer.selection,NSRange(location:destination,length:0));XCTAssertEqual(editor.text.selectedRange(),buffer.selection)
  }
  XCTAssertEqual(buffer.text,saved.content);XCTAssertEqual(buffer.base,saved.content);XCTAssertEqual(buffer.sourceEditorState?()?.identity,identity);XCTAssertTrue(editor.text.undoManager===manager);XCTAssertEqual(editor.scroll.contentView.bounds.origin,scroll);let disk=try await engine.file(file);XCTAssertEqual(disk.content,saved.content)
 }
 #endif
}
