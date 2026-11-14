<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Search within the existing native shell

For maintainers changing workspace services independently of the editor renderer.

The floating search reads current workspace targets and uses existing file/source navigation. It never claims that an empty provider index proves an absent symbol. Read workspace starts the same bounded acquisition as the analysis report without activating files. Only source-current symbol results are selectable; command reasons stay visible. Keep the entry in existing menus and keyboard handling, with no new toolbar or code-area allocation.

<ui:copy id="simple-search-everywhere-apple-view">
import SwiftUI
struct SearchEverywhereView:View {
    @Bindable var workspace:Workspace
    var commands:[WorkspaceCommand]
    @State private var query=""
    @State private var kind="all"
    @State private var selected=0
    @FocusState private var typing:Bool
    var body:some View {
        let files=workspace.rootFiles+workspace.buffers.values.map(\.file)
        let targets=everywhereTargets(files,knowledge:Array(workspace.analysis.knowledge.values),current:{workspace.buffers[$0]?.text},commands:commands)
        let result=searchEverywhere(targets,query:query,kind:kind)
        let index=min(selected,max(0,result.items.count-1))
        VStack(alignment:.leading,spacing:8) {
            HStack {Text("Search Everywhere").font(.headline);Spacer();Button("Close"){workspace.showEverywhere=false;workspace.currentBuffer?.focusSource?()}}
            TextField("Search files, types, members and actions",text:$query).focused($typing).onSubmit {if result.items.indices.contains(index),result.items[index].reason==nil {Task {await workspace.openEverywhere(result.items[index],commands:commands)}}}
                .onKeyPress(.downArrow) {selected=min(index+1,max(0,result.items.count-1));return .handled}
                .onKeyPress(.upArrow) {selected=max(0,index-1);return .handled}
            Picker("Search scope",selection:$kind){Text("All").tag("all");Text("Files").tag("file");Text("Types").tag("type");Text("Members").tag("member");Text("Other symbols").tag("symbol");Text("Actions").tag("action")}
            HStack {Text("\(result.items.count) of \(result.total) matches").font(.caption);Spacer();Button(workspace.analysis.running ? "Reading…":"Read workspace"){workspace.startAnalysis()}.disabled(workspace.analysis.running)}
            Text("Symbols reflect source-current provider reads; missing results may need analysis or language support. Files and actions work independently.").font(.caption)
            if SimpleCommandCatalog.descriptors.isEmpty {Text("Command catalog missing. Reinstall the app; source files remain readable.").font(.caption)}
            ScrollViewReader {proxy in ScrollView {VStack(alignment:.leading,spacing:4){ForEach(Array(result.items.enumerated()),id:\.element.id){at,target in
                VStack(alignment:.leading,spacing:2) {
                    Button {Task {await workspace.openEverywhere(target,commands:commands)}} label:{VStack(alignment:.leading){Text(target.label);Text(target.kind+" · "+target.detail).font(.caption)}}.disabled(target.reason != nil).buttonStyle(.plain).padding(.vertical,4).background(at==index ? Color.accentColor.opacity(0.12):Color.clear).accessibilityValue(at==index ? "Selected":"")
                    if let reason=target.reason {Text(reason).font(.caption)}
                }.frame(maxWidth:.infinity,alignment:.leading).id(target.id)
            }}}.onChange(of:selected){_,_ in if result.items.indices.contains(index) {proxy.scrollTo(result.items[index].id,anchor:.center)}}}
        }.onChange(of:query){_,_ in selected=0}.onChange(of:kind){_,_ in selected=0}.padding().frame(minWidth:280,idealWidth:480,maxWidth:560,minHeight:280,idealHeight:440,maxHeight:520).onAppear{typing=true}
        .onKeyPress(.escape) {workspace.showEverywhere=false;workspace.currentBuffer?.focusSource?();return .handled}
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/SearchEverywhereView.swift"><ui:paste select="#simple-search-everywhere-apple-view"/></ui:file>
</ui:doc>
