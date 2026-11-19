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
extension RecordedRunUITests {
    @MainActor func testNativeAgentSendAppliesBothDraftsAndReplaysHistory() throws {
        continueAfterFailure = false
        let environment = ProcessInfo.processInfo.environment
        let origin = try XCTUnwrap(environment["HICKORY_IOS_PARITY_ORIGIN"])
        let prompt = try XCTUnwrap(environment["HICKORY_IOS_PROPOSAL_PROMPT"])
        let app = XCUIApplication()
        app.launchEnvironment["HICKORY_API_ORIGIN"] = origin
        app.launchEnvironment["HICKORY_OPEN_FILE"] = "note.py"
        app.launch()

        func menu(_ name: String, _ action: String) {
            let menu = app.buttons[name].firstMatch
            XCTAssertTrue(menu.waitForExistence(timeout: 15), name)
            menu.tap()
            let item = app.buttons[action].firstMatch
            XCTAssertTrue(item.waitForExistence(timeout: 8), action)
            item.tap()
        }
        func source(_ path: String, expected: String? = nil) throws -> XCUIElement {
            let open = app.buttons.matching(NSPredicate(format: "identifier == %@ OR label == %@ OR label == %@", path, path, path + " •")).firstMatch
            let editor = app.textViews[path].firstMatch
            if !editor.exists || !editor.isHittable {
                XCTAssertTrue(open.waitForExistence(timeout: 12), path)
                open.tap()
            }
            XCTAssertTrue(editor.waitForExistence(timeout: 12), path)
            if let expected {
                let exact = XCTNSPredicateExpectation(predicate: NSPredicate(format: "value == %@", expected), object: editor)
                XCTAssertEqual(XCTWaiter.wait(for: [exact], timeout: 8), .completed, path)
            }
            return editor
        }
        func reveal(_ element: XCUIElement, upward: Bool) {
            for _ in 0..<5 where !element.isHittable {
                if upward { app.swipeUp() } else { app.swipeDown() }
            }
            XCTAssertTrue(element.isHittable, element.debugDescription)
        }
        func acknowledgeChange(_ statusPrefix: String) {
            let preview = app.staticTexts["Workspace change preview"].firstMatch
            XCTAssertTrue(preview.waitForExistence(timeout: 12))
            let status = app.staticTexts.matching(NSPredicate(format: "label BEGINSWITH %@", statusPrefix)).firstMatch
            XCTAssertTrue(status.waitForExistence(timeout: 8), statusPrefix)
            app.buttons["Close"].firstMatch.tap()
            XCTAssertTrue(preview.waitForNonExistence(timeout: 10))
        }

        let note = "note.py", construction = "construction.md"
        let noteView = try source(note, expected: "value=1\n")
        let noteFrame = noteView.frame.integral
        let constructionView = try source(construction)
        let originalConstruction = try XCTUnwrap(constructionView.value as? String)
        XCTAssertTrue(originalConstruction.contains("generated = 1"))
        let constructionFrame = constructionView.frame.integral
        let changedConstruction = originalConstruction.replacingOccurrences(of: "generated = 1", with: "generated = 41")

        menu("View", "Agent")
        let conversation = app.scrollViews["Conversation"].firstMatch
        XCTAssertTrue(conversation.waitForExistence(timeout: 15))
        let composer = app.textViews.matching(NSPredicate(format: "identifier == %@ OR label == %@ OR label == %@",
            "conversation/draft.md", "conversation/draft.md", "Message")).firstMatch
        XCTAssertTrue(composer.waitForExistence(timeout: 12))
        composer.tap()
        XCTAssertTrue(app.keyboards.firstMatch.waitForExistence(timeout: 8))
        composer.typeText(prompt)
        let entered = XCTNSPredicateExpectation(predicate: NSPredicate(format: "value == %@", prompt), object: composer)
        XCTAssertEqual(XCTWaiter.wait(for: [entered], timeout: 10), .completed)
        let send = app.buttons["Send"].firstMatch
        XCTAssertTrue(send.isHittable && send.isEnabled)
        send.tap()
        let cleared = XCTNSPredicateExpectation(predicate: NSPredicate(format: "value == '' OR value == nil"), object: composer)
        XCTAssertEqual(XCTWaiter.wait(for: [cleared], timeout: 10), .completed)
        XCTAssertTrue(app.keyboards.firstMatch.waitForNonExistence(timeout: 10))
        XCTAssertTrue(conversation.waitForExistence(timeout: 12))
        XCTAssertGreaterThan(conversation.frame.height, 0)
        let userCard = conversation.descendants(matching: .any)["Recorded user message"].firstMatch
        let archived = userCard.textViews.matching(NSPredicate(format: "value == %@", prompt)).firstMatch
        XCTAssertTrue(archived.waitForExistence(timeout: 20), "The exact native message was not visible in the recorded user card")
        NSLog("HickoryUIKitParity native-proposal-send-archived-exact-message")

        menu("View", "Agent proposal review")
        let review = app.staticTexts["Agent proposal review"].firstMatch
        XCTAssertTrue(review.waitForExistence(timeout: 12))
        let read = app.buttons["Read Explain and change the source together"].firstMatch
        XCTAssertTrue(read.waitForExistence(timeout: 15))
        read.tap()
        let digest = app.staticTexts.matching(NSPredicate(format: "label BEGINSWITH %@", "Complete SHA256 ")).firstMatch
        XCTAssertTrue(digest.waitForExistence(timeout: 15))
        let apply = app.buttons["Apply complete draft batch"].firstMatch
        reveal(apply, upward: true)
        XCTAssertTrue(apply.isEnabled)
        apply.tap()
        acknowledgeChange("Applied 2 source drafts. Not saved or verified.")
        if review.exists {
            let close = app.buttons["Close"].firstMatch
            reveal(close, upward: false)
            close.tap()
            XCTAssertTrue(review.waitForNonExistence(timeout: 10))
        }

        func drafts(noteText: String, constructionText: String) throws {
            XCTAssertEqual(try source(note, expected: noteText).frame.integral, noteFrame)
            XCTAssertEqual(try source(construction, expected: constructionText).frame.integral, constructionFrame)
        }
        try drafts(noteText: "value=41\n", constructionText: changedConstruction)
        menu("View", "Agent")
        menu("Edit", "Undo workspace change")
        acknowledgeChange("Workspace undo applied to 2 drafts. Not saved or verified.")
        try drafts(noteText: "value=1\n", constructionText: originalConstruction)
        menu("View", "Agent")
        menu("Edit", "Redo workspace change")
        acknowledgeChange("Workspace redo applied to 2 drafts. Not saved or verified.")
        try drafts(noteText: "value=41\n", constructionText: changedConstruction)
        NSLog("HickoryUIKitParity native-proposal-send-applied-and-replayed-two-drafts")
    }
}
extension RecordedRunUITests {
    @MainActor func testNativeGitReviewStagesOnlySavedBytesAndRetainsDraft() throws {
        continueAfterFailure = false
        let environment = ProcessInfo.processInfo.environment
        let origin = try XCTUnwrap(environment["HICKORY_IOS_PARITY_ORIGIN"])
        let baseline = try XCTUnwrap(environment["HICKORY_IOS_GIT_BASELINE"])
        let saved = try XCTUnwrap(environment["HICKORY_IOS_GIT_SAVED"])
        let draftMarker = try XCTUnwrap(environment["HICKORY_IOS_GIT_DRAFT_MARKER"])
        let path = "note.py"
        let app = XCUIApplication()
        app.launchEnvironment["HICKORY_API_ORIGIN"] = origin
        app.launchEnvironment["HICKORY_OPEN_FILE"] = path
        app.launch()

        func tap(_ title: String) {
            let button = app.buttons[title].firstMatch
            XCTAssertTrue(button.waitForExistence(timeout: 15), title)
            XCTAssertTrue(button.isHittable && button.isEnabled, title)
            button.tap()
        }
        func read(_ phrase: String) {
            let label = app.descendants(matching: .any).matching(NSPredicate(format: "label CONTAINS %@ OR value CONTAINS %@", phrase, phrase)).firstMatch
            XCTAssertTrue(label.waitForExistence(timeout: 12), phrase)
        }
        func diff(_ side: String) {
            tap("\(side) \(path)")
            let removed = "-" + baseline.trimmingCharacters(in: .newlines)
            let added = "+" + saved.trimmingCharacters(in: .newlines)
            let exact = app.descendants(matching: .any).matching(NSPredicate(format:
                "(label CONTAINS %@ AND label CONTAINS %@ AND NOT label CONTAINS %@) OR (value CONTAINS %@ AND value CONTAINS %@ AND NOT value CONTAINS %@)",
                removed, added, draftMarker, removed, added, draftMarker)).firstMatch
            XCTAssertTrue(exact.waitForExistence(timeout: 15), "The \(side) diff must show the exact saved hunk")
        }

        let editor = app.textViews[path].firstMatch
        XCTAssertTrue(editor.waitForExistence(timeout: 20))
        tap("View"); tap("Agent")
        tap(path)
        let loaded = XCTNSPredicateExpectation(predicate: NSPredicate(format: "value == %@", saved), object: editor)
        XCTAssertEqual(XCTWaiter.wait(for: [loaded], timeout: 10), .completed)
        let frame = editor.frame.integral
        editor.tap()
        XCTAssertTrue(app.keyboards.firstMatch.waitForExistence(timeout: 8))
        editor.typeText(draftMarker)
        let draft = try XCTUnwrap(editor.value as? String)
        XCTAssertTrue(draft.contains(draftMarker)); XCTAssertNotEqual(draft, saved)
        tap("Agent")
        XCTAssertTrue(app.keyboards.firstMatch.waitForNonExistence(timeout: 10))
        tap("View"); tap("Repository changes")
        read("Saved working tree and index. Unsaved editor drafts are not included.")
        diff("Working tree")
        tap("Stage saved file")
        read("Saved paths staged.")
        diff("Staged")
        tap("Unstage file")
        read("Saved paths unstaged.")
        tap("Close")
        tap(path)
        XCTAssertTrue(editor.waitForExistence(timeout: 12))
        XCTAssertEqual(editor.value as? String, draft)
        XCTAssertEqual(editor.frame.integral, frame)
        NSLog("HickoryUIKitParity native-git-review-staged-saved-only-and-retained-draft")
    }
}
