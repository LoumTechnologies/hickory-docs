package com.hickorydocs.simple
import kotlinx.coroutines.runBlocking
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Test
class WorkspaceEntrySelectionTest {
 private fun original()=JSONObject().put("version",1).put("entry_protocol",1).put("workspace","a".repeat(32)).put("namespace","/workspace").put("entry",JSONObject().put("version",1).put("identity","b".repeat(32)).put("generation",7).put("path","project.md")).put("guard","c".repeat(64)).put("source_guard","opaque physical observation").put("source_hash","sha256:"+"d".repeat(64)).put("movable",true).put("source_effects",false)
 private fun refuses(action:()->Unit){try{action();fail("Malformed entry authority was accepted")}catch(expected:IllegalArgumentException){}catch(expected:IllegalStateException){}}
 @Test fun originalProofDoesNotFollowMutableResponseOrReturnedBody(){val response=original();val proof=WorkspaceEntrySelectionProof(response);response.put("namespace","/other");proof.body().put("namespace","/mutated");assertEquals("/workspace",proof.namespace);assertEquals("project.md",proof.entry.path);assertEquals("/workspace",proof.body().getString("namespace"))}
 @Test fun strictTypesAndGenerationBoundsRefuseIncompleteAuthority(){for((key,value)in listOf("unexpected" to 1,"source_effects" to "false","source_effects" to 0,"movable" to 1,"source_guard" to JSONObject.NULL,"source_hash" to "wrong","guard" to "C".repeat(64))){refuses{WorkspaceEntrySelectionProof(original().put(key,value))}};for(value in listOf(-1L,0x100000000L)){val body=original();body.getJSONObject("entry").put("generation",value);refuses{WorkspaceEntrySelectionProof(body)}}}
 @Test fun positiveAbsenceCannotBorrowSourceOrMoveAuthority(){val response=original();response.getJSONObject("entry").put("path",JSONObject.NULL);refuses{WorkspaceEntrySelectionProof(response)};response.remove("source_guard");response.remove("source_hash");response.put("movable",false);assertNull(WorkspaceEntrySelectionProof(response).entry.path)}
 @Test fun noEffectRequiresTheExactRequestAndOriginalAddress(){
  val response=original();val proof=WorkspaceEntrySelectionProof(response);val before=response.getJSONObject("entry")
  val after=JSONObject(before.toString()).put("generation",8).put("path","moved.spec")
  val request="a".repeat(32);val token="e".repeat(64)
  val reply=JSONObject().put("version",1).put("workspace",proof.workspace).put("namespace",proof.namespace)
   .put("before_entry",before).put("after_entry",after).put("context",token).put("token",token)
   .put("kind","move").put("title","Move").put("edit",JSONObject()).put("source_files_modified",false)
  val intent=WorkspaceEntryIntent(reply,proof,"move","moved.spec")
  fun outcome(consumed:Any,retained:JSONObject)=JSONObject().put("version",1).put("state","no_effect").put("request_id",request)
   .put("before_entry",before).put("after_entry",after).put("reservation_consumed",consumed).put("current_entry",retained)
  val retained=JSONObject(before.toString()).put("generation",8)
  assertTrue(workspaceEntryPublicationHadNoEffect(outcome(false,before),request,intent))
  assertTrue(workspaceEntryPublicationHadNoEffect(outcome(true,retained),request,intent))
  refuses{workspaceEntryPublicationHadNoEffect(outcome(true,after),request,intent)}
  refuses{workspaceEntryPublicationHadNoEffect(outcome(1,retained),request,intent)}
 }
 @Test fun asynchronousAcquisitionNeverRebindsOriginalLease()=runBlocking {val response=original();var live=false;var reads=0;try{WorkspaceEntrySelections.read({reads++;response},"a".repeat(32),"/workspace"){live};fail("stale source must refuse before request")}catch(expected:IllegalStateException){};assertEquals(0,reads);live=true;try{WorkspaceEntrySelections.read({live=false;response},"a".repeat(32),"/workspace"){live};fail("late reply must refuse")}catch(expected:IllegalStateException){};live=true;val frozen=WorkspaceEntrySelections.read({response},"a".repeat(32),"/workspace"){live};assertEquals("project.md",frozen.entryPath);live=false;assertFalse(frozen.current())}
}
