import XCTest
@testable import HickorySimple
@MainActor final class WorkspaceAnalysisTests: XCTestCase {
    private var files: [FileNode] { (0..<5).map {FileNode(name:"\($0).py",path:"\($0).py",dir:false,language:"python")} }
    private func buffer(_ file:FileNode) -> DocumentBuffer {DocumentBuffer(file:file,text:FileText(path:file.path,content:"source",hash:"base"))}
    func testDrainBeforeRestart() async {
        let model=WorkspaceAnalysis();var loads=0;var active=0;var peak=0;var held:[CheckedContinuation<Void,Never>]=[]
        let entered=expectation(description:"two bounded reads")
        let task=Task {await model.run(files,load:{file in loads += 1;return self.buffer(file)},outline:{_,_ in
            active += 1;peak=max(peak,active);if active==2 {entered.fulfill()}
            await withCheckedContinuation {held.append($0)};active -= 1;return []
        })}
        await fulfillment(of:[entered],timeout:2);model.stop();XCTAssertTrue(model.running)
        let restart=await model.run(files,load:{_ in XCTFail("must not load");throw CancellationError()},outline:{_,_ in []});XCTAssertFalse(restart)
        held.forEach {$0.resume()};_ = await task.value
        XCTAssertEqual(loads,2);XCTAssertEqual(peak,2);XCTAssertEqual(model.completed,2);XCTAssertFalse(model.running)
    }
    func testStaleAndUnavailableAreIndependent() async {
        let model=WorkspaceAnalysis(),draft=buffer(files[0]);var reads=0
        _ = await model.run(files,load:{file in file.path=="0.py" ? draft:self.buffer(file)},outline:{file,_ in
            reads += 1;if file.path=="0.py" {draft.text="draft"};if file.path=="1.py" {throw EngineError(message:"provider unavailable")};return []
        })
        XCTAssertEqual(reads,5);XCTAssertEqual(model.completed,5);XCTAssertEqual(draft.text,"draft");XCTAssertEqual(model.knowledge["0.py"]?.state,"stale");XCTAssertNil(model.current("0.py",source:"source"))
        XCTAssertEqual(model.knowledge["1.py"]?.notice,"provider unavailable");XCTAssertNotNil(model.current("2.py",source:"source"));XCTAssertNil(model.current("2.py",source:"changed"))
    }
}
