<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Drive native search through real key events

For maintainers changing workspace services independently of the editor renderer.

Use a real AppKit window and SwiftUI search field. Move from the first file result to the second with Down, then press Return. The existing workspace opener must activate that retained file and dismiss the search state without replacing its draft. This is a focused macOS keyboard witness; it does not establish iOS/Android pointer, IME or assistive-technology acceptance.

<ui:copy id="simple-search-everywhere-apple-view-tests">
import XCTest
@testable import HickorySimple
#if os(macOS)
import AppKit
import SwiftUI
@MainActor final class SearchEverywhereViewTests:XCTestCase {
    func testDownAndReturnOpenSecondRetainedFileWithoutChangingDraft() async throws {
        let workspace=Workspace(),a=FileNode(name:"a.py",path:"a.py",dir:false),b=FileNode(name:"b.py",path:"b.py",dir:false)
        workspace.rootFiles=[a,b];workspace.buffers[a.path]=DocumentBuffer(file:a,text:FileText(path:a.path,content:"source",hash:"base"));workspace.buffers[b.path]=DocumentBuffer(file:b,text:FileText(path:b.path,content:"source",hash:"base"))
        workspace.buffers[b.path]?.text="retained draft";workspace.showEverywhere=true
        let host=NSHostingView(rootView:SearchEverywhereView(workspace:workspace,commands:[])),window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:480),styleMask:[.titled],backing:.buffered,defer:false)
        window.contentView=host;window.makeKeyAndOrderFront(nil);host.layoutSubtreeIfNeeded();defer {window.orderOut(nil)}
        func find(_ view:NSView)->NSTextField? {if let field=view as? NSTextField,field.placeholderString=="Search files, types, members and actions" {return field};return view.subviews.lazy.compactMap {find($0)}.first}
        for _ in 0..<200 {if find(host) != nil {break};try await Task.sleep(for:.milliseconds(10))}
        let field=try XCTUnwrap(find(host));XCTAssertTrue(window.makeFirstResponder(field))
        for _ in 0..<200 {if window.firstResponder is NSTextView {break};try await Task.sleep(for:.milliseconds(10))}
        XCTAssertTrue(window.firstResponder is NSTextView)
        func key(_ code:UInt16,_ characters:String) throws {
            let event=try XCTUnwrap(NSEvent.keyEvent(with:.keyDown,location:.zero,modifierFlags:[],timestamp:ProcessInfo.processInfo.systemUptime,windowNumber:window.windowNumber,context:nil,characters:characters,charactersIgnoringModifiers:characters,isARepeat:false,keyCode:code))
            window.sendEvent(event)
        }
        try key(125,"\u{f701}");try await Task.sleep(for:.milliseconds(30));try key(36,"\r")
        for _ in 0..<200 {if workspace.active==b.path {break};try await Task.sleep(for:.milliseconds(10))}
        XCTAssertEqual(workspace.active,b.path);XCTAssertFalse(workspace.showEverywhere);XCTAssertEqual(workspace.buffers[b.path]?.text,"retained draft");XCTAssertTrue(workspace.buffers[b.path]?.dirty==true)
    }
}
#endif
</ui:copy>

<ui:file path="apps/apple-simple/Tests/SearchEverywhereViewTests.swift"><ui:paste select="#simple-search-everywhere-apple-view-tests"/></ui:file>
</ui:doc>
