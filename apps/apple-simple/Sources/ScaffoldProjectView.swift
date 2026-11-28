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
