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
