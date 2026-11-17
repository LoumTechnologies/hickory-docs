<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Bound native review JSON before decoding nested material

For Android maintainers admitting retained agent intentions or private editor snapshots.

Hashing establishes the exact bytes being reviewed; it does not make a deeply nested payload safe to parse. Bound UTF-8, Unicode scalars, nesting and container count first. The same scanner serves both review mechanisms, while each caller retains its own protocol and byte budget. It has no provider, UI or publication policy.

<ui:copy id="native-review-json-android-budget">
package com.hickorydocs.simple
import org.json.JSONObject
import org.json.JSONTokener
internal fun nativeReviewTextBytes(text:String,limit:Long,label:String):Long {
 var bytes=0L;var index=0
 while(index<text.length){val c=text[index];bytes+=when{c.isHighSurrogate()->{check(index+1<text.length&&text[index+1].isLowSurrogate()){ "$label contains invalid Unicode." };index+=2;4};c.isLowSurrogate()->error("$label contains invalid Unicode.");else->{index++;when{c.code<128->1;c.code<2048->2;else->3}}};check(bytes<=limit){"$label exceeds its UTF8 budget."}}
 return bytes
}
internal fun nativeReviewMaterialBudget(text:String,maxBytes:Int,label:String){
 nativeReviewTextBytes(text,maxBytes.toLong(),label);var depth=0;var containers=0;var quoted=false;var escaped=false
 for(c in text){if(quoted){if(escaped)escaped=false else if(c=='\\')escaped=true else if(c=='"')quoted=false}else when(c){'"'->quoted=true;'{','['->{depth++;containers++;check(depth<=64&&containers<=65536){"$label structure exceeds its review budget."}};'}',']'->{depth--;check(depth>=0){"Malformed $label structure."}}}}
 check(!quoted&&depth==0){"Malformed $label material."}
}
internal fun nativeReviewProtocolJSON(text:String,maxBytes:Int,label:String,versions:Set<String>):JSONObject {
 nativeReviewMaterialBudget(text,maxBytes,label);val stack=mutableListOf<MutableSet<String>?>();var index=0
 while(index<text.length){when(text[index]){'{'->{stack.add(mutableSetOf());index++};'['->{stack.add(null);index++};'}',']'->{check(stack.isNotEmpty()){ "Malformed $label structure." };stack.removeAt(stack.lastIndex);index++};'"'->{val start=index++;var escaped=false;while(index<text.length){val c=text[index++];if(escaped){escaped=false;continue};if(c=='\\'){escaped=true;continue};if(c=='"')break};val end=index;var at=index;while(at<text.length&&text[at].isWhitespace())at++
  if(at<text.length&&text[at]==':'){val key=JSONTokener(text.substring(start,end)).nextValue() as? String ?: error("Malformed $label key.");val keys=stack.lastOrNull() ?: error("Malformed $label object.");check(keys.add(key)){"Duplicate $label JSON key refused."};if(key in versions){at++;while(at<text.length&&text[at].isWhitespace())at++;if(at<text.length&&(text[at]=='-'||text[at].isDigit())){val from=at;while(at<text.length&&!text[at].isWhitespace()&&text[at] !in ",}]")at++;check(text.substring(from,at).matches(Regex("0|[1-9][0-9]*"))){"$label requires canonical integer protocol versions."}}}}
 };else->index++}}
 return JSONObject(text)
}
</ui:copy>

<ui:file path="apps/android-simple/app/src/main/java/com/hickorydocs/simple/NativeReviewJSON.kt"><ui:paste select="#native-review-json-android-budget"/></ui:file>
</ui:doc>
