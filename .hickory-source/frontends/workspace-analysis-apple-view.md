<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read workspace analysis without moving code

For maintainers changing workspace services independently of the editor renderer.

Expose acquisition through the existing Edit menu and status bar. The report floats over code and has bounded, filterable pages; starting it never activates a source pane. It names generated and unclassified exclusions and keeps zero-symbol reads distinct from a clean-project claim. Closing the report leaves background work alive; Stop drains admitted reads without starting more files. Opening source remains a separate action through the existing Files or navigation surfaces.

<ui:copy id="simple-workspace-analysis-apple-view">
import SwiftUI
struct WorkspaceAnalysisView: View {
    @Bindable var workspace: Workspace
    @State private var query=""
    @State private var page=0
    var body: some View {
        let model=workspace.analysis
        let reports=model.knowledge.values.filter {query.isEmpty || ($0.path+" "+$0.notice).localizedCaseInsensitiveContains(query)}.sorted {$0.path<$1.path}
        let last=max(0,(reports.count+99)/100-1),selected=min(page,last)
        VStack(alignment:.leading,spacing:8) {
            HStack {Text("Workspace analysis").font(.headline);Spacer();Button("Close"){workspace.showAnalysis=false}}
            Text("\(model.completed)/\(model.scope.files.count) attempted · \(model.knowledge.values.filter {$0.source != nil}.count) sources read · \(model.scope.generated) generated and \(model.scope.unclassified) unclassified excluded.")
            Text("Provider symbols and diagnostics arrive independently. Zero symbols may mean no declarations, unsupported analysis or indexing. This is not verification.").font(.caption)
            HStack {Button("Read workspace"){workspace.startAnalysis()}.disabled(model.running);Button(model.stopping ? "Stopping…":"Stop"){model.stop()}.disabled(!model.running || model.stopping)}
            TextField("Filter analysis",text:$query).onChange(of:query){_,_ in page=0}
            ScrollView {VStack(alignment:.leading,spacing:8) {ForEach(Array(reports.dropFirst(selected*100).prefix(100))) {item in
                Text(item.path).bold();Text("\(item.source != nil && workspace.buffers[item.path]?.text != item.source ? "stale":item.state) · \(item.locations.count) symbols · \(item.notice)").font(.caption)
            }}}
            if last>0 {HStack {Button("Previous results"){page=selected-1}.disabled(selected==0);Text("\(selected+1)/\(last+1)");Button("Next results"){page=selected+1}.disabled(selected==last)}}
        }.padding().frame(minWidth:280,idealWidth:420,maxWidth:480,minHeight:260,idealHeight:400,maxHeight:480)
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/WorkspaceAnalysisView.swift"><ui:paste select="#simple-workspace-analysis-apple-view"/></ui:file>
</ui:doc>
