import XCTest
#if os(macOS)
import AppKit
import SwiftUI
#endif
@testable import HickorySimple
final class NativeSourceIdentityTests:XCTestCase {
    let first="caf\u{e9}.py",second="cafe\u{301}.py"
    func testMapKeepsExactNamesAndPendingReadIdentities(){
        XCTAssertEqual(first,second);XCTAssertNotEqual(NativeSourceKey(first),NativeSourceKey(second))
        var sources:NativeSourceMap<Int>=[:];sources[first]=1;sources[second]=2
        XCTAssertEqual(sources.count,2);XCTAssertEqual(sources[first],1);XCTAssertEqual(sources[second],2)
        let captured=sources.mapValues{String($0)};XCTAssertEqual(captured[first],"1");XCTAssertEqual(captured[second],"2")
        sources.removeValue(forKey:first);XCTAssertNil(sources[first]);XCTAssertEqual(sources[second],2)
    }
    func testTypedSourceAndUtilityAddressesRemainSeparateThroughAtomicRouting()throws{
        var layout=PaneLayout();layout.activateSource("files",in:"document-group");layout.activateUtility(.agent,in:"document-group");layout.activateSource("agent",in:"document-group")
        XCTAssertNotEqual(NativeTabIdentity.source(NativeSourceKey("files")),.utility(.files));XCTAssertEqual(layout.nodes["document-group"]?.tabs.count,3);XCTAssertTrue(layout.nodes["files-group"]!.tabs.containsUtility(.files))
        let moved=try layout.remappingTabs([Data(nativeSourceTab("files").utf8):nativeSourceTab("terminal"),Data(nativeSourceTab("agent").utf8):nativeSourceTab("files")])
        XCTAssertTrue(moved.nodes["files-group"]!.tabs.containsUtility(.files));XCTAssertTrue(moved.nodes["document-group"]!.tabs.containsUtility(.agent));XCTAssertTrue(moved.nodes["document-group"]!.tabs.containsTab(nativeSourceTab("files")));XCTAssertTrue(moved.nodes["document-group"]!.tabs.containsTab(nativeSourceTab("terminal")));XCTAssertEqual(moved.nodes["document-group"]?.tabs.count,3)
        XCTAssertThrowsError(try layout.remappingTabs([Data(nativeSourceTab("files").utf8):nativeSourceTab("agent")]))
    }
    @MainActor func testUnnamedStateBelongsToBufferRatherThanFilenamePrefix()throws{
        let named=DocumentBuffer(file:FileNode(name:"Untitled-real.txt",path:"Untitled-real.txt",dir:false),text:FileText(path:"Untitled-real.txt",content:"saved named source",hash:"loaded",save_guard:"loaded-guard")),unnamed=DocumentBuffer(file:FileNode(name:"Untitled",path:"arbitrary-draft-address",dir:false),text:FileText(path:"arbitrary-draft-address",content:"unnamed source",hash:""),isUntitledDraft:true)
        let space=Workspace();space.buffers[named.file.path]=named;space.buffers[unnamed.file.path]=unnamed;let capture=try resourceOwnershipSnapshot(space)
        XCTAssertFalse(named.isUntitledDraft);XCTAssertTrue(unnamed.isUntitledDraft);XCTAssertEqual(capture.named[named.file.path],named.text);XCTAssertEqual(capture.guards[named.file.path],"loaded-guard");XCTAssertNil(capture.named[unnamed.file.path]);XCTAssertEqual(capture.unaddressed[unnamed.id.uuidString],unnamed.text)
    }
    func testLanguageFramesRefuseMergedContributorsAndBooleanRequestIdentity()throws{
        let aliases=Data(#"{"jsonrpc":"2.0","id":1,"result":{"changes":{"hick:///é.py":[],"hick:///e\u0301.py":[]}}}"#.utf8)
        XCTAssertThrowsError(try nativeLanguageJSONObject(aliases));XCTAssertThrowsError(try nativeLanguageJSONObject(Data(#"{"id":true,"result":[]}"#.utf8)))
        XCTAssertNoThrow(try nativeLanguageJSONObject(Data(#"{"jsonrpc":"2.0","id":1,"result":{"version":1.5}}"#.utf8)))
    }
    func testWorkspacePlansKeepCanonicalDistinctTargets()throws {
        let sources:NativeSourceMap<SourceSnapshot>=[first:SourceSnapshot(text:"one",writable:true),second:SourceSnapshot(text:"two",writable:true)]
        let edit:[String:Any]=["changes":[languageURI(first):[["range":["start":["line":0,"character":0],"end":["line":0,"character":3]],"newText":"first"]],languageURI(second):[["range":["start":["line":0,"character":0],"end":["line":0,"character":3]],"newText":"second"]]]]
        let plan=try planWorkspaceEdit("Two exact sources",raw:edit,sources:sources);XCTAssertEqual(plan.patches.count,2);XCTAssertEqual(plan.patches.first{exactUTF8($0.path,first)}?.before,"one");XCTAssertEqual(plan.patches.first{exactUTF8($0.path,second)}?.before,"two");XCTAssertNoThrow(try validateReferencedTargets(plan,referencePaths:[first,second]))
        var partial=plan;partial.patches.removeAll{exactUTF8($0.path,first)};XCTAssertThrowsError(try validateReferencedTargets(partial,referencePaths:[first,second]))
    }
    @MainActor func testObservedDebuggerSourceAddressPreservesRawSpelling()throws{let debug=NativeDebugger(engine:try Engine(address:"http://identity.invalid"));defer{debug.close()};debug.owner=first;debug.owner=second;XCTAssertTrue(exactUTF8(debug.owner,second));debug.launchSource="é";debug.launchSource="e\u{301}";XCTAssertTrue(exactUTF8(debug.launchSource,"e\u{301}"))}
    @MainActor func testReferenceAcquisitionDoesNotDeduplicateCanonicalNames()async throws {
        var round=0,opened:[NativeSourceKey]=[]
        let value=try await acquireReferenceSources([:],query:{round+=1;return [self.first,self.second,self.first]},acquire:{path in opened.append(NativeSourceKey(path));return SourceSnapshot(text:exactUTF8(path,self.first) ? "one":"two",writable:true)},current:{true})
        XCTAssertEqual(round,2);XCTAssertEqual(Set(opened).count,2);XCTAssertEqual(value.sources.count,2);XCTAssertEqual(value.references.count,2)
    }
    #if os(macOS)
    @MainActor func testReservedPhysicalNamesMountCodeAndKeepUtilityTabsReachable()async throws {
        _ = NSApplication.shared;NSApp.setActivationPolicy(.regular);let configuration=URLSessionConfiguration.ephemeral;configuration.protocolClasses=[ResponseProtocol.self]
        let space=Workspace(),engine=try Engine(address:"http://identity.invalid",session:URLSession(configuration:configuration));space.engine=engine
        ResponseProtocol.status=200;ResponseProtocol.responsesByPath=[:];ResponseProtocol.resetRequests();ResponseProtocol.beforeResponse={request in let path=URLComponents(url:request.url!,resolvingAgainstBaseURL:false)?.queryItems?.first{$0.name=="path"}?.value ?? "";let text="readable code for "+path+"\n";ResponseProtocol.response=(try? JSONSerialization.data(withJSONObject:["path":path,"content":text,"hash":nativeSourceHash(text),"save_guard":"guard-"+path])) ?? Data()}
        let host=NSHostingView(rootView:WorkspaceView(workspace:space)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:840,height:560),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil)
        defer{ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:];space.sourceTestIndex?.stop();for b in space.buffers.values{b.language?.close()};window.orderOut(nil)}
        func find(_ view:NSView,_ source:String)->NSTextView?{if let text=view as? NSTextView,exactUTF8(text.string,source){return text};return view.subviews.lazy.compactMap{find($0,source)}.first}
        for path in ["files","agent","terminal","Untitled-real.txt"]{
            await space.open(FileNode(name:path,path:path,dir:false));let buffer=try XCTUnwrap(space.buffers[path]);XCTAssertTrue(space.currentBuffer === buffer);XCTAssertFalse(buffer.isUntitledDraft)
            for _ in 0..<200{host.layoutSubtreeIfNeeded();if find(host,buffer.text) != nil{break};try await Task.sleep(for:.milliseconds(10))};let editor=try XCTUnwrap(find(host,buffer.text)),identity=try XCTUnwrap(buffer.sourceEditorState?()).identity;editor.setSelectedRange(NSRange(location:1,length:0));editor.insertText("!",replacementRange:editor.selectedRange());let draft=buffer.text,selection=editor.selectedRange()
            space.showUtility(.files);XCTAssertNil(space.currentBuffer);XCTAssertTrue(space.layout.nodes["files-group"]!.tabs.containsUtility(.files));space.showSource(path)
            for _ in 0..<200{host.layoutSubtreeIfNeeded();if find(host,draft) != nil{break};try await Task.sleep(for:.milliseconds(10))};XCTAssertTrue(find(host,draft) === editor);XCTAssertEqual(buffer.sourceEditorState?()?.identity,identity);XCTAssertEqual(editor.selectedRange(),selection);editor.undoManager?.undo();XCTAssertEqual(buffer.text,"readable code for "+path+"\n");editor.undoManager?.redo();XCTAssertTrue(exactUTF8(buffer.text,draft))
        }
        let capture=try resourceOwnershipSnapshot(space);XCTAssertEqual(capture.named["Untitled-real.txt"],space.buffers["Untitled-real.txt"]?.text);XCTAssertNotNil(capture.guards["Untitled-real.txt"])
        for utility in [NativeUtilityTab.files,.agent,.terminal]{space.showUtility(utility);XCTAssertNil(space.currentBuffer);XCTAssertTrue(space.layout.groups().contains{$0.tabs.containsUtility(utility)})};XCTAssertEqual(space.buffers.count,4)
    }
    @MainActor func testExactFileWireOpeningKeepsBothNativeEditorsAndUndo()async throws {
        _ = NSApplication.shared;NSApp.setActivationPolicy(.regular)
        let configuration=URLSessionConfiguration.ephemeral;configuration.protocolClasses=[ResponseProtocol.self]
        let engine=try Engine(address:"http://identity.invalid",session:URLSession(configuration:configuration)),space=Workspace();space.engine=engine
        ResponseProtocol.status=200;ResponseProtocol.responsesByPath=[:];ResponseProtocol.resetRequests()
        ResponseProtocol.beforeResponse={request in
            let path=URLComponents(url:request.url!,resolvingAgainstBaseURL:false)?.queryItems?.first{$0.name=="path"}?.value ?? ""
            let text=exactUTF8(path,self.first) ? "first source\n":"second source\n"
            ResponseProtocol.response=(try? JSONSerialization.data(withJSONObject:["path":path,"content":text,"hash":nativeSourceHash(text),"save_guard":"loaded-"+Data(path.utf8).base64EncodedString()])) ?? Data()
        }
        let firstFile=FileNode(name:first,path:first,dir:false),secondFile=FileNode(name:second,path:second,dir:false);space.rootFiles=[firstFile,secondFile]
        await space.open(firstFile);let firstBuffer=try XCTUnwrap(space.buffers[first]);XCTAssertTrue(space.currentBuffer === firstBuffer)
        let host=NSHostingView(rootView:WorkspaceView(workspace:space)),window=NSWindow(contentRect:NSRect(x:0,y:0,width:840,height:560),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=host;window.makeKeyAndOrderFront(nil)
        defer{ResponseProtocol.beforeResponse=nil;ResponseProtocol.responsesByPath=[:];space.sourceTestIndex?.stop();for buffer in space.buffers.values{buffer.language?.close()};window.orderOut(nil)}
        func find(_ view:NSView)->NSTextView?{if let text=view as? NSTextView{return text};return view.subviews.lazy.compactMap{find($0)}.first}
        func mounted(_ buffer:DocumentBuffer)async throws->NSTextView{for _ in 0..<200{host.layoutSubtreeIfNeeded();if let text=find(host),exactUTF8(text.string,buffer.text){return text};try await Task.sleep(for:.milliseconds(10))};throw EngineError(message:"Exact source editor did not mount")}
        let firstEditor=try await mounted(firstBuffer),firstIdentity=try XCTUnwrap(firstBuffer.sourceEditorState?()).identity,geometry=try XCTUnwrap(firstEditor.enclosingScrollView).convert(firstEditor.enclosingScrollView!.bounds,to:host)
        firstEditor.setSelectedRange(NSRange(location:1,length:0));firstEditor.insertText("!",replacementRange:firstEditor.selectedRange());let firstDraft=firstBuffer.text,selection=firstEditor.selectedRange()
        await space.open(secondFile);let secondBuffer=try XCTUnwrap(space.buffers[second]);XCTAssertFalse(firstBuffer === secondBuffer);XCTAssertTrue(space.currentBuffer === secondBuffer);XCTAssertEqual(space.buffers.count,2)
        let secondEditor=try await mounted(secondBuffer);XCTAssertFalse(firstEditor === secondEditor);secondEditor.setSelectedRange(NSRange(location:2,length:0));secondEditor.insertText("?",replacementRange:secondEditor.selectedRange());let secondDraft=secondBuffer.text
        space.show(first);let returned=try await mounted(firstBuffer);XCTAssertTrue(returned === firstEditor);XCTAssertEqual(firstBuffer.sourceEditorState?()?.identity,firstIdentity);XCTAssertEqual(firstEditor.selectedRange(),selection);XCTAssertTrue(exactUTF8(firstBuffer.text,firstDraft));XCTAssertEqual(firstEditor.enclosingScrollView!.convert(firstEditor.enclosingScrollView!.bounds,to:host),geometry)
        firstEditor.undoManager?.undo();XCTAssertEqual(firstBuffer.text,"first source\n");XCTAssertTrue(exactUTF8(secondBuffer.text,secondDraft));firstEditor.undoManager?.redo();XCTAssertTrue(exactUTF8(firstBuffer.text,firstDraft))
        XCTAssertEqual(space.layout.nodes["document-group"]?.tabs.count,2);XCTAssertNotEqual(firstFile.id,secondFile.id)
        XCTAssertNotEqual(languageURI(first),languageURI(second));XCTAssertEqual(ResponseProtocol.requestCount("/api/file"),2)
    }
    #endif
    func testTabsSelectMoveRemoveAndSwapExactNames()throws{
        var layout=PaneLayout();layout.activate(first,in:"document-group");layout.activate(second,in:"document-group")
        XCTAssertEqual(layout.nodes["document-group"]?.tabs.count,2);XCTAssertTrue(exactUTF8(layout.nodes["document-group"]?.active,second))
        layout.activate(first,in:"document-group");XCTAssertTrue(exactUTF8(layout.nodes["document-group"]?.active,first))
        let swapped=try layout.remappingTabs([Data(first.utf8):second,Data(second.utf8):first]);XCTAssertTrue(exactUTF8(swapped.nodes["document-group"]?.tabs.first,second));XCTAssertTrue(exactUTF8(swapped.nodes["document-group"]?.active,second))
        layout.move(second,to:"document-group",before:first);XCTAssertTrue(exactUTF8(layout.nodes["document-group"]?.tabs.first,second))
        layout.remove(first);XCTAssertEqual(layout.nodes["document-group"]?.tabs.count,1);XCTAssertTrue(exactUTF8(layout.nodes["document-group"]?.tabs.first,second))
    }
}
