import Foundation

func validateNativeJSON(_ data:Data,canonicalVersionKeys:Set<String>=[])throws {
    let bytes=Array(data);var objects:[[String:Data]]=[];var index=0
    while index<bytes.count {
        let byte=bytes[index]
        if byte==123{objects.append([:]);index+=1;continue}
        if byte==125{if !objects.isEmpty{objects.removeLast()};index+=1;continue}
        guard byte==34 else{index+=1;continue}
        let start=index;index+=1
        while index<bytes.count {if bytes[index]==92{index+=2;continue};if bytes[index]==34{break};index+=1}
        guard index<bytes.count else{throw EngineError(message:"Malformed provider JSON string.")}
        index+=1;var next=index;while next<bytes.count,[UInt8(9),10,13,32].contains(bytes[next]){next+=1}
        if next<bytes.count,bytes[next]==58,!objects.isEmpty {
            let key=try JSONDecoder().decode(String.self,from:Data(bytes[start..<index])),position=objects.count-1
            guard objects[position][key]==nil else{throw EngineError(message:"Duplicate or canonically equivalent JSON keys are unsupported by this native host. Provider admission was refused without merging contributors.")}
            objects[position][key]=Data(key.utf8)
            if canonicalVersionKeys.contains(key) {
                var value=next+1;while value<bytes.count,[UInt8(9),10,13,32].contains(bytes[value]){value+=1}
                if value<bytes.count,(bytes[value]==45 || (48...57).contains(bytes[value])) {
                    var end=value;while end<bytes.count,![UInt8(9),10,13,32,44,125,93].contains(bytes[end]){end+=1}
                    let token=bytes[value..<end]
                    guard !token.isEmpty,token.allSatisfy({(48...57).contains($0)}),token.count==1 || token.first != 48 else{throw EngineError(message:"Protocol admission requires a canonical integer version; numeric aliases were refused.")}
                }
            }
        }
    }
}

struct NativeSourceKey:Hashable,Sendable {
    let bytes:Data
    init(_ source:String){bytes=Data(source.utf8)}
    var string:String {String(decoding:bytes,as:UTF8.self)}
}
extension String {var nativeSourceKey:NativeSourceKey {NativeSourceKey(self)}}
extension Array where Element==String {
    func containsSource(_ source:String)->Bool {contains{NativeSourceKey($0)==NativeSourceKey(source)}}
    func firstSourceIndex(_ source:String)->Int? {firstIndex{NativeSourceKey($0)==NativeSourceKey(source)}}
}
struct NativeSourceMap<Value>:ExpressibleByDictionaryLiteral,Sequence {
    private var storage:[NativeSourceKey:Value]=[:]
    init(){}
    init<S:Sequence>(_ elements:S) where S.Element==(key:String,value:Value) {for(key,value)in elements{storage[NativeSourceKey(key)]=value}}
    init(dictionaryLiteral elements:(String,Value)...){for(key,value)in elements{storage[NativeSourceKey(key)]=value}}
    subscript(_ key:String)->Value? {get{storage[NativeSourceKey(key)]}set{storage[NativeSourceKey(key)]=newValue}}
    var count:Int {storage.count}
    var isEmpty:Bool {storage.isEmpty}
    var keys:[String] {storage.keys.map(\.string)}
    var values:[Value] {Array(storage.values)}
    @discardableResult mutating func removeValue(forKey key:String)->Value? {storage.removeValue(forKey:NativeSourceKey(key))}
    mutating func removeAll(){storage.removeAll()}
    func makeIterator()->AnyIterator<(key:String,value:Value)> {
        var iterator=storage.makeIterator()
        return AnyIterator{iterator.next().map{(key:$0.key.string,value:$0.value)}}
    }
    func mapValues<T>(_ transform:(Value)throws->T)rethrows->NativeSourceMap<T> {
        var result=NativeSourceMap<T>();for(key,value)in storage{result.storage[key]=try transform(value)};return result
    }
}
/** Utility routes and source addresses are separate identities even when their displayed spelling matches. */
enum NativeUtilityTab:String,Hashable,Sendable {case files,agent,terminal}
enum NativeTabIdentity:Hashable,Sendable {
    case source(NativeSourceKey)
    case utility(NativeUtilityTab)
    case retained(UUID)
    case unavailable(NativeSourceKey)
    init(address:String){
        if let utility=NativeUtilityTab(rawValue:address){self = .utility(utility)}
        else if address.hasPrefix("\0source:") {
            let raw=String(address.dropFirst("\0source:".count))
            if let bytes=Data(base64Encoded:raw),let path=String(data:bytes,encoding:.utf8){self = .source(NativeSourceKey(path))}else{self = .unavailable(NativeSourceKey(address))}
        }else if address.hasPrefix("\0resource:"),let id=UUID(uuidString:String(address.dropFirst("\0resource:".count))){self = .retained(id)}
        else if address.hasPrefix("\0"){self = .unavailable(NativeSourceKey(address))}
        else{self = .source(NativeSourceKey(address))}
    }
    var address:String {switch self {
        case .source(let key):let path=key.string;return NativeUtilityTab(rawValue:path) != nil || path.hasPrefix("\0") ? "\0source:"+key.bytes.base64EncodedString():path
        case .utility(let utility):return utility.rawValue
        case .retained(let id):return "\0resource:"+id.uuidString
        case .unavailable(let key):return key.string
    }}
    var sourcePath:String? {if case .source(let key)=self{return key.string};return nil}
    var utility:NativeUtilityTab? {if case .utility(let utility)=self{return utility};return nil}
}
func nativeSourceTab(_ path:String)->String {NativeTabIdentity.source(NativeSourceKey(path)).address}
extension Array where Element==String {
    func containsTab(_ address:String)->Bool {contains{NativeTabIdentity(address:$0)==NativeTabIdentity(address:address)}}
    func containsUtility(_ utility:NativeUtilityTab)->Bool {contains{NativeTabIdentity(address:$0) == .utility(utility)}}
    func firstTabIndex(_ address:String)->Int? {firstIndex{NativeTabIdentity(address:$0)==NativeTabIdentity(address:address)}}
}
