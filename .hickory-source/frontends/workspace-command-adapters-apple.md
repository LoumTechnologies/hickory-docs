<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Adapt shared commands to native workspace operations

For maintainers changing workspace services independently of the editor renderer.

Bind command IDs to existing workspace operations at the UI boundary. Source/save/refactoring methods remain the authoritative implementations. A fresh context getter reads the current buffer and navigation trail when a retained command executes. Window and connection actions are supplied by the scene/activity rather than stored in the workspace, avoiding an activity lifetime or window API in the model. The same command values feed menus and Search Everywhere.

<ui:copy id="simple-workspace-command-adapters-apple">
import Foundation
extension Workspace {
    func commands(ui:[String:()->Void]=[:])->[WorkspaceCommand] {
        var handlers:[String:()->Void]=[
            "file.open":{self.chooseFileCommand("Open File")},"file.directory":{self.chooseFileCommand("Open Directory")},"file.new":{self.newTab()},
            "file.save":{Task {await self.save()}},"file.saveAs":{self.chooseFileCommand("Save As")},"file.saveAll":{Task {await self.saveAll()}},"file.refresh":{Task {await self.refresh()}},"file.close":{if let tab=self.active {self.close(tab)}},
            "shell.search":{self.showEverywhere=true},"edit.find":{self.currentBuffer?.showFind=true},"workspace.find":{self.showWorkspaceSearch=true},
            "symbol.rename":{self.changes.open("rename",workspace:self)},"symbol.actions":{self.changes.open("actions",workspace:self)},"document.format":{self.changes.open("format",workspace:self)},
            "workspace.undo":{self.changes.open("undo",workspace:self)},"workspace.redo":{self.changes.open("redo",workspace:self)},
            "document.attribution":{if let path=self.currentBuffer?.file.path{self.gitAttribution.close();self.gitAttribution=GitAttributionModel(path:path);self.showReview=false;self.showHistory=false;self.showAttribution=true}},"workspace.review":{self.showAttribution=false;self.showHistory=false;self.showReview=true},"workspace.history":{self.showAttribution=false;self.gitHistory.close();self.gitHistory=GitHistoryModel(request:.log(nil));self.showReview=false;self.showHistory=true},"document.history":{self.showAttribution=false;self.gitHistory.close();self.gitHistory=GitHistoryModel(request:.log(self.currentBuffer?.file.path));self.showReview=false;self.showHistory=true},"workspace.coverage":{self.showCoverage=true},"workspace.construction":{self.showConstruction=true},"workspace.tests":{self.showTests=true},"workspace.problems":{self.showProblems=true},"workspace.analysis":{self.showAnalysis=true},"problem.next":{Task {await self.navigateProblem(1)}},"problem.previous":{Task {await self.navigateProblem(-1)}},
            "navigation.back":{Task {await self.navigateHistory(-1)}},"navigation.forward":{Task {await self.navigateHistory(1)}},
            "editor.tools":{Task {await self.currentBuffer?.language?.inspect()};Task {await self.currentBuffer?.language?.complete()}},
            "editor.undo":{_ = self.currentBuffer?.undoSource?()},"editor.redo":{_ = self.currentBuffer?.redoSource?()},
            "editor.snippetNext":{_ = self.currentBuffer?.snippets.move(1)},"editor.snippetPrevious":{_ = self.currentBuffer?.snippets.move(-1)},"editor.snippetFinish":{_ = self.currentBuffer?.snippets.finish()},"editor.snippetChoices":{self.currentBuffer?.showSnippetTools=true},
            "editor.rectangle":{self.currentBuffer?.selectRectangle()},"editor.cursorBelow":{self.currentBuffer?.addCursor(1)},"editor.cursorAbove":{self.currentBuffer?.addCursor(-1)},
            "view.files":{self.show("files")},"view.agent":{self.show("agent")},"view.terminal":{Task {await self.openTerminal()}},"view.splitRight":{self.split("horizontal")},"view.splitBelow":{self.split("vertical")},"view.move":{self.moveToNext()}
        ]
        for query in SemanticQuery.allCases {handlers["semantic."+query.rawValue]={Task {await self.currentBuffer?.language?.navigate(query)}}}
        handlers.merge(ui){_,new in new}
        return workspaceCommands({CommandContext(editor:self.currentBuffer != nil,writable:self.currentBuffer?.readOnly==false,dirty:self.currentBuffer?.dirty ?? false,saving:self.currentBuffer?.saving ?? false,back:self.navigationTrail.canBack,forward:self.navigationTrail.canForward,tab:self.active != nil,groups:self.layout.groups().count,snippet:self.currentBuffer?.snippets.active != nil,snippetChoices:self.currentBuffer?.snippets.choices.isEmpty==false,sourceUndo:self.currentBuffer?.canUndoSource?()==true,sourceRedo:self.currentBuffer?.canRedoSource?()==true)},handlers:handlers)
    }
    func openEverywhere(_ target:SearchTarget,commands:[WorkspaceCommand]) async {
        if let id=target.commandID {
            guard let command=commands.first(where:{$0.id==id}),command.reason==nil else {status=target.reason ?? "Command unavailable. Refresh Search Everywhere.";return}
            showEverywhere=false;command.run();return
        }
        guard let path=target.path else {return}
        if let location=target.location,let source=target.source {
            guard analysis.current(path,source:source) != nil,buffers[path]?.text==source else {status="Symbol source changed. Read the workspace again.";return}
            await openSemanticLocation(location,allowDirty:true,expectedSource:source);showEverywhere=false;return
        }
        func find(_ files:[FileNode])->FileNode? {for file in files {if file.path==path {return file};if let value=find(file.children ?? []) {return value}};return nil}
        guard let file=buffers[path]?.file ?? find(rootFiles),!file.dir else {status="File no longer listed. Refresh the workspace.";return}
        await open(file);showEverywhere=false
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/WorkspaceCommandAdapters.swift"><ui:paste select="#simple-workspace-command-adapters-apple"/></ui:file>
</ui:doc>
