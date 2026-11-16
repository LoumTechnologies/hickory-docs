package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
// Contract: .hickory-source/frontends/contract.md (resource namespace preservation).
class ResourceEditorsTest {
 class Editor {var selection=2;var scroll=108;val undo=mutableListOf("old")}
 @Test fun moveDeleteRestoreAndCreateKeepOpaqueEditorState(){
  val editor=Editor();val a=ResourceSource("a","dirty");val capture=ResourceEditorCapture(editor,"dirty")
  val move=routeResourceEditors(listOf(ResourceEditorPatch("a.py",a,null),ResourceEditorPatch("b.py",null,a)),mapOf("a" to capture));assertSame(editor,move.single().editor);assertEquals("b.py",move.single().to)
  val other=Editor();val b=ResourceSource("b","B");val swapped=routeResourceEditors(listOf(ResourceEditorPatch("a.py",a,b),ResourceEditorPatch("b.py",b,a)),mapOf("a" to capture,"b" to ResourceEditorCapture(other,"B")));assertSame(editor,swapped.first{it.to=="b.py"}.editor);assertSame(other,swapped.first{it.to=="a.py"}.editor)
  val deleted=routeResourceEditors(listOf(ResourceEditorPatch("b.py",a,null)),mapOf("a" to capture));assertNull(deleted.single().to);assertSame(editor,deleted.single().editor)
  val restored=routeResourceEditors(listOf(ResourceEditorPatch("b.py",null,ResourceSource("restored","dirty"))),emptyMap(),mapOf("restored" to capture));assertSame(editor,restored.single().editor)
  val created=routeResourceEditors<Editor>(listOf(ResourceEditorPatch("new.py",null,ResourceSource("new",""))),emptyMap());assertNull(created.single().editor);assertEquals("",created.single().after)
  assertEquals(2,editor.selection);assertEquals(108,editor.scroll);assertEquals(listOf("old"),editor.undo)
 }
 @Test fun confirmedReceiptRequiresWholeNamespaceAndSeparateSavedBaseline(){
  val saved=ResourceSaved("file","saved","a".repeat(64),false,false,null);val absent=ResourceSaved("missing")
  val patches=listOf(ResourceEditorPatch("old.py",ResourceSource("a","dirty"),null),ResourceEditorPatch("new.py",null,ResourceSource("a","dirty")))
  val receipt=ResourcePublication(1,"request","workspace","context",true,"record",listOf("old.py","new.py"),mapOf("old.py" to absent,"new.py" to saved),listOf(ResourcePublishedFile("new.py",saved,"fresh")))
  fun confirm(value:ResourcePublication<String>)=confirmedResourceFiles(patches,value,"request","workspace","context",mapOf("old.py" to absent,"new.py" to saved)){it to true}
  assertEquals("saved",confirm(receipt).getValue("new.py").saved.text)
  for(value in listOf(receipt.copy(complete=false),receipt.copy(request="other"),receipt.copy(files=receipt.files+receipt.files),receipt.copy(applied=listOf("new.py")))){try{confirm(value);fail("Expected refusal")}catch(_:IllegalArgumentException){}}
 }
 @Test fun completePreflightAndByteExactNamespaces(){
  val editor=Editor();val capture=ResourceEditorCapture(editor,"a");val a=ResourceSource("a","a");val patch=ResourceEditorPatch("a.py",a,null)
  fun refuses(block:()->Unit){try{block();fail("Expected refusal")}catch(_:IllegalArgumentException){}}
  refuses{routeResourceEditors(listOf(patch,patch),mapOf("a" to capture))};refuses{routeResourceEditors<Editor>(listOf(patch),emptyMap())}
  refuses{routeResourceEditors(listOf(patch),mapOf("a" to ResourceEditorCapture(editor,"later")))};refuses{routeResourceEditors(listOf(patch),mapOf("a" to capture),mapOf("a" to capture))}
  refuses{routeResourceEditors(listOf(patch,ResourceEditorPatch("b.py",ResourceSource("b","b"),null)),mapOf("a" to capture,"b" to ResourceEditorCapture(editor,"b")))}
  assertEquals(2,routeResourceEditors<Editor>(listOf(ResourceEditorPatch("é.py",null,ResourceSource("é","")),ResourceEditorPatch("e\u0301.py",null,ResourceSource("e\u0301",""))),emptyMap()).size)
  assertEquals(listOf("old"),editor.undo)
 }
}
