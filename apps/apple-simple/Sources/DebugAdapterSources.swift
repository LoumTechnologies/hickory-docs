import Foundation
import Observation
import CoreFoundation
import SwiftUI
struct NativeAdapterSourceBinding:Equatable,Sendable {
    static func ==(a:Self,b:Self)->Bool{exactUTF8(a.session,b.session)&&a.stop==b.stop&&a.thread==b.thread&&a.frame==b.frame&&a.reference==b.reference}
    var session:String;var stop:UInt64;var thread:Int64;var frame:Int;var reference:Int
    var request:[String:Any]{["op":"source","session":session,"stop":stop,"thread":thread,"frame":frame,"reference":reference]}
    func matches(_ event:[String:Any])->Bool {exactUTF8(event["session"] as? String,session)&&adapterSourceStop(event["stop"])==stop&&adapterSourceID(event["thread"]).map(Int64.init)==thread&&adapterSourceID(event["frame"])==frame&&adapterSourceID(event["reference"])==reference}
}
func adapterSourceID(_ raw:Any?)->Int? {guard let number=raw as? NSNumber,CFGetTypeID(number) != CFBooleanGetTypeID() else{return nil};let value=number.doubleValue;guard value.isFinite,value.rounded()==value,value>=Double(Int32.min),value<=Double(Int32.max) else{return nil};return Int(value)}
private func adapterSourceStop(_ raw:Any?)->UInt64? {guard let number=raw as? NSNumber,CFGetTypeID(number) != CFBooleanGetTypeID() else{return nil};let value=number.stringValue;guard !value.isEmpty,value.utf8.allSatisfy({(48...57).contains($0)}) else{return nil};return UInt64(value)}
struct NativeAdapterSourceFacts:Sendable {var hash:String;var scalars:Int;var starts:[Int]}
func nativeAdapterSourceFacts(_ content:String,mime:String?=nil,budget:Int=8*1024*1024)throws->NativeAdapterSourceFacts {
    guard budget>=0,content.utf8.count<=budget else{throw EngineError(message:"Adapter source exceeds its bounded UTF8 reading capacity.")}
    if let mime {guard mime.utf8.count<=256,!mime.unicodeScalars.contains(where:{CharacterSet.controlCharacters.contains($0)}) else{throw EngineError(message:"Adapter source MIME metadata is malformed or exceeds its capacity.")}}
    var scalars=0,offset=0,starts=[0]
    for scalar in content.unicodeScalars {if scalars>0&&scalars%4000==0{starts.append(offset)};offset += scalar.value>0xffff ? 2:1;scalars += 1}
    return NativeAdapterSourceFacts(hash:nativeSourceHash(content),scalars:scalars,starts:starts)
}
struct NativeAdapterSourceReading {var binding:NativeAdapterSourceBinding;var request:String;var content:String;var mime:String?;var facts:NativeAdapterSourceFacts
    func page(_ index:Int)->String{guard facts.starts.indices.contains(index) else{return ""};let start=facts.starts[index],end=index+1<facts.starts.count ? facts.starts[index+1]:content.utf16.count;return (content as NSString).substring(with:NSRange(location:start,length:end-start))}
}
@MainActor @Observable final class NativeAdapterSourceModel {
    private struct Pending {var binding:NativeAdapterSourceBinding;var request:String;var inspection:UUID;var generation:UUID;var processing=false}
    private var inspection:UUID?;private var generation=UUID();private var pending:Pending?
    private(set) var reading:NativeAdapterSourceReading?;private(set) var busy=false;private(set) var current=false;private(set) var notice=""
    @ObservationIgnored var validate:@Sendable(String,String?)async throws->NativeAdapterSourceFacts={text,mime in try await Task.detached(priority:.userInitiated){try nativeAdapterSourceFacts(text,mime:mime)}.value}
    func open()->UUID{retire("Previous adapter text is historical until explicitly refreshed.");let token=UUID();inspection=token;return token}
    func close(_ token:UUID){guard inspection==token else{return};inspection=nil;retire("Adapter-source inspector closed. Captured text remains historical.")}
    func retire(_ reason:String="Paused source observation changed. Captured adapter text is historical."){generation=UUID();pending=nil;busy=false;current=false;if reading != nil || !notice.isEmpty{notice=reason}}
    func cancel(_ token:UUID){if inspection==token{retire("Pending source reading cancelled locally. Late replies cannot replace captured text.")}}
    func begin(_ binding:NativeAdapterSourceBinding?,capability:Any?,inspection token:UUID)->[String:Any]? {
        guard inspection==token else{notice="Open the current debugger inspector before reading adapter source.";return nil}
        guard let number=capability as? NSNumber,CFGetTypeID(number)==CFBooleanGetTypeID(),number.boolValue else{notice="This engine/provider does not advertise adapter source content. No workspace text was substituted.";return nil}
        guard let binding,binding.reference>0,binding.reference<=Int(Int32.max) else{notice="Choose a current paused frame with a positive adapter source reference. No disk fallback is used.";return nil}
        let id=UUID().uuidString;generation=UUID();pending=Pending(binding:binding,request:id,inspection:token,generation:generation);busy=true;current=false;notice="Reading selected frame adapter-owned source…";var request=binding.request;request["request_id"]=id;return request
    }
    private func owns(_ value:Pending)->Bool{pending?.request==value.request&&inspection==value.inspection&&generation==value.generation}
    @discardableResult func receive(_ event:[String:Any],binding:()->NativeAdapterSourceBinding?)async->Bool {
        guard var held=pending,exactUTF8(event["request_id"] as? String,held.request),owns(held),!held.processing else{return false}
        guard event["event"] as? String=="adapter_source",held.binding.matches(event),binding()==held.binding,let content=event["content"] as? String,let hash=event["source_hash"] as? String,hash.utf8.count==64&&hash.utf8.allSatisfy({(48...57).contains($0)||(97...102).contains($0)}) else{return reject(held.request,"Adapter source reply is malformed or belongs to an obsolete stopped frame.")}
        let mime:String?;if let raw=event["mime_type"],!(raw is NSNull){guard let text=raw as? String else{return reject(held.request,"Adapter source MIME metadata is malformed.")};mime=text}else{mime=nil}
        held.processing=true;pending=held
        let facts:NativeAdapterSourceFacts;do{facts=try await validate(content,mime)}catch{return reject(held.request,error.localizedDescription)}
        guard owns(held) else{return false};guard binding()==held.binding else{retire();return false}
        guard exactUTF8(facts.hash,hash) else{return reject(held.request,"Adapter source bytes do not match the reported hash.")}
        reading=NativeAdapterSourceReading(binding:held.binding,request:held.request,content:content,mime:mime,facts:facts);pending=nil;busy=false;current=true;notice=content.isEmpty ? "Adapter reported empty literal source content.":"Adapter-owned literal text only. No workspace edit, source position or breakpoint authority is implied.";return true
    }
    private func reject(_ request:String,_ message:String)->Bool{_ = refuse(request,message);return false}
    @discardableResult func refuse(_ request:String,_ message:String)->Bool{guard let held=pending,exactUTF8(held.request,request),owns(held) else{return false};pending=nil;busy=false;current=false;notice=message+" Captured text is retained; refresh explicitly.";return true}
    func matches(_ binding:NativeAdapterSourceBinding?)->Bool{current&&reading?.binding==binding}
}
struct NativeAdapterSourceView:View {
    var debug:NativeDebugger;var inspection:UUID?
    @State private var page=0
    var body:some View {
        let model=debug.adapterSources,binding=debug.adapterSourceBinding
        if binding != nil || model.reading != nil || !model.notice.isEmpty {
            VStack(alignment:.leading,spacing:5){
                Text("Adapter-owned source").font(.caption.bold())
                Button("Read selected frame adapter source"){if let inspection{debug.readAdapterSource(inspection)}}.disabled(reason: inspection==nil ? "Pause the debugger and select a frame first." : binding==nil ? "This frame is not bound to the current debugger stop." : !debug.canReadAdapterSource ? "The current adapter does not provide this frame’s source." : model.busy ? "An operation is already in progress." : nil)
                if binding != nil && !debug.canReadAdapterSource{Text("Source content capability is unavailable. Workspace or disk text is not a fallback.").font(.caption)}
                if model.busy{Button("Cancel pending source reading"){if let inspection{model.cancel(inspection)}}}
                if !model.notice.isEmpty{Text(model.notice).font(.caption).textSelection(.enabled)}
                if let reading=model.reading {
                    let index=min(max(0,page),reading.facts.starts.count-1)
                    Text("\(model.matches(binding) ? "Current":"Historical") adapter text · pause \(reading.binding.stop) · thread \(reading.binding.thread) · frame \(reading.binding.frame) · reference \(reading.binding.reference)").font(.caption)
                    Text("\(reading.mime ?? "MIME unavailable") · SHA256 \(reading.facts.hash)").font(.caption).textSelection(.enabled)
                    Text(reading.page(index)).font(.system(.caption,design:.monospaced)).textSelection(.enabled).accessibilityLabel("Read-only literal adapter source page \(index+1)")
                    Text("Page \(index+1)/\(reading.facts.starts.count) · \(reading.facts.scalars) Unicode scalars. No workspace mapping is implied.").font(.caption)
                    HStack{Button("Previous adapter source page"){page=index-1}.disabled(reason: index==0 ? "You’re on the first page." : nil);Button("Next adapter source page"){page=index+1}.disabled(reason: index+1>=reading.facts.starts.count ? "You’re on the last page." : nil)}
                }
            }.onChange(of:model.reading?.request){_,_ in page=0}
        }
    }
}
