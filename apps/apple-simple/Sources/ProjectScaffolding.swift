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
