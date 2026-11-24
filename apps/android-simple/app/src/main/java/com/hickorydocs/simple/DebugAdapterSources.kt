package com.hickorydocs.simple
import androidx.compose.runtime.*
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.material3.*
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.*
import org.json.JSONObject
import java.util.UUID

data class NativeAdapterSourceBinding(val scope:NativeDebugScope,val reference:Long){init{require(reference in 1..Int.MAX_VALUE.toLong())}}
data class NativeAdapterSourceFacts(val hash:String,val scalars:Int,val starts:List<Int>)
internal fun nativeAdapterSourceFacts(content:String,budget:Int=8*1024*1024):NativeAdapterSourceFacts{
 require(budget>=0&&content.length<=budget){"Adapter source exceeds its UTF8 reading budget."};var offset=0;var scalars=0;var bytes=0L;val starts=mutableListOf(0)
 while(offset<content.length){val char=content[offset];val cost=when{char.isHighSurrogate()->{require(offset+1<content.length&&content[offset+1].isLowSurrogate()){"Adapter source contains an unpaired surrogate."};offset+=2;4};char.isLowSurrogate()->error("Adapter source contains an unpaired surrogate.");else->{offset++;when{char.code<=0x7f->1;char.code<=0x7ff->2;else->3}}};bytes+=cost;require(bytes<=budget){"Adapter source exceeds its UTF8 reading budget."};scalars++;if(scalars%4000==0&&offset<content.length)starts.add(offset)}
 return NativeAdapterSourceFacts(debugHash(content),scalars,starts.toList())
}
data class NativeAdapterSourceReading(val binding:NativeAdapterSourceBinding,val request:String,val content:String,val hash:String,val mime:String?,val facts:NativeAdapterSourceFacts){
 fun page(index:Int):String{require(index in facts.starts.indices);return content.substring(facts.starts[index],facts.starts.getOrNull(index+1) ?: content.length)}
}
class NativeAdapterSourceModel(private val validate:suspend(String,String?)->NativeAdapterSourceFacts={text,mime->withContext(Dispatchers.Default){if(mime!=null)nativeAdapterSourceFacts(mime,256);nativeAdapterSourceFacts(text)}}){
 private data class Pending(val binding:NativeAdapterSourceBinding,val request:String,val inspection:String,val generation:Long,var processing:Boolean=false)
 private var inspection:String?=null;private var generation=0L;private var pending:Pending?=null
 var reading by mutableStateOf<NativeAdapterSourceReading?>(null);private set
 var busy by mutableStateOf(false);private set
 var notice by mutableStateOf("");private set
 var current by mutableStateOf(false);private set
 fun open():String{retire("Previous adapter text remains a historical reading until explicitly refreshed.");return UUID.randomUUID().toString().also{inspection=it}}
 fun close(token:String){if(inspection==token){inspection=null;retire("Adapter-source inspector closed. Captured text remains historical.")}}
 fun retire(reason:String="Paused source observation changed; captured adapter text is historical."){generation++;pending=null;busy=false;current=false;if(reading!=null||notice.isNotEmpty())notice=reason}
 fun cancel(token:String){if(inspection==token)retire("Pending source reading cancelled locally. Late replies cannot replace captured text.")}
 fun begin(binding:NativeAdapterSourceBinding?,capability:Any?,token:String):JSONObject?{
  if(token!=inspection){notice="Open the current debugger inspector before reading adapter source.";return null}
  if(capability!=true){notice="This engine/provider does not advertise adapter source content. No workspace text was substituted.";return null}
  if(binding==null){notice="Choose a current paused frame with a positive adapter source reference. No workspace fallback is used.";return null}
  val request=UUID.randomUUID().toString();pending=Pending(binding,request,token,++generation);busy=true;current=false;notice="Reading the selected frame's adapter-owned text…"
  return binding.scope.put(JSONObject().put("op","source").put("reference",binding.reference).put("request_id",request))
 }
 private fun owns(value:Pending)=pending===value&&inspection==value.inspection&&generation==value.generation
 suspend fun receive(event:JSONObject,binding:()->NativeAdapterSourceBinding?):Boolean{
  val value=pending ?: return false;if(event.optString("request_id")!=value.request||!owns(value)||value.processing)return false
  if(event.optString("event")!="adapter_source"||!value.binding.scope.matches(event)||debugInteger(event.opt("reference"))!=value.binding.reference||binding()!=value.binding)return refuse(value.request,"Adapter source reply belongs to an obsolete or foreign frame. Previous text retained.")
  val content=event.opt("content") as? String ?: return refuse(value.request,"Adapter returned no literal source content.")
  val hash=event.opt("source_hash") as? String ?: return refuse(value.request,"Adapter source hash is unavailable.")
  if(content.length>8*1024*1024||!hash.matches(Regex("[a-f0-9]{64}")))return refuse(value.request,"Adapter source exceeds its reading budget or has an invalid source hash.")
  val mime=when(val raw=event.opt("mime_type")){null,JSONObject.NULL->null;is String->raw.takeIf{it.length<=256&&!it.any{char->char.isISOControl()}} ?: return refuse(value.request,"Adapter returned invalid source MIME metadata.");else->return refuse(value.request,"Adapter returned invalid source MIME metadata.")}
  value.processing=true
  val facts=try{validate(content,mime)}catch(cancelled:CancellationException){refuse(value.request,"Adapter source reading cancelled. Refresh explicitly if still paused.");throw cancelled}catch(error:Exception){return refuse(value.request,error.message ?: "Adapter source validation failed.")}
  if(!owns(value))return false
  if(binding()!=value.binding){retire("Source observation changed before confirmation. Captured text retained.");return false}
  if(facts.hash!=hash)return refuse(value.request,"Adapter source bytes do not match their source hash. Previous text retained.")
  reading=NativeAdapterSourceReading(value.binding,value.request,content,hash,mime,facts);pending=null;busy=false;current=true;notice=if(content.isEmpty())"Adapter reported empty literal source content." else "Adapter-owned source only. It grants no workspace position, edit or breakpoint authority.";return true
 }
 fun refuse(request:String,message:String):Boolean{val value=pending?.takeIf{it.request==request&&owns(it)} ?: return false;pending=null;busy=false;current=false;notice=message+" Captured text is retained; refresh explicitly.";return true}
 fun matches(binding:NativeAdapterSourceBinding?)=current&&reading?.binding==binding
}
@Composable fun NativeAdapterSourceView(debug:NativeDebugger,inspection:String?){
 val model=debug.adapterSources;val binding=debug.adapterSourceBinding();val capable=debug.capabilities.opt("source_content")==true
 if(binding!=null||model.reading!=null||model.notice.isNotEmpty())Column{
  Text("Adapter-owned source",fontSize=12.sp)
  if(binding!=null&&!capable)Text("This engine/provider does not advertise source content. Workspace or disk text is not a fallback.",fontSize=12.sp)
  WithDisabledReason(if (inspection == null) "Pause the debugger and select a frame first." else if (binding == null) "This frame is not bound to the current debugger stop." else if (!(capable)) "The current adapter does not provide this frame’s source." else if (model.busy) "An operation is already in progress." else null) { available -> TextButton(enabled = available,onClick={inspection?.let{debug.readAdapterSource(it)}}){Text("Read selected frame adapter source")} }
  if(model.busy)TextButton(onClick={inspection?.let{model.cancel(it)}}){Text("Cancel pending source reading")}
  if(model.notice.isNotEmpty())Text(model.notice,fontSize=12.sp)
  model.reading?.let{value->var page by remember(value){mutableIntStateOf(0)};val index=page.coerceIn(value.facts.starts.indices)
   Text("${if(model.matches(binding))"Current" else "Historical"} adapter text · pause ${value.binding.scope.stop} · thread ${value.binding.scope.thread} · frame ${value.binding.scope.frame} · reference ${value.binding.reference}",fontSize=11.sp)
   Text("${value.mime ?: "MIME unavailable"} · SHA256 ${value.hash}",fontSize=11.sp)
   SelectionContainer{Text(value.page(index),fontFamily=FontFamily.Monospace,fontSize=12.sp)}
   Text("Page ${index+1}/${value.facts.starts.size} · ${value.facts.scalars} Unicode scalars. Read-only literal text; no workspace mapping is implied.",fontSize=11.sp)
   Row{WithDisabledReason(if (index <= 0) "You’re on the first page." else null) { available -> TextButton(enabled = available,onClick={page=index-1}){Text("Previous adapter source page")} };WithDisabledReason(if (index >= value.facts.starts.lastIndex) "You’re on the last page." else null) { available -> TextButton(enabled = available,onClick={page=index+1}){Text("Next adapter source page")} }}
  }
 }
}
