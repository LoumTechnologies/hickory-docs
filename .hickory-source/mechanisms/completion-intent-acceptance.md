<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Retire offered and pending completion intentions after source or cursor round trips

Equal bytes and a restored cursor do not establish that an old offer is still current. Production native offers retain their source revision and selection intent; pending acceptance captures them again before resolution. Browser offers and acceptance reuse the editor adapter’s opaque input token, which ordinary source/selection transactions advance and undo never restores. Capability arrivals without input retain the existing editor state and token.

These focused native witnesses try both an old offered completion and a resolver that returns after source edit/undo or selection A→B→A. They require no provider or editor commit on refusal, preserve the original text and cursor, and keep the injected-resolution test seam explicit. Existing AppKit/Compose history witnesses exercise the usable successful path separately. Physical pointer, keyboard, device and accessibility acceptance remains unverified.

<ui:copy id="completion-intent-apple-tests">
import XCTest
@testable import HickorySimple
@MainActor final class CompletionIntentTests:XCTestCase {
 private func fixture()throws->(DocumentBuffer,NativeLanguage){let buffer=DocumentBuffer(file:FileNode(name:"a.py",path:"a.py",dir:false),text:FileText(path:"a.py",content:"va\n",hash:"base"));let service=NativeLanguage(engine:try Engine(address:"http://fixture.invalid"),buffer:buffer);buffer.language=service;buffer.selection=NSRange(location:2,length:0);return(buffer,service)}
 private func offer(_ buffer:DocumentBuffer)->NativeCompletion{NativeCompletion(id:1,label:"value",detail:"",range:NSRange(location:0,length:2),insert:"value",source:buffer.text,selection:buffer.selection,original:["label":"value"],sourceIdentity:buffer.sourceEditIdentity,selectionIdentity:buffer.selectionIdentity)}
 private func resolved()->[String:Any]{["label":"value","textEdit":["range":["start":["line":0,"character":0],"end":["line":0,"character":2]],"newText":"value"]]}
 private func roundTrip(_ buffer:DocumentBuffer,source:Bool){if source{let original=buffer.text;buffer.text="xy\n";buffer.text=original}else{let original=buffer.selection;buffer.selection=NSRange(location:0,length:0);buffer.selection=original}}
 func testOfferedCompletionCannotReviveAfterSourceOrSelectionRoundTrip()async throws {
  for source in [true,false]{let(buffer,service)=try fixture();defer{service.close()};let item=offer(buffer);roundTrip(buffer,source:source);var calls=0,commits=0;buffer.applyCompletionDraft={_,_ in commits+=1;return true};let accepted=await service.accept(item,resolution:{_ in calls+=1;return self.resolved()});XCTAssertFalse(accepted);XCTAssertEqual(calls,0);XCTAssertEqual(commits,0);XCTAssertEqual(buffer.text,"va\n");XCTAssertEqual(buffer.selection,NSRange(location:2,length:0));XCTAssertTrue(service.completionNotice.contains("out of date"))}
 }
 func testPendingCompletionCannotReviveAfterSourceOrSelectionRoundTrip()async throws {
  for source in [true,false]{let(buffer,service)=try fixture();defer{service.close()};let item=offer(buffer);var commits=0;buffer.applyCompletionDraft={_,_ in commits+=1;return true};let accepted=await service.accept(item,resolution:{_ in self.roundTrip(buffer,source:source);return self.resolved()});XCTAssertFalse(accepted);XCTAssertEqual(commits,0);XCTAssertEqual(buffer.text,"va\n");XCTAssertEqual(buffer.selection,NSRange(location:2,length:0));XCTAssertTrue(service.completionNotice.contains("changed"))}
 }
}
</ui:copy>
<ui:file path="apps/apple-simple/Tests/CompletionIntentTests.swift"><ui:paste select="#completion-intent-apple-tests"/></ui:file>

<ui:copy id="completion-intent-android-tests">
package com.hickorydocs.simple
import androidx.compose.ui.text.TextRange
import kotlinx.coroutines.runBlocking
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
class CompletionIntentTest {
 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun offeredCompletionCannotReviveAfterSourceOrSelectionRoundTrip()=runBlocking {
  for(source in listOf(true,false)){val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText("va\n","base"));val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service;buffer.state.edit{selection=TextRange(2)};try{val item=NativeLanguageCompletion("value","",NativeSourceEdit(0,2,"value"),buffer.text,buffer.state.selection,JSONObject().put("label","value"),buffer.sourceRevision);roundTrip(buffer,source);var calls=0;val accepted=service.accept(item){calls++;resolved()};assertFalse(accepted);assertEquals(0,calls);assertEquals("va\n",buffer.text);assertEquals(TextRange(2),buffer.state.selection);assertTrue(service.completionNotice.contains("out of date"))}finally{service.close()}}
 }
 @OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
 @Test fun pendingCompletionCannotReviveAfterSourceOrSelectionRoundTrip()=runBlocking {
  for(source in listOf(true,false)){val buffer=DocumentBuffer(FileNode("a.py","a.py",false,emptyList(),null,null),FileText("va\n","base"));val service=NativeLanguage(Engine("http://127.0.0.1:1"),buffer,this);buffer.language=service;buffer.state.edit{selection=TextRange(2)};try{val item=NativeLanguageCompletion("value","",NativeSourceEdit(0,2,"value"),buffer.text,buffer.state.selection,JSONObject().put("label","value"),buffer.sourceRevision);val accepted=service.accept(item){roundTrip(buffer,source);resolved()};assertFalse(accepted);assertEquals("va\n",buffer.text);assertEquals(TextRange(2),buffer.state.selection);assertTrue(service.completionNotice.contains("changed"))}finally{service.close()}}
 }
 private fun resolved()=JSONObject("""{"label":"value","textEdit":{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":2}},"newText":"value"}}""")
 private fun roundTrip(buffer:DocumentBuffer,source:Boolean){if(source){assertTrue(buffer.applySourceEdits(listOf(NativeSourceEdit(0,2,"xy")),buffer.text));assertTrue(buffer.snippets.undoSource())}else{assertTrue(buffer.selectSourceRange(0,0,buffer.text));assertTrue(buffer.selectSourceRange(2,2,buffer.text))}}
}
</ui:copy>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/CompletionIntentTest.kt"><ui:paste select="#completion-intent-android-tests"/></ui:file>
</ui:doc>
