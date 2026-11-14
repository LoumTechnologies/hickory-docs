<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Inspect source tests from the existing gutter on apple

For maintainers changing workspace services independently of the editor renderer.

The editor supplies text immediately; a separate bounded source index reads test declarations in the background. A source-bound hint uses the existing right-hand action column and overflow grid. Clicking it activates the existing buffer, then refreshes the full floating test view with current drafts before offering execution. Cached gutter facts never authorize a run. Other saved construction contributors can differ from retained drafts, so this fresh inspection is required even when the primary text still matches.

<ui:copy id="source-test-gutters-apple">
import Foundation
extension DocumentBuffer {
    func readTestSources(){testIndex?.enqueue(file.path,source:text,current:{[weak self] in self?.text})}
    func sourceTestRows()->[(line:Int,candidate:SourceTestCandidate)] {indexedTestRows(testIndex?.current(file.path,source:text))}
}
extension Workspace {
    func inspectTestSource(_ buffer:DocumentBuffer,line:Int) async {
        let rows=buffer.sourceTestRows().filter {$0.line==line}
        guard !rows.isEmpty else {status="Source changed. Wait for test inspection or use Source tests in Edit.";return}
        let index=sourceTestIndex,source=buffer.text
        await open(buffer.file)
        guard sourceTestIndex === index,buffer.text==source,currentBuffer === buffer else {return}
        let model=testSources
        model.query=rows.count==1 ? rows[0].candidate.name:"";showTests=true
        await model.refresh(self)
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Sources/SourceTestGutters.swift"><ui:paste select="#source-test-gutters-apple"/></ui:file>
</ui:doc>
