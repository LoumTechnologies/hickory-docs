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
