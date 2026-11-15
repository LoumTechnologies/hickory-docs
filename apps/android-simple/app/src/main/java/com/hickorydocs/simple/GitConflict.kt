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
 suspend fun choose(stage:Int,load:suspend (String)->ConflictReading,offer:suspend (ConflictStage,ConflictReading,()->Boolean)->Unit){
  val value=value;val item=value?.stages?.firstOrNull{it.stage==stage};if(value==null||item==null||item.text==null){notice="Choose an available regular text stage first.";return}
  val id=++ticket;val current={id==ticket&&this.value===value};loading=true;notice="Checking captured conflict stages…"
  try{val fresh=load(value.path);if(!current())return;validateConflict(fresh,value.path);check(fresh.fingerprint==value.fingerprint&&fresh.stages.any{it==item}){"Conflict stages changed. Inspect them again before choosing a resolution."};offer(item,value,current);if(current())notice="Prepared a source change preview. Nothing saved or staged."}catch(cancelled:CancellationException){throw cancelled}catch(error:Exception){if(current())notice=error.message.orEmpty()}finally{if(id==ticket)loading=false}
 }

}
@Composable fun GitConflictView(workspace:Workspace,path:String){
 val engine=workspace.engine;val model=remember(path,engine){GitConflictModel()};val scope=rememberCoroutineScope();var selected by remember(path){mutableIntStateOf(2)};var page by remember(path){mutableIntStateOf(0)}
 suspend fun choose(stage:Int){
  if(engine==null){model.notice="Connect to a workspace server first.";return};val epoch=workspace.sourceNavigationEpoch
  model.choose(stage,{value->conflictReading(engine.request("GET","/api/git/conflict?path=${URLEncoder.encode(value,"UTF-8")}"))}){item,value,current->
   val valid={current()&&workspace.sourceNavigationEpoch==epoch&&workspace.engine===engine};check(valid()){"Workspace changed. Prepare the conflict again."}
   fun find(files:List<FileNode>):FileNode?{for(file in files){if(file.path==value.path)return file;find(file.children)?.let{return it}};return null}
   val file=(workspace.buffers[value.path]?.file ?: find(workspace.rootFiles))?.takeUnless{it.dir} ?: error("Conflict source is no longer listed. Refresh files and try again.")
   val buffer=workspace.loadBuffer(file);check(valid()){"Workspace changed. Prepare the conflict again."}
   check(!buffer.readOnly&&!buffer.saving){"Choose an authored, editable source that is not saving."}
   val replacement=item.text ?: error("Text unavailable");val source=buffer.text;val rows=source.split('\n')
   val end=JSONObject().put("line",rows.size-1).put("character",rows.last().length)
   val edit=JSONObject().put("changes",JSONObject().put(languageURI(value.path),org.json.JSONArray().put(JSONObject().put("range",JSONObject().put("start",JSONObject().put("line",0).put("character",0)).put("end",end)).put("newText",replacement))))
   workspace.changes.offer("Use Git stage ${item.stage} · ${item.oid}",edit,mapOf(value.path to source),workspace);workspace.showReview=false
  }
 }
 DisposableEffect(model){onDispose{model.close()}}
 Column(verticalArrangement=Arrangement.spacedBy(6.dp)){
  TextButton(enabled=!model.loading,onClick={scope.launch{if(engine==null){model.notice="Connect to a workspace server first.";return@launch};page=0;model.read(path){value->conflictReading(engine.request("GET","/api/git/conflict?path=${URLEncoder.encode(value,"UTF-8")}"))}}}){Text("Inspect conflict stages")}
  if(model.notice.isNotEmpty())Text(model.notice,fontSize=12.sp)
  model.value?.let{value->
   Text("Stage 1: base · Stage 2 and Stage 3 retain Git’s meanings, including during rebase.",fontSize=12.sp)
   Row{value.stages.forEach{stage->TextButton(onClick={selected=stage.stage;page=0}){Text("Stage ${stage.stage}${if(stage.stage==1)" · Base" else ""}")}}}
   (value.stages.firstOrNull{it.stage==selected} ?: value.stages.firstOrNull())?.let{stage->
    Text("Stage ${stage.stage} · mode ${stage.mode}",fontSize=12.sp);SelectionContainer{Text(stage.oid,fontFamily=FontFamily.Monospace,fontSize=12.sp)}
    stage.text?.let{text->TextButton(enabled=!model.loading,onClick={scope.launch{choose(stage.stage)}}){Text("Preview this stage as source")};val rows=text.split('\n');val last=((rows.size+99)/100-1).coerceAtLeast(0);val index=page.coerceAtMost(last);val preview=rows.drop(index*100).take(100).joinToString("\n")
     Text("Lines ${index*100+1}–${rows.size.coerceAtMost((index+1)*100)}${if(preview.length>200000)" · page preview capped at 200,000 characters" else ""}",fontSize=12.sp)
     SelectionContainer{Text(preview.take(200000),fontFamily=FontFamily.Monospace,fontSize=12.sp,modifier=Modifier.horizontalScroll(rememberScrollState()))}
     Row{TextButton(enabled=index>0,onClick={page=index-1}){Text("Previous stage lines")};TextButton(enabled=index<last,onClick={page=index+1}){Text("Next stage lines")}}
    } ?: Text(stage.unavailable ?: "Text unavailable",fontSize=12.sp)
   }
  }
 }
}
