import Foundation
/** Linguistic equivalence cannot establish a code or protocol-byte revision. */
func exactUTF8(_ lhs:String?,_ rhs:String?)->Bool {
    switch (lhs,rhs) {
    case (.none,.none):return true
    case (.some(let lhs),.some(let rhs)):
        if let equal=lhs.utf8.withContiguousStorageIfAvailable({left in
            rhs.utf8.withContiguousStorageIfAvailable {right in
                guard left.count==right.count else {return false}
                if left.baseAddress==right.baseAddress {return true}
                return left.elementsEqual(right)
            }
        }).flatMap({$0}) {return equal}
        return lhs.utf8.elementsEqual(rhs.utf8)
    default:return false
    }
}
/** Observation must publish byte-distinct text even when String equality says equal. */
struct ExactSourceText:Equatable,Sendable {
    let text:String
    init(_ text:String){self.text=text}
    static func ==(lhs:Self,rhs:Self)->Bool {exactUTF8(lhs.text,rhs.text)}
}
func exactSourceValues(_ values:[String:String])->[Data:ExactSourceText] {
    Dictionary(uniqueKeysWithValues:values.map {(Data($0.key.utf8),ExactSourceText($0.value))})
}
