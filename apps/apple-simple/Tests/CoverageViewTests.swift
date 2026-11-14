import XCTest
import WebKit
#if os(macOS)
import AppKit
#endif
@testable import HickorySimple
@MainActor final class CoverageViewTests:XCTestCase {
    #if os(macOS)
    func testBundledCoverageSurfaceUsesWorkspaceBoundNativeMessages()throws {
        let workspace=Workspace(),state=workspace.coverage
        let source="let value = 1\n",file=FileNode(name:"draft.swift",path:"draft.swift",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:source,hash:nativeSourceHash(source)))
        workspace.buffers[file.path]=buffer;buffer.text+="// unsaved\n";buffer.selection=NSRange(location:4,length:2)
        let coordinator=CoverageWebCoordinator(workspace),view=coverageWebView(coordinator)
        let window=NSWindow(contentRect:NSRect(x:0,y:0,width:740,height:640),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=view
        defer{coordinator.destroy();window.orderOut(nil)}
        func pump(_ condition:()->Bool){let deadline=Date().addingTimeInterval(15);while !condition()&&Date()<deadline{RunLoop.current.run(until:Date().addingTimeInterval(0.02))}}
        func evaluate(_ script:String)throws->Any?{var result:Result<Any?,Error>?;view.evaluateJavaScript(script){value,error in result=error.map(Result.failure) ?? .success(value)};pump{result != nil};return try XCTUnwrap(result).get()}
        pump{coordinator.ready};XCTAssertTrue(coordinator.ready,"Bundled coverage renderer must load in actual WebKit")
        guard coordinator.ready else{return}
        var rendered=false;pump{rendered=(try? evaluate("!!document.querySelector('[aria-label=\"Tests and code coverage\"]')")) as? Bool ?? false;return rendered};XCTAssertTrue(rendered)
        _ = try evaluate("webkit.messageHandlers.coverage.postMessage({kind:'select',workspace:'obsolete',id:'old'});null")
        RunLoop.current.run(until:Date().addingTimeInterval(0.05));XCTAssertNil(state.selected)
        let identity=String(data:try JSONSerialization.data(withJSONObject:state.identity,options:[.fragmentsAllowed]),encoding:.utf8)!
        _ = try evaluate("webkit.messageHandlers.coverage.postMessage({kind:'select',workspace:\(identity),id:'chosen'});null");pump{state.selected=="chosen"};XCTAssertEqual(state.selected,"chosen")
        workspace.coverage=SimpleCoverageState()
        _ = try evaluate("webkit.messageHandlers.coverage.postMessage({kind:'select',workspace:\(identity),id:'late'});null")
        RunLoop.current.run(until:Date().addingTimeInterval(0.05));XCTAssertEqual(state.selected,"chosen");XCTAssertNil(workspace.coverage.selected)
        XCTAssertTrue(exactUTF8(buffer.text,source+"// unsaved\n"));XCTAssertTrue(buffer.dirty);XCTAssertEqual(buffer.selection,NSRange(location:4,length:2))
    }
    #endif
}
