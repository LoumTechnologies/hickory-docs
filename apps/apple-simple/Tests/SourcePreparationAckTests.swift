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
