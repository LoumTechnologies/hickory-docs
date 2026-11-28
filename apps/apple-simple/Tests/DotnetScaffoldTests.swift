#if os(macOS)
import Foundation
import XCTest
@testable import HickorySimple

final class DotnetScaffoldTests: XCTestCase {
    func testInstalledTemplatesGenerateProjectsWithoutReplacingExistingFiles() throws {
        let files = FileManager.default
        let parent = files.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
        let config = parent.appendingPathComponent("fixture/.template.config", isDirectory: true)
        try files.createDirectory(at: config, withIntermediateDirectories: true)
        defer { try? files.removeItem(at: parent) }

        var provider = DotnetScaffoldProvider()
        provider.environment["DOTNET_CLI_HOME"] = parent.appendingPathComponent("cli").path
        do {
            try provider.run(["--version"], in: parent)
        } catch let error as ScaffoldFailure where error.message.contains("could not find the dotnet executable") {
            throw XCTSkip(error.message)
        }

        try """
        {
          "identity": "Hickory.Scaffold.Tests",
          "name": "Hickory Template Fixture",
          "shortName": "hickory-fixture",
          "sourceName": "Seed",
          "tags": { "type": "project", "language": "C#" },
          "symbols": {
            "title": { "type": "parameter", "datatype": "text", "isRequired": true, "replaces": "TITLE" },
            "debugMode": { "type": "parameter", "datatype": "bool", "defaultValue": "true", "replaces": "MODE" },
            "flavor": {
              "type": "parameter", "datatype": "choice", "defaultValue": "one",
              "allowMultipleValues": true, "replaces": "FLAVOR",
              "choices": [{ "choice": "one" }, { "choice": "two" }]
            }
          }
        }
        """.write(to: config.appendingPathComponent("template.json"), atomically: true, encoding: .utf8)
        try "TITLE MODE FLAVOR".write(
            to: config.deletingLastPathComponent().appendingPathComponent("Seed.txt"), atomically: true, encoding: .utf8
        )
        try provider.run(["new", "install", config.deletingLastPathComponent().path], in: parent)

        let templates = try provider.templates(in: parent)
        let fixture = try XCTUnwrap(templates.first { $0.identifier == "hickory-fixture" && $0.variant == "C#" })
        let options = try provider.options(for: fixture, in: parent)
        XCTAssertEqual(options.first { $0.id == "--title" }?.required, true)
        XCTAssertEqual(options.first { $0.id == "--debugMode" }?.defaultValue, "true")
        let flavor = try XCTUnwrap(options.first { $0.id == "--flavor" })
        guard case .choice(let choices, let multiple) = flavor.kind else { return XCTFail("Expected a choice field.") }
        XCTAssertEqual(choices, ["one", "two"])
        XCTAssertTrue(multiple)
        XCTAssertThrowsError(try provider.create(ScaffoldRequest(
            template: fixture, parent: parent, name: "Missing", options: options, values: [:]
        )))
        XCTAssertFalse(files.fileExists(atPath: parent.appendingPathComponent("Missing").path))

        let request = ScaffoldRequest(template: fixture, parent: parent, name: "Custom Project", options: options, values: [
            "--title": ["--force"], "--debugMode": ["false"], "--flavor": ["one", "two"]
        ])
        let destination = try provider.create(request)
        let generated = destination.appendingPathComponent("Custom Project.txt")
        XCTAssertEqual(try String(contentsOf: generated, encoding: .utf8), "--force False one|two")
        try "Existing work".write(to: generated, atomically: true, encoding: .utf8)
        XCTAssertThrowsError(try provider.create(request))
        XCTAssertEqual(try String(contentsOf: generated, encoding: .utf8), "Existing work")

        let console = try XCTUnwrap(templates.first { $0.identifier == "console" && $0.variant == "C#" })
        let consoleOptions = try provider.options(for: console, in: parent)
        let framework = try XCTUnwrap(consoleOptions.first { $0.id == "--framework" })
        guard case .choice(let frameworks, _) = framework.kind else { return XCTFail("Expected SDK frameworks.") }
        let selectedFramework = try XCTUnwrap(frameworks.last)
        let demo = try provider.create(ScaffoldRequest(
            template: console, parent: parent, name: "Demo", options: consoleOptions,
            values: ["--framework": [selectedFramework], "--use-program-main": ["true"], "--no-restore": ["true"]]
        ))
        XCTAssertTrue(try String(contentsOf: demo.appendingPathComponent("Demo.csproj"), encoding: .utf8)
            .contains("<TargetFramework>\(selectedFramework)</TargetFramework>"))
        XCTAssertTrue(try String(contentsOf: demo.appendingPathComponent("Program.cs"), encoding: .utf8).contains("static void Main"))
    }
}
#endif
