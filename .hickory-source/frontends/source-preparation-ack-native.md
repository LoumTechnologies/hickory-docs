<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Acknowledge current source preparation before asking for coordinates

For maintainers acquiring previously unopened or dirty provider targets. A sent change notification does not prove the engine installed its source map. Native background readers send the exact captured text through `hick/captureDocument`, validate its current source hash and opaque preparation revision, and only then ask for symbols. Plain files normally report `parsed=false`; a doc-backed literate target requires successful parsing. Caret motion does not invalidate background information, while source and provider epoch changes still refuse the read.

The acknowledgement is effectless preparation, not Save or test evidence. Typed booleans and exact UTF-8 hashes prevent a malformed or stale reply from renewing older target authority. Original host acquisition leases remain fixed across this wait and across the complete provider requery.

<ui:copy id="source-preparation-ack-apple-tests">
import XCTest
@testable import HickorySimple
final class SourcePreparationAckTests:XCTestCase {
 func testPlainAndLiterateSourcePreparationProofs()throws {
  let source="value = 'λ'\n",good:[String:Any]=["processed":true,"current":true,"parsed":false,"open_files":1,"source_hash":"sha256:"+nativeSourceHash(source),"source_revision":String(repeating:"a",count:64),"source_files_modified":false]
  XCTAssertNoThrow(try validateNativeSourcePreparation(good,source:source));XCTAssertThrowsError(try validateNativeSourcePreparation(good,source:source,requireParsed:true));var parsed=good;parsed["parsed"]=true;XCTAssertNoThrow(try validateNativeSourcePreparation(parsed,source:source,requireParsed:true))
  let bad:[(String,Any)]=[("processed",false),("current",false),("processed",1),("current","true"),("parsed","false"),("open_files",-1),("source_hash","sha256:"+String(repeating:"0",count:64)),("source_revision","legacy-counter"),("source_revision",true),("source_files_modified",true),("source_files_modified",0)]
  for(key,value)in bad{var changed=good;changed[key]=value;XCTAssertThrowsError(try validateNativeSourcePreparation(changed,source:source),key)}
  var missing=good;missing.removeValue(forKey:"source_revision");XCTAssertThrowsError(try validateNativeSourcePreparation(missing,source:source));XCTAssertThrowsError(try validateNativeSourcePreparation(good,source:source+" "))
 }
}
</ui:copy>
<ui:file path="apps/apple-simple/Tests/SourcePreparationAckTests.swift"><ui:paste select="#source-preparation-ack-apple-tests"/></ui:file>

<ui:copy id="source-preparation-ack-android-tests">
package com.hickorydocs.simple
import org.junit.Test
import org.junit.Assert.*
import org.json.JSONObject
class SourcePreparationAckTest {
 @Test fun plainAndLiterateSourcePreparationProofs(){
  val source="value = 'λ'\n";val good=JSONObject().put("processed",true).put("current",true).put("parsed",false).put("open_files",1).put("source_hash","sha256:"+debugHash(source)).put("source_revision","a".repeat(64)).put("source_files_modified",false)
  validateNativeSourcePreparation(good,source);assertTrue(runCatching{validateNativeSourcePreparation(good,source,true)}.isFailure);validateNativeSourcePreparation(JSONObject(good.toString()).put("parsed",true),source,true)
  val bad=listOf("processed" to false,"current" to false,"processed" to 1,"current" to "true","parsed" to "false","open_files" to -1,"source_hash" to "sha256:"+"0".repeat(64),"source_revision" to "legacy-counter","source_revision" to true,"source_files_modified" to true,"source_files_modified" to 0)
  for((key,value)in bad)assertTrue(key,runCatching{validateNativeSourcePreparation(JSONObject(good.toString()).put(key,value),source)}.isFailure)
  val missing=JSONObject(good.toString());missing.remove("source_revision");assertTrue(runCatching{validateNativeSourcePreparation(missing,source)}.isFailure);assertTrue(runCatching{validateNativeSourcePreparation(good,source+" ")}.isFailure)
 }
}
</ui:copy>
<ui:file path="apps/android-simple/app/src/test/java/com/hickorydocs/simple/SourcePreparationAckTest.kt"><ui:paste select="#source-preparation-ack-android-tests"/></ui:file>
</ui:doc>
