import Foundation

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
