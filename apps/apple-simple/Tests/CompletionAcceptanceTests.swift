import XCTest
#if os(macOS)
import AppKit
#endif
@testable import HickorySimple
@MainActor final class CompletionAcceptanceTests:XCTestCase {
    func settle(_ condition:@escaping ()->Bool)async{let end=Date().addingTimeInterval(2);while !condition()&&Date()<end{try? await Task.sleep(for:.milliseconds(1))}}
    func edit(_ line:Int,_ from:Int,_ to:Int,_ text:String)->[String:Any]{["range":["start":["line":line,"character":from],"end":["line":line,"character":to]],"newText":text]}
    func testActualMacEditorAcceptsImportsAndCaretInOneUndo()async throws {
        #if os(macOS)
        let source="header\nva\n",file=FileNode(name:"a.py",path:"a.py",dir:false)
        let buffer=DocumentBuffer(file:file,text:FileText(path:file.path,content:source,hash:"base")),service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer);buffer.language=service
        let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll
        defer{service.close();window.contentView=nil}
        editor.text.setSelectedRange(NSRange(location:9,length:0));buffer.selection=editor.text.selectedRange();let original:[String:Any]=["label":"value","data":["hickory_completion":"held"]]
        let item=NativeCompletion(id:1,label:"value",detail:"",range:NSRange(location:7,length:2),insert:"value",source:source,selection:buffer.selection,original:original)
        let accepted=await service.accept(item,resolution:{_ in ["label":"value","textEdit":self.edit(1,0,2,"value"),"additionalTextEdits":[self.edit(0,0,6,"longheader")]]})
        XCTAssertTrue(accepted);XCTAssertEqual(buffer.text,"longheader\nvalue\n");XCTAssertEqual(editor.text.selectedRange().location,16);XCTAssertEqual(buffer.base,source);XCTAssertEqual(buffer.hash,"base")
        buffer.saving=true;editor.text.undoManager?.undo();XCTAssertEqual(buffer.text,source);buffer.saving=false
        editor.text.undoManager?.redo();XCTAssertEqual(buffer.text,"longheader\nvalue\n");XCTAssertEqual(editor.text.selectedRange().location,16)
        buffer.text="external";editor.text.string="external";editor.text.undoManager?.undo()
        XCTAssertEqual(buffer.text,"external");XCTAssertEqual(editor.text.string,"external");XCTAssertTrue(service.completionNotice.contains("history retired"));XCTAssertFalse(editor.text.undoManager?.canUndo ?? false)
        #endif
    }
    func testNewerChoiceAndDismissalRetireHeldResolutionWithoutOverwritingSource()async throws {
        let source="va\n",buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"va\n",hash:"base"))
        let service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer);buffer.language=service;buffer.selection=NSRange(location:2,length:0);defer{service.close()}
        var commits=0
        buffer.applyCompletionDraft={draft,initial in
            let edits=(draft.plan.patches.first?.edits ?? []).map{NativeSourceEdit(range:NSRange(location:$0.from,length:$0.to-$0.from),text:$0.text)},before=draft.before,selected=NSRange(location:draft.selection.from,length:draft.selection.to-draft.selection.from)
            guard exactUTF8(buffer.text,before),buffer.selection==initial else{return false};commits+=1
            let text=NSMutableString(string:before);for edit in edits.sorted(by:{$0.range.location>$1.range.location}){text.replaceCharacters(in:edit.range,with:edit.text)};buffer.text=text as String;buffer.selection=selected;return true
        }
        var held:[String:CheckedContinuation<[String:Any],Error>]=[:]
        let resolve:([String:Any]) async throws->[String:Any]={item in try await withCheckedThrowingContinuation{held[item["label"] as! String]=$0}}
        func item(_ label:String)->NativeCompletion{NativeCompletion(id:1,label:label,detail:"",range:NSRange(location:0,length:2),insert:label,source:source,selection:NSRange(location:2,length:0),original:["label":label,"data":["hickory_completion":label]])}
        func result(_ label:String)->[String:Any]{["label":label,"textEdit":edit(0,0,2,label),"additionalTextEdits":[edit(0,0,0,"# import\n")]]}
        let first=Task{await service.accept(item("old"),resolution:resolve)};await settle{held["old"] != nil}
        let second=Task{await service.accept(item("new"),resolution:resolve)};await settle{held["new"] != nil}
        try XCTUnwrap(held["new"]).resume(returning:result("new"));let secondAccepted=await second.value;XCTAssertTrue(secondAccepted);let notice=service.completionNotice
        try XCTUnwrap(held["old"]).resume(returning:result("old"));let firstAccepted=await first.value;XCTAssertFalse(firstAccepted);XCTAssertEqual(commits,1);XCTAssertEqual(buffer.text,"# import\nnew\n");XCTAssertEqual(service.completionNotice,notice)
        buffer.text=source;buffer.selection=NSRange(location:2,length:0)
        let pending=Task{await service.accept(item("dismissed"),resolution:resolve)};await settle{held["dismissed"] != nil};service.cancelCompletion()
        try XCTUnwrap(held["dismissed"]).resume(returning:result("dismissed"));let pendingAccepted=await pending.value;XCTAssertFalse(pendingAccepted);XCTAssertEqual(buffer.text,source);XCTAssertEqual(commits,1)
    }
}
