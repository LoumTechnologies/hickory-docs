import XCTest
#if os(macOS)
import AppKit
#endif
@testable import HickorySimple
@MainActor final class SourceSnippetInteractionTests:XCTestCase {
    private func uppercaseTransform() throws -> SnippetTransform {
        let url=URL(fileURLWithPath:#filePath).deletingLastPathComponent().deletingLastPathComponent()
            .deletingLastPathComponent().appendingPathComponent("simple-contract/snippet-derived-cases.json")
        let vectors=try XCTUnwrap(JSONSerialization.jsonObject(with:Data(contentsOf:url)) as? [String:Any])
        let programs=try XCTUnwrap(vectors["programs"] as? [String:Any])
        return try readSnippetTransform(XCTUnwrap(programs["upper"]))
    }
    func testComputedInputAndNativeHistoryAreOneAction()throws {
        #if os(macOS)
        let transform=try uppercaseTransform()
        let source="foo FOO",values=[SnippetField(index:1,from:0,to:3,choices:[],id:0),SnippetField(index:1,from:4,to:7,choices:[],id:1,transform:transform),SnippetField(index:0,from:7,to:7,choices:[],id:2)]
        XCTAssertEqual(snippetHistoryCost(SourceSnippetHistoryContext(fields:values,active:1,current:0)),3*64+transform.bytes)
        let buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:source,hash:"base")),editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll;defer{window.contentView=nil}
        editor.text.setSelectedRange(NSRange(location:0,length:3));buffer.selection=editor.text.selectedRange();buffer.snippets.restore(beginSnippet(source,fields:values))
        let originalSelection=editor.text.selectedRange(),canUndo=editor.text.undoManager?.canUndo
        editor.text.insertText(String(repeating:"x",count:65537),replacementRange:originalSelection)
        XCTAssertEqual(buffer.text,source)
        XCTAssertEqual(editor.text.selectedRange(),originalSelection)
        XCTAssertEqual(editor.text.undoManager?.canUndo,canUndo)
        XCTAssertNotNil(buffer.snippets.active)
        XCTAssertTrue(buffer.snippets.notice.contains("Current draft is retained"))
        editor.text.insertText("λ😀",replacementRange:editor.text.selectedRange());XCTAssertEqual(Array(buffer.text.utf8),Array("λ😀 Λ😀".utf8));XCTAssertEqual(editor.text.selectedRange(),NSRange(location:3,length:0));XCTAssertNotNil(buffer.snippets.active)
        editor.text.undoManager?.undo();XCTAssertEqual(buffer.text,source);XCTAssertNotNil(buffer.snippets.active);editor.text.undoManager?.redo();XCTAssertEqual(Array(buffer.text.utf8),Array("λ😀 Λ😀".utf8));XCTAssertEqual(editor.text.selectedRange(),NSRange(location:3,length:0));XCTAssertNotNil(buffer.snippets.active)
        editor.text.setSelectedRange(NSRange(location:4,length:3));buffer.selection=editor.text.selectedRange();editor.text.insertText("bar",replacementRange:editor.text.selectedRange());XCTAssertEqual(Array(buffer.text.utf8),Array("λ😀 bar".utf8));XCTAssertNil(buffer.snippets.active)
        #endif
    }
    func testActualMacRefusedMarkedCandidateKeepsEarlierUndoAndRedo() throws {
        #if os(macOS)
        let saved="foo FOO"
        let buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:saved,hash:"base"))
        let editor=MacEditor(buffer)
        let window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false)
        window.contentView=editor.scroll
        window.makeFirstResponder(editor.text)
        defer { window.contentView=nil }
        XCTAssertTrue(editor.applySourceEdits([NativeSourceEdit(range:NSRange(location:0,length:0),text:"!")],source:saved))
        let source="!foo FOO",selected=NSRange(location:1,length:3)
        XCTAssertTrue(editor.selectSourceRange(selected,source:source))
        let fields=[SnippetField(index:1,from:1,to:4,choices:[],id:0),
                    SnippetField(index:1,from:5,to:8,choices:[],id:1,transform:try uppercaseTransform()),
                    SnippetField(index:0,from:8,to:8,choices:[],id:2)]
        buffer.snippets.restore(beginSnippet(source,fields:fields))
        let mark=NSRange(location:NSNotFound,length:0)
        editor.text.setMarkedText("に",selectedRange:NSRange(location:1,length:0),replacementRange:mark)
        XCTAssertEqual(buffer.text,"!に FOO")
        editor.text.insertText(String(repeating:"x",count:65537),replacementRange:mark)
        XCTAssertFalse(editor.text.hasMarkedText())
        XCTAssertEqual(buffer.text,source)
        XCTAssertEqual(editor.text.selectedRange(),selected)
        XCTAssertNotNil(buffer.snippets.active)
        XCTAssertTrue(buffer.snippets.notice.contains("Current draft is retained"))
        let manager=try XCTUnwrap(editor.text.undoManager)
        manager.undo()
        XCTAssertEqual(buffer.text,saved)
        manager.redo()
        XCTAssertEqual(buffer.text,source)
        XCTAssertFalse(manager.canRedo)
        #endif
    }
    func fields()->[SnippetField]{[SnippetField(index:1,from:0,to:1,choices:["a","long"],id:0),SnippetField(index:1,from:4,to:5,choices:["a","long"],id:1),SnippetField(index:0,from:5,to:5,choices:[],id:2)]}
    func testActualMacParallelInputAndUndoRetainNativeSelections() throws {
        #if os(macOS)
        let source="aa / bb"
        let buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:source,hash:"base"))
        let editor=MacEditor(buffer)
        let window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false)
        window.contentView=editor.scroll
        defer {window.contentView=nil}
        let ranges=[NSRange(location:0,length:2),NSRange(location:5,length:2)]
        XCTAssertTrue(buffer.selectMultiple(ranges,source:source))
        func expansion(_ name:String,_ range:NSRange)->ParallelSnippetExpansion {
            let n=name.utf16.count
            return ParallelSnippetExpansion(edit:NativeSourceEdit(range:range,text:name+":"+name),fields:[
                SnippetField(index:1,from:0,to:n,choices:[],id:0),
                SnippetField(index:1,from:n+1,to:n+n+1,choices:[],id:1),
                SnippetField(index:0,from:n+n+1,to:n+n+1,choices:[],id:2),
            ])
        }
        let expansions=[expansion("a",ranges[0]),expansion("beta",ranges[1])]
        let intent=buffer.snippets.intent
        XCTAssertFalse(buffer.snippets.applyParallel(source:source,expansions:expansions,primary:0,expectedIntent:intent,admitted:{false}))
        XCTAssertTrue(buffer.snippets.applyParallel(source:source,expansions:expansions,primary:0,expectedIntent:intent,admitted:{true}))
        let beforeInput=buffer.multiRanges
        XCTAssertEqual(buffer.text,"a:a / beta:beta")
        editor.text.insertText(String(repeating:"x",count:262145),replacementRange:editor.text.selectedRange())
        XCTAssertEqual(buffer.text,"a:a / beta:beta")
        XCTAssertEqual(buffer.multiRanges,beforeInput)
        XCTAssertNotNil(buffer.snippets.parallel)
        editor.text.insertText("λ😀",replacementRange:editor.text.selectedRange())
        let afterInput=buffer.multiRanges
        XCTAssertEqual(buffer.text,"λ😀:λ😀 / λ😀:λ😀")
        XCTAssertEqual(editor.text.selectedRange(),afterInput.first)
        editor.text.undoManager?.undo()
        XCTAssertEqual(buffer.text,"a:a / beta:beta")
        XCTAssertEqual(buffer.multiRanges,beforeInput)
        editor.text.undoManager?.undo()
        XCTAssertEqual(buffer.text,source)
        XCTAssertEqual(buffer.multiRanges,ranges)
        XCTAssertNil(buffer.snippets.parallel)
        editor.text.undoManager?.redo()
        XCTAssertNotNil(buffer.snippets.parallel)
        editor.text.undoManager?.redo()
        XCTAssertEqual(buffer.multiRanges,afterInput)
        XCTAssertTrue(buffer.snippets.move(1))
        XCTAssertEqual(buffer.multiRanges.map(\.location),[7,17])
        XCTAssertTrue(buffer.snippets.move(-1))
        let outer=try XCTUnwrap(buffer.snippets.parallel)
        let nested=outer.selections.map{expansion("q",$0)}
        XCTAssertTrue(buffer.snippets.applyParallel(source:outer.source,expansions:nested,
            primary:outer.primaryIndex,expectedIntent:buffer.snippets.intent,admitted:{true}))
        XCTAssertEqual(buffer.text,"q:q:q:q / q:q:q:q")
        XCTAssertTrue(buffer.snippets.parallel?.peers.allSatisfy{$0.session?.scopes.count==1}==true)
        XCTAssertTrue(buffer.snippets.move(1))
        XCTAssertTrue(buffer.snippets.parallel?.peers.allSatisfy{$0.session?.scopes.isEmpty==true}==true)
        editor.text.undoManager?.undo()
        XCTAssertEqual(buffer.text,"λ😀:λ😀 / λ😀:λ😀")
        XCTAssertEqual(buffer.base,source)
        #endif
    }
    func testActualMacMarkedTextCommitsMirrorsAfterNativeCandidateAndKeepsEarlierUndo() {
        #if os(macOS)
        let saved="a + a"
        let buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:saved,hash:"base"))
        let editor=MacEditor(buffer)
        let window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false)
        window.contentView=editor.scroll
        window.makeFirstResponder(editor.text)
        defer {window.contentView=nil}
        let prior=NSRange(location:0,length:0)
        XCTAssertTrue(editor.applySourceEdits([NativeSourceEdit(range:prior,text:"!")],source:saved))
        let source="!a + a"
        let choice=NSRange(location:1,length:1)
        XCTAssertTrue(editor.selectSourceRange(choice,source:source))
        let fields=[
            SnippetField(index:1,from:1,to:2,choices:[],id:0),
            SnippetField(index:1,from:5,to:6,choices:[],id:1),
            SnippetField(index:0,from:6,to:6,choices:[],id:2),
        ]
        buffer.snippets.restore(beginSnippet(source,fields:fields))
        let mark=NSRange(location:NSNotFound,length:0)
        editor.text.setMarkedText("に",selectedRange:NSRange(location:1,length:0),replacementRange:mark)
        XCTAssertTrue(editor.text.hasMarkedText())
        XCTAssertEqual(buffer.text,"!に + a")
        editor.text.setMarkedText("日本",selectedRange:NSRange(location:2,length:0),replacementRange:mark)
        XCTAssertTrue(editor.text.hasMarkedText())
        XCTAssertEqual(buffer.text,"!日本 + a")
        editor.text.insertText("日本語",replacementRange:mark)
        XCTAssertFalse(editor.text.hasMarkedText())
        XCTAssertEqual(buffer.text,"!日本語 + 日本語")
        XCTAssertEqual(editor.text.selectedRange(),NSRange(location:4,length:0))
        XCTAssertNotNil(buffer.snippets.active)
        editor.text.undoManager?.undo()
        XCTAssertEqual(buffer.text,source)
        XCTAssertEqual(editor.text.selectedRange(),choice)
        XCTAssertNotNil(buffer.snippets.active)
        editor.text.undoManager?.redo()
        XCTAssertEqual(buffer.text,"!日本語 + 日本語")
        editor.text.undoManager?.undo()
        editor.text.undoManager?.undo()
        XCTAssertEqual(buffer.text,saved)
        XCTAssertEqual(buffer.base,saved)
        #endif
    }

    func testMacUnmarkRetainsOrdinaryInputWhenSnippetIntentChanged() {
        #if os(macOS)
        let source="a + a"
        let buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:source,hash:"base"))
        let editor=MacEditor(buffer)
        let window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false)
        window.contentView=editor.scroll
        window.makeFirstResponder(editor.text)
        defer {window.contentView=nil}
        XCTAssertTrue(editor.selectSourceRange(NSRange(location:0,length:1),source:source))
        buffer.snippets.restore(beginSnippet(source,fields:fields()))
        let mark=NSRange(location:NSNotFound,length:0)
        editor.text.setMarkedText("に",selectedRange:NSRange(location:1,length:0),replacementRange:mark)
        XCTAssertEqual(buffer.text,"に + a")
        buffer.snippets.restore(nil)
        editor.text.unmarkText()
        XCTAssertFalse(editor.text.hasMarkedText())
        XCTAssertEqual(buffer.text,"に + a")
        XCTAssertNil(buffer.snippets.active)
        editor.text.undoManager?.undo()
        XCTAssertEqual(buffer.text,source)
        #endif
    }

    func testHistoryCostChargesEveryEmptyChoice() {
        let field=SnippetField(index:1,from:0,to:0,choices:Array(repeating:"",count:4096),id:0)
        XCTAssertEqual(snippetHistoryCost(SourceSnippetHistoryContext(fields:[field],active:1,current:0)),64+4096*32)
        var unicode=field;unicode.choices=["😀e\u{301}"]
        XCTAssertEqual(snippetHistoryCost(SourceSnippetHistoryContext(fields:[unicode],active:1,current:0)),64+32+7)
    }
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
    func testNestedCompletionRestoresScopeThroughNativeHistory()async throws {
        #if os(macOS)
        let source="a + a",buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:source,hash:"base")),service:NativeLanguage
        service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer);buffer.language=service
        let editor=MacEditor(buffer),window=NSWindow(contentRect:NSRect(x:0,y:0,width:500,height:300),styleMask:[.titled],backing:.buffered,defer:false);window.contentView=editor.scroll;defer{window.contentView=nil}
        editor.text.setSelectedRange(NSRange(location:0,length:1));buffer.selection=editor.text.selectedRange();buffer.snippets.restore(beginSnippet(source,fields:fields()))
        let syntax="${1:x}:$1$0",inner:[[String:Any]]=[["id":0,"parent":NSNull(),"index":1,"from":0,"to":1,"choices":[]],["id":1,"parent":NSNull(),"index":1,"from":2,"to":3,"choices":[]],["id":2,"parent":NSNull(),"index":0,"from":3,"to":3,"choices":[]]]
        let raw:[String:Any]=["label":"nested","insertTextFormat":2,"textEdit":["range":["start":["line":0,"character":0],"end":["line":0,"character":1]],"newText":syntax],"data":["hickory_snippet":["version":1,"syntax":syntax,"text":"x:x","fields":inner]]]
        let item=NativeCompletion(id:2,label:"nested",detail:"",range:NSRange(location:0,length:1),insert:syntax,source:source,selection:buffer.selection,original:raw)
        let accepted=await service.accept(item,resolution:{_ in raw});XCTAssertTrue(accepted);XCTAssertEqual(buffer.text,"x:x + x:x");XCTAssertEqual(buffer.snippets.active?.scopes.count,1)
        XCTAssertEqual(buffer.snippets.input(source:buffer.text,range:NSRange(location:0,length:1),text:"long"),true);XCTAssertEqual(buffer.text,"long:long + long:long")
        editor.text.undoManager?.undo();XCTAssertEqual(buffer.snippets.active?.scopes.count,1);editor.text.undoManager?.redo();XCTAssertEqual(buffer.snippets.active?.scopes.count,1);XCTAssertEqual(editor.text.selectedRange().location,4)
        XCTAssertTrue(buffer.snippets.move(1));XCTAssertEqual(editor.text.selectedRange().location,9);XCTAssertEqual(buffer.snippets.active?.active,1);XCTAssertTrue(buffer.snippets.move(1));XCTAssertEqual(editor.text.selectedRange().location,21);XCTAssertNil(buffer.snippets.active)
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
