package com.hickorydocs.simple
import kotlinx.coroutines.*
import okhttp3.*
import okio.ByteString
import okio.ByteString.Companion.toByteString
import org.json.JSONObject
import org.json.JSONArray
import java.util.UUID
class NativeLanguageConnection(private val engine:Engine,private val scope:CoroutineScope,private val openSocket:(Request,WebSocketListener)->WebSocket={request,listener->client.newWebSocket(request,listener)}) {
 private data class Subscriber(val uri:String,val receive:(JSONObject)->Unit,val failed:(Throwable)->Unit)
 private data class Document(val text:String,val version:Int)
 private data class Pending(val owner:UUID,val reply:CompletableDeferred<Any>)
 private class Effects {
  val actions=mutableListOf<()->Unit>();val failures=linkedMapOf<UUID,Pair<Subscriber,Throwable>>();val finish=mutableListOf<()->Unit>()
  fun deliver(){var error:Throwable?=null;fun perform(action:()->Unit){try{action()}catch(cause:Throwable){if(error==null)error=cause else if(error!==cause)error!!.addSuppressed(cause)}};for((subscriber,cause)in failures.values)perform{subscriber.failed(cause)};for(action in actions)perform(action);for(action in finish)perform(action);error?.let{throw it}}
 }
 private val gate=Any();private var activeEffects:Effects?=null
 private val subscribers=linkedMapOf<UUID,Subscriber>();private val documents=linkedMapOf<String,Document>();private val pending=mutableMapOf<Int,Pending>()
 private var nextID=1;private var epoch=0L;private var retryAfter:Long?=null;private var invalidating=false;private var socket:WebSocket?=null
 private fun <T> state(block:(Effects)->T):T {if(Thread.holdsLock(gate))return block(checkNotNull(activeEffects));val effects=Effects();var failure:Throwable?=null;try{return synchronized(gate){activeEffects=effects;try{block(effects)}finally{activeEffects=null}}}catch(error:Throwable){failure=error;throw error}finally{try{effects.deliver()}catch(error:Throwable){if(failure==null)throw error else if(failure!==error)failure.addSuppressed(error)}}}
 val active get()=synchronized(gate){socket!=null}
 fun attach(uri:String,source:String,version:Int,receive:(JSONObject)->Unit,failed:(Throwable)->Unit):UUID=state{effects->
  val token=UUID.randomUUID();val subscriber=Subscriber(uri,receive,failed)
  if(documents.containsKey(uri)){effects.failures[token]=subscriber to IllegalStateException("Reuse the workspace language session for this source: $uri");return@state token}
  subscribers[token]=subscriber;documents[uri]=Document(source,version)
  try{if(!startLocked(effects))sendDocumentLocked(uri,false,effects)}catch(error:Throwable){effects.failures.putIfAbsent(token,subscriber to error)};token
 }
 fun detach(token:UUID):Unit=state{effects->
  val entry=subscribers.remove(token) ?: return@state
  val abandoned=pending.filterValues{it.owner==token};for((id,request)in abandoned){pending.remove(id);effects.actions.add{request.reply.cancel()}}
  documents.remove(entry.uri)
  if(socket!=null)try{sendLocked(JSONObject().put("jsonrpc","2.0").put("method","textDocument/didClose").put("params",JSONObject().put("textDocument",JSONObject().put("uri",entry.uri))),effects)}catch(_:IllegalStateException){/* sendLocked already retired the failed wire and scheduled every remaining owner. */}
  if(subscribers.isEmpty())closeLocked(effects)
 }
 fun change(owner:UUID,source:String,version:Int):Unit=state{effects->
  val subscriber=subscribers[owner] ?: return@state
  if(subscriber.uri !in documents)return@state;documents[subscriber.uri]=Document(source,version)
  try{if(!startLocked(effects))sendDocumentLocked(subscriber.uri,true,effects)}catch(error:Throwable){effects.failures.putIfAbsent(owner,subscriber to error)}
 }
 suspend fun request(method:String,params:JSONObject,owner:UUID):Any {
  val context=currentCoroutineContext()
  val (id,answer)=state{effects->
   context.ensureActive();val subscriber=subscribers[owner] ?: error("Language source is closed. Retry language services.")
   params.optJSONObject("textDocument")?.let{check(it.opt("uri") is String&&it.getString("uri")==subscriber.uri){"Language request source identity changed."}}
   startLocked(effects);check(nextID<Int.MAX_VALUE){"Language request capacity exhausted. Reopen language services."}
   val id=nextID++;val answer=CompletableDeferred<Any>();pending[id]=Pending(owner,answer)
   try{sendLocked(JSONObject().put("jsonrpc","2.0").put("id",id).put("method",method).put("params",params),effects)}catch(error:Throwable){pending.remove(id);effects.actions.add{answer.completeExceptionally(error)};throw error};id to answer
  }
  try{return withTimeout(12000){answer.await()}}finally{state{pending.remove(id)}}
 }
 fun close()=state{effects->closeLocked(effects)}
 private fun closeLocked(effects:Effects){
  epoch++;val old=socket;socket=null;retryAfter=null;invalidating=false
  for(request in pending.values)effects.actions.add{request.reply.cancel()};pending.clear()
  for((id,subscriber)in subscribers)effects.failures[id]=subscriber to CancellationException("Language connection closed");subscribers.clear();documents.clear()
  if(old!=null)effects.actions.add{old.close(1000,"closed");Unit}
 }
 private fun incoming(action:()->Unit){scope.launch(Dispatchers.Default){withContext(scope.coroutineContext.minusKey(Job)){action()}}}
 private fun startLocked(effects:Effects):Boolean {
  if(socket!=null)return false
  check(!invalidating){"Language connection invalidation is still being delivered. Retry language services shortly."}
  check(retryAfter?.let{System.nanoTime()-it>=0} ?: true){"Language connection unavailable. Retry language services shortly."}
  val current=++epoch
  try{val created=openSocket(Request.Builder().url(engine.debugURL).build(),object:WebSocketListener(){
   override fun onMessage(webSocket:WebSocket,bytes:ByteString){val raw=bytes.toByteArray();if(raw.size<2||raw[0]!=2.toByte())return;val message=runCatching{JSONObject(raw.copyOfRange(1,raw.size).toString(Charsets.UTF_8))}.getOrNull() ?: return;incoming{receive(current,message)}}
   override fun onFailure(webSocket:WebSocket,t:Throwable,response:Response?){incoming{fail(current,t)}}
   override fun onClosing(webSocket:WebSocket,code:Int,reason:String){incoming{state{effects->if(epoch==current){effects.actions.add{webSocket.close(code,reason);Unit};failLocked(IllegalStateException("Language connection closing: $reason"),effects)}}}}
   override fun onClosed(webSocket:WebSocket,code:Int,reason:String){incoming{fail(current,IllegalStateException("Language connection closed: $reason"))}}
  });if(epoch!=current){effects.actions.add{created.cancel()};throw IllegalStateException("Language connection retired during opening.")};socket=created;for(uri in documents.keys.sorted())sendDocumentLocked(uri,false,effects)}catch(error:Throwable){if(epoch==current)failLocked(error,effects);throw error};return true
 }
 private fun sendDocumentLocked(uri:String,change:Boolean,effects:Effects){val document=documents[uri] ?: return;val params=JSONObject().put("textDocument",JSONObject().put("uri",uri).put("version",document.version));if(change)params.put("contentChanges",JSONArray().put(JSONObject().put("text",document.text)))else params.getJSONObject("textDocument").put("languageId","hick").put("text",document.text);sendLocked(JSONObject().put("jsonrpc","2.0").put("method",if(change)"textDocument/didChange" else "textDocument/didOpen").put("params",params),effects)}
 private fun sendLocked(message:JSONObject,effects:Effects){val wire=socket ?: error("Language connection is closed. Retry language services.");if(!wire.send((byteArrayOf(2)+message.toString().toByteArray(Charsets.UTF_8)).toByteString())){val error=IllegalStateException("Language connection is closed. Retry language services.");failLocked(error,effects);throw error}}
 private fun receive(current:Long,message:JSONObject)=state{effects->
  if(epoch!=current)return@state
  if(message.has("id")&&!message.has("method")){val id=message.opt("id") as? Int ?: return@state;val request=pending.remove(id) ?: return@state;if(request.owner !in subscribers){effects.actions.add{request.reply.cancel()};return@state};if(message.has("error")){val error=IllegalStateException(message.optJSONObject("error")?.optString("message") ?: "Language request refused");effects.actions.add{request.reply.completeExceptionally(error)}}else{val result=message.opt("result") ?: JSONObject.NULL;effects.actions.add{request.reply.complete(result)}};return@state}
  if(message.has("id")&&message.has("method")){val response=JSONObject().put("jsonrpc","2.0").put("id",message.get("id"));if(message.optString("method")=="workspace/applyEdit")response.put("result",JSONObject().put("applied",false).put("failureReason","A source preview is required."))else response.put("error",JSONObject().put("code",-32601).put("message","Client method unavailable"));sendLocked(response,effects);return@state}
  val target=if(message.optString("method")=="textDocument/publishDiagnostics")message.optJSONObject("params")?.optString("uri") else null
  for((id,listener)in subscribers)if(target==null||target==listener.uri)effects.actions.add{val valid=synchronized(gate){epoch==current&&subscribers[id]===listener};if(valid)listener.receive(message)}
 }
 private fun fail(current:Long,error:Throwable):Unit=state{effects->if(epoch==current)failLocked(error,effects)}
 private fun failLocked(error:Throwable,effects:Effects){
  val retired=++epoch;val old=socket;socket=null;retryAfter=System.nanoTime()+1_000_000_000L;invalidating=true
  for(request in pending.values)effects.actions.add{request.reply.completeExceptionally(error)};pending.clear()
  for((id,subscriber)in subscribers)effects.failures[id]=subscriber to error
  if(old!=null)effects.actions.add{old.cancel()}
  effects.finish.add{synchronized(gate){if(epoch==retired)invalidating=false}}
 }
 companion object{private val client=OkHttpClient.Builder().retryOnConnectionFailure(false).build()}
}
