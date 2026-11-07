<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Execute the shared pane scenarios in Swift

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

These cases consume the same JSON as the browser tests. They check user-visible tab membership and active-tab validity independently of view framework and internal split identifiers.

<ui:copy id="simple-7d7b2a4b09-0">
// Contract: .hickory-source/frontends/contract.md
import XCTest
@testable import HickorySimple

final class ParityTests: XCTestCase {
    struct Fixture: Decodable { var cases: [Scenario] }
    struct Scenario: Decodable { var name: String; var steps: [Step]; var expected: [[String]] }
    struct Step: Decodable { var op: String; var group: Int?; var tab: String?; var axis: String?; var id: String? }
    func testSharedLayoutScenarios() throws {
        let apps = URL(fileURLWithPath: #filePath).deletingLastPathComponent().deletingLastPathComponent().deletingLastPathComponent()
        let data = try Data(contentsOf: apps.appendingPathComponent("simple-contract/layout-cases.json"))
        for scenario in try JSONDecoder().decode(Fixture.self, from: data).cases {
            var layout = PaneLayout()
            for step in scenario.steps {
                let group = step.group.map { layout.groups()[$0].id } ?? ""
                switch step.op {
                case "open": layout.activate(step.tab!, in: group)
                case "move": layout.move(step.tab!, to: group)
                case "remove": layout.remove(step.tab!)
                case "split": _ = layout.split(group, axis: step.axis!, newID: step.id!)
                default: XCTFail("Unknown shared operation: \(step.op)")
                }
            }
            XCTAssertEqual(layout.groups().map(\.tabs), scenario.expected, scenario.name)
            for group in layout.groups() { XCTAssertTrue(group.active == nil || group.tabs.contains(group.active!), scenario.name) }
        }
    }
    func testInvalidMoveRetainsTab() {
        var layout = PaneLayout(); layout.activate("notes.md", in: "document-group")
        layout.move("notes.md", to: "missing")
        XCTAssertEqual(layout.nodes["document-group"]?.tabs, ["notes.md"])
    }
    func testResizeBounds() {
        var layout = PaneLayout(); layout.resize("root", ratio: 10); XCTAssertEqual(layout.nodes["root"]?.ratio, 0.9)
        layout.resize("root", ratio: -10); XCTAssertEqual(layout.nodes["root"]?.ratio, 0.1)
    }
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/apple-simple/Tests/ParityTests.swift"><ui:paste select="#simple-7d7b2a4b09-0"/></ui:file>

</ui:doc>
