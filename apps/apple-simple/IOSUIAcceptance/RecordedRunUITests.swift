import XCTest

final class RecordedRunUITests: XCTestCase {
    @MainActor func testRecordedUserAndAssistantRunsShowTerminalWithoutChangingSource() throws {
        continueAfterFailure = false
        let origin = try XCTUnwrap(ProcessInfo.processInfo.environment["HICKORY_IOS_PARITY_ORIGIN"])
        let app = XCUIApplication()
        app.launchEnvironment["HICKORY_API_ORIGIN"] = origin
        app.launch()

        let view = app.buttons["View"].firstMatch
        XCTAssertTrue(view.waitForExistence(timeout: 20))
        view.tap()
        app.buttons["Agent"].firstMatch.tap()
        let conversation = app.scrollViews["Conversation"].firstMatch
        XCTAssertTrue(conversation.waitForExistence(timeout: 20))

        let cases = [("user", "user", "print(value + 1)", "value = 41", "42"),
                     ("assistant", "agent", "print(value + 1)", "value = 52", "53")]
        for (side, buffer, statement, input, expected) in cases {
            let card = conversation.descendants(matching: .any)["Recorded \(side) message"].firstMatch
            let editor = card.textViews["hick-agent-message:///recorded-fence-host-\(buffer).md"].firstMatch
            let title = "Run recorded python fence on line 3"
            let run = card.buttons[title].firstMatch
            let overflow = card.buttons["More gutter icons on line 3"].firstMatch
            for _ in 0..<5 where !run.isHittable && !overflow.isHittable {
                if !editor.exists { _ = editor.waitForExistence(timeout: 3) }
                if run.isHittable || overflow.isHittable { break }
                let start = conversation.coordinate(withNormalizedOffset: CGVector(dx: 0.02, dy: 0.78))
                let end = conversation.coordinate(withNormalizedOffset: CGVector(dx: 0.02, dy: 0.28))
                start.press(forDuration: 0.04, thenDragTo: end)
            }
            XCTAssertTrue(editor.exists, "Recorded \(side) source is unavailable")
            let before = try XCTUnwrap(editor.value as? String)
            XCTAssertTrue(before.contains(statement) && before.contains(input))
            let frame = editor.frame
            if run.isHittable {
                run.tap()
            } else {
                XCTAssertTrue(overflow.isHittable, "Recorded \(side) Run gutter is unavailable")
                overflow.tap()
                let more = app.descendants(matching: .any)["More actions on line 3"].firstMatch
                let revealed = more.buttons[title].firstMatch
                XCTAssertTrue(revealed.waitForExistence(timeout: 10))
                revealed.tap()
            }

            let terminal = app.descendants(matching: .any)["Recorded fence terminal"].firstMatch
            XCTAssertTrue(terminal.waitForExistence(timeout: 20))
            let web = terminal.webViews.firstMatch
            XCTAssertTrue(web.waitForExistence(timeout: 15))
            XCTAssertFalse(web.frame.isEmpty)
            let line = "\\s*\(expected)\\s*"
            let output = web.descendants(matching: .any).matching(NSPredicate(format: "label MATCHES %@ OR value MATCHES %@", line, line)).firstMatch
            XCTAssertTrue(output.waitForExistence(timeout: 15), "The \(side) terminal did not expose output \(expected)")
            XCTAssertTrue(terminal.staticTexts["Exit code 0"].waitForExistence(timeout: 15))
            let screenshot = XCTAttachment(screenshot: XCUIScreen.main.screenshot())
            screenshot.name = "Recorded \(side) Run output \(expected)"
            screenshot.lifetime = .keepAlways
            add(screenshot)
            terminal.buttons["Close"].tap()
            XCTAssertTrue(terminal.waitForNonExistence(timeout: 10))
            XCTAssertEqual(editor.value as? String, before)
            XCTAssertEqual(editor.frame.integral, frame.integral)
            NSLog("HickoryUIKitParity recorded-run-\(side)-terminal-kept-editor")
        }
    }

    @MainActor func testRecordedConstructionShowsOnlyTheSelectedLocalTransition() throws {
        continueAfterFailure = false
        let environment = ProcessInfo.processInfo.environment
        let origin = try XCTUnwrap(environment["HICKORY_IOS_PARITY_ORIGIN"])
        let side = try XCTUnwrap(environment["HICKORY_IOS_CONSTRUCTION_SIDE"])
        XCTAssertTrue(side == "user" || side == "assistant")
        let buffer = side == "user" ? "user" : "agent"
        let document = side == "user" ? "request.md" : "response.md"
        let app = XCUIApplication()
        app.launchEnvironment["HICKORY_API_ORIGIN"] = origin
        app.launch()
        let view = app.buttons["View"].firstMatch
        XCTAssertTrue(view.waitForExistence(timeout: 20))
        view.tap()
        app.buttons["Agent"].firstMatch.tap()
        let conversation = app.scrollViews["Conversation"].firstMatch
        XCTAssertTrue(conversation.waitForExistence(timeout: 20))
        let card = conversation.descendants(matching: .any)["Recorded \(side) message"].firstMatch
        let editor = card.textViews["hick-agent-message:///declared-construction-\(buffer).md"].firstMatch
        XCTAssertTrue(editor.waitForExistence(timeout: 15))
        let source = try XCTUnwrap(editor.value as? String)
        XCTAssertTrue(source.contains("<hick:patch id=\"selected\">"))
        let frame = editor.frame
        let action = "Inspect recorded construction patch on line 3"
        let inspect = card.buttons[action].firstMatch
        if inspect.isHittable {
            inspect.tap()
        } else {
            let overflow = card.buttons["More gutter icons on line 3"].firstMatch
            XCTAssertTrue(overflow.waitForExistence(timeout: 15))
            overflow.tap()
            app.buttons[action].firstMatch.tap()
        }
        let reading = app.descendants(matching: .any)["Recorded construction reading"].firstMatch
        XCTAssertTrue(reading.waitForExistence(timeout: 20))
        XCTAssertTrue(reading.staticTexts["\(document) · selected"].waitForExistence(timeout: 10))
        let output = reading.descendants(matching: .any)["materialized.txt"].firstMatch
        XCTAssertTrue(output.waitForExistence(timeout: 15))
        output.tap()
        let row = { (text: String) in reading.staticTexts.matching(NSPredicate(format: "label MATCHES %@", "\\s*\(text)\\s*")).firstMatch }
        XCTAssertTrue(row("old").waitForExistence(timeout: 10))
        XCTAssertTrue(row("middle").waitForExistence(timeout: 10))
        XCTAssertFalse(row("final").exists)
        XCTAssertFalse(reading.buttons.matching(NSPredicate(format: "label MATCHES %@", "(?i).*(Apply|Save|Run).*")).firstMatch.exists)
        let screenshot = XCTAttachment(screenshot: XCUIScreen.main.screenshot())
        screenshot.name = "Recorded construction patch Before and After"
        screenshot.lifetime = .keepAlways
        add(screenshot)
        reading.buttons["Close"].tap()
        XCTAssertTrue(reading.waitForNonExistence(timeout: 10))
        XCTAssertEqual(editor.value as? String, source)
        XCTAssertEqual(editor.frame.integral, frame.integral)
        NSLog("HickoryUIKitParity recorded-construction-\(side)-read-immutable-local-transition")
    }
}
