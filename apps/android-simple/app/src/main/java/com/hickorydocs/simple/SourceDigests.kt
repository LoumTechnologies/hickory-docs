package com.hickorydocs.simple
import androidx.compose.runtime.*
import kotlinx.coroutines.*
class SourceDigests(private val scope:CoroutineScope,private val hash:suspend (String)->String={source->withContext(Dispatchers.Default){java.security.MessageDigest.getInstance("SHA-256").digest(source.toByteArray(Charsets.UTF_8)).joinToString(""){"%02x".format(it)}}}) {
    private data class Record(val source:String,val hash:String)
    private data class Request(val source:String,val current:()->String?)
    private var records by mutableStateOf<Map<String,Record>>(emptyMap());private val pending=linkedMapOf<String,Request>();private var active=false;private var closed=false
    fun current(path:String,source:String)=records[path]?.takeIf{it.source==source}?.hash
    fun enqueue(path:String,source:String,current:()->String?){if(closed||this.current(path,source)!=null)return;pending[path]=Request(source,current);pump()}
    fun stop(){closed=true;pending.clear()}
    private fun pump(){if(closed||active)return;val next=pending.entries.firstOrNull() ?: return;val path=next.key;val request=next.value;pending.remove(path);active=true
        scope.launch{try{val value=hash(request.source);if(!closed&&request.current()==request.source&&path !in pending)records=records+(path to Record(request.source,value))}
        catch(cancelled:CancellationException){throw cancelled}catch(_:Exception){}finally{active=false;pump()}}
    }
}
