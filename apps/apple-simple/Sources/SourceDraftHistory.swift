import Foundation
@MainActor final class SourceDraftHistory {
    private var retirementObservers:[ObjectIdentifier:NSObjectProtocol]=[:]
    private func retire(_ manager:UndoManager) {
        guard manager.isUndoing || manager.isRedoing else{manager.removeAllActions();return}
        let key=ObjectIdentifier(manager)
        guard retirementObservers[key]==nil else{return}
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
        var before:String;var after:String;var edits:[NativeSourceEdit];var inverse:[NativeSourceEdit]
        var selectionBefore:NSRange;var selectionAfter:NSRange;var title:String
        var reversed:Step {Step(before:after,after:before,edits:inverse,inverse:edits,selectionBefore:selectionAfter,selectionAfter:selectionBefore,title:title)}
    }
    typealias Apply=([NativeSourceEdit],String,NSRange)->Bool
    func commit(_ plan:ChangePlan,path:String,before:String,selectionBefore:NSRange,selectionAfter:NSRange,manager:UndoManager?,read:@escaping ()->String,apply:@escaping Apply,notice:@escaping (String)->Void)->Bool {
        guard plan.patches.count<=1,exactUTF8(read(),before) else{return false}
        let patch=plan.patches.first
        if let patch{guard exactUTF8(patch.path,path),exactUTF8(patch.before,before) else{return false}}
        let step=Step(before:before,after:patch?.after ?? before,edits:(patch?.edits ?? []).map{NativeSourceEdit(range:NSRange(location:$0.from,length:$0.to-$0.from),text:$0.text)},inverse:(invertChangePlan(plan).patches.first?.edits ?? []).map{NativeSourceEdit(range:NSRange(location:$0.from,length:$0.to-$0.from),text:$0.text)},selectionBefore:selectionBefore,selectionAfter:selectionAfter,title:plan.title)
        guard perform(step,manager:manager,read:read,apply:apply) else{return false}
        if !exactUTF8(step.before,step.after),let manager{register(step.reversed,manager:manager,read:read,apply:apply,notice:notice)}
        return true
    }
    private func perform(_ step:Step,manager:UndoManager?,read:()->String,apply:Apply)->Bool {
        guard exactUTF8(read(),step.before) else{return false}
        manager?.disableUndoRegistration();defer{manager?.enableUndoRegistration()}
        return apply(step.edits,step.before,step.selectionAfter) && exactUTF8(read(),step.after)
    }
    private func register(_ step:Step,manager:UndoManager,read:@escaping ()->String,apply:@escaping Apply,notice:@escaping (String)->Void) {
        let group = !manager.isUndoing && !manager.isRedoing
        if group{manager.beginUndoGrouping()}
        manager.registerUndo(withTarget:self){[weak manager] history in
            guard let manager else{return}
            guard history.perform(step,manager:manager,read:read,apply:apply) else{
                history.retire(manager);notice("Undo history retired because source or permissions changed. Current draft is retained.");return
            }
            history.register(step.reversed,manager:manager,read:read,apply:apply,notice:notice)
        }
        manager.setActionName(step.title)
        if group{manager.endUndoGrouping()}
    }
}
