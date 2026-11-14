package com.hickorydocs.simple
import kotlinx.coroutines.*
import okhttp3.*
import okio.ByteString
import okio.ByteString.Companion.toByteString
import org.json.JSONObject
import org.json.JSONArray
import java.util.UUID
class NativeLanguageConnection(private val engine: Engine,private val scope: CoroutineScope) {
    private data class Subscriber(val uri:String,val receive:(JSONObject)->Unit,val failed:(Throwable)->Unit)
    private data class Document(val text:String,val version:Int)
    private data class Pending(val owner:UUID,val reply:CompletableDeferred<Any>)
    private val subscribers=linkedMapOf<UUID,Subscriber>()
    private val documents=linkedMapOf<String,Document>()
    private val pending=mutableMapOf<Int,Pending>()
    private var nextID=1;private var epoch=0;private var retryAfter:Long?=null
    private var socket:WebSocket?=null
    val active get()=socket!=null
    fun attach(uri:String,source:String,version:Int,receive:(JSONObject)->Unit,failed:(Throwable)->Unit):UUID {
        val token=UUID.randomUUID();val existing=documents.containsKey(uri)
        if(existing) {failed(IllegalStateException("Reuse the workspace language session for this source: $uri"));return token}
        subscribers[token]=Subscriber(uri,receive,failed);documents[uri]=Document(source,version)
        runCatching {if(!startIfNeeded()) sendDocument(uri,existing)}.onFailure(failed)
        return token
    }
    fun detach(token:UUID) {
        val entry=subscribers.remove(token) ?: return
        val abandoned=pending.filterValues {it.owner==token};for((id,request) in abandoned) {pending.remove(id);request.reply.cancel()}
        if(subscribers.values.none {it.uri==entry.uri}) {
            documents.remove(entry.uri)
            if(active) runCatching {send(JSONObject().put("jsonrpc","2.0").put("method","textDocument/didClose").put("params",JSONObject().put("textDocument",JSONObject().put("uri",entry.uri))))}
        }
        if(subscribers.isEmpty()) close()
    }
    fun change(owner:UUID,source:String,version:Int) {
        val uri=subscribers[owner]?.uri ?: return
        if(uri !in documents) return;documents[uri]=Document(source,version)
        runCatching {if(!startIfNeeded()) sendDocument(uri,true)}
    }
    suspend fun request(method:String,params:JSONObject,owner:UUID):Any {
        val subscriber=subscribers[owner] ?: error("Language source is closed. Retry language services.")
        params.optJSONObject("textDocument")?.let {check(it.optString("uri")==subscriber.uri) {"Language request source identity changed."}}
        startIfNeeded()
        val id=nextID++;val answer=CompletableDeferred<Any>();pending[id]=Pending(owner,answer)
        try {send(JSONObject().put("jsonrpc","2.0").put("id",id).put("method",method).put("params",params));return withTimeout(12000) {answer.await()}}
        finally {pending.remove(id)}
    }
    fun close() {
        epoch++;socket?.close(1000,"closed");socket=null;retryAfter=null
        val listeners=subscribers.values.toList();subscribers.clear();documents.clear();listeners.forEach {it.failed(CancellationException("Language connection closed"))}
        val replies=pending.values.toList();pending.clear();replies.forEach {it.reply.cancel()}
    }
    private fun startIfNeeded():Boolean {
        if(active) return false
        check(retryAfter?.let {System.nanoTime()-it>=0} ?: true) {"Language connection unavailable. Retry language services shortly."}
        val current=++epoch
        socket=client.newWebSocket(Request.Builder().url(engine.debugURL).build(),object:WebSocketListener() {
            override fun onMessage(webSocket:WebSocket,bytes:ByteString) {
                val raw=bytes.toByteArray();if(raw.size<2 || raw[0]!=2.toByte()) return
                val message=runCatching {JSONObject(raw.copyOfRange(1,raw.size).toString(Charsets.UTF_8))}.getOrNull() ?: return
                scope.launch {if(epoch==current) receive(message)}
            }
            override fun onFailure(webSocket:WebSocket,t:Throwable,response:Response?) {scope.launch {if(epoch==current) fail(t)}}
            override fun onClosing(webSocket:WebSocket,code:Int,reason:String) {scope.launch {if(epoch==current) {webSocket.close(code,reason);fail(IllegalStateException("Language connection closing: $reason"))}}}
            override fun onClosed(webSocket:WebSocket,code:Int,reason:String) {scope.launch {if(epoch==current) fail(IllegalStateException("Language connection closed: $reason"))}}
        })
        for(uri in documents.keys.sorted()) sendDocument(uri,false)
        return true
    }
    private fun sendDocument(uri:String,change:Boolean) {
        val document=documents[uri] ?: return
        val params=JSONObject().put("textDocument",JSONObject().put("uri",uri).put("version",document.version))
        if(change) params.put("contentChanges",JSONArray().put(JSONObject().put("text",document.text)))
        else params.getJSONObject("textDocument").put("languageId","hick").put("text",document.text)
        send(JSONObject().put("jsonrpc","2.0").put("method",if(change) "textDocument/didChange" else "textDocument/didOpen").put("params",params))
    }
    private fun send(message:JSONObject) {
        val wire=socket ?: error("Language connection is closed. Retry language services.")
        if(!wire.send((byteArrayOf(2)+message.toString().toByteArray(Charsets.UTF_8)).toByteString())) {val error=IllegalStateException("Language connection is closed. Retry language services.");fail(error);throw error}
    }
    private fun receive(message:JSONObject) {
        if(message.has("id") && !message.has("method")) {
            val request=pending.remove(message.optInt("id")) ?: return
            if(message.has("error")) request.reply.completeExceptionally(IllegalStateException(message.optJSONObject("error")?.optString("message") ?: "Language request refused"))
            else request.reply.complete(message.opt("result") ?: JSONObject.NULL);return
        }
        if(message.has("id") && message.has("method")) {
            val response=JSONObject().put("jsonrpc","2.0").put("id",message.get("id"))
            if(message.optString("method")=="workspace/applyEdit") response.put("result",JSONObject().put("applied",false).put("failureReason","A source preview is required."))
            else response.put("error",JSONObject().put("code",-32601).put("message","Client method unavailable"))
            send(response);return
        }
        val target=if(message.optString("method")=="textDocument/publishDiagnostics") message.optJSONObject("params")?.optString("uri") else null
        for(listener in subscribers.values.toList()) if(target==null || target==listener.uri) listener.receive(message)
    }
    private fun fail(error:Throwable) {
        epoch++;socket?.cancel();socket=null;retryAfter=System.nanoTime()+1_000_000_000L
        subscribers.values.toList().forEach {it.failed(error)}
        val replies=pending.values.toList();pending.clear();replies.forEach {it.reply.completeExceptionally(error)}
    }
    companion object {private val client=OkHttpClient.Builder().retryOnConnectionFailure(false).build()}
}
