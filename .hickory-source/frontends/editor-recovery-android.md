<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Recover Android editor sessions without source publication

For maintainers connecting private checkpoints to the existing compact recovery dialog.

## Admit typed snapshots independently of providers

Source text remains available while private storage is slow or unavailable. Checkpoints retain original baselines only as comparison material, explicit named/unnamed/generated identities and source views. Literal names do not determine source kind. Hash exact retained material before decoding it, then validate every source and scalar boundary. Recovery never grants a physical save guard.

<ui:copy id="editor-recovery-android-codec">
@file:OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
package com.hickorydocs.simple
import androidx.compose.runtime.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.*
import org.json.JSONObject
import org.json.JSONArray
import org.json.JSONTokener
private fun recoveryToken()=java.util.UUID.randomUUID().toString().replace("-","")
private fun recoveryHex(s:String,n:Int)=s.matches(Regex("[a-f0-9]{$n}"))
private fun recoveryString(raw:JSONObject,key:String)=raw.get(key) as? String ?: error("Malformed private recovery $key.")
private fun recoveryUInt(raw:JSONObject,key:String):Long{val n=raw.get(key);check(n is Int||n is Long){"Malformed private recovery $key."};return (n as Number).toLong().also{check(it in 0..4294967295L){"Private recovery counter exceeds its protocol."}}}
private fun recoveryBool(raw:JSONObject,key:String)=raw.get(key) as? Boolean ?: error("Malformed private recovery $key.")
private fun recoverySlot(value:String)=value.matches(Regex("0|[1-9][0-9]{0,19}"))&&value.toULongOrNull()!=null
private fun recoveryStrings(raw:JSONArray):List<String> =(0 until raw.length()).map{raw.get(it) as? String ?: error("Malformed private recovery string.")}
data class EditorRecoveryAddress(val kind:String,val label:String,val owner:String?=null,val original:String?=null){fun json()=JSONObject().put("kind",kind).apply{when(kind){"source"->put("path",label);"generated"->{put("path",label);put("owner",owner)};"unnamed"->put("name",label);"detached"->{put("name",label);put("original_path",original ?: JSONObject.NULL)}}}}
data class EditorRecoveryBaseline(val text:String,val guard:String?){fun json()=JSONObject().put("text",text).put("save_guard",guard ?: JSONObject.NULL)}
data class EditorRecoveryViewport(val anchor:Int,val head:Int,val x:Double,val y:Double){fun json()=JSONObject().put("selection",JSONObject().put("anchor",anchor).put("head",head)).put("scroll_x",x).put("scroll_y",y)}
data class EditorRecoveryBuffer(val id:String,val address:EditorRecoveryAddress,val text:String,val baseline:EditorRecoveryBaseline?,val view:EditorRecoveryViewport){fun json()=JSONObject().put("id",id).put("address",address.json()).put("text",text).put("baseline",baseline?.json() ?: JSONObject.NULL).put("view",view.json())}
data class EditorRecoverySnapshot(val host:String,val buffers:List<EditorRecoveryBuffer>,val layout:String){fun json()=JSONObject().put("host",host).put("buffers",JSONArray(buffers.map{it.json()})).put("layout",JSONTokener(layout).nextValue())}
data class EditorRecoverySession(val client:String,val slot:String,val workspace:String,val namespace:String,val revision:Long,val request:String,val hash:String,val host:String,val count:Int)
data class EditorRecoveryCatalogue(val runtime:String,val workspace:String,val namespace:String,val slot:String,val currentSlot:String,val slots:List<String>,val sessions:List<EditorRecoverySession>,val notices:List<String>)
fun editorRecoverySnapshot(raw:JSONObject):EditorRecoverySnapshot {
 val host=recoveryString(raw,"host");check(host in listOf("browser","macos","ios","android")){"Unsupported private snapshot host."};val rows=raw.getJSONArray("buffers");check(rows.length()<=256){"Private recovery exceeds256buffers."};val ids=mutableSetOf<String>();var bytes=0L
 val buffers=(0 until rows.length()).map{index->val b=rows.getJSONObject(index);val id=recoveryString(b,"id");check(recoveryHex(id,32)&&ids.add(id)){"Private buffer identities are ambiguous."};val a=b.getJSONObject("address");val kind=recoveryString(a,"kind");check(kind in listOf("source","unnamed","generated","detached")){"Unsupported private source kind."};val label=recoveryString(a,if(kind in listOf("source","generated"))"path" else "name");check(label.isNotEmpty()&&'\u0000' !in label){"Malformed private source address."};nativeReviewTextBytes(label,16384,"Private source address")
  val address=EditorRecoveryAddress(kind,label,if(kind=="generated")recoveryString(a,"owner") else null,if(kind=="detached"&&!a.isNull("original_path"))recoveryString(a,"original_path") else null)
  nativeReviewTextBytes(address.json().toString(),32768,"Private source metadata");val text=recoveryString(b,"text");bytes+=nativeReviewTextBytes(text,32L*1024*1024,"Private source");val baseline=if(b.isNull("baseline"))null else b.getJSONObject("baseline").let{base->val saved=recoveryString(base,"text");bytes+=nativeReviewTextBytes(saved,32L*1024*1024,"Private baseline");val guard=if(base.isNull("save_guard"))null else recoveryString(base,"save_guard");if(guard!=null)nativeReviewTextBytes(guard,16384,"Private baseline guard");EditorRecoveryBaseline(saved,guard)};check(bytes<=32L*1024*1024){"Private text and baselines exceed32MiB."}
  val v=b.getJSONObject("view");val selected=v.getJSONObject("selection");val anchor=recoveryUInt(selected,"anchor");val head=recoveryUInt(selected,"head");fun scalar(n:Long)=n<=text.length&&!(n>0&&n<text.length&&text[n.toInt()-1].isHighSurrogate()&&text[n.toInt()].isLowSurrogate())
  check(scalar(anchor)&&scalar(head)){"Private selection splits a Unicode scalar."};check(v.get("scroll_x") is Number&&v.get("scroll_y") is Number){"Malformed private viewport coordinates."};val x=v.getDouble("scroll_x");val y=v.getDouble("scroll_y");check(listOf(x,y).all{it.isFinite()&&it in 0.0..1e9}){"Invalid private recovery viewport."};EditorRecoveryBuffer(id,address,text,baseline,EditorRecoveryViewport(anchor.toInt(),head.toInt(),x,y))
 }
 val layoutValue=raw.get("layout");val layout=if(layoutValue is String)JSONObject.quote(layoutValue) else layoutValue.toString();nativeReviewTextBytes(layout,512L*1024,"Private pane layout");return EditorRecoverySnapshot(host,java.util.Collections.unmodifiableList(buffers),layout)
}
private fun editorRecoveryCatalogue(raw:JSONObject,runtime:String,namespace:String):EditorRecoveryCatalogue {
 check(raw.get("version")==1&&raw.get("recovery_protocol")==1&&recoveryString(raw,"engine_workspace_id")==runtime&&recoveryString(raw,"namespace")==namespace&&!recoveryBool(raw,"source_effects")){"Foreign or unsupported private recovery catalogue."};val workspace=recoveryString(raw,"workspace_id");val slot=recoveryString(raw,"window_slot");val current=recoveryString(raw,"current_window_slot");check(recoveryHex(workspace,64)&&recoverySlot(slot)&&recoverySlot(current)){"Malformed private workspace or window identity."};val slots=recoveryStrings(raw.getJSONArray("window_slots"));check(slots.all(::recoverySlot)){"Invalid retained window slot."};val rows=raw.getJSONArray("sessions");check(rows.length()<=32){"Private session page exceeds its limit."}
 val sessions=(0 until rows.length()).map{index->val row=rows.getJSONObject(index);val client=recoveryString(row,"client_id");val hash=recoveryString(row,"snapshot_hash");val request=recoveryString(row,"request_id");val physical=recoveryString(row,"workspace_id");val host=recoveryString(row,"host");val revision=recoveryUInt(row,"revision");val count=recoveryUInt(row,"buffers");check(recoveryHex(client,32)&&recoveryHex(hash,64)&&recoveryHex(request,32)&&recoveryHex(physical,64)&&revision>0&&count<=256&&host in listOf("browser","macos","ios","android")&&recoveryString(row,"window_slot")==slot&&!recoveryBool(row,"source_effects")){"Malformed private session identity."};EditorRecoverySession(client,slot,physical,recoveryString(row,"namespace"),revision,request,hash,host,count.toInt())}
 return EditorRecoveryCatalogue(runtime,workspace,namespace,slot,current,slots,sessions,recoveryStrings(raw.getJSONArray("notices")))
}
</ui:copy>

## Prepare complete source and layout copies before adoption

Preserve every existing TextFieldState and its Undo. A recovered source gets a new unnamed identity, original text/view and no save guard; generated text stays read-only. Validate all pane nodes before importing any buffer. Existing utility instances remain in place. Captured layouts from another host recover text in a new source group without claiming that host's opaque pane representation was understood.

<ui:copy id="editor-recovery-android-adapter">
private fun editorRecoveryRoster(w:Workspace)=(w.buffers.values+w.retainedResourceEditors.values+w.retainedResourceTabs.values).distinct().sortedBy{it.debugIdentity}
fun captureEditorRecovery(w:Workspace):EditorRecoverySnapshot {
 val rows=editorRecoveryRoster(w);check(rows.size<=256){"Private recovery exceeds256buffers; readable text is unaffected."}
 val buffers=rows.map{b->check(b.state.composition==null&&!b.saving){"Finish input or saving before the next private checkpoint."};val named=w.buffers[b.file.path]===b&&!b.resourceDeleted&&!b.isUntitledDraft;val address=when{b.file.generatedBy!=null->EditorRecoveryAddress("generated",b.file.path,b.file.generatedBy);named->EditorRecoveryAddress("source",b.file.path);b.isUntitledDraft->EditorRecoveryAddress("unnamed",b.file.name);else->EditorRecoveryAddress("detached",b.file.name,original=b.file.path)};val baseline=if(b.isUntitledDraft||b.resourceDeleted||b.readOnly)b.recoveryBaseline ?: EditorRecoveryBaseline(b.base,b.saveGuard) else EditorRecoveryBaseline(b.base,b.saveGuard);EditorRecoveryBuffer(b.debugIdentity.replace("-",""),address,b.text,baseline,EditorRecoveryViewport(b.state.selection.start,b.state.selection.end,0.0,b.sourceScroll.value.toDouble()))}
 val nodes=JSONArray(w.layout.nodes.values.sortedBy{it.id}.map{n->JSONObject().put("id",n.id).put("tabs",JSONArray(n.tabs)).put("active",n.active ?: JSONObject.NULL).put("axis",n.axis ?: JSONObject.NULL).put("first",n.first ?: JSONObject.NULL).put("second",n.second ?: JSONObject.NULL).put("ratio",n.ratio)})
 val tabs=JSONArray(w.layout.groups().flatMap{g->g.tabs.mapNotNull{address->w.bufferForTab(address)?.let{JSONObject().put("address",address).put("id",it.debugIdentity.replace("-",""))}}});val layout=JSONObject().put("root",w.layout.rootID).put("focused",w.focused).put("nodes",nodes).put("tabs",tabs)
 return editorRecoverySnapshot(EditorRecoverySnapshot("android",buffers,layout.toString()).json())
}
fun editorRecoveryLease(w:Workspace):()->Boolean {
 val namespace=w.captureRetainedSourceNamespace();val server=w.engine;val runtime=server?.sourceWorkspaceID;val root=server?.sourceRootAddress;val layout=w.layout;val focus=w.focused;val rows=editorRecoveryRoster(w).map{Triple(it,it.state.selection,it.sourceScroll.value)}
 return {w.engine===server&&server?.sourceWorkspaceID==runtime&&server?.sourceRootAddress==root&&namespace()&&w.layout===layout&&w.focused==focus&&rows.all{(b,selection,scroll)->b.state.selection==selection&&b.sourceScroll.value==scroll&&b.state.composition==null}}
}
fun restoreEditorRecovery(snapshot:EditorRecoverySnapshot,w:Workspace){
 nativeReviewMaterialBudget(snapshot.layout,512*1024,"Private pane layout");val checked=editorRecoverySnapshot(snapshot.json());val prefix="recovered-"+recoveryToken();val added=linkedMapOf<String,DocumentBuffer>();val paths=linkedMapOf<String,String>()
 for(row in checked.buffers){val path=prefix+"/"+row.id;check(w.buffers[path]==null){"Recovered draft address was reused."};val file=FileNode("Recovered "+row.address.label,path,false,emptyList(),null,if(row.address.kind=="generated")row.address.owner else null);val b=DocumentBuffer(file,FileText(row.text,""),untitledDraft=row.address.kind!="generated",initialScroll=row.view.y.toInt());b.base="";b.recoveryBaseline=row.baseline;b.state.edit{this.selection=TextRange(row.view.anchor,row.view.head)};b.saveGuardReason="Private session recovery grants no save authority. Use guarded Save As to publish this draft.";added[path]=b;paths[row.id]=path}
 var recovered=PaneLayout(prefix+"-sources",mapOf(prefix+"-sources" to PaneNode(prefix+"-sources",tabs=added.keys.map(::sourceTab),active=added.keys.firstOrNull()?.let(::sourceTab))));var focus=recovered.rootID
 if(checked.host=="android"){
  val saved=JSONObject(checked.layout);val rawNodes=saved.getJSONArray("nodes");check(rawNodes.length()<=512){"Captured pane tree exceeds its budget."};val source=linkedMapOf<String,JSONObject>();for(i in 0 until rawNodes.length()){val node=rawNodes.getJSONObject(i);val id=recoveryString(node,"id");check(source.put(id,node)==null){"Duplicate captured pane identity."}}
  val placements=linkedMapOf<String,String>();val rawTabs=saved.getJSONArray("tabs");check(rawTabs.length()<=512){"Captured pane placements exceed their budget."};for(i in 0 until rawTabs.length()){val row=rawTabs.getJSONObject(i);val address=recoveryString(row,"address");val id=recoveryString(row,"id");check(id in paths&&placements.put(address,id)==null){"Ambiguous private source placement."}}
  val nodes=linkedMapOf<String,PaneNode>();val seen=mutableSetOf<String>();val placed=mutableSetOf<String>()
  fun project(id:String,depth:Int=0):String {check(depth<=32&&seen.add(id)){"Malformed captured pane tree."};val raw=source[id] ?: error("Missing captured pane node.");val next=prefix+"-"+id
   if(!raw.isNull("first")&&!raw.isNull("second")){val axis=recoveryString(raw,"axis");val ratio=raw.getDouble("ratio");check(axis in listOf("horizontal","vertical")&&ratio.isFinite()&&ratio in 0.1..0.9&&raw.getJSONArray("tabs").length()==0){"Malformed private split proportions."};nodes[next]=PaneNode(next,axis=axis,first=project(recoveryString(raw,"first"),depth+1),second=project(recoveryString(raw,"second"),depth+1),ratio=ratio)}
   else{check(raw.isNull("first")&&raw.isNull("second")&&raw.isNull("axis")){"Malformed captured pane group."};val tabs=recoveryStrings(raw.getJSONArray("tabs")).mapNotNull{address->placements[address]?.let{id->check(placed.add(id)){"One recovered editor occupies multiple pane addresses."};sourceTab(paths.getValue(id))}};val active=if(raw.isNull("active"))null else placements[recoveryString(raw,"active")]?.let{sourceTab(paths.getValue(it))};nodes[next]=PaneNode(next,tabs=tabs,active=active ?: tabs.firstOrNull())};return next
  }
  val root=project(recoveryString(saved,"root"));recovered=PaneLayout(root,nodes);val missing=paths.filterKeys{it !in placed}.values.map(::sourceTab)
  if(missing.isNotEmpty()){val group=prefix+"-unplaced";val root=prefix+"-unplaced-split";recovered=recovered.copy(rootID=root,nodes=recovered.nodes+mapOf(group to PaneNode(group,tabs=missing,active=missing.first()),root to PaneNode(root,axis="horizontal",first=recovered.rootID,second=group,ratio=0.75)))}
  val desired=prefix+"-"+recoveryString(saved,"focused");focus=recovered.groups().firstOrNull{it.id==desired&&it.tabs.isNotEmpty()}?.id ?: recovered.groups().firstOrNull{it.tabs.isNotEmpty()}?.id ?: recovered.rootID
 }
 check(recovered.nodes.keys.none{it in w.layout.nodes}){"Recovered layout address collision."};val root=prefix+"-session";val next=PaneLayout(root,w.layout.nodes+recovered.nodes+(root to PaneNode(root,axis="horizontal",first=w.layout.rootID,second=recovered.rootID,ratio=0.5)))
 // No owned namespace changes precede this point.
 for((path,b)in added)w.buffers[path]=b;w.layout=next;w.focused=focus
}
</ui:copy>

## Record exact requests and retire stale reviews

Recording has its own busy state; review never blocks source input. Cancellation after POST retains the original request. A later checkpoint retries that request before recording newer text. Review admits complete sealed material and then holds the original host lease until explicit restoration.

<ui:copy id="editor-recovery-android-controller">
class EditorRecoveryModel(private val transport:(suspend(Engine,String,String,JSONObject?)->JSONObject)?=null){
 var showing by mutableStateOf(false)
 var busy by mutableStateOf(false);private set
 var notice by mutableStateOf("");private set
 var catalogue by mutableStateOf<EditorRecoveryCatalogue?>(null);private set
 var snapshot by mutableStateOf<EditorRecoverySnapshot?>(null);private set
 var revision by mutableLongStateOf(0);private set
 private var client=recoveryToken();private var bound:Engine?=null;private var runtime="";private var namespace="";private var last:String?=null;private var writing=false;private var ticket=0L;private var current:()->Boolean={false}
 private data class Pending(val body:String,val physical:String,val namespace:String,val slot:String,val request:String,val base:Long,val snapshot:String)
 private var pending:Pending?=null
 private suspend fun call(e:Engine,method:String,path:String,body:JSONObject?=null)=transport?.invoke(e,method,path,body) ?: e.request(method,path,body)
 private suspend fun list(w:Workspace,slot:String?=null):EditorRecoveryCatalogue {val e=w.engine ?: error("Connect before private session recovery.");val runtime=e.sourceWorkspaceID ?: error("Refresh Files before private recovery.");val root=e.sourceRootAddress ?: error("Refresh Files before private recovery.");val raw=call(e,"GET","/api/workspace/recovery"+(slot?.let{"?window_slot=$it"} ?: ""));check(w.engine===e&&e.sourceWorkspaceID==runtime&&e.sourceRootAddress==root){"Private recovery connection changed."};return withContext(Dispatchers.Default){editorRecoveryCatalogue(raw,runtime,root)}}
 suspend fun captureLoop(w:Workspace){while(currentCoroutineContext().isActive){checkpoint(w);delay(1500)}}
 suspend fun checkpoint(w:Workspace){val e=w.engine ?: return;val runtime=e.sourceWorkspaceID ?: return;val root=e.sourceRootAddress ?: return;if(writing)return;writing=true
  try{if(bound!==e||this.runtime!=runtime||namespace!=root){bound=e;this.runtime=runtime;namespace=root;client=recoveryToken();revision=0;last=null;pending=null};val captured=captureEditorRecovery(w).json().toString();if(pending==null&&captured==last)return
   if(pending==null){val meta=list(w);check(w.engine===e&&e.sourceWorkspaceID==runtime&&e.sourceRootAddress==root){"Private checkpoint connection changed."};val request=recoveryToken();val body=JSONObject().put("version",1).put("workspace",runtime).put("window_slot",meta.currentSlot).put("client_id",client).put("request_id",request).put("base_revision",revision).put("snapshot",JSONObject(captured));pending=Pending(body.toString(),meta.workspace,root,meta.currentSlot,request,revision,captured)}
   val held=pending!!;check(w.engine===e&&e.sourceWorkspaceID==runtime&&e.sourceRootAddress==root){"Private checkpoint connection changed."};val raw=call(e,"POST","/api/workspace/recovery/checkpoint",JSONObject(held.body));check(raw.get("version")==1&&raw.get("recovery_protocol")==1&&recoveryString(raw,"client_id")==client&&recoveryString(raw,"window_slot")==held.slot&&recoveryString(raw,"workspace_id")==held.physical&&recoveryString(raw,"namespace")==held.namespace&&recoveryString(raw,"request_id")==held.request&&held.base<4294967295L&&recoveryUInt(raw,"revision")==held.base+1&&recoveryHex(recoveryString(raw,"snapshot_hash"),64)&&!recoveryBool(raw,"source_effects")&&!recoveryBool(raw,"source_saved")){"Private checkpoint acknowledgment changed. Exact request retained."};revision=held.base+1;last=held.snapshot;pending=null;notice="Private editor checkpoint retained · source files unchanged."
  }catch(e:CancellationException){throw e}catch(e:Exception){notice=e.message.orEmpty()+" Readable drafts remain available; no source save was claimed."}finally{writing=false}
 }
 fun close(){ticket++;showing=false;busy=false;snapshot=null;current={false}}
 suspend fun open(w:Workspace,slot:String?=null){showing=true;val id=++ticket;busy=true;snapshot=null;current={false};try{val value=list(w,slot);if(id==ticket){catalogue=value;notice="Preview a private session. Recovery adds draft copies without replacing open editors or saving source."}}catch(e:CancellationException){throw e}catch(e:Exception){if(id==ticket)notice=e.message.orEmpty()}finally{if(id==ticket)busy=false}}
 suspend fun read(row:EditorRecoverySession,w:Workspace){if(busy)return;val e=w.engine ?: return;val runtime=e.sourceWorkspaceID ?: return;val lease=editorRecoveryLease(w);val id=++ticket;busy=true;snapshot=null;current={false}
  try{val raw=call(e,"GET","/api/workspace/recovery/${row.client}?revision=${row.revision}&window_slot=${row.slot}");check(raw.get("version")==1&&raw.get("recovery_protocol")==1&&recoveryString(raw,"engine_workspace_id")==runtime&&recoveryString(raw,"client_id")==row.client&&recoveryString(raw,"window_slot")==row.slot&&recoveryString(raw,"workspace_id")==row.workspace&&recoveryString(raw,"namespace")==row.namespace&&recoveryUInt(raw,"revision")==row.revision&&recoveryString(raw,"request_id")==row.request&&recoveryString(raw,"snapshot_hash")==row.hash&&!recoveryBool(raw,"source_effects")&&!recoveryBool(raw,"source_saved")){"Complete private snapshot receipt changed."};val material=recoveryString(raw,"material")
   val parsed=withContext(Dispatchers.Default){nativeReviewMaterialBudget(material,64*1024*1024,"Private recovery");check(debugHash(material)==row.hash){"Complete private snapshot fingerprint changed."};editorRecoverySnapshot(nativeReviewProtocolJSON(material,64*1024*1024,"Private snapshot",emptySet()))};val authority=raw.getJSONArray("authority");check(parsed.host==row.host&&parsed.buffers.size==row.count&&authority.length()==parsed.buffers.size&&parsed.buffers.all{b->(0 until authority.length()).count{val a=authority.getJSONObject(it);recoveryString(a,"buffer_id")==b.id&&!recoveryBool(a,"write_permission")&&a.get("baseline_current") is Boolean}==1}){"Private source observation changed or claimed write permission."};if(id!=ticket)return;check(w.engine===e&&e.sourceWorkspaceID==runtime&&lease()){"Host editor lease changed. Preview again."};snapshot=parsed;current=lease;notice="Complete checkpoint fingerprint verified. Recovery grants no source write permission."
  }catch(e:CancellationException){throw e}catch(e:Exception){if(id==ticket)notice=e.message.orEmpty()}finally{if(id==ticket)busy=false}
 }
 val canRestore get()=!busy&&snapshot!=null&&current()
 fun restore(w:Workspace):Boolean{val value=snapshot;if(!canRestore||value==null){notice="Host editor lease changed. Preview again; open drafts were preserved.";return false};return try{restoreEditorRecovery(value,w);snapshot=null;current={false};notice="Recovered draft copies and layout. Existing editors and saved source were retained.";true}catch(e:Exception){notice=e.message.orEmpty();false}}
}
</ui:copy>

<ui:copy id="editor-recovery-android-view">
@Composable fun EditorRecoveryView(w:Workspace){val m=w.editorRecovery;val scope=rememberCoroutineScope();var index by remember{mutableIntStateOf(0)};var page by remember{mutableIntStateOf(0)}
 LaunchedEffect(m){if(!m.busy)m.open(w)}
 Column(Modifier.padding(12.dp).widthIn(max=680.dp).heightIn(max=640.dp).verticalScroll(rememberScrollState()),verticalArrangement=Arrangement.spacedBy(8.dp)){
  Text("Editor session recovery");Text(m.notice,fontSize=12.sp);Text("Private checkpoints are draft recovery, not source saves. Recovery adds copies beside existing editors; utility panes stay in place.",fontSize=12.sp)
  TextButton(enabled=!m.busy,onClick={scope.launch{m.open(w,m.catalogue?.slot)}}){Text("Refresh sessions")}
  m.catalogue?.let{catalogue->Row{catalogue.slots.forEach{slot->TextButton(enabled=!m.busy,onClick={scope.launch{m.open(w,slot)}}){Text("Window $slot")}}};catalogue.notices.forEach{Text(it,fontSize=12.sp)};catalogue.sessions.forEach{row->TextButton(enabled=!m.busy,onClick={index=0;page=0;scope.launch{m.read(row,w)}}){Text("Read ${row.host} · ${row.count} buffers · ${row.client} · revision ${row.revision}")}}}
  m.snapshot?.let{snapshot->if(snapshot.buffers.isNotEmpty()){snapshot.buffers.forEachIndexed{i,b->TextButton(onClick={index=i;page=0}){Text("${b.address.kind} · ${b.address.label}")}};val row=snapshot.buffers[index.coerceIn(snapshot.buffers.indices)];val total=row.text.codePointCount(0,row.text.length);val count=((total+15999)/16000).coerceAtLeast(1);val shown=page.coerceIn(0,count-1);val start=row.text.offsetByCodePoints(0,shown*16000);val end=row.text.offsetByCodePoints(start,(total-shown*16000).coerceAtMost(16000));Text(row.text.substring(start,end),fontSize=12.sp);Text("Text page ${shown+1}/$count · ${if(row.address.kind=="generated")"read-only generated text" else "unnamed draft; guarded Save As required"}",fontSize=12.sp);Row{TextButton(enabled=shown>0,onClick={page=shown-1}){Text("Previous text")};TextButton(enabled=shown+1<count,onClick={page=shown+1}){Text("Next text")}};TextButton(enabled=m.canRestore,onClick={m.restore(w)}){Text("Recover complete session as draft copies")}}}
  TextButton(onClick={m.close();w.currentBuffer?.focusSource?.invoke()}){Text("Close")}
 }
}
</ui:copy>

## Check exact retries, immutable readings and complete adoption

Focused fixtures bind engine metadata through the ordinary Files API and keep provider work absent. The opted-in case below uses two fresh test processes against the owned real engine. It verifies retained native text state and scroll; it does not claim rendered device input or accessibility acceptance.

<ui:copy id="editor-recovery-android-tests">
@file:OptIn(androidx.compose.foundation.ExperimentalFoundationApi::class)
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
import org.json.JSONArray
import kotlinx.coroutines.*
import androidx.compose.ui.text.TextRange
class EditorRecoveryTest {
 private fun buffer(path:String,text:String="draft 😀",unnamed:Boolean=false,owner:String?=null)=DocumentBuffer(FileNode(path,path,false,emptyList(),null,owner),FileText(text,"old","original"),untitledDraft=unnamed)
 private fun meta()=JSONObject().put("version",1).put("recovery_protocol",1).put("engine_workspace_id","e".repeat(32)).put("workspace_id","a".repeat(64)).put("namespace","/owned").put("window_slot","0").put("current_window_slot","0").put("window_slots",JSONArray().put("0")).put("sessions",JSONArray()).put("notices",JSONArray()).put("source_effects",false)
 private suspend fun host(block:suspend(Workspace,DocumentBuffer)->Unit)=coroutineScope {
  val server=com.sun.net.httpserver.HttpServer.create(java.net.InetSocketAddress("127.0.0.1",0),0);server.createContext("/api/files"){exchange->val bytes=JSONObject().put("root","Owned").put("root_path","/owned").put("workspace_id","e".repeat(32)).put("tree",JSONArray()).toString().toByteArray();exchange.sendResponseHeaders(200,bytes.size.toLong());exchange.responseBody.use{it.write(bytes)}};server.start()
  val w=Workspace(this);w.engine=Engine("http://127.0.0.1:${server.address.port}");w.engine!!.files();val b=buffer("files");w.buffers[b.file.path]=b;w.show(sourceTab(b.file.path));try{block(w,b)}finally{server.stop(0)}
 }
 @Test fun rawReviewRefusesDuplicateAliasedVersionsAndExcessiveNesting(){for(raw in listOf("{\"version\":1,\"version\":1}","{\"version\":1,\"recovery_protocol\":1.0}","[".repeat(65)+"0"+"]".repeat(65))){assertTrue(runCatching{nativeReviewProtocolJSON(raw,1024,"Recovery",setOf("version","recovery_protocol"))}.isFailure)}}
 @Test fun captureKeepsReservedLiteralUnnamedAndGeneratedSourcesDistinct()=runBlocking {
  val w=Workspace(this);for(path in listOf("files","agent","terminal","Untitled-real.txt","é.txt","e\u0301.txt")){val b=buffer(path);w.buffers[path]=b;w.layout=w.layout.activate(sourceTab(path),"document-group")};val note=buffer("note","anonymous",true);w.buffers[note.file.path]=note;val generated=buffer("out.py","output",owner="project.md");w.buffers[generated.file.path]=generated
  val snapshot=captureEditorRecovery(w);assertEquals(8,snapshot.buffers.size);assertEquals(6,snapshot.buffers.count{it.address.kind=="source"});assertEquals(1,snapshot.buffers.count{it.address.kind=="unnamed"});assertEquals(1,snapshot.buffers.count{it.address.kind=="generated"});assertTrue(snapshot.buffers.all{it.baseline?.guard=="original"})
 }
 @Test fun coldCopiesKeepGeneratedTextReadonlyAndExistingTextStateUndo()=runBlocking {
  val old=Workspace(this);val source=buffer("files");val output=buffer("out.py","generated",owner="project.md");old.buffers[source.file.path]=source;old.buffers[output.file.path]=output;old.show(sourceTab("files"));val snapshot=captureEditorRecovery(old)
  val fresh=Workspace(this);val independent=buffer("files","initial");fresh.buffers["files"]=independent;fresh.show(sourceTab("files"));independent.applySourceEdits(listOf(NativeSourceEdit(0,independent.text.length,"independent draft")),independent.text);val state=independent.state;val selection=state.selection;restoreEditorRecovery(snapshot,fresh)
  assertSame(independent,fresh.buffers["files"]);assertSame(state,independent.state);assertEquals(selection,state.selection);assertEquals("independent draft",independent.text);val recovered=fresh.buffers.values.filter{it!==independent};assertEquals(2,recovered.size);assertTrue(recovered.all{it.saveGuard==null&&it.recoveryBaseline?.guard=="original"});assertEquals(1,recovered.count{it.readOnly});assertEquals(1,recovered.count{it.isUntitledDraft});assertTrue(independent.snippets.undoSource());assertEquals("initial",independent.text)
 }
 @Test fun malformedLayoutAndScalarSelectionNeverImportPartialSources()=runBlocking {
  val w=Workspace(this);val b=buffer("files");w.buffers[b.file.path]=b;val snapshot=captureEditorRecovery(w);val target=Workspace(this);assertTrue(runCatching{restoreEditorRecovery(snapshot.copy(layout="{}"),target)}.isFailure);assertTrue(target.buffers.isEmpty());val raw=snapshot.json();raw.getJSONArray("buffers").getJSONObject(0).getJSONObject("view").getJSONObject("selection").put("head",7);assertTrue(runCatching{editorRecoverySnapshot(raw)}.isFailure)
  for(value in listOf("[".repeat(65)+"0"+"]".repeat(65),"\uD800")){assertTrue(runCatching{nativeReviewMaterialBudget(value,1024,"Fixture")}.isFailure)}
 }
 @Test fun fullLeaseRetiresSelectionScrollAndSameSizedRetainedOwnerReplacement()=runBlocking {
  val w=Workspace(this);val b=buffer("files");w.buffers[b.file.path]=b;val note=buffer("note","old",true);w.retainedResourceEditors["note"]=note;val first=editorRecoveryLease(w);assertTrue(first());b.state.edit{this.selection=TextRange(1)};assertFalse(first());val second=editorRecoveryLease(w);b.sourceScroll.scrollTo(20);assertFalse(second());val third=editorRecoveryLease(w);w.retainedResourceEditors["note"]=buffer("note","old",true);assertFalse(third())
 }
 @Test fun uncertainCheckpointRetainsExactOriginalRequestBeforeNewerCapture()=runBlocking {host{w,b->
  val requests=mutableListOf<String>();var lose=true;val model=EditorRecoveryModel{_,method,_,body->if(method=="GET")meta() else {val value=body!!;requests.add(value.toString());if(lose){lose=false;error("Lost response")};JSONObject().put("version",1).put("recovery_protocol",1).put("client_id",value.getString("client_id")).put("window_slot","0").put("workspace_id","a".repeat(64)).put("namespace","/owned").put("request_id",value.getString("request_id")).put("revision",value.getLong("base_revision")+1).put("snapshot_hash","f".repeat(64)).put("source_effects",false).put("source_saved",false)}}
  model.checkpoint(w);b.applySourceEdits(listOf(NativeSourceEdit(0,b.text.length,"newer draft")),b.text);model.checkpoint(w);assertEquals(requests[0],requests[1]);assertEquals(1L,model.revision);model.checkpoint(w);assertEquals(2L,model.revision);assertNotEquals(requests[1],requests[2])
 }}
 @Test fun sealedReadRefusesChangedBytesAndSourceWriteClaims()=runBlocking {for(mode in listOf("valid","material","permission","source_saved"))host{w,b->
  val value=captureEditorRecovery(w);val material=value.json().toString();val hash=debugHash(material);val row=EditorRecoverySession("c".repeat(32),"0","a".repeat(64),"/owned",1,"d".repeat(32),hash,"android",1)
  val envelope=JSONObject().put("version",1).put("recovery_protocol",1).put("engine_workspace_id","e".repeat(32)).put("client_id",row.client).put("window_slot","0").put("workspace_id",row.workspace).put("namespace","/owned").put("revision",1).put("request_id",row.request).put("snapshot_hash",hash).put("material",material+if(mode=="material")" " else "").put("source_effects",false).put("source_saved",mode=="source_saved").put("authority",JSONArray().put(JSONObject().put("buffer_id",value.buffers.single().id).put("baseline_current",true).put("write_permission",mode=="permission")))
  val model=EditorRecoveryModel{_,_,_,_->envelope};model.read(row,w);if(mode=="valid"){assertTrue(model.notice,model.canRestore);b.advanceSourceRevision();assertFalse(model.restore(w))}else assertNull(mode,model.snapshot);assertEquals("draft 😀",b.text);assertEquals(1,w.buffers.size)
 }}
 @Test(timeout=120000) fun liveOwnedProcessRestartRecovery()=runBlocking {
  val origin=System.getenv("HICKORY_NATIVE_SESSION_ORIGIN");org.junit.Assume.assumeNotNull(origin);val root=java.io.File(System.getenv("HICKORY_NATIVE_SESSION_WORKSPACE") ?: error("Use the owned session fixture"));val phase=System.getenv("HICKORY_NATIVE_SESSION_PHASE") ?: error("Select capture or restore");check(root.resolve(".hickory-native-session-fixture").readText()=="owned-native-session-v1\n"){"Refuse non-owned session fixture"};println("NATIVE_SESSION_PROCESS_ID="+ProcessHandle.current().pid())
  val background=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined);val w=Workspace(background);val e=Engine(origin!!);w.engine=e;val (_,tree)=e.files();w.rootFiles=tree
  fun find(path:String,nodes:List<FileNode>):FileNode?{for(node in nodes){if(node.path==path)return node;find(path,node.children)?.let{return it}};return null}
  val b=w.loadBuffer(find(if(phase=="capture")"files" else "note.py",tree) ?: error("Missing fixture source"));w.show(sourceTab(b.file.path));val state=b.state;val model=w.editorRecovery;var loop:Job?=null
  try{
   if(phase=="capture"){
    assertTrue(b.applySourceEdits(listOf(NativeSourceEdit(b.text.length,b.text.length,"\nunsaved λ😀")),b.text));b.state.edit{this.selection=TextRange(9,12)};b.sourceScroll.scrollTo(180);val note=buffer("anonymous","anonymous retained note",true);w.buffers[note.file.path]=note;val source=b.text;loop=background.launch{model.captureLoop(w)}
    withTimeout(15000){while(model.revision==0L)delay(20)};model.open(w);val row=model.catalogue!!.sessions.first{it.host=="android"&&it.count==2};model.read(row,w);val saved=model.snapshot!!.buffers.first{it.address.kind=="source"&&it.address.label=="files"};assertEquals(source,saved.text);assertEquals(9,saved.view.anchor);assertEquals(12,saved.view.head);assertEquals(180.0,saved.view.y,0.0);assertTrue(model.snapshot!!.buffers.any{it.address.kind=="unnamed"&&it.text=="anonymous retained note"});assertFalse(root.resolve("files").readText().contains("unsaved"));return@runBlocking
   }
   assertEquals("restore",phase);assertTrue(b.applySourceEdits(listOf(NativeSourceEdit(b.text.length,b.text.length,"# independent new draft")),b.text));b.state.edit{this.selection=TextRange(2,4)};val source=b.text;val selection=state.selection;val scroll=b.sourceScroll.value
   val commands=commandDescriptors(javaClass.classLoader!!.getResourceAsStream("commands.json")!!.bufferedReader().use{it.readText()});val action=w.commands(commands).first{it.id=="workspace.recovery"};assertNull(action.reason);action.run();assertTrue(w.showRecovery);w.showRecovery=false;model.open(w);val row=model.catalogue!!.sessions.first{it.host=="android"&&it.count==2};model.read(row,w);val saved=model.snapshot!!.buffers.first{it.address.kind=="source"&&it.address.label=="files"};assertTrue(model.notice,model.canRestore)
   b.state.edit{this.selection=TextRange(3)};assertFalse(model.restore(w));assertEquals(1,w.buffers.size);b.state.edit{this.selection=selection};model.read(row,w);assertTrue(model.notice,model.restore(w));val recovered=w.buffers.values.single{it.isUntitledDraft&&it.text==saved.text};assertNull(recovered.saveGuard);assertSame(recovered,w.currentBuffer);assertEquals(TextRange(saved.view.anchor,saved.view.head),recovered.state.selection);assertEquals(saved.view.y.toInt(),recovered.sourceScroll.value);assertTrue(w.buffers.values.any{it.isUntitledDraft&&it.text=="anonymous retained note"})
   assertSame(b,w.buffers["note.py"]);assertSame(state,b.state);assertEquals(source,b.text);assertEquals(selection,state.selection);assertEquals(scroll,b.sourceScroll.value);assertTrue(b.snippets.undoSource());assertEquals("value=1\n",b.text);assertFalse(root.resolve("files").readText().contains("unsaved"));assertEquals("value=1\n",root.resolve("note.py").readText())
  }finally{loop?.cancelAndJoin();model.close();w.buffers.values.forEach{it.language?.close()};background.cancel()}
 }
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/EditorRecovery.kt"><ui:paste select="#editor-recovery-android-codec"/><ui:paste select="#editor-recovery-android-adapter"/><ui:paste select="#editor-recovery-android-controller"/><ui:paste select="#editor-recovery-android-view"/></ui:file>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/EditorRecoveryTest.kt"><ui:paste select="#editor-recovery-android-tests"/></ui:file>
</ui:doc>
