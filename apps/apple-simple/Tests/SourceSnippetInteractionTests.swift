import XCTest
#if os(macOS)
import AppKit
#endif
@testable import HickorySimple
@MainActor final class SourceSnippetInteractionTests:XCTestCase {
    func fields()->[SnippetField]{[SnippetField(index:1,from:0,to:1,choices:["a","long"],id:0),SnippetField(index:1,from:4,to:5,choices:["a","long"],id:1),SnippetField(index:0,from:5,to:5,choices:[],id:2)]}
    func testActualMacInputAndHistoryCarryFieldState()throws {
        #if os(macOS)
        let source="a + a",buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"a + a",hash:"base")),editor:MacEditor
        editor=MacEditor(buffer);let window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll;defer{window.contentView=nil}
        editor.text.setSelectedRange(NSRange(location:0,length:1));buffer.selection=editor.text.selectedRange();buffer.snippets.restore(beginSnippet(source,fields:fields()))
        XCTAssertTrue(editor.textView(editor.text,doCommandBy:NSSelectorFromString("insertBacktab:")));XCTAssertEqual(buffer.text,source);XCTAssertNotNil(buffer.snippets.active)
        editor.text.insertText("long",replacementRange:editor.text.selectedRange())
        XCTAssertEqual(buffer.text,"long + long");XCTAssertEqual(editor.text.selectedRange(),NSRange(location:4,length:0));XCTAssertEqual(buffer.snippets.active?.source,"long + long");XCTAssertEqual(buffer.base,source)
        editor.text.undoManager?.undo();XCTAssertEqual(buffer.text,source);XCTAssertEqual(buffer.snippets.active?.source,source)
        editor.text.undoManager?.redo();XCTAssertEqual(buffer.text,"long + long");XCTAssertEqual(buffer.snippets.active?.source,"long + long")
        XCTAssertTrue(buffer.snippets.move(1));XCTAssertEqual(editor.text.selectedRange().location,11);XCTAssertNil(buffer.snippets.active)
        #endif
    }
    func testPreparedNativeEnterMirrorsItsTextAndCaret()throws {
        #if os(macOS)
        let source="call() + call()",buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"call() + call()",hash:"base")),editor:MacEditor
        editor=MacEditor(buffer);let window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll;defer{window.contentView=nil}
        let fields=[SnippetField(index:1,from:0,to:6,choices:[],id:0),SnippetField(index:1,from:9,to:15,choices:[],id:1),SnippetField(index:0,from:15,to:15,choices:[],id:2)]
        editor.text.setSelectedRange(NSRange(location:5,length:0));buffer.selection=editor.text.selectedRange();buffer.snippets.restore(beginSnippet(source,fields:fields))
        XCTAssertEqual(buffer.snippets.input(source:source,range:NSRange(location:5,length:0),text:"\n  \n",caret:8),true);XCTAssertEqual(buffer.text,"call(\n  \n) + call(\n  \n)");XCTAssertEqual(editor.text.selectedRange().location,8)
        editor.text.undoManager?.undo();XCTAssertEqual(buffer.text,source);editor.text.undoManager?.redo();XCTAssertEqual(editor.text.selectedRange().location,8)
        #endif
    }
    func testProviderCompletionCombinesImportsAndRefusesInteriorEdits()async throws {
        #if os(macOS)
        let source="a + a",buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"a + a",hash:"base")),service:NativeLanguage
        service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer);buffer.language=service
        let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll;defer{service.close();window.contentView=nil}
        editor.text.setSelectedRange(NSRange(location:0,length:1));buffer.selection=editor.text.selectedRange();buffer.snippets.restore(beginSnippet(source,fields:fields()))
        let item=NativeCompletion(id:1,label:"long",detail:"",range:NSRange(location:0,length:1),insert:"long",source:source,selection:buffer.selection,original:["label":"long"])
        let edit:[String:Any]=["range":["start":["line":0,"character":0],"end":["line":0,"character":1]],"newText":"long"]
        let accepted=await service.accept(item,resolution:{_ in ["label":"long","textEdit":edit]});XCTAssertTrue(accepted);XCTAssertEqual(buffer.text,"long + long");XCTAssertEqual(buffer.snippets.active?.source,"long + long")
        editor.text.undoManager?.undo();XCTAssertEqual(buffer.text,source);XCTAssertEqual(buffer.snippets.active?.source,source)
        let imported=await service.accept(item,resolution:{_ in ["label":"long","textEdit":edit,"additionalTextEdits":[["range":["start":["line":0,"character":0],"end":["line":0,"character":0]],"newText":"# import\n"],["range":["start":["line":0,"character":5],"end":["line":0,"character":5]],"newText":"\n# end"]]]})
        XCTAssertTrue(imported);XCTAssertEqual(buffer.text,"# import\nlong + long\n# end");XCTAssertEqual(editor.text.selectedRange().location,13);XCTAssertEqual(buffer.snippets.active?.fields.map{$0.from},[9,16,20])
        editor.text.undoManager?.undo();XCTAssertEqual(buffer.text,source);XCTAssertEqual(buffer.snippets.active?.source,source);editor.text.undoManager?.redo();XCTAssertEqual(editor.text.selectedRange().location,13);editor.text.undoManager?.undo()
        let refused=await service.accept(item,resolution:{_ in ["label":"long","textEdit":edit,"additionalTextEdits":[["range":["start":["line":0,"character":2],"end":["line":0,"character":2]],"newText":"\nimport other"]]]})
        XCTAssertFalse(refused);XCTAssertEqual(buffer.text,source);XCTAssertTrue(service.completionNotice.contains("overlaps template"))
        #endif
    }
    func testEqualPositionNavigationRetiresHeldCompletion()async throws {
        #if os(macOS)
        let source="a",buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"a",hash:"base")),service:NativeLanguage
        service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer);buffer.language=service
        let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll;defer{service.close();window.contentView=nil}
        editor.text.setSelectedRange(NSRange(location:0,length:1));buffer.selection=editor.text.selectedRange();let selected=buffer.selection
        buffer.snippets.restore(beginSnippet(source,fields:[SnippetField(index:2,from:0,to:1,choices:[],id:1,parent:0),SnippetField(index:1,from:0,to:1,choices:[],id:0),SnippetField(index:0,from:1,to:1,choices:[],id:2)]))
        var held:CheckedContinuation<[String:Any],Error>?
        let work=Task{await service.accept(NativeCompletion(id:1,label:"long",detail:"",range:selected,insert:"long",source:source,selection:selected,original:["label":"long"]),resolution:{_ in try await withCheckedThrowingContinuation{held=$0}})}
        let end=Date().addingTimeInterval(2);while held==nil&&Date()<end{await Task.yield()}
        XCTAssertTrue(buffer.snippets.move(1));XCTAssertEqual(buffer.selection,selected);try XCTUnwrap(held).resume(returning:["label":"long","insertText":"long"]);let accepted=await work.value;XCTAssertFalse(accepted);XCTAssertEqual(buffer.text,source)
        #endif
    }
    func testActualMacChoiceNeedsNoLanguageConnection()throws {
        #if os(macOS)
        let source="a + a",buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"a + a",hash:"base")),editor:MacEditor
        editor=MacEditor(buffer);let window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll;defer{window.contentView=nil}
        editor.text.setSelectedRange(NSRange(location:0,length:1));buffer.selection=editor.text.selectedRange();buffer.snippets.restore(beginSnippet(source,fields:fields()))
        XCTAssertTrue(buffer.snippets.choose("long"));XCTAssertEqual(buffer.text,"long + long");XCTAssertEqual(editor.text.selectedRange().location,4);XCTAssertNil(buffer.language)
        editor.text.undoManager?.undo();XCTAssertEqual(buffer.text,source);XCTAssertEqual(buffer.snippets.active?.source,source)
        #endif
    }
    func testActualMacAcceptsSourceEqualCompletion()async throws {
        #if os(macOS)
        let buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"va",hash:"base")),service:NativeLanguage
        service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer);buffer.language=service
        let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll;defer{service.close();window.contentView=nil}
        editor.text.setSelectedRange(NSRange(location:2,length:0));buffer.selection=editor.text.selectedRange()
        let item=NativeCompletion(id:1,label:"va",detail:"",range:NSRange(location:0,length:2),insert:"va",source:"va",selection:buffer.selection,original:["label":"va"])
        let accepted=await service.accept(item,resolution:{_ in ["label":"va","textEdit":["range":["start":["line":0,"character":0],"end":["line":0,"character":2]],"newText":"va"]]})
        XCTAssertTrue(accepted);XCTAssertEqual(buffer.text,"va");XCTAssertEqual(buffer.base,"va");XCTAssertFalse(editor.text.undoManager?.canUndo ?? false)
        #endif
    }
    func testSavingRetiresMirroringAndLeavesOrdinaryNativeInput()throws {
        #if os(macOS)
        let buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"a + a",hash:"base")),editor:MacEditor
        editor=MacEditor(buffer);let window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll;defer{window.contentView=nil}
        editor.text.setSelectedRange(NSRange(location:0,length:1));buffer.selection=editor.text.selectedRange();buffer.snippets.restore(beginSnippet(buffer.text,fields:fields()));buffer.saving=true
        editor.text.insertText("x",replacementRange:editor.text.selectedRange());XCTAssertEqual(buffer.text,"x + a");XCTAssertNil(buffer.snippets.active)
        #endif
    }
}
