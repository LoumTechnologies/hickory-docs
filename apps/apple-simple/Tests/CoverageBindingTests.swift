import XCTest
@testable import HickorySimple
@MainActor final class CoverageBindingTests:XCTestCase {
    func testMixedOwnerMarksRetireImmediatelyAfterExactByteEdit() async throws {
        let source="// é\nvalue\n",path="owner.swift",hash=nativeSourceHash(source)
        let record:[String:Any]=["id":"measured","source_changed_during_run":false,"identity":["source_hashes":[path:hash,"generated.swift":String(repeating:"a",count:64)]],"evidence":["report":["files":["generated.swift":["line_counts":false,"lines":["1":1,"2":0]]]]],"mappings":["generated.swift":["1":["path":path,"line":2],"2":["path":path,"line":2]]]]
        let configuration=URLSessionConfiguration.ephemeral;configuration.protocolClasses=[ResponseProtocol.self]
        ResponseProtocol.status=200;ResponseProtocol.responsesByPath=[:];ResponseProtocol.beforeResponse=nil;ResponseProtocol.response=try JSONSerialization.data(withJSONObject:record)
        let session=URLSession(configuration:configuration);defer{session.invalidateAndCancel()}
        let workspace=Workspace();workspace.engine=try Engine(address:"http://coverage.test",session:session)
        let buffer=DocumentBuffer(file:FileNode(name:path,path:path,dir:false),text:FileText(path:path,content:source,hash:hash));buffer.coverageSource=workspace.coverage;workspace.buffers[path]=buffer
        let state=workspace.coverage;defer{state.stop()};state.select("measured",workspace:workspace)
        func wait(_ condition:()->Bool)async {let end=Date().addingTimeInterval(2);while !condition()&&Date()<end{try? await Task.sleep(for:.milliseconds(2))}}
        await wait{!buffer.coverageRows().isEmpty};let row=try XCTUnwrap(buffer.coverageRows().first)
        XCTAssertEqual(row.line,1);XCTAssertEqual(row.state,"mixed");XCTAssertNil(row.count);XCTAssertEqual(row.origins.count,2)
        buffer.text="// e\u{301}\nvalue\n";XCTAssertTrue(buffer.dirty);XCTAssertTrue(buffer.coverageRows().isEmpty)
        buffer.readCoverageSource();await wait{state.digests.current(path,source:buffer.text) != nil};XCTAssertTrue(buffer.coverageRows().isEmpty)
        buffer.text=source;buffer.readCoverageSource();await wait{!buffer.coverageRows().isEmpty};XCTAssertEqual(buffer.coverageRows().first?.state,"mixed")
        state.select(nil,workspace:workspace);XCTAssertTrue(buffer.coverageRows().isEmpty)
    }
}
