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
