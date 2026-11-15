package com.hickorydocs.simple
fun Workspace.commands(descriptors:List<CommandDescriptor>,ui:Map<String,()->Unit> = emptyMap()):List<WorkspaceCommand> {
    val handlers=mutableMapOf<String,()->Unit>(
        "file.open" to {chooseFileCommand("Open File")},"file.directory" to {chooseFileCommand("Open Directory")},"file.new" to {newTab()},
        "file.save" to {runCommand {save()}},"file.saveAs" to {chooseFileCommand("Save As")},"file.saveAll" to {runCommand {saveAll()}},"file.refresh" to {runCommand {refresh()}},"file.close" to {active?.let {close(it)};Unit},
        "shell.search" to {showEverywhere=true},"edit.find" to {currentBuffer?.showFind=true;Unit},"workspace.find" to {showWorkspaceSearch=true},
        "symbol.rename" to {changes.open("rename",this)},"symbol.actions" to {changes.open("actions",this)},"document.format" to {changes.open("format",this)},
        "workspace.undo" to {changes.open("undo",this)},"workspace.redo" to {changes.open("redo",this)},"workspace.coverage" to {showCoverage=true},"workspace.tests" to {showTests=true},"workspace.problems" to {showProblems=true},"workspace.analysis" to {showAnalysis=true},
        "problem.next" to {runCommand {navigateProblem(1)}},"problem.previous" to {runCommand {navigateProblem(-1)}},"navigation.back" to {runCommand {navigateHistory(-1)}},"navigation.forward" to {runCommand {navigateHistory(1)}},
        "editor.undo" to {currentBuffer?.snippets?.undoSource();Unit},"editor.redo" to {currentBuffer?.snippets?.redoSource();Unit},
        "editor.snippetNext" to {currentBuffer?.snippets?.move(1);Unit},"editor.snippetPrevious" to {currentBuffer?.snippets?.move(-1);Unit},"editor.snippetFinish" to {currentBuffer?.snippets?.finish();Unit},"editor.snippetChoices" to {currentBuffer?.showSnippetTools=true;Unit},
        "editor.tools" to {runCommand {currentBuffer?.language?.inspect()};runCommand {currentBuffer?.language?.complete()}},"editor.rectangle" to {currentBuffer?.selectRectangle();Unit},"editor.cursorBelow" to {currentBuffer?.addCursor(1);Unit},"editor.cursorAbove" to {currentBuffer?.addCursor(-1);Unit},
        "view.files" to {show("files")},"view.agent" to {show("agent")},"view.terminal" to {runCommand {openTerminal()}},"view.splitRight" to {split("horizontal")},"view.splitBelow" to {split("vertical")},"view.move" to {moveNext()}
    )
    for(query in SemanticQuery.entries)handlers["semantic."+query.name.replaceFirstChar {it.lowercase()}]={runCommand {currentBuffer?.language?.navigate(query)}}
    handlers.putAll(ui)
    return workspaceCommands({CommandContext(currentBuffer!=null,currentBuffer?.readOnly==false,currentBuffer?.dirty ?: false,currentBuffer?.saving ?: false,navigationTrail.canBack,navigationTrail.canForward,active!=null,layout.groups().size,currentBuffer?.snippets?.active!=null,currentBuffer?.snippets?.choices?.isNotEmpty()==true,currentBuffer?.snippets?.canUndo==true,currentBuffer?.snippets?.canRedo==true)},handlers,descriptors)
}
suspend fun Workspace.openEverywhere(target:SearchTarget,commands:List<WorkspaceCommand>) {
    target.commandID?.let {id->val command=commands.firstOrNull {it.id==id};if(command==null||command.reason!=null){status=target.reason ?: "Command unavailable. Refresh Search Everywhere.";return};showEverywhere=false;command.run();return}
    val path=target.path ?: return;val location=target.location;val source=target.source
    if(location!=null && source!=null) {
        if(analysis.current(path,source)==null||buffers[path]?.text!=source){status="Symbol source changed. Read the workspace again.";return}
        openSemanticLocation(location,allowDirty=true,expectedSource=source);showEverywhere=false;return
    }
    fun find(files:List<FileNode>):FileNode? {for(file in files){if(file.path==path)return file;find(file.children)?.let {return it}};return null}
    val file=(buffers[path]?.file ?: find(rootFiles))?.takeUnless {it.dir} ?: run {status="File no longer listed. Refresh the workspace.";return}
    open(file);showEverywhere=false
}
