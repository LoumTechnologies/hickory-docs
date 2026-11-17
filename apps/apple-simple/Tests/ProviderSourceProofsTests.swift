import XCTest
@testable import HickorySimple
final class ProviderSourceProofsTests:XCTestCase {
 func row(_ uri:String="hick:///a",_ hash:String="sha256:"+String(repeating:"0",count:64))->[String:Any]{["uri":uri,"source_hash":hash]}
 func action(_ rows:Any)->[String:Any]{["data":["hickory_action":"opaque:1","hickory_sources":rows]]}
 func testDiscoversContextAndRejectsMalformedEvidence()throws {
  XCTAssertEqual(try providerSourceProofPaths(action([row(),row("hick:///b")])),["a","b"])
  let bad:[Any]=[[:],action([]),action([row("file:///a")]),action([row("hick:///a?x")]),action([row("hick:///../a")]),action([row("hick:///a","sha256:"+String(repeating:"A",count:64))]),action([["uri":"hick:///a","source_hash":"bad","extra":true]]),action([row(),row()]),action([row("hick:///b"),row()]),action([row("hick:///%61"),row()])]
  for value in bad{XCTAssertThrowsError(try providerSourceProofs(value))}
 }
 func testFrozenUTF8ContextHashAndExistingEditCoverage()throws {
  let original=SourceSnapshot(text:"é😀",writable:false),proof=action([row("hick:///a","sha256:"+nativeSourceHash("é😀"))])
  try validateProviderSourceProofs(proof,sources:["a":original],requiredExisting:["a"])
  XCTAssertThrowsError(try validateProviderSourceProofs(proof,sources:["a":SourceSnapshot(text:"e\u{301}😀",writable:false)],requiredExisting:[]))
  XCTAssertThrowsError(try validateProviderSourceProofs(proof,sources:[:],requiredExisting:[]))
  XCTAssertThrowsError(try validateProviderSourceProofs(proof,sources:["a":original],requiredExisting:["unproved-existing"]))
  try validateProviderSourceProofs(proof,sources:["a":original],requiredExisting:[]) // A caller excludes newly created destinations.
 }
}
