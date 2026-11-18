import Foundation
struct SnippetVariableClock:Codable,Equatable {let unix_millis:Int64;let utc_offset_minutes:Int}
struct SnippetClipboard:Codable,Equatable {let available:Bool;let text:String?
 static func ==(a:Self,b:Self)->Bool{a.available==b.available && ((a.text==nil&&b.text==nil)||(a.text != nil&&b.text != nil&&Array(a.text!.utf8)==Array(b.text!.utf8)))}
}
struct SnippetVariableSnapshot:Codable,Equatable {
 let clock:SnippetVariableClock?;let random_hex:String?;var clipboard:SnippetClipboard?=nil
 static func capture()->Self {let now=Date();return Self(clock:SnippetVariableClock(unix_millis:Int64((now.timeIntervalSince1970*1000).rounded(.towardZero)),utc_offset_minutes:TimeZone.current.secondsFromGMT(for:now)/60),random_hex:UUID().uuidString.replacingOccurrences(of:"-",with:""))}
 var body:[String:Any]{["clock":clock.map{["unix_millis":$0.unix_millis,"utc_offset_minutes":Int64($0.utc_offset_minutes)]} as Any? ?? NSNull(),"random_hex":random_hex as Any? ?? NSNull(),"clipboard":clipboard.map{["available":$0.available,"text":$0.text as Any? ?? NSNull()]} as Any? ?? NSNull()]}
}
#if os(macOS)
import AppKit
#elseif os(iOS)
import UIKit
#endif
@MainActor func captureSnippetClipboard(read:(()->String?)?=nil)->SnippetClipboard {
 let value:String?
 if let read{value=read()}else{
 #if os(macOS)
 value=NSPasteboard.general.string(forType:.string)
 #elseif os(iOS)
 value=UIPasteboard.general.string
 #else
 value=nil
 #endif
 }
 guard let value,value.utf16.prefix(1048577).count<=1048576,value.utf8.prefix(1048577).count<=1048576 else{return SnippetClipboard(available:false,text:nil)};return SnippetClipboard(available:true,text:value)
}
func validSnippetContextSelection(_ source:String,_ from:Int,_ to:Int)->Bool {
 let ns=source as NSString
 func boundary(_ at:Int)->Bool{guard at>=0,at<=ns.length else{return false};if at>0,at<ns.length{let before=ns.character(at:at-1),after=ns.character(at:at);if before>=0xD800&&before<=0xDBFF&&after>=0xDC00&&after<=0xDFFF||before==13&&after==10{return false}};return true}
 return from<=to&&boundary(from)&&boundary(to)
}
func bindSnippetVariables(_ source:String,from:Int,to:Int,proof:Any,snapshot:SnippetVariableSnapshot = .capture())throws->[String:Any] {
 guard validSnippetContextSelection(source,from,to),let expected=proof as? [String:Any],let hash=expected["source_hash"] as? String,let revision=expected["source_revision"] as? String,hash=="sha256:"+nativeSourceHash(source),revision.count==64,revision.utf8.allSatisfy({(48...57).contains($0)||(97...102).contains($0)}) else{throw EngineError(message:"Snippet source context changed. Request completions again.")}
 return snapshot.body.merging(["version":1,"expected":["source_hash":hash,"source_revision":revision],"selection":["from":from,"to":to]]){_,new in new}
}

@MainActor func resolveSnippetCompletion(_ original:[String:Any],source:String,from:Int,to:Int,current:()->Bool,request:([String:Any])async throws->[String:Any],clipboard:(()->String?)?=nil)async throws->[String:Any]{
 func check()throws{guard current() else{throw EngineError(message:"Source, cursor or provider changed. Request completions again.")}};try check()
 guard let initialData=original["data"] as? [String:Any],initialData.keys.contains("hickory_context") else{let result=try await request(original);try check();return result}
 var snapshot=SnippetVariableSnapshot.capture(),item=original
 for attempt in 0..<2{try check();guard let data=item["data"] as? [String:Any],let proof=data["hickory_context"] else{throw EngineError(message:"Completion context proof is unavailable. Request again.")};let bound=try bindSnippetVariables(source,from:from,to:to,proof:proof,snapshot:snapshot);try check();var params=item,wire=data;wire["hickory_context"]=bound;params["data"]=wire;let result=try await request(params);try check();let reply=result["data"] as? [String:Any] ?? [:]
  guard let expected=bound["expected"] as? [String:Any],let proof=reply["hickory_context"] as? [String:Any],let hash=proof["source_hash"] as? String,let revision=proof["source_revision"] as? String,hash==expected["source_hash"] as? String,revision==expected["source_revision"] as? String else{throw EngineError(message:"Completion source context changed. Request again.")}
  guard let raw=reply["hickory_context_request"] else{return result};guard attempt==0,let requested=raw as? [String:Any],requested.count==2,sourceChangeInteger(requested["version"])==1,let names=requested["variables"] as? [String],names==["CLIPBOARD"] else{throw EngineError(message:"Unsupported or repeated completion context request. No edits were applied.")};snapshot.clipboard=captureSnippetClipboard(read:clipboard);try check();item=result
 }
 throw EngineError(message:"Completion context is unavailable. No edits were applied.")
}
func snippetContextNotice(_ item:[String:Any])throws->String {guard let data=item["data"] as? [String:Any],let raw=data["hickory_context_unavailable"] else{return ""};guard let names=raw as? [String],names.count<=40,names.allSatisfy({$0.utf8.count<=64}) else{throw EngineError(message:"Malformed completion context availability")};guard !names.isEmpty else{return ""};return names.allSatisfy({$0=="CLIPBOARD"}) ? "Clipboard unavailable; its fallback or empty text was used.":"Some requested context values were unavailable; their fallback or empty text was used."}
