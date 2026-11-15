package com.hickorydocs.simple
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import java.net.URLEncoder
import org.json.JSONObject

data class RecoveryPage(val version:Int,val workspace:String,val ids:List<String>,val next:String?,val ignored:Int)
data class RecoverySummary(val kind:String,val bytes:Int?,val sha256:String?)
data class RecoveryPath(val path:String,val snapshots:Map<String,RecoverySummary>)
data class RecoveryReading(val version:Int,val workspace:String,val id:String,val fingerprint:String,val title:String,val titleTruncated:Boolean,val paths:List<RecoveryPath>,val allApplied:Boolean,val unrecognized:Int,val draftText:String?,val savedText:String?)
private fun JSONObject.recoveryText(key:String)=if(isNull(key))null else getString(key)
fun recoveryPage(raw:JSONObject):RecoveryPage {val ids=raw.getJSONArray("ids");return RecoveryPage(raw.getInt("version"),raw.getString("workspace"),(0 until ids.length()).map{ids.getString(it)},raw.recoveryText("next"),raw.getInt("ignored"))}
fun recoveryReading(raw:JSONObject):RecoveryReading {val paths=raw.getJSONArray("paths");return RecoveryReading(raw.getInt("version"),raw.getString("workspace"),raw.getString("id"),raw.getString("fingerprint"),raw.getString("title"),raw.getBoolean("title_truncated"),(0 until paths.length()).map{val row=paths.getJSONObject(it);RecoveryPath(row.getString("path"),listOf("saved_before","saved_after","draft_before","draft_after").associateWith{key->val value=row.getJSONObject(key);RecoverySummary(value.getString("kind"),if(value.has("bytes"))value.getInt("bytes") else null,value.recoveryText("sha256"))})},raw.getBoolean("all_paths_recorded_applied"),raw.getJSONArray("unrecognized_files").length(),raw.recoveryText("draft_text"),raw.recoveryText("saved_text"))}
class RecoveryReader {
 var page by mutableStateOf<RecoveryPage?>(null);var value by mutableStateOf<RecoveryReading?>(null);var notice by mutableStateOf("");var loading by mutableStateOf(false);private var ticket=0L
 fun close(){ticket++;loading=false;page=null;value=null}
 suspend fun list(after:String?=null,load:suspend(String)->RecoveryPage){val id=++ticket;loading=true;value=null;notice="Reading retained recovery identities…"
  try{val next=load("/api/changes/resource-recoveries"+(after?.let{"?after="+URLEncoder.encode(it,"UTF-8")} ?: ""));if(id!=ticket)return
   check(next.version==1&&next.workspace.isNotEmpty()&&next.ids.size<=100&&next.ids.all{it.matches(Regex("[0-9a-f]{32}"))}&&next.ids.zipWithNext().all{it.first<it.second}&&(after==null||next.ids.all{it>after}&&next.workspace==page?.workspace)&&(next.next==null||next.next==next.ids.lastOrNull())){"Recovery page identity changed. Refresh from the beginning."}
   page=next;notice=if(next.ids.isEmpty())"No retained resource recovery records." else "Choose a retained record. Identities are sorted, not chronological."
  }catch(error:CancellationException){throw error}catch(error:Exception){if(id==ticket){page=null;notice=error.message.orEmpty()}}finally{if(id==ticket)loading=false}
 }
 suspend fun read(record:String,path:String?=null,snapshot:String="draft_before",load:suspend(String)->RecoveryReading){val workspace=page?.workspace;val previous=value?.takeIf{it.id==record};val id=++ticket;loading=true;value=null;notice="Reading retained source evidence…"
  try{check(workspace!=null&&page?.ids?.contains(record)==true){"Refresh recovery identities before selecting a record."};val parts=snapshot.split('_');check(parts.size==2&&parts[0] in listOf("saved","draft")&&parts[1] in listOf("before","after")){"Choose a supported recovery snapshot."}
   val reply=load("/api/changes/resource-recovery?id="+URLEncoder.encode(record,"UTF-8")+(path?.let{"&path="+URLEncoder.encode(it,"UTF-8")+"&"+parts[0]+"="+parts[1]} ?: ""));if(id!=ticket)return
   check(reply.version==1&&reply.workspace==workspace&&reply.id==record&&reply.fingerprint.matches(Regex("[0-9a-f]{64}"))&&(previous==null||previous.fingerprint==reply.fingerprint)&&(path==null||reply.paths.any{it.path==path})){"Recovery evidence changed. Refresh identities and read again."}
   value=reply;notice="Retained evidence only. Current source and editor drafts are unchanged."
  }catch(error:CancellationException){throw error}catch(error:Exception){if(id==ticket)notice=error.message.orEmpty()}finally{if(id==ticket)loading=false}
 }
}
@Composable fun SourceRecoveryView(workspace:Workspace){
 val model=remember{RecoveryReader()};val scope=rememberCoroutineScope();val engine=workspace.engine;var path by remember{mutableStateOf("")};var snapshot by remember{mutableStateOf("draft_before")};var recordMenu by remember{mutableStateOf(false)};var pathMenu by remember{mutableStateOf(false)}
 suspend fun list(after:String?=null){if(engine==null){model.notice="Connect to a workspace server first.";return};path="";model.list(after){recoveryPage(engine.request("GET",it))}}
 suspend fun read(id:String,path:String?=null,snapshot:String="draft_before"){if(engine==null)return;model.read(id,path,snapshot){recoveryReading(engine.request("GET",it))}}
 LaunchedEffect(engine){list()};DisposableEffect(model){onDispose{model.close()}}
 Column(Modifier.widthIn(max=680.dp).heightIn(max=680.dp).padding(16.dp).verticalScroll(rememberScrollState()),verticalArrangement=Arrangement.spacedBy(8.dp)){
  Text("Source recovery",style=MaterialTheme.typography.titleMedium);Row{TextButton(enabled=!model.loading,onClick={scope.launch{list()}}){Text("Refresh records")};TextButton(onClick={workspace.showRecovery=false;workspace.currentBuffer?.focusSource?.invoke()}){Text("Close")}}
  Text(model.notice,fontSize=12.sp)
  model.page?.let{page->Box{TextButton(enabled=!model.loading,onClick={recordMenu=true}){Text(model.value?.id ?: "Choose retained record")};DropdownMenu(expanded=recordMenu,onDismissRequest={recordMenu=false}){page.ids.forEach{id->DropdownMenuItem(text={Text(id)},onClick={recordMenu=false;path="";scope.launch{read(id)}})}}};if(page.ignored>0)Text("${page.ignored} unrecognized entries omitted.",fontSize=12.sp);page.next?.let{next->TextButton(enabled=!model.loading,onClick={scope.launch{list(next)}}){Text("Next 100 identities")}}}
  model.value?.let{value->Text(value.title+if(value.titleTruncated)"…" else "",style=MaterialTheme.typography.titleSmall);Text((if(value.allApplied)"Every path has an applied receipt." else "The record does not confirm every path was applied.")+" These receipts describe the recorded operation, not current disk state.",fontSize=12.sp);if(value.unrecognized>0)Text("${value.unrecognized} unrecognized files in this record.",fontSize=12.sp)
   Box{TextButton(enabled=!model.loading,onClick={pathMenu=true}){Text(path.ifEmpty{"Choose recorded source"})};DropdownMenu(expanded=pathMenu,onDismissRequest={pathMenu=false}){value.paths.forEach{item->DropdownMenuItem(text={Text(item.path)},onClick={pathMenu=false;path=item.path;scope.launch{read(value.id,path,snapshot)}})}}}
   listOf("saved_before","draft_before","saved_after","draft_after").forEach{side->TextButton(enabled=!model.loading,onClick={snapshot=side;if(path.isNotEmpty())scope.launch{read(value.id,path,side)}}){Text((if(snapshot==side)"✓ " else "")+side.replace('_',' '))}}
   value.paths.find{it.path==path}?.snapshots?.get(snapshot)?.let{selected->SelectionContainer{Text(if(selected.kind=="missing")"Source absent in this snapshot." else "${selected.bytes} UTF-8 bytes · ${selected.sha256}",fontSize=12.sp)};val text=if(snapshot.startsWith("saved"))value.savedText else value.draftText;if(text!=null){SelectionContainer{Text(text.take(32000),fontFamily=FontFamily.Monospace,fontSize=12.sp)};if(text.length>32000)Text("First 32,000 characters shown.",fontSize=12.sp)}}
  }
 }
}
