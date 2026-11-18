import Foundation
import CryptoKit
@MainActor final class SourceDraftHistory {
    private struct ContextEntry {var cost:Int;var restore:()->Void}
    private static var contexts:[UUID:ContextEntry]=[:]
    private static var contextOrder:[UUID]=[]
    private static var contextBytes=0
    private static func hold<Context>(_ value:Context,cost:Int,restore:@escaping (Context)->Void)->UUID {
        let id=UUID(),weight=max(0,cost)
        guard weight<=32*1024*1024 else{return id}
        contexts[id]=ContextEntry(cost:weight,restore:{restore(value)});contextOrder.append(id);contextBytes+=weight
        while contextBytes>32*1024*1024 || contextOrder.count>4096 {
            let old=contextOrder.removeFirst();if let value=contexts.removeValue(forKey:old){contextBytes-=value.cost}
        }
        return id
    }
    private var retirementObservers:[ObjectIdentifier:NSObjectProtocol]=[:]
    private func retire(_ manager:UndoManager) {
        guard manager.isUndoing || manager.isRedoing else{manager.removeAllActions();return}
        let key=ObjectIdentifier(manager);guard retirementObservers[key]==nil else{return}
        let event=manager.isUndoing ? NSNotification.Name.NSUndoManagerDidUndoChange : NSNotification.Name.NSUndoManagerDidRedoChange
        retirementObservers[key]=NotificationCenter.default.addObserver(forName:event,object:manager,queue:nil){[weak self,weak manager] _ in
            MainActor.assumeIsolated {
                guard let self else{return}
                if let token=self.retirementObservers.removeValue(forKey:key){NotificationCenter.default.removeObserver(token)}
                manager?.removeAllActions()
            }
        }
    }
    private struct Step {
        var before:SHA256.Digest;var after:SHA256.Digest;var edits:[NativeSourceEdit];var inverse:[NativeSourceEdit]
        var selectionBefore:NSRange;var selectionAfter:NSRange;var title:String;var contextBefore:UUID;var contextAfter:UUID
        var reversed:Step {Step(before:after,after:before,edits:inverse,inverse:edits,selectionBefore:selectionAfter,selectionAfter:selectionBefore,title:title,contextBefore:contextAfter,contextAfter:contextBefore)}
    }
    typealias Apply=([NativeSourceEdit],String,NSRange)->Bool
    private func identity(_ source:String)->SHA256.Digest{SHA256.hash(data:Data(source.utf8))}
    func commit<Context>(_ plan:ChangePlan,path:String,before:String,selectionBefore:NSRange,selectionAfter:NSRange,contextBefore:Context,contextAfter:Context,manager:UndoManager?,read:@escaping ()->String,apply:@escaping Apply,notice:@escaping (String)->Void,restoreContext:@escaping (Context)->Void,contextCost:(Context)->Int,discardContext:@escaping ()->Void,alreadyObserved:Bool=false)->Bool {
        guard plan.patches.count<=1 else{return false}
        let patch=plan.patches.first
        if let patch{guard exactUTF8(patch.path,path),exactUTF8(patch.before,before) else{return false}}
        if alreadyObserved {
            guard exactUTF8(read(),patch?.after ?? before) else{return false}
            if let patch {
                guard patch.edits.allSatisfy({$0.from>=0 && $0.to>=$0.from && $0.to<=before.utf16.count}) else{return false}
                let edits=patch.edits.map{NativeSourceEdit(range:NSRange(location:$0.from,length:$0.to-$0.from),text:$0.text)}
                guard validSourceEdits(edits,length:before.utf16.count) else{return false}
                let observed=NSMutableString(string:before)
                for edit in edits.reversed(){observed.replaceCharacters(in:edit.range,with:edit.text)}
                guard exactUTF8(observed as String,patch.after) else{return false}
            }
        } else if !exactUTF8(read(),before) {return false}
        let first=Self.hold(contextBefore,cost:contextCost(contextBefore),restore:restoreContext),last=Self.hold(contextAfter,cost:contextCost(contextAfter),restore:restoreContext)
        let step=Step(before:identity(before),after:identity(patch?.after ?? before),edits:(patch?.edits ?? []).map{NativeSourceEdit(range:NSRange(location:$0.from,length:$0.to-$0.from),text:$0.text)},inverse:(invertChangePlan(plan).patches.first?.edits ?? []).map{NativeSourceEdit(range:NSRange(location:$0.from,length:$0.to-$0.from),text:$0.text)},selectionBefore:selectionBefore,selectionAfter:selectionAfter,title:plan.title,contextBefore:first,contextAfter:last)
        if alreadyObserved {
            restoreContext(contextAfter)
        } else {
            guard perform(step,manager:manager,read:read,apply:apply,discardContext:discardContext,forwardContext:{restoreContext(contextAfter)}) else{return false}
        }
        if step.before != step.after,let manager{register(step.reversed,manager:manager,read:read,apply:apply,notice:notice,discardContext:discardContext)}
        return true
    }
    private func perform(_ step:Step,manager:UndoManager?,read:()->String,apply:Apply,discardContext:()->Void,forwardContext:(()->Void)?=nil)->Bool {
        let source=read();guard identity(source)==step.before else{return false}
        manager?.disableUndoRegistration();defer{manager?.enableUndoRegistration()}
        guard apply(step.edits,source,step.selectionAfter),identity(read())==step.after else{return false}
        if let forwardContext{forwardContext()}else if let context=Self.contexts[step.contextAfter]{context.restore()}else{discardContext()}
        return true
    }
    private func register(_ step:Step,manager:UndoManager,read:@escaping ()->String,apply:@escaping Apply,notice:@escaping (String)->Void,discardContext:@escaping ()->Void) {
        let group = !manager.isUndoing && !manager.isRedoing
        // Separate consecutive explicit source actions even in one host event.
        // Leave caller-owned nested groups and undo/redo replay intact.
        if group,manager.groupsByEvent,manager.groupingLevel==1{manager.endUndoGrouping();manager.beginUndoGrouping()}
        if group{manager.beginUndoGrouping()}
        manager.registerUndo(withTarget:self){[weak manager] history in
            guard let manager else{return}
            guard history.perform(step,manager:manager,read:read,apply:apply,discardContext:discardContext) else{
                history.retire(manager);notice("Undo history retired because source or permissions changed. Current draft is retained.");return
            }
            history.register(step.reversed,manager:manager,read:read,apply:apply,notice:notice,discardContext:discardContext)
        }
        manager.setActionName(step.title)
        if group{manager.endUndoGrouping()}
    }
}
