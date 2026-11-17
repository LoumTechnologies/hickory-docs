<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Share Android language transport across source buffers

For maintainers changing workspace services independently of the editor renderer.

Language knowledge belongs to source buffers, while wire lifetime belongs to the workspace. One shared connection owns request IDs, pending replies, document snapshots and reconnection replay. Notifications route by source URI; capability announcements fan out. Each URI has one source session in a workspace; views borrow it. Detaching a source cancels only its requests and closes its document. Duplicate sessions cannot overwrite the held source or borrow another owner's request authority. A failed connection invalidates every subscriber before replies resume, retaining readable source. The next read may reconnect and replay all held full-text documents. This ordinary transport object can be replaced without rewriting editor widgets.

Every shared transport map, request ID and epoch is admitted through one short monitor. Socket sends only enqueue into OkHttp; no coroutine waits under that monitor. Deferred completions, subscriber callbacks and socket retirement execute afterward. Subscriber failure notifications precede pending-reply completion, including synchronous continuation resumption. Failure delivery temporarily blocks reconnection, so a new wire cannot publish replies before old subscribers are invalidated. Cancellation cleanup re-enters the same monitor instead of racing teardown iteration. Callback exceptions are reported after the remaining notifications drain rather than hiding a collection race.

<ui:copy id="simple-android-language-connection">
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
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/LanguageConnection.kt"><ui:paste select="#simple-android-language-connection"/></ui:file>
## Reproduce teardown races without relying on scheduler luck

The fake socket is an enqueue-only boundary around the real connection. A coordinated start races already-pending requests with close, responses and cancellation across 24 generations. Independent owners attach/change/detach concurrently; stale wire epochs cannot deliver into a replacement source. A synchronous factory retirement and a throwing subscriber prove that callbacks run after admission releases, remaining listeners drain, and reconnect invalidation is released even on failure. The existing real WebSocket fixture and the five-case TypeScript host flow remain separate usable-path acceptance.

<ui:copy id="android-language-connection-concurrency-tests">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import kotlinx.coroutines.*
import okhttp3.*
import okio.ByteString
import okio.ByteString.Companion.toByteString
import org.json.JSONObject
import java.util.concurrent.CopyOnWriteArrayList
import java.util.concurrent.LinkedBlockingQueue
import java.util.concurrent.CompletableFuture
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicInteger
class LanguageConnectionConcurrencyTest {
 private class Wire(val listener:WebSocketListener,val auto:Boolean=false):WebSocket {
  val messages=LinkedBlockingQueue<JSONObject>();val accepting=AtomicBoolean(true);val closed=AtomicInteger();val cancelled=AtomicInteger()
  override fun request()=Request.Builder().url("http://127.0.0.1:1/debug").build()
  override fun queueSize()=0L
  override fun send(text:String)=false
  override fun send(bytes:ByteString):Boolean {if(!accepting.get())return false;val raw=bytes.toByteArray();val message=JSONObject(raw.copyOfRange(1,raw.size).toString(Charsets.UTF_8));messages.add(message);if(auto&&message.has("id"))reply(message.getInt("id"));return true}
  override fun close(code:Int,reason:String?):Boolean{closed.incrementAndGet();return true}
  override fun cancel(){cancelled.incrementAndGet()}
  fun reply(id:Int){listener.onMessage(this,(byteArrayOf(2)+JSONObject().put("jsonrpc","2.0").put("id",id).put("result",JSONObject().put("ok",true)).toString().toByteArray()).toByteString())}
  fun diagnostics(uri:String){listener.onMessage(this,(byteArrayOf(2)+JSONObject().put("jsonrpc","2.0").put("method","textDocument/publishDiagnostics").put("params",JSONObject().put("uri",uri)).toString().toByteArray()).toByteString())}
 }
 private fun params(uri:String)=JSONObject().put("textDocument",JSONObject().put("uri",uri))
 private suspend fun until(message:String,test:()->Boolean){withTimeout(5000){while(!test())delay(2)};assertTrue(message,test())}
 @Test(timeout=60000) fun closeResponseCancellationAndFinallyCleanupShareOneStateAdmission()=runBlocking {
  repeat(24){
   val errors=CopyOnWriteArrayList<Throwable>();val scope=CoroutineScope(SupervisorJob()+Dispatchers.Default+CoroutineExceptionHandler{_,error->errors.add(error)});val wires=CopyOnWriteArrayList<Wire>();val failed=AtomicInteger();val connection=NativeLanguageConnection(Engine("http://127.0.0.1:1"),scope){_,listener->Wire(listener).also{wires.add(it)}}
   try{val uri="hick:///a.py";val owner=connection.attach(uri,"original",1,{},{failed.incrementAndGet()});val wire=wires.single();val jobs=(0 until 64).map{scope.async{runCatching{connection.request("fixture/block",params(uri),owner)}}};until("Every request must be admitted before teardown"){wire.messages.count{it.has("id")}==64};val ids=wire.messages.filter{it.has("id")}.map{it.getInt("id")};assertEquals(64,ids.toSet().size)
    val start=CompletableDeferred<Unit>();val close=scope.async{start.await();connection.close()};val replies=scope.async{start.await();ids.forEach(wire::reply)};val cancellations=scope.async{start.await();jobs.filterIndexed{index,_->index%2==0}.forEach{it.cancel()}};start.complete(Unit);awaitAll(close,replies,cancellations);jobs.forEach{runCatching{it.await()}};assertTrue(jobs.all{it.isCompleted});assertFalse(connection.active);assertEquals(1,failed.get());assertTrue(errors.toString(),errors.isEmpty());assertTrue(runCatching{connection.request("fixture/late",params(uri),owner)}.isFailure)
   }finally{connection.close();scope.cancel()}
  }
 }
 @Test(timeout=15000) fun failureInvalidatesOwnersBeforePendingContinuationsResume()=runBlocking {
  val errors=CopyOnWriteArrayList<Throwable>();val scope=CoroutineScope(SupervisorJob()+Dispatchers.Default+CoroutineExceptionHandler{_,error->errors.add(error)});val wires=CopyOnWriteArrayList<Wire>();val connection=NativeLanguageConnection(Engine("http://127.0.0.1:1"),scope){_,listener->Wire(listener).also{wires.add(it)}};val intent=AtomicInteger();val observed=AtomicInteger(-1);val held=java.util.concurrent.atomic.AtomicReference<Deferred<Result<Any>>?>()
  try{val uri="hick:///a.py";val owner=connection.attach(uri,"source",1,{},{if(it.message=="owned ordering failure"){assertFalse("A pending continuation resumed before owner invalidation",held.get()?.isCompleted ?: false);intent.incrementAndGet()}});val query=scope.async(Dispatchers.Unconfined){val result=runCatching{connection.request("fixture/hold",params(uri),owner)};observed.set(intent.get());result};held.set(query);val wire=wires.single();until("Request not pending"){wire.messages.any{it.has("id")}};wire.listener.onFailure(wire,IllegalStateException("owned ordering failure"),null);assertTrue(query.await().isFailure);assertEquals(1,observed.get());until("Failure callbacks did not finish"){!connection.active};assertTrue(errors.toString(),errors.isEmpty())
  }finally{connection.close();scope.cancel()}
 }
 @Test(timeout=30000) fun concurrentSourceOwnersCannotOverwriteDetachOrReuseAnotherSource()=runBlocking {
  val errors=CopyOnWriteArrayList<Throwable>();val scope=CoroutineScope(SupervisorJob()+Dispatchers.Default+CoroutineExceptionHandler{_,error->errors.add(error)});val wires=CopyOnWriteArrayList<Wire>();val connection=NativeLanguageConnection(Engine("http://127.0.0.1:1"),scope){_,listener->Wire(listener,true).also{wires.add(it)}}
  try{val anchor=connection.attach("hick:///anchor.py","anchor",1,{},{})
   (0 until 128).map{index->scope.async{val uri="hick:///source$index.py";val owner=connection.attach(uri,"$index",1,{},{});connection.change(owner,"new$index",2);assertTrue(runCatching{connection.request("fixture/wrong",params("hick:///foreign.py"),owner)}.isFailure);val query=async{runCatching{connection.request("fixture/fast",params(uri),owner)}};yield();connection.detach(owner);query.await();assertTrue(runCatching{connection.request("fixture/late",params(uri),owner)}.isFailure)}}.awaitAll()
   assertTrue(connection.active);assertTrue((connection.request("fixture/fast",params("hick:///anchor.py"),anchor) as JSONObject).getBoolean("ok"));val ids=wires.flatMap{it.messages.filter{row->row.has("id")}.map{row->row.getInt("id")}};assertEquals(ids.size,ids.toSet().size);assertTrue(errors.toString(),errors.isEmpty())
  }finally{connection.close();scope.cancel()}
 }
 @Test(timeout=15000) fun fastFactoryRetirementCannotPublishItsOldWireAndCallbacksRunOutsideTheMonitor()=runBlocking {
  val scope=CoroutineScope(SupervisorJob()+Dispatchers.Unconfined);lateinit var connection:NativeLanguageConnection;val wires=CopyOnWriteArrayList<Wire>();val failed=AtomicInteger()
  connection=NativeLanguageConnection(Engine("http://127.0.0.1:1"),scope){_,listener->connection.close();Wire(listener).also{wires.add(it)}}
  try{val owner=connection.attach("hick:///a.py","text",1,{},{failed.incrementAndGet();assertFalse(CompletableFuture.supplyAsync{connection.active}.get(2,TimeUnit.SECONDS))});assertFalse(connection.active);assertEquals(1,failed.get());assertEquals(1,wires.single().cancelled.get());assertTrue(runCatching{connection.request("fixture/late",params("hick:///a.py"),owner)}.isFailure)}finally{connection.close();scope.cancel()}
 }
 @Test(timeout=20000) fun staleEpochRepliesAndDiagnosticsCannotReachReattachedSource()=runBlocking {
  val scope=CoroutineScope(SupervisorJob()+Dispatchers.Default);val wires=CopyOnWriteArrayList<Wire>();val connection=NativeLanguageConnection(Engine("http://127.0.0.1:1"),scope){_,listener->Wire(listener).also{wires.add(it)}}
  try{val uri="hick:///a.py";connection.attach(uri,"old",1,{},{});val old=wires.single();connection.close();val notifications=AtomicInteger();val owner=connection.attach(uri,"new",1,{notifications.incrementAndGet()},{});val wire=wires.last();val query=scope.async{connection.request("fixture/hold",params(uri),owner)};until("Current request not queued"){wire.messages.any{it.has("id")}};val id=wire.messages.first{it.has("id")}.getInt("id");old.reply(id);old.diagnostics(uri);delay(50);assertFalse(query.isCompleted);assertEquals(0,notifications.get());wire.reply(id);assertTrue((query.await() as JSONObject).getBoolean("ok"));wire.diagnostics(uri);until("Current diagnostics not delivered"){notifications.get()==1}
  }finally{connection.close();scope.cancel()}
 }
 @Test(timeout=20000) fun throwingFailureListenerCannotStrandPendingRepliesOrReconnectionInvalidation()=runBlocking {
  val errors=CopyOnWriteArrayList<Throwable>();val scope=CoroutineScope(SupervisorJob()+Dispatchers.Default+CoroutineExceptionHandler{_,error->errors.add(error)});val wires=CopyOnWriteArrayList<Wire>();val connection=NativeLanguageConnection(Engine("http://127.0.0.1:1"),scope){_,listener->Wire(listener).also{wires.add(it)}};val failed=AtomicInteger()
  try{val a=connection.attach("hick:///a.py","a",1,{},{if(it.message=="owned wire failure")throw IllegalStateException("owned listener failure")});val b=connection.attach("hick:///b.py","b",1,{},{failed.incrementAndGet()});val qa=scope.async{runCatching{connection.request("fixture/hold",params("hick:///a.py"),a)}};val qb=scope.async{runCatching{connection.request("fixture/hold",params("hick:///b.py"),b)}};val old=wires.single();until("Both requests must be pending"){old.messages.count{it.has("id")}==2};old.listener.onFailure(old,IllegalStateException("owned wire failure"),null);assertTrue(qa.await().isFailure);assertTrue(qb.await().isFailure);until("All listeners must be notified despite one failure"){failed.get()==1&&errors.size==1};assertEquals("owned listener failure",errors.single().message);delay(1100);val next=scope.async{connection.request("fixture/retry",params("hick:///b.py"),b)};until("New wire request not queued"){wires.size==2&&wires.last().messages.any{it.has("id")}};val wire=wires.last();wire.reply(wire.messages.first{it.has("id")}.getInt("id"));assertTrue((next.await() as JSONObject).getBoolean("ok"));connection.detach(a)
  }finally{connection.close();scope.cancel()}
 }
}
</ui:copy>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/LanguageConnectionConcurrencyTest.kt"><ui:paste select="#android-language-connection-concurrency-tests"/></ui:file>

</ui:doc>
