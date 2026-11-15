<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Inspect immutable Git conflict stages

For maintainers changing workspace services independently of the editor renderer.

Conflict inspection is an optional capability inside the existing repository review. It validates literal source identity, protocol version, immutable object names, stage uniqueness and explicit text availability. A fixed read ticket retires held/closed/foreign replies. Stage labels preserve Git’s meaning during rebase. Source text and index remain separate from this observation; this view does not publish a resolution. Page the retained stage text without reserving code space, and disclose a cap for a very long page.

<ui:copy id="git-conflict-android">
package com.hickorydocs.simple
import androidx.compose.runtime.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.material3.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import org.json.JSONObject
import java.net.URLEncoder
data class ConflictStage(val stage:Int,val mode:String,val oid:String,val text:String?,val unavailable:String?)
data class ConflictReading(val version:Int,val path:String,val fingerprint:String,val stages:List<ConflictStage>,val indexModified:Boolean,val sourceFilesModified:Boolean)
fun conflictReading(raw:JSONObject):ConflictReading {
 val rows=raw.getJSONArray("stages")
 return ConflictReading(raw.get("version") as? Int ?: error("Malformed conflict version."),raw.getString("path"),raw.getString("fingerprint"),(0 until rows.length()).map{val value=rows.getJSONObject(it);ConflictStage(value.get("stage") as? Int ?: error("Malformed conflict stage."),value.getString("mode"),value.getString("oid"),if(value.isNull("text"))null else value.getString("text"),if(value.isNull("unavailable"))null else value.getString("unavailable"))},raw.getBoolean("index_modified"),raw.getBoolean("source_files_modified"))
}
fun validateConflict(value:ConflictReading,path:String){
 fun hex(text:String,lengths:List<Int>)=text.length in lengths&&text.all{it in '0'..'9'||it in 'a'..'f'||it in 'A'..'F'}
 check(value.version==1&&value.path==path&&!value.indexModified&&!value.sourceFilesModified&&hex(value.fingerprint,listOf(64))&&value.stages.size in 1..3){"Conflict identity changed or unsupported response. Refresh the conflict."}
 val seen=mutableSetOf<Int>();for(stage in value.stages)check(stage.stage in 1..3&&seen.add(stage.stage)&&hex(stage.oid,listOf(40,64))&&stage.mode.length==6&&stage.mode.all{it in '0'..'7'}&&((stage.text!=null&&stage.unavailable==null&&stage.mode in listOf("100644","100755"))||(stage.text==null&&stage.unavailable!=null))){"Malformed conflict stage. Refresh repository changes."}
}
class GitConflictModel {
 var value by mutableStateOf<ConflictReading?>(null);var notice by mutableStateOf("");var loading by mutableStateOf(false);private var ticket=0
 fun close(){ticket++;loading=false;value=null}
 suspend fun read(path:String,load:suspend (String)->ConflictReading){val id=++ticket;value=null;loading=true;notice="Reading unmerged stages…"
 try{val result=load(path);if(id!=ticket)return;validateConflict(result,path);value=result;notice="Captured immutable Git stages. Working text and unsaved drafts are separate."}catch(cancelled:CancellationException){throw cancelled}catch(error:Exception){if(id==ticket)notice=error.message.orEmpty()}finally{if(id==ticket)loading=false}}
}
@Composable fun GitConflictView(workspace:Workspace,path:String){
 val engine=workspace.engine;val model=remember(path,engine){GitConflictModel()};val scope=rememberCoroutineScope();var selected by remember(path){mutableIntStateOf(2)};var page by remember(path){mutableIntStateOf(0)}
 DisposableEffect(model){onDispose{model.close()}}
 Column(verticalArrangement=Arrangement.spacedBy(6.dp)){
  TextButton(enabled=!model.loading,onClick={scope.launch{if(engine==null){model.notice="Connect to a workspace server first.";return@launch};page=0;model.read(path){value->conflictReading(engine.request("GET","/api/git/conflict?path=${URLEncoder.encode(value,"UTF-8")}"))}}}){Text("Inspect conflict stages")}
  if(model.notice.isNotEmpty())Text(model.notice,fontSize=12.sp)
  model.value?.let{value->
   Text("Stage 1: base · Stage 2 and Stage 3 retain Git’s meanings, including during rebase.",fontSize=12.sp)
   Row{value.stages.forEach{stage->TextButton(onClick={selected=stage.stage;page=0}){Text("Stage ${stage.stage}${if(stage.stage==1)" · Base" else ""}")}}}
   (value.stages.firstOrNull{it.stage==selected} ?: value.stages.firstOrNull())?.let{stage->
    Text("Stage ${stage.stage} · mode ${stage.mode}",fontSize=12.sp);SelectionContainer{Text(stage.oid,fontFamily=FontFamily.Monospace,fontSize=12.sp)}
    stage.text?.let{text->val rows=text.split('\n');val last=((rows.size+99)/100-1).coerceAtLeast(0);val index=page.coerceAtMost(last);val preview=rows.drop(index*100).take(100).joinToString("\n")
     Text("Lines ${index*100+1}–${rows.size.coerceAtMost((index+1)*100)}${if(preview.length>200000)" · page preview capped at 200,000 characters" else ""}",fontSize=12.sp)
     SelectionContainer{Text(preview.take(200000),fontFamily=FontFamily.Monospace,fontSize=12.sp,modifier=Modifier.horizontalScroll(rememberScrollState()))}
     Row{TextButton(enabled=index>0,onClick={page=index-1}){Text("Previous stage lines")};TextButton(enabled=index<last,onClick={page=index+1}){Text("Next stage lines")}}
    } ?: Text(stage.unavailable ?: "Text unavailable",fontSize=12.sp)
   }
  }
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/GitConflict.kt"><ui:paste select="#git-conflict-android"/></ui:file>
## Retire stale conflict observations

Release held conflict reads out of order, close during a pending read, and supply a byte-distinct foreign path. Only the latest exact identity may display. Validate immutable object names, unique stages and explicit binary/text availability. These are service-model witnesses, not native physical input acceptance.

<ui:copy id="git-conflict-android-tests">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
class GitConflictTest {
 private fun reading(path:String,text:String="captured")=ConflictReading(1,path,"a".repeat(64),listOf(ConflictStage(2,"100644","b".repeat(40),text,null)),false,false)
 @Test fun olderClosedAndForeignRepliesAreRetired()=runBlocking{
  val model=GitConflictModel();val held=mutableListOf<CompletableDeferred<ConflictReading>>();val load:suspend (String)->ConflictReading={CompletableDeferred<ConflictReading>().also{held.add(it)}.await()}
  suspend fun settle(count:Int){withTimeout(2000){while(held.size<count)yield()}}
  val first=async{model.read("a.txt",load)};settle(1);val second=async{model.read("b.txt",load)};settle(2)
  held[1].complete(reading("b.txt","new"));second.await();held[0].complete(reading("a.txt","old"));first.await();assertEquals("b.txt",model.value?.path)
  val closed=async{model.read("a.txt",load)};settle(3);model.close();held[2].complete(reading("a.txt"));closed.await();assertNull(model.value)
  model.read("é.txt"){reading("e\u0301.txt")};assertNull(model.value);assertTrue(model.notice.contains("identity changed"))
 }
 @Test fun immutableStageAndAvailabilityValidation(){
  val value=reading("a.txt");validateConflict(value,"a.txt")
  for(bad in listOf(value.copy(stages=value.stages+value.stages),value.copy(stages=listOf(value.stages[0].copy(oid="main"))),value.copy(stages=listOf(value.stages[0].copy(mode="120000"))),value.copy(stages=listOf(value.stages[0].copy(text=null))))){assertTrue(runCatching{validateConflict(bad,"a.txt")}.isFailure)}
  validateConflict(value.copy(stages=listOf(value.stages[0].copy(text=null,unavailable="Binary"))),"a.txt")
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/GitConflictTest.kt"><ui:paste select="#git-conflict-android-tests"/></ui:file>
</ui:doc>
