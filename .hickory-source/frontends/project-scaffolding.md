<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Create a project from an installed template

For maintainers adding a scaffolding tool to the macOS File menu. A provider
supplies a catalog, the fields for a template variant, and the creation
operation. The form can render those fields and ask for a destination without
knowing how the tool exposes its templates or accepts parameters.

## Give each provider the same form model

Use the template identifier, variant and category together so a short name
shared across languages or template types keeps its own form. Retain SDK
defaults as hints. Sending only deliberate selections keeps conditional
defaults and omitted parameters under the provider's control.

<ui:copy id="scaffold-form-model">
#if os(macOS)
import Foundation

protocol ScaffoldProvider: Sendable {
    var id: String { get }
    var title: String { get }
    func templates(in parent: URL) throws -> [ScaffoldTemplate]
    func options(for template: ScaffoldTemplate, in parent: URL) throws -> [ScaffoldOption]
    func create(_ request: ScaffoldRequest) throws -> URL
}

struct ScaffoldTemplate: Hashable, Identifiable, Sendable {
    let name: String
    let identifier: String
    let variant: String?
    let category: String?

    var id: Self { self }
}

struct ScaffoldOption: Identifiable, Sendable {
    enum Kind: Sendable {
        case text, toggle, choice([String], multiple: Bool)
    }

    let id: String
    let label: String
    let help: String
    let kind: Kind
    let defaultValue: String?
    let required: Bool
}

struct ScaffoldRequest: Sendable {
    let template: ScaffoldTemplate
    let parent: URL
    let name: String
    let options: [ScaffoldOption]
    let values: [String: [String]]

    var folderName: String { name.trimmingCharacters(in: .whitespacesAndNewlines) }

    var validationError: String? {
        if folderName.isEmpty { return "Enter a name for the project folder." }
        if [".", ".."].contains(folderName) || folderName.hasPrefix("-")
            || folderName.rangeOfCharacter(from: CharacterSet(charactersIn: "/:").union(.controlCharacters)) != nil {
            return "Enter a single folder name without slashes, colons, control characters, or a leading dash."
        }
        for option in options where option.required {
            if values[option.id, default: []].allSatisfy({ $0.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }) {
                return "Enter a value for \(option.label)."
            }
        }
        return nil
    }
}

struct ScaffoldFailure: LocalizedError {
    let message: String
    var errorDescription: String? { message }
}
</ui:copy>

## Run the installed .NET CLI where the project will live

The destination's parent determines SDK selection through `global.json`. Use
that location for discovery and help, and start creation inside the new folder.
Graphical launches often inherit a short PATH, so also look in common Microsoft
installer, Homebrew and user installation locations. Use the SDK's English
output for parsing while retaining its diagnostic text for the user.

<ui:copy id="scaffold-dotnet-process">

struct DotnetScaffoldProvider: ScaffoldProvider {
    let id = "dotnet"
    let title = ".NET Project…"
    var environment = ProcessInfo.processInfo.environment

    func templates(in parent: URL) throws -> [ScaffoldTemplate] {
        let output = try run(["new", "list", "--columns", "language", "type"], in: parent)
        return try catalog(from: output)
    }

    func options(for template: ScaffoldTemplate, in parent: URL) throws -> [ScaffoldOption] {
        let output = try run(command(for: template) + ["--help"], in: parent)
        return try fields(from: output)
    }

    private func command(for template: ScaffoldTemplate) -> [String] {
        var arguments = ["new", template.identifier]
        if let variant = template.variant { arguments += ["--language", variant] }
        if let category = template.category { arguments += ["--type", category] }
        return arguments
    }

    @discardableResult
    func run(_ arguments: [String], in directory: URL) throws -> String {
        let home = FileManager.default.homeDirectoryForCurrentUser.path
        let path = (environment["PATH"] ?? "").split(separator: ":").map(String.init)
            + ["/usr/local/share/dotnet", "/opt/homebrew/bin", "/usr/local/bin", "\(home)/.dotnet", "/usr/bin", "/bin"]
        guard let executable = path.lazy.filter({ $0.hasPrefix("/") })
            .map({ URL(fileURLWithPath: $0, isDirectory: true).appendingPathComponent("dotnet") })
            .first(where: { FileManager.default.isExecutableFile(atPath: $0.path) }) else {
            throw ScaffoldFailure(message: "Install a .NET SDK to use .NET templates. Hickory could not find the dotnet executable.")
        }

        let process = Process()
        let output = Pipe()
        var childEnvironment = environment
        childEnvironment["PATH"] = path.joined(separator: ":")
        childEnvironment["DOTNET_CLI_UI_LANGUAGE"] = "en-US"
        childEnvironment["DOTNET_CLI_TELEMETRY_OPTOUT"] = "1"
        childEnvironment["DOTNET_GENERATE_ASPNET_CERTIFICATE"] = "false"
        childEnvironment["DOTNET_NOLOGO"] = "1"
        process.environment = childEnvironment
        process.executableURL = executable
        process.currentDirectoryURL = directory
        process.arguments = arguments
        process.standardInput = FileHandle.nullDevice
        process.standardOutput = output
        process.standardError = output
        try process.run()
        // Drain before waiting so a full pipe can't block the child process.
        let bytes = output.fileHandleForReading.readDataToEndOfFile()
        process.waitUntilExit()
        let result = String(decoding: bytes, as: UTF8.self)
        guard process.terminationStatus == 0 else {
            let detail = result.trimmingCharacters(in: .whitespacesAndNewlines).suffix(6000)
            throw ScaffoldFailure(message: ".NET exited with code \(process.terminationStatus).\n\(detail)")
        }
        return result
    }
</ui:copy>

## Derive the form from the catalog and help

The CLI aligns catalog columns with a dashed separator. Slice at the column
starts so empty language or type cells retain their meaning. Read each selected
variant's template options separately. Their help distinguishes booleans,
choices, repeated choices and free-form fields. Keep other help alongside the
field, including conditional requirements, which the SDK evaluates.

<ui:copy id="scaffold-dotnet-metadata">

    private func catalog(from output: String) throws -> [ScaffoldTemplate] {
        var columns: [Int] = []
        var templates: [ScaffoldTemplate] = []
        // The CLI also has subcommands in this position. Use a template alias
        // that cannot be interpreted as one of those commands.
        let subcommands: Set<String> = ["create", "details", "install", "list", "search", "uninstall", "update"]
        for line in output.components(separatedBy: .newlines) {
            let row = line as NSString
            if line.range(of: #"^-+(  -+){3}\s*$"#, options: .regularExpression) != nil {
                let runs = try NSRegularExpression(pattern: "-+").matches(in: line, range: NSRange(location: 0, length: row.length))
                columns = runs.map(\.range.location) + [Int.max]
                continue
            }
            guard columns.count == 5, !line.trimmingCharacters(in: .whitespaces).isEmpty else { continue }
            let cells = zip(columns, columns.dropFirst()).map { start, end in
                let low = min(start, row.length)
                return row.substring(with: NSRange(location: low, length: min(end, row.length) - low))
                    .trimmingCharacters(in: .whitespaces)
            }
            guard let alias = cells[1].split(separator: ",").map({ $0.trimmingCharacters(in: .whitespaces) })
                .first(where: {
                    !$0.hasSuffix("...") && !subcommands.contains($0.lowercased())
                        && $0.range(of: #"^[\p{L}\p{N}_.][\p{L}\p{N}_.-]*$"#, options: .regularExpression) != nil
                }) else { continue }
            let variants = cells[2].split(separator: ",").map { $0.trimmingCharacters(in: CharacterSet(charactersIn: " []")) }
            for language in variants.isEmpty ? [nil] : variants.map(Optional.some) {
                templates.append(ScaffoldTemplate(
                    name: cells[0], identifier: alias, variant: language,
                    category: cells[3].isEmpty ? nil : cells[3]
                ))
            }
        }
        guard !columns.isEmpty else {
            throw ScaffoldFailure(message: "Hickory could not read the installed .NET template list. Check the SDK selected for this folder with dotnet --version.")
        }
        var seen: Set<ScaffoldTemplate> = []
        return templates.filter { seen.insert($0).inserted }
    }

    private func fields(from output: String) throws -> [ScaffoldOption] {
        let lines = output.components(separatedBy: .newlines)
        guard let heading = lines.firstIndex(where: { $0.trimmingCharacters(in: .whitespaces) == "Template options:" }) else {
            throw ScaffoldFailure(message: "The SDK did not return options for this template and language.\n\(output.suffix(1600))")
        }
        var fields: [ScaffoldOption] = []
        let flag = try NSRegularExpression(pattern: #"^  (?:-[^ ,]+, )?(--[\p{L}\p{N}][\p{L}\p{N}_-]*)"#)
        var block: [String] = []
        var option: String?
        for line in lines.dropFirst(heading + 1) + [""] {
            let row = line as NSString
            let match = flag.firstMatch(in: line, range: NSRange(location: 0, length: row.length))
            if match != nil || line.trimmingCharacters(in: .whitespaces).isEmpty {
                if let option { fields.append(field(option, from: block)) }
                block = []
                option = match.map { row.substring(with: $0.range(at: 1)) }
                if match == nil { break }
            }
            if option != nil { block.append(line) }
        }
        return fields
    }

    private func field(_ flag: String, from lines: [String]) -> ScaffoldOption {
        var kind = ""
        var choices: [String] = []
        var help: [String] = []
        var defaultValue: String?
        var required = false
        var multiple = false
        var readingChoices = false
        var readingDefault = false
        var choiceIndent: Int?
        for (index, raw) in lines.enumerated() {
            var line = raw.trimmingCharacters(in: .whitespaces)
            var indent = raw.prefix(while: { $0 == " " }).count
            if index == 0 || line.hasPrefix("<") {
                // Help can start on the flag line or after a wrapped value placeholder.
                guard let gap = line.range(of: "  ") else { continue }
                indent += line.distance(from: line.startIndex, to: gap.lowerBound)
                let rest = line[gap.lowerBound...]
                indent += rest.prefix(while: { $0 == " " }).count
                line = rest.trimmingCharacters(in: .whitespaces)
            }
            if line.hasPrefix("Type:") {
                kind = line.dropFirst("Type:".count).trimmingCharacters(in: .whitespaces)
                readingChoices = kind == "choice"
                readingDefault = false
                choiceIndent = indent + 2
            } else if line.hasPrefix("Default:") {
                defaultValue = line.dropFirst("Default:".count).trimmingCharacters(in: .whitespaces)
                readingChoices = false
                readingDefault = true
            } else if line.hasPrefix("Required:") {
                required = line.dropFirst("Required:".count)
                    .trimmingCharacters(in: CharacterSet(charactersIn: " *")).lowercased() == "true"
                readingChoices = false
                readingDefault = false
            } else if line.hasPrefix("Multiple values are allowed:") {
                multiple = line.lowercased().hasSuffix("true")
                readingChoices = false
                readingDefault = false
            } else if readingChoices {
                if indent == choiceIndent {
                    let value = line.components(separatedBy: "  ")[0]
                    if !value.isEmpty { choices.append(value) }
                }
            } else if readingDefault {
                defaultValue = [defaultValue ?? "", line].filter { !$0.isEmpty }.joined(separator: " ")
            } else {
                help.append(line)
            }
        }
        let label = String(flag.dropFirst(2))
        let control: ScaffoldOption.Kind = switch kind {
        case "bool": .toggle
        case "choice" where !choices.isEmpty: .choice(choices, multiple: multiple)
        default: .text
        }
        return ScaffoldOption(
            id: flag, label: label, help: help.joined(separator: " "), kind: control,
            defaultValue: defaultValue, required: required
        )
    }
</ui:copy>

## Reserve the destination before running the template

The form's validation catches empty required fields and names that could escape
the chosen parent. Creation reserves a fresh directory in one operation. Run
the template inside that directory and pass the destination explicitly. Pass
each value as part of its option argument so a text value like `--force` stays
literal. The template engine checks constraints and runs its usual post-actions.
If execution fails after the directory is reserved, keep its contents and
report the path so the user can inspect the result.

<ui:copy id="scaffold-dotnet-create">

    func create(_ request: ScaffoldRequest) throws -> URL {
        if let problem = request.validationError { throw ScaffoldFailure(message: problem) }
        let folder = request.parent.appendingPathComponent(request.folderName, isDirectory: true)
        let arguments = command(for: request.template)
            + ["--name", request.folderName, "--output", folder.path, "--no-update-check"]
            + request.options.flatMap { option in
                request.values[option.id, default: []].filter { !$0.isEmpty }.map { "\(option.id)=\($0)" }
            }
        do {
            try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: false)
        } catch {
            throw ScaffoldFailure(message: "Could not create \(folder.path). Choose an unused name in a writable folder.\n\(error.localizedDescription)")
        }
        do {
            try run(arguments, in: folder)
        } catch {
            throw ScaffoldFailure(message: "\(error.localizedDescription)\nProject folder: \(folder.path)")
        }
        return folder
    }
}
#endif
</ui:copy>

## Exercise discovery and creation together

Install an isolated fixture through the real CLI. The field and output assertions
verify that required text, a default-true toggle and a repeated choice come from
that install. Run a built-in console template as well to check language and
framework fields distributed by the SDK. Then try the same destination again
and verify the original contents are intact.

<ui:copy id="scaffold-dotnet-witness">
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
</ui:copy>

<ui:file path="apps/apple-simple/Sources/ProjectScaffolding.swift"><ui:paste select="#scaffold-form-model" /><ui:paste select="#scaffold-dotnet-process" /><ui:paste select="#scaffold-dotnet-metadata" /><ui:paste select="#scaffold-dotnet-create" /></ui:file>
<ui:file path="apps/apple-simple/Tests/DotnetScaffoldTests.swift"><ui:paste select="#scaffold-dotnet-witness" /></ui:file>

## Render the tool’s choices in a native project window

The form owns the destination, selection and edited values. Bind an option list
to the template and catalog revision that produced it so a form from an earlier
folder cannot be submitted.

<ui:copy id="scaffold-view-state">
#if os(macOS)
import SwiftUI
import UniformTypeIdentifiers

struct ScaffoldProjectView: View {
    let provider: any ScaffoldProvider
    var finish: (URL?) -> Void

    @State private var parent = FileManager.default.homeDirectoryForCurrentUser
    @State private var choosingFolder = false
    @State private var revision = UUID()
    @State private var templates: [ScaffoldTemplate]?
    @State private var catalogError: String?
    @State private var query = ""
    @State private var selection: ScaffoldTemplate?
    @State private var form: (context: TemplateContext, options: [ScaffoldOption])?
    @State private var optionsError: String?
    @State private var values: [String: [String]] = [:]
    @State private var name = ""
    @State private var creating = false
    @State private var creationError: String?

    private struct TemplateContext: Equatable, Sendable {
        let template: ScaffoldTemplate
        let parent: URL
        let revision: UUID
    }

    private var context: TemplateContext? {
        selection.map { TemplateContext(template: $0, parent: parent, revision: revision) }
    }

    private var request: ScaffoldRequest? {
        guard let form, form.context == context else { return nil }
        return ScaffoldRequest(template: form.context.template, parent: parent, name: name, options: form.options, values: values)
    }

</ui:copy>

Use a searchable catalog and a standard form inside the project window. Choose
controls from the provider's field kinds, surface defaults beside the input,
and keep ordinary editor commands and drafts in their original workspace.

<ui:copy id="scaffold-view-controls">
    var body: some View {
        VStack(spacing: 0) {
            HSplitView {
                catalog
                    .frame(minWidth: 210, idealWidth: 260)
                settings
                    .frame(minWidth: 350)
            }
            .disabled(creating)
            Divider()
            HStack {
                if creating { ProgressView("Creating project…").controlSize(.small) }
                Spacer()
                Button("Cancel", role: .cancel) { finish(nil) }
                    .keyboardShortcut(.cancelAction)
                    .disabled(creating)
                Button("Create", action: create)
                    .keyboardShortcut(.defaultAction)
                    .disabled(creating || request == nil || request?.validationError != nil)
            }
            .padding()
        }
        .frame(minWidth: 620, minHeight: 450)
        .fileImporter(isPresented: $choosingFolder, allowedContentTypes: [.folder]) { result in
            switch result {
            case .success(let folder):
                guard folder != parent else { return }
                parent = folder
                reloadCatalog()
            case .failure(let error):
                creationError = error.localizedDescription
            }
        }
        .task(id: revision) { await loadCatalog(in: parent, revision: revision) }
        .task(id: context) { await loadOptions(context) }
    }

    private var catalog: some View {
        VStack(alignment: .leading, spacing: 8) {
            TextField("Search templates", text: $query)
                .textFieldStyle(.roundedBorder)
                .padding([.horizontal, .top])
            if let templates {
                let visible = templates.filter { template in
                    query.isEmpty || [template.name, template.identifier, template.variant ?? "", template.category ?? ""]
                        .contains { $0.localizedCaseInsensitiveContains(query) }
                }
                if visible.isEmpty {
                    VStack(alignment: .leading, spacing: 8) {
                        Text(templates.isEmpty ? "No installed templates are available for this location." : "No templates match your search.")
                        if templates.isEmpty { Button("Reload Templates", action: reloadCatalog) }
                    }
                    .padding()
                } else {
                    List(visible, selection: Binding(get: { selection }, set: select)) { template in
                        HStack {
                            Text(template.name)
                            Spacer()
                            if let variant = template.variant {
                                Text(variant).foregroundStyle(.secondary)
                            }
                        }
                        .tag(template)
                        .help(template.identifier)
                    }
                }
            } else if let catalogError {
                ScrollView {
                    VStack(alignment: .leading, spacing: 8) {
                        Text(catalogError).textSelection(.enabled)
                        Button("Retry", action: reloadCatalog)
                    }
                    .padding()
                }
            } else {
                ProgressView("Loading templates…").padding()
            }
            Spacer(minLength: 0)
        }
    }

    private var settings: some View {
        Form {
            Section("Destination") {
                TextField("Name", text: $name)
                LabeledContent("Location") {
                    HStack {
                        Text(parent.path).lineLimit(1).truncationMode(.middle)
                            .help(parent.path)
                        Button("Choose…") { choosingFolder = true }
                    }
                }
            }
            Section("Template Options") {
                if let request {
                    if request.options.isEmpty { Text("This template has no additional options.").foregroundStyle(.secondary) }
                    ForEach(request.options) { option in optionControl(option) }
                } else if let optionsError {
                    Text(optionsError).textSelection(.enabled)
                    Button("Reload Templates", action: reloadCatalog)
                } else if selection != nil {
                    ProgressView("Loading options…")
                } else {
                    Text("Select a template.").foregroundStyle(.secondary)
                }
            }
            if let creationError {
                Section {
                    Text(creationError).foregroundStyle(.red).textSelection(.enabled)
                }
            } else if let problem = request?.validationError, !name.isEmpty {
                Text(problem).foregroundStyle(.secondary)
            }
        }
        .formStyle(.grouped)
    }

    @ViewBuilder
    private func optionControl(_ option: ScaffoldOption) -> some View {
        let label = option.required ? "\(option.label) (required)" : option.label
        let value = Binding(get: { values[option.id]?.first ?? "" }, set: { values[option.id] = [$0] })
        let defaultLabel = option.defaultValue.flatMap { $0.isEmpty ? nil : "Template default (\($0))" } ?? "Template default"
        VStack(alignment: .leading, spacing: 4) {
            switch option.kind {
            case .text:
                LabeledContent(label) {
                    TextField(label, text: value, prompt: option.defaultValue.map { Text($0) })
                        .labelsHidden()
                }
            case .toggle:
                if let fallback = option.defaultValue.flatMap({ Bool($0.lowercased()) }), !option.required {
                    Toggle(label, isOn: Binding(
                        get: { Bool(value.wrappedValue) ?? fallback },
                        set: { value.wrappedValue = $0 == fallback ? "" : String($0) }
                    ))
                } else {
                    Picker(label, selection: value) {
                        Text(defaultLabel).tag("")
                        Text("Yes").tag("true")
                        Text("No").tag("false")
                    }
                }
            case .choice(let choices, multiple: false):
                Picker(label, selection: value) {
                    Text(defaultLabel).tag("")
                    ForEach(choices, id: \.self) { Text($0).tag($0) }
                }
            case .choice(let choices, multiple: true):
                Text(label)
                ForEach(choices, id: \.self) { choice in
                    Toggle(choice, isOn: Binding(
                        get: { values[option.id, default: []].contains(choice) },
                        set: { enabled in
                            var selected = Set(values[option.id, default: []])
                            if enabled { selected.insert(choice) } else { selected.remove(choice) }
                            values[option.id] = choices.filter { selected.contains($0) }
                        }
                    ))
                }
                if values[option.id, default: []].isEmpty, option.defaultValue != nil {
                    Text(defaultLabel).foregroundStyle(.secondary)
                }
            }
            if !option.help.isEmpty {
                Text(option.help).font(.caption).foregroundStyle(.secondary)
            }
        }
    }

</ui:copy>

Changing the location starts a fresh catalog read because it can select a
different SDK. Changing template clears its parameter values. Run discovery and
creation off the main thread and check the selection again before installing a
reply. One explicit Create submits the current form.

<ui:copy id="scaffold-view-actions">
    private func select(_ template: ScaffoldTemplate?) {
        guard selection != template else { return }
        selection = template
        form = nil
        optionsError = nil
        creationError = nil
        values = [:]
    }

    private func reloadCatalog() {
        select(nil)
        templates = nil
        catalogError = nil
        creationError = nil
        revision = UUID()
    }

    private func loadCatalog(in folder: URL, revision requested: UUID) async {
        let provider = provider
        let load = Task.detached(priority: .userInitiated) { try provider.templates(in: folder) }
        let result = await load.result
        guard !Task.isCancelled, revision == requested, parent == folder else { return }
        switch result {
        case .success(let found): templates = found
        case .failure(let error): catalogError = error.localizedDescription
        }
    }

    private func loadOptions(_ requested: TemplateContext?) async {
        guard let requested else { return }
        let provider = provider
        let load = Task.detached(priority: .userInitiated) {
            try provider.options(for: requested.template, in: requested.parent)
        }
        let result = await load.result
        guard !Task.isCancelled, context == requested else { return }
        switch result {
        case .success(let options): form = (requested, options)
        case .failure(let error): optionsError = error.localizedDescription
        }
    }

    private func create() {
        guard let request, request.validationError == nil, !creating else { return }
        creating = true
        creationError = nil
        let provider = provider
        Task {
            defer { creating = false }
            let operation = Task.detached(priority: .userInitiated) { try provider.create(request) }
            switch await operation.result {
            case .success(let folder): finish(folder)
            case .failure(let error): creationError = error.localizedDescription
            }
        }
    }
}
#endif
</ui:copy>

<ui:file path="apps/apple-simple/Sources/ScaffoldProjectView.swift"><ui:paste select="#scaffold-view-state" /><ui:paste select="#scaffold-view-controls" /><ui:paste select="#scaffold-view-actions" /></ui:file>
</ui:doc>
