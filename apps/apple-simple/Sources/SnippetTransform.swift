import Foundation
import CoreFoundation
let supportedSnippetTransformPrograms=[2,3]
struct SnippetTransform {let regex:[Any];let format:[Any];let flags:String;let groups:Int;let bytes:Int;var v3:RegexV3Program?=nil}
struct SnippetTransformBudget {var work=0}
func readSnippetTransform(_ raw:Any)throws->SnippetTransform {
 if let value=raw as? [String:Any],let version=value["version"] as? NSNumber,CFGetTypeID(version) != CFBooleanGetTypeID(),version.doubleValue==3{let program=try readRegexV3(value);return SnippetTransform(regex:[],format:program.format,flags:program.flags,groups:program.groups,bytes:program.bytes,v3:program)}
 var count=0,bytes=0
 func tick(_ n:Int=1)throws{count+=n;guard count<=512 else{throw EngineError(message:"Snippet transform program limit exceeded")}}
 func integer(_ raw:Any,_ max:Int=1114111)throws->Int{guard let n=raw as? NSNumber,CFGetTypeID(n) != CFBooleanGetTypeID(),let value=Int(n.stringValue),value>=0,value<=max else{throw EngineError(message:"Malformed transform integer")};return value}
 func boolean(_ raw:Any)throws->Bool{guard let n=raw as? NSNumber,CFGetTypeID(n)==CFBooleanGetTypeID() else{throw EngineError(message:"Malformed transform Boolean")};return n.boolValue}
 func text(_ raw:Any)throws->String{guard let text=raw as? String else{throw EngineError(message:"Malformed transform text")};bytes+=text.utf8.count;guard bytes<=16384 else{throw EngineError(message:"Transform metadata limit exceeded")};return text}
 guard let raw=raw as? [String:Any],try integer(raw["version"] as Any)==2,raw["case_map"] as? String==snippetCaseVersion else{throw EngineError(message:"Unsupported snippet transform version")};let groups=try integer(raw["groups"] as Any,32),flags=try text(raw["flags"] as Any)
 guard flags.allSatisfy({"gimsu".contains($0)}),Set(flags).count==flags.count else{throw EngineError(message:"Unsupported transform flags")}
 func node(_ raw:Any,_ depth:Int)throws->[Any]{try tick();guard depth<=64,let v=raw as? [Any],let op=v.first as? String else{throw EngineError(message:"Malformed transform node")}
  switch op {
  case "seq","alt":guard v.count==2,let children=v[1] as? [Any],children.count<=512 else{throw EngineError(message:"Malformed transform branches")};return [op,try children.map{try node($0,depth+1)}]
  case "capture":guard v.count==3 else{throw EngineError(message:"Malformed transform capture")};return [op,try integer(v[1],groups),try node(v[2],depth+1)]
  case "repeat":guard v.count==5 else{throw EngineError(message:"Malformed transform repetition")};let min=try integer(v[2],256),max=v[3] is NSNull ? nil:try integer(v[3],256);guard max==nil||max!>=min else{throw EngineError(message:"Backwards transform repetition")};return [op,try node(v[1],depth+1),min,max as Any? ?? NSNull(),try boolean(v[4])]
  case "class":guard v.count==3,let ranges=v[1] as? [Any] else{throw EngineError(message:"Malformed transform class")};let values=try ranges.map{raw->[Int] in try tick();guard let range=raw as? [Any],range.count==2 else{throw EngineError(message:"Malformed transform range")};let low=try integer(range[0]),high=try integer(range[1]);guard low<=high else{throw EngineError(message:"Backwards transform range")};return [low,high]};return [op,values,try boolean(v[2])]
  case "dot":guard v.count==1 else{throw EngineError(message:"Malformed transform dot")};return [op]
  case "assert":guard v.count==2,let kind=v[1] as? String,["start","end","word","nonword"].contains(kind) else{throw EngineError(message:"Malformed transform assertion")};return [op,kind]
  default:throw EngineError(message:"Unsupported transform operation")
  }
 }
 func format(_ raw:Any,_ depth:Int)throws->[Any]{guard depth<=32,let values=raw as? [Any],values.count<=512 else{throw EngineError(message:"Malformed transform format")};return try values.map{raw->Any in try tick();if raw is String{return try text(raw)};guard let op=raw as? [Any],let kind=op.first as? String else{throw EngineError(message:"Malformed transform format")};if kind=="capture",op.count==3,let mode=op[2] as? String,["","upcase","downcase","capitalize","camelcase","pascalcase"].contains(mode){return [kind,try integer(op[1],32),mode]};if kind=="if",op.count==4{return [kind,try integer(op[1],32),try format(op[2],depth+1),try format(op[3],depth+1)]};throw EngineError(message:"Unsupported transform format")}}
 let regex=try node(raw["regex"] as Any,0),format=try format(raw["format"] as Any,0),data=try JSONSerialization.data(withJSONObject:["version":2,"case_map":snippetCaseVersion,"regex":regex,"format":format,"groups":groups,"flags":flags]);return SnippetTransform(regex:regex,format:format,flags:flags,groups:groups,bytes:data.count)
}
private struct TransformMatch {var at:Int;var captures:[Range<Int>?]}
private struct TransformEvaluator {
 let text:[Unicode.Scalar];let flags:String;var work:Int
 mutating func tick(_ n:Int=1)throws{guard n>=0,work>=0,work<=100000-n else{throw EngineError(message:"Snippet transform work limit exceeded")};work+=n}
 func newline(_ c:Unicode.Scalar?)->Bool{c.map{[10,13,0x2028,0x2029].contains(Int($0.value))} ?? false}
 func word(_ c:Unicode.Scalar?)->Bool{guard let c else{return false};return c.value>=48&&c.value<=57||c.value>=65&&c.value<=90||c.value==95||c.value>=97&&c.value<=122||flags.contains("i")&&flags.contains("u")&&(c.value==383||c.value==8490)}
 func string(_ from:Int,_ to:Int)->String{text[from..<to].map(String.init).joined()}
 mutating func matches(_ node:[Any],_ state:TransformMatch,_ depth:Int)throws->[TransformMatch]{try tick();guard depth<=64 else{throw EngineError(message:"Transform evaluation nesting limit exceeded")};let op=node[0] as! String;var result:[TransformMatch]=[]
  switch op {
  case "seq":var states=[state];for child in node[1] as! [[Any]]{var next:[TransformMatch]=[];for state in states{next+=try matches(child,state,depth+1);guard next.count<=1024 else{throw EngineError(message:"Transform state limit exceeded")}};states=next;if states.isEmpty{break}};result=states
  case "alt":for child in node[1] as! [[Any]]{result+=try matches(child,state,depth+1);guard result.count<=1024 else{throw EngineError(message:"Transform state limit exceeded")}}
  case "capture":let start=state.at,index=node[1] as! Int;result=try matches(node[2] as! [Any],state,depth+1);for i in result.indices{result[i].captures[index]=start..<result[i].at}
  case "repeat":let min=node[2] as! Int,max=Swift.min(node[3] as? Int ?? text.count+1,4096),lazy=node[4] as! Bool;var levels=[[state]],total=1
   for _ in 0..<max{var next:[TransformMatch]=[];let previous=levels.last!;for state in previous{for value in try matches(node[1] as! [Any],state,depth+1){if value.at != state.at || levels.count<=min{next.append(value)}};guard next.count<=1024 else{throw EngineError(message:"Transform state limit exceeded")}};if next.isEmpty{break};levels.append(next);total+=next.count;guard total<=1024 else{throw EngineError(message:"Transform state limit exceeded")}}
   if min<levels.count{let indexes=Array(min..<levels.count);for i in lazy ? indexes:Array(indexes.reversed()){result+=levels[i]}}
  case "assert":let before=state.at>0 ? text[state.at-1]:nil,next=state.at<text.count ? text[state.at]:nil,boundary=word(before) != word(next),kind=node[1] as! String
   let yes=kind=="start" ? state.at==0 || flags.contains("m")&&newline(before):kind=="end" ? state.at==text.count || newline(next)&&(flags.contains("m")||state.at+1==text.count||state.at+2==text.count&&next?.value==13&&text.last?.value==10):kind=="word" ? boundary:!boundary;if yes{result=[state]}
  case "class","dot":if state.at<text.count{let ch=text[state.at];let yes:Bool;if op=="dot"{yes=flags.contains("s") || !newline(ch)}else{let ranges=node[1] as! [[Int]];try tick(ranges.count);func contains(_ c:Unicode.Scalar)->Bool{ranges.contains{$0[0]<=Int(c.value)&&Int(c.value)<=$0[1]}};let found=contains(ch);yes=found != (node[2] as! Bool)};if yes{var next=state;next.at+=1;result=[next]}}
  default:throw EngineError(message:"Unsupported transform operation")
  };try tick(result.count);return result
 }
 mutating func format(_ ops:[Any],_ captures:[Range<Int>?],_ depth:Int)throws->String{guard depth<=32 else{throw EngineError(message:"Transform format nesting limit exceeded")};var output=""
  for raw in ops{try tick();let value:String;if let literal=raw as? String{value=literal}else{let op=raw as! [Any],index=op[1] as! Int,captured=index<captures.count ? captures[index].map{string($0.lowerBound,$0.upperBound)} ?? "":"";if op[0] as! String=="if"{value=try format(op[captured.isEmpty ? 3:2] as! [Any],captures,depth+1)}else{switch op[2] as! String {
   case "upcase":value=snippetCase(captured,"upper")
   case "downcase":value=snippetCase(captured,"lower")
   case "capitalize":value=captured.unicodeScalars.first.map{snippetCase(String($0),"upper")+captured.unicodeScalars.dropFirst().map(String.init).joined()} ?? ""
   case "camelcase","pascalcase":let words=snippetCaseWords(captured);let camel=op[2] as! String=="camelcase";value=words.isEmpty ? captured:words.enumerated().map{index,word in let scalar=word.unicodeScalars.first!;return (index==0&&camel ? snippetCase(String(scalar),"lower"):snippetCase(String(scalar),"upper"))+word.unicodeScalars.dropFirst().map(String.init).joined()}.joined()
   default:value=captured
  }}};guard value.utf8.count<=1048576-output.utf8.count else{throw EngineError(message:"Transform output limit exceeded")};output+=value};return output
 }
}
func applySnippetTransform(_ program:SnippetTransform,_ input:String,budget:inout SnippetTransformBudget)throws->String {
 if let v3=program.v3{return try applyRegexV3(v3,input,work:&budget.work)}
 guard input.utf8.count<=65536 else{throw EngineError(message:"Transform input limit exceeded")};var evaluator=TransformEvaluator(text:Array(input.unicodeScalars),flags:program.flags,work:budget.work),output="",cursor=0,search=0,matched=false;let count=evaluator.text.count
 while search<=count{var found:(Int,TransformMatch)?;for start in search...count{let initial=TransformMatch(at:start,captures:Array(repeating:nil,count:program.groups+1));if var state=try evaluator.matches(program.regex,initial,0).first{state.captures[0]=start..<state.at;found=(start,state);break}};guard let(start,state)=found else{break};matched=true;let prefix=evaluator.string(cursor,start),replacement=try evaluator.format(program.format,state.captures,0);guard prefix.utf8.count+replacement.utf8.count<=1048576-output.utf8.count else{throw EngineError(message:"Transform output limit exceeded")};output+=prefix+replacement;cursor=state.at;if !program.flags.contains("g"){break};search=state.at==start ? state.at+1:state.at}
 if !matched,program.format.contains(where:{raw in guard let op=raw as? [Any],op.first as? String=="if",let branch=op[3] as? [Any] else{return false};return !branch.isEmpty}){let result=try evaluator.format(program.format,Array(repeating:nil,count:program.groups+1),0);budget.work=evaluator.work;return result}
 let suffix=evaluator.string(cursor,count);guard suffix.utf8.count<=1048576-output.utf8.count else{throw EngineError(message:"Transform output limit exceeded")};budget.work=evaluator.work;return output+suffix
}
