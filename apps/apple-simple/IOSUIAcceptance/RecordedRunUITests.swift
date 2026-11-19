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
extension RecordedRunUITests {
    @MainActor func testNativeGitAttributionOpensImmutableCommitAndRetainsDraft() throws {
        continueAfterFailure = false
        let environment = ProcessInfo.processInfo.environment
        let origin = try XCTUnwrap(environment["HICKORY_IOS_PARITY_ORIGIN"])
        let baseline = try XCTUnwrap(environment["HICKORY_IOS_GIT_BASELINE"])
        let saved = try XCTUnwrap(environment["HICKORY_IOS_GIT_SAVED"])
        let draftMarker = try XCTUnwrap(environment["HICKORY_IOS_GIT_DRAFT_MARKER"])
        let commit = try XCTUnwrap(environment["HICKORY_IOS_GIT_COMMIT"])
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
        func text(_ value: String) -> XCUIElement {
            app.descendants(matching: .any).matching(NSPredicate(format: "label == %@ OR value == %@", value, value)).firstMatch
        }
        func read(_ value: String) {
            XCTAssertTrue(text(value).waitForExistence(timeout: 12), value)
        }
        func readCommittedBlob() {
            read(commit)
            let lines = baseline.split(separator: "\n").map(String.init)
            XCTAssertEqual(lines.count, 2)
            let blob = app.descendants(matching: .any).matching(NSPredicate(format:
                "(label CONTAINS %@ AND label CONTAINS %@ AND NOT label CONTAINS %@) OR (value CONTAINS %@ AND value CONTAINS %@ AND NOT value CONTAINS %@)",
                lines[0], lines[1], draftMarker, lines[0], lines[1], draftMarker)).firstMatch
            XCTAssertTrue(blob.waitForExistence(timeout: 15), "The immutable reading must display both committed lines")
        }
        func leaveKeyboard() {
            tap("Agent")
            XCTAssertTrue(app.keyboards.firstMatch.waitForNonExistence(timeout: 10))
            tap(path)
        }

        let editor = app.textViews[path].firstMatch
        XCTAssertTrue(editor.waitForExistence(timeout: 20))
        tap("View"); tap("Agent"); tap(path)
        let loaded = XCTNSPredicateExpectation(predicate: NSPredicate(format: "value == %@", saved), object: editor)
        XCTAssertEqual(XCTWaiter.wait(for: [loaded], timeout: 10), .completed)
        let frame = editor.frame.integral
        editor.tap()
        XCTAssertTrue(app.keyboards.firstMatch.waitForExistence(timeout: 8))
        editor.typeText(draftMarker)
        let draft = try XCTUnwrap(editor.value as? String)
        XCTAssertTrue(draft.contains(draftMarker)); XCTAssertNotEqual(draft, saved)
        leaveKeyboard()

        tap("View"); tap("Saved file attribution")
        read("Saved attribution · \(path)")
        read("Saved-file lines only. Unsaved editor changes are not included.")
        let lines = saved.split(separator: "\n").map(String.init)
        XCTAssertEqual(lines.count, 2)
        for line in lines { read(line) }
        read("Full immutable commit reference unavailable.")
        XCTAssertFalse(app.buttons["Read commit for saved line 2"].exists)
        tap("Read commit for saved line 1")
        let attributedHistory = "Commit from saved attribution · \(path):1"
        read(attributedHistory)
        readCommittedBlob()
        tap("Close")
        XCTAssertTrue(text(attributedHistory).waitForNonExistence(timeout: 10))
        leaveKeyboard()

        tap("View"); tap("Saved file history")
        let fileHistory = "Saved history · \(path)"
        read(fileHistory)
        tap("Read commit \(commit)")
        readCommittedBlob()
        tap("Close")
        XCTAssertTrue(text(fileHistory).waitForNonExistence(timeout: 10))
        leaveKeyboard()
        XCTAssertTrue(editor.waitForExistence(timeout: 12))
        XCTAssertEqual(editor.value as? String, draft)
        XCTAssertEqual(editor.frame.integral, frame)
        NSLog("HickoryUIKitParity native-git-history-read-immutable-commit-and-retained-draft")
    }
}
extension RecordedRunUITests {
    @MainActor func testNativePythonDefinitionAndHistoryAcrossFiles() throws {
        continueAfterFailure = false
        let environment = ProcessInfo.processInfo.environment
        let origin = try XCTUnwrap(environment["HICKORY_IOS_PARITY_ORIGIN"])
        let mainSource = try XCTUnwrap(environment["HICKORY_IOS_PYTHON_CALLER"])
        let helperSource = try XCTUnwrap(environment["HICKORY_IOS_PYTHON_DEFINITION"])
        let app = XCUIApplication()
        app.launchEnvironment["HICKORY_API_ORIGIN"] = origin
        app.launchEnvironment["HICKORY_OPEN_FILE"] = "main.py"
        app.launch()

        func tap(_ button: XCUIElement, _ description: String) {
            XCTAssertTrue(button.waitForExistence(timeout: 15), description)
            XCTAssertTrue(button.isHittable && button.isEnabled, description)
            button.tap()
        }
        func command(_ item: String, menu: String = "Edit") {
            tap(app.buttons[menu].firstMatch, menu)
            let button = app.buttons[item].firstMatch
            if menu == "Edit" && !button.isHittable {
                let list = app.collectionViews.firstMatch
                XCTAssertTrue(list.waitForExistence(timeout: 5))
                // The collection extends under the clipped menu. Start the drag inside its visible middle.
                for _ in 0..<6 where !button.isHittable {
                    let start = list.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.46))
                    let end = list.coordinate(withNormalizedOffset: CGVector(dx: 0.5, dy: 0.21))
                    start.press(forDuration: 0.05, thenDragTo: end)
                }
            }
            tap(button, item)
        }
        func tab(_ path: String) -> XCUIElement {
            let strip = app.scrollViews.containing(.button, identifier: "Close \(path)").firstMatch
            return strip.buttons[path].firstMatch
        }
        func editor(_ path: String, text: String) -> XCUIElement {
            let view = app.textViews[path].firstMatch
            XCTAssertTrue(view.waitForExistence(timeout: 15), path)
            let matches = XCTNSPredicateExpectation(predicate: NSPredicate(format: "value == %@", text), object: view)
            XCTAssertEqual(XCTWaiter.wait(for: [matches], timeout: 8), .completed, path)
            return view
        }
        func releaseKeyboard(in path: String) {
            tap(app.buttons["Agent"].firstMatch, "Agent tab")
            XCTAssertTrue(app.keyboards.firstMatch.waitForNonExistence(timeout: 10))
            tap(tab(path), path)
        }

        _ = editor("main.py", text: mainSource)
        command("Agent", menu: "View")
        tap(tab("main.py"), "main.py tab")
        let caller = editor("main.py", text: mainSource)
        let callerFrame = caller.frame.integral
        command("Find and replace")
        let find = app.textFields["Find"].firstMatch
        XCTAssertTrue(find.waitForExistence(timeout: 12))
        find.tap()
        find.typeText("greeting(")
        let entered = XCTNSPredicateExpectation(predicate: NSPredicate(format: "value == %@", "greeting("), object: find)
        XCTAssertEqual(XCTWaiter.wait(for: [entered], timeout: 8), .completed)
        let keyboardIntro = app.staticTexts.matching(NSPredicate(format: "label BEGINSWITH %@", "Speed up your typing by sliding your finger")).firstMatch
        if keyboardIntro.waitForExistence(timeout: 3) {
            tap(app.buttons["Continue"].firstMatch, "Dismiss the first-use keyboard introduction")
            XCTAssertTrue(keyboardIntro.waitForNonExistence(timeout: 8))
        }
        find.tap()
        find.typeText("\n")
        tap(app.buttons["Close search"].firstMatch, "Close search with selected call")
        releaseKeyboard(in: "main.py")

        command("Go to definition")
        let target = app.buttons["helper.py:4:5"].firstMatch
        XCTAssertTrue(target.waitForExistence(timeout: 20), "Expected the real Python declaration at helper.py:4:5")
        if !target.isHittable {
            let panel = app.scrollViews.containing(.button, identifier: "helper.py:4:5").firstMatch
            XCTAssertTrue(panel.waitForExistence(timeout: 5), app.debugDescription)
            for _ in 0..<4 where !target.isHittable {
                let start = panel.coordinate(withNormalizedOffset: CGVector(dx: 0.20, dy: 0.72))
                let end = panel.coordinate(withNormalizedOffset: CGVector(dx: 0.20, dy: 0.25))
                start.press(forDuration: 0.05, thenDragTo: end)
            }
        }
        tap(target, "exact Python definition")
        _ = editor("helper.py", text: helperSource)
        releaseKeyboard(in: "helper.py")
        let helperFrame = editor("helper.py", text: helperSource).frame.integral
        XCTAssertTrue(app.textViews["main.py"].waitForNonExistence(timeout: 8))

        command("Navigate back")
        _ = editor("main.py", text: mainSource)
        releaseKeyboard(in: "main.py")
        XCTAssertEqual(editor("main.py", text: mainSource).frame.integral, callerFrame)
        XCTAssertTrue(app.textViews["helper.py"].waitForNonExistence(timeout: 8))
        command("Navigate forward")
        _ = editor("helper.py", text: helperSource)
        releaseKeyboard(in: "helper.py")
        XCTAssertEqual(editor("helper.py", text: helperSource).frame.integral, helperFrame)
        NSLog("HickoryUIKitParity native-python-definition-and-back-forward")
    }
}
extension RecordedRunUITests {
    @MainActor func testRecordedFenceDebugThroughNativeGutter() throws {
        continueAfterFailure = false
        let environment = ProcessInfo.processInfo.environment
        let origin = try XCTUnwrap(environment["HICKORY_IOS_PARITY_ORIGIN"])
        let side = try XCTUnwrap(environment["HICKORY_IOS_RECORDED_DEBUG_SIDE"])
        XCTAssertTrue(["user", "assistant"].contains(side))
        let buffer = side == "user" ? "user" : "agent"
        let expected = side == "user" ? "42" : "53"
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
        let editor = card.textViews["hick-agent-message:///recorded-fence-host-\(buffer).md"].firstMatch
        func screenshot(_ phase: String) {
            let attachment = XCTAttachment(screenshot: XCUIScreen.main.screenshot())
            attachment.name = "Recorded \(side) Debug \(phase)"
            attachment.lifetime = .keepAlways
            add(attachment)
        }
        func firstAction(_ line: Int) -> XCUIElement {
            let overflow = card.buttons["More gutter icons on line \(line)"].firstMatch
            let run = card.buttons["Run recorded python fence on line \(line)"].firstMatch
            return overflow.exists ? overflow : run
        }
        for _ in 0..<7 where !firstAction(3).isHittable || !firstAction(8).isHittable {
            let start = conversation.coordinate(withNormalizedOffset: CGVector(dx: 0.02, dy: 0.78))
            let end = conversation.coordinate(withNormalizedOffset: CGVector(dx: 0.02, dy: 0.37))
            if firstAction(3).isHittable && firstAction(8).isHittable { break }
            start.press(forDuration: 0.04, thenDragTo: end)
        }
        XCTAssertTrue(editor.exists)
        XCTAssertTrue(firstAction(3).isHittable && firstAction(8).isHittable, "Both real fence anchors must be on screen before tapping the line-number rail")
        let archived = try XCTUnwrap(editor.value as? String)
        XCTAssertTrue(archived.contains(side == "user" ? "value = 41" : "value = 52"))
        XCTAssertTrue(archived.contains("print(value + 1)"))
        let originalFrame = editor.frame.integral
        let breakpoint = card.buttons["Toggle breakpoint on line 4"].firstMatch
        XCTAssertTrue(breakpoint.waitForExistence(timeout: 10) && breakpoint.isHittable && breakpoint.isEnabled)
        XCTAssertEqual(breakpoint.value as? String, "No breakpoint")
        XCTAssertFalse(card.buttons["Toggle breakpoint on line 3"].exists)
        breakpoint.tap()
        let pending = XCTNSPredicateExpectation(predicate: NSPredicate(format: "value == %@", "pending"), object: breakpoint)
        XCTAssertEqual(XCTWaiter.wait(for: [pending], timeout: 7), .completed)
        XCTAssertEqual(editor.value as? String, archived)
        XCTAssertEqual(editor.frame.integral, originalFrame)
        NSLog("HickoryUIKitParity recorded-debug-ui-\(side)-breakpoint-tap")
        NSLog("HickoryUIKitParity recorded-debug-ui-\(side)-native-line-button-4-pending")
        screenshot("authored breakpoint")

        func tapGutter(_ label: String, line: Int) {
            let direct = card.buttons[label].firstMatch
            let more = card.buttons["More gutter icons on line \(line)"].firstMatch
            if direct.waitForExistence(timeout: 7), direct.isHittable { direct.tap(); return }
            XCTAssertTrue(more.waitForExistence(timeout: 10) && more.isHittable, "Missing native gutter action \(label)")
            more.tap()
            let overflow = app.descendants(matching: .any)["More actions on line \(line)"].firstMatch
            let revealed = overflow.buttons[label].firstMatch
            XCTAssertTrue(revealed.waitForExistence(timeout: 9) && revealed.isHittable, "Missing overflow action \(label)")
            revealed.tap()
        }
        func paused(_ line: Int) {
            let ready = card.buttons["Continue on line \(line)"].firstMatch
            XCTAssertTrue(ready.waitForExistence(timeout: 24), "No native pause controls on recorded line \(line)")
            XCTAssertTrue(ready.isHittable)
            XCTAssertEqual(editor.value as? String, archived)
            screenshot("paused on source line \(line)")
        }
        tapGutter("Debug recorded python fence on line 3", line: 3)
        NSLog("HickoryUIKitParity recorded-debug-ui-\(side)-launch-tap")
        paused(4)
        NSLog("HickoryUIKitParity recorded-debug-ui-\(side)-paused-4")
        tapGutter("Over on line 4", line: 4)
        paused(5)
        NSLog("HickoryUIKitParity recorded-debug-ui-\(side)-stepped-5")
        tapGutter("Inspect on line 5", line: 5)
        let atFrame = app.staticTexts["Source line 5 · paused"].firstMatch
        XCTAssertTrue(atFrame.waitForExistence(timeout: 12))
        NSLog("HickoryUIKitParity recorded-debug-ui-\(side)-inspect-mapped-5")
        let expression = app.textFields["Expression"].firstMatch
        let inspector = app.scrollViews.containing(.staticText, identifier: "Source line 5 · paused").firstMatch
        XCTAssertTrue(inspector.exists, "The debugger's own scroll container is unavailable")
        let initialInspectorFrame = inspector.frame
        let keyboard = app.keyboards.firstMatch
        func screenPoint(_ point: CGPoint) -> XCUICoordinate {
            app.coordinate(withNormalizedOffset: .zero).withOffset(CGVector(dx: point.x - app.frame.minX, dy: point.y - app.frame.minY))
        }
        func reveal(_ element: XCUIElement, upward: Bool) {
            for _ in 0..<5 where !element.isHittable {
                let bounds = (inspector.exists ? inspector.frame : initialInspectorFrame).intersection(app.frame)
                let bottom = keyboard.exists && !keyboard.frame.isEmpty ? min(bounds.maxY, keyboard.frame.minY) : bounds.maxY
                let height = bottom - bounds.minY
                XCTAssertGreaterThan(height, 55, "The inspector has no scroll viewport above the keyboard")
                let low = CGPoint(x: bounds.midX, y: bounds.minY + height * 0.32)
                let high = CGPoint(x: bounds.midX, y: bounds.minY + height * 0.78)
                let start = screenPoint(upward ? high : low)
                let end = screenPoint(upward ? low : high)
                start.press(forDuration: 0.03, thenDragTo: end)
            }
            XCTAssertTrue(element.isHittable, element.exists ? element.debugDescription : "Debugger control is no longer mounted")
        }
        reveal(expression, upward: true)
        let inputFrame = expression.frame
        XCTAssertTrue(initialInspectorFrame.contains(CGPoint(x: inputFrame.midX, y: inputFrame.midY)))
        NSLog("HickoryUIKitParity \(side) inspector=\(initialInspectorFrame) expression-before-focus=\(inputFrame)")
        screenshot("Expression before focus")
        expression.tap()
        let keyboardOpened = keyboard.waitForExistence(timeout: 8)
        if !keyboardOpened { screenshot("Expression after focus without keyboard") }
        XCTAssertTrue(keyboardOpened, "The expression tap did not open a native keyboard")
        screenshot("native expression keyboard")
        // UIKit can retire the placeholder-labelled accessibility element as
        // the text field gains focus. Send real keys to the focused app.
        app.typeText("value + 1")
        let watch = app.buttons["Watch"].firstMatch
        reveal(watch, upward: true)
        XCTAssertTrue(watch.isEnabled, "The native expression did not enable Watch")
        watch.tap()
        let value = app.staticTexts["value + 1 = \(expected)"].firstMatch
        XCTAssertTrue(value.waitForExistence(timeout: 12), "The mapped frame did not show the expected Watch")
        reveal(value, upward: true)
        screenshot("watch \(expected)")
        NSLog("HickoryUIKitParity recorded-debug-ui-\(side)-watched-\(expected)")
        let resume = app.buttons["Continue"].firstMatch
        reveal(resume, upward: false)
        resume.tap()
        let finished = app.staticTexts["Source line 5 · finished"].firstMatch
        XCTAssertTrue(finished.waitForExistence(timeout: 22))
        screenshot("continued to finished")
        // Tap outside a popover or dismiss the iPhone sheet with its ordinary
        // drag gesture before sampling the underlying recorded editor.
        if !editor.isHittable {
            let leftOfInspector = CGPoint(x: app.frame.minX + 10, y: initialInspectorFrame.midY)
            let aboveInspector = CGPoint(x: app.frame.midX, y: max(app.frame.minY + 35, initialInspectorFrame.minY - 25))
            let outside = initialInspectorFrame.minX - app.frame.minX > 25 ? leftOfInspector : aboveInspector
            XCTAssertFalse(initialInspectorFrame.contains(outside))
            screenPoint(outside).tap()
            if !editor.isHittable, finished.exists {
                let top = CGPoint(x: app.frame.midX, y: app.frame.minY + 85)
                let bottom = CGPoint(x: app.frame.midX, y: app.frame.maxY - 35)
                screenPoint(top).press(forDuration: 0.03, thenDragTo: screenPoint(bottom))
            }
        }
        XCTAssertEqual(editor.value as? String, archived)
        XCTAssertEqual(editor.frame.integral, originalFrame)
        NSLog("HickoryUIKitParity recorded-debug-ui-\(side)-finished-immutable")
    }
}
