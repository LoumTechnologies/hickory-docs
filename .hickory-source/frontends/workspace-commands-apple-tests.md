<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Recheck commands after context changes

For maintainers changing workspace services independently of the editor renderer.

A search result may outlive the workspace state that enabled it. Resolve Save once, change the state and invoke that retained command: it must refuse while saving, read-only or without an editor, then use current eligibility when valid again. Unknown requirements and missing adapters must not become enabled. Native witnesses load the same bundled descriptors as production, catching packaging failures without opening an editor.

<ui:copy id="simple-workspace-commands-apple-tests">
import XCTest
@testable import HickorySimple
@MainActor final class WorkspaceCommandsTests: XCTestCase {
    func testBundledCommandsRecheckContext() throws {
        XCTAssertFalse(SimpleCommandCatalog.descriptors.isEmpty,"Rebuild the bundled command descriptors.")
        var context=CommandContext(editor:true,writable:true,dirty:true,saving:false,back:false,forward:false,tab:true,groups:1),called=0
        let commands=workspaceCommands({context},handlers:["file.save":{called += 1}])
        let save=try XCTUnwrap(commands.first {$0.id=="file.save"});XCTAssertNil(save.reason)
        context.saving=true;save.run();context.saving=false;context.writable=false;save.run();context.writable=true;context.editor=false;save.run();XCTAssertEqual(called,0)
        context.editor=true;save.run();XCTAssertEqual(called,1);XCTAssertNotNil(commands.first {$0.id=="file.open"}?.reason)
        XCTAssertNotNil(commandReason("unknown",context));XCTAssertNotNil(commandReason("back",context));XCTAssertNotNil(commandReason("multi",context))
    }
}
</ui:copy>

<ui:file path="apps/apple-simple/Tests/WorkspaceCommandsTests.swift"><ui:paste select="#simple-workspace-commands-apple-tests"/></ui:file>
</ui:doc>
