<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Local Install Is a Launchable macOS App

For: Maintainers publishing installers and people checking what a download contains.

Given a checkout on an Apple Silicon Mac with the desktop build prerequisites
and write access to `/Applications`, when `just local-install` succeeds, then
`/Applications/Hickory Docs.app` contains a native release build with the UI
embedded and is registered with macOS as **Hickory Docs**. It can be launched
from Applications without a running development server or this checkout. If
the per-user LaunchServices service is unavailable, the installer warns but
still succeeds with the valid app in Applications; macOS registers it when the
service returns.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/release/a-local-install-is-a-launchable-macos-app.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-dfe61b90bcaf"></a>
## `scripts/default-markdown-editor.rs`

Related contracts: [A Local Install Is a Launchable macOS App](contract.md).

<a id="f-dfe61b90bcaf-BRIDGE"></a>
### `BRIDGE`

Installer-only macOS helper. The Swift bridge calls AppKit's current,
asynchronous API: legacy LaunchServices returns before consent completes.
Swift ships with the same Xcode tools required to build the desktop app.

<storycode:copy id="f-dfe61b90bcaf-BRIDGE">
//! Installer-only macOS helper. The Swift bridge calls AppKit's current,
//! asynchronous API: legacy LaunchServices returns before consent completes.
//! Swift ships with the same Xcode tools required to build the desktop app.
use std::process::{Command, ExitCode};

const BRIDGE: &str = r#"
import AppKit
import UniformTypeIdentifiers

let arguments = Array(CommandLine.arguments.dropFirst())
let checkOnly = arguments.contains("--check")
let paths = arguments.filter { $0 != "--check" }
let application = URL(fileURLWithPath: paths[0]).standardizedFileURL.resolvingSymlinksInPath()
let workspace = NSWorkspace.shared
func log(_ message: String) {
    print(message)
    fflush(stdout)
}
func warn(_ message: String) {
    fputs(message + "\n", stderr)
}
func canonical(_ url: URL?) -> URL? {
    url?.standardizedFileURL.resolvingSymlinksInPath()
}
guard let bundleID = Bundle(url: application)?.bundleIdentifier else {
    warn("Could not read the installed application's bundle identifier at \(application.path).")
    exit(1)
}
guard let markdown = UTType(filenameExtension: "md") else {
    warn("macOS could not resolve the Markdown content type for .md files.")
    exit(1)
}
func currentDefault() -> URL? {
    canonical(workspace.urlForApplication(toOpen: markdown))
}
func isDefault() -> Bool {
    currentDefault() == application
}
func describeDefault() -> String {
    currentDefault()?.path ?? "no application selected"
}
log("Markdown content type: \(markdown.identifier)")
log("Requested default: \(application.path) (\(bundleID))")
log("Current Markdown default: \(describeDefault())")
let copies = workspace.urlsForApplications(withBundleIdentifier: bundleID)
for copy in copies {
    log("Registered Hickory Docs copy: \(copy.path)")
}
for app in workspace.runningApplications where app.bundleIdentifier == bundleID {
    log("Hickory Docs is running from: \(app.bundleURL?.path ?? "unknown path"). Quit and reopen it to use the installed build.")
}
if checkOnly { exit(isDefault() ? 0 : 1) }
// A default is associated with the bundle identifier. LaunchServices can
// resolve that identifier to a debug copy even after a successful request.
// Unregister only copies within this checkout's build tree; preserve files.
if paths.count > 1 {
    let buildRoot = URL(fileURLWithPath: paths[1]).standardizedFileURL.resolvingSymlinksInPath().path + "/"
    for copy in copies where canonical(copy) != application && canonical(copy)!.path.hasPrefix(buildRoot) {
        log("Unregistering checkout build copy: \(copy.path)")
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister")
        process.arguments = ["-u", copy.path]
        do {
            try process.run()
            process.waitUntilExit()
            if process.terminationStatus != 0 {
                warn("Warning: unregistering \(copy.path) exited \(process.terminationStatus).")
            }
        } catch {
            warn("Warning: could not unregister \(copy.path): \(error)")
        }
    }
}
if isDefault() {
    log("Hickory Docs in Applications is already the default editor for .md files.")
    exit(0)
}
log("Making Hickory Docs the default Markdown editor. Confirm if macOS asks.")
// One request; completion runs after the system consent prompt. Poll only
// the readback, never issue another request or another consent prompt.
workspace.setDefaultApplication(at: application, toOpen: markdown) { error in
    if let error {
        let detail = error as NSError
        warn("Could not change the Markdown default: \(detail.localizedDescription) (domain=\(detail.domain), code=\(detail.code), details=\(detail.userInfo)).")
        warn("Current Markdown default: \(describeDefault())")
        exit(1)
    }
    log("macOS accepted the default-editor request; checking the selected app.")
    let deadline = Date().addingTimeInterval(5)
    func verify() {
        if isDefault() {
            log("Hickory Docs in Applications is the default editor for .md files.")
            exit(0)
        }
        if Date() >= deadline {
            warn("macOS accepted the request but still resolves .md files to: \(describeDefault())")
            warn("Expected: \(application.path). Another registered copy with the same bundle identifier may be taking precedence.")
            exit(1)
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.25) { verify() }
    }
    DispatchQueue.main.async { verify() }
}
RunLoop.main.run()
"#;

</storycode:copy>
<a id="f-dfe61b90bcaf-main"></a>
### `main`

<storycode:copy id="f-dfe61b90bcaf-main">
fn main() -> ExitCode {
    let Some(application) = std::env::args_os().nth(1) else {
        eprintln!("Expected the installed .app path");
        return ExitCode::FAILURE;
    };
    match Command::new("/usr/bin/swift")
        .args(["-e", BRIDGE])
        .arg(application)
        .args(std::env::args_os().skip(2))
        .status()
    {
        Ok(status) if status.success() => ExitCode::SUCCESS,
        Ok(_) => ExitCode::FAILURE,
        Err(error) => {
            eprintln!("Could not run the macOS default-editor helper: {error}");
            ExitCode::FAILURE
        }
    }
}
</storycode:copy>

### Assemble `scripts/default-markdown-editor.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="scripts/default-markdown-editor.rs"><storycode:paste select="#f-dfe61b90bcaf-BRIDGE"/><storycode:paste select="#f-dfe61b90bcaf-main"/></storycode:file>

## Implementation review

### Implementation

- [apps/desktop/src-tauri/src/lib.rs](../../../mechanisms/apps/desktop/src-tauri/src/lib.rs.md#output-42caba993099)
- [apps/desktop/src-tauri/src/server.rs](../../../mechanisms/apps/desktop/src-tauri/src/server.rs.md#output-821f8d0a36be)
- [apps/web/src/views/WorkspaceView.tsx](../../../mechanisms/apps/web/src/views/WorkspaceView.tsx.md#output-47afe3575948)
- [scripts/build-python-agent-runtime.sh](../../../mechanisms/scripts/build-python-agent-runtime.sh.md#output-5f2e12b18746)
- [scripts/default-markdown-editor.rs](contract.md#output-dfe61b90bcaf)
- [scripts/dist-desktop.sh](../../../mechanisms/scripts/dist-desktop.sh.md#output-928dfd6fa282)

### Tests and fixtures

- [apps/web/src/App.test.tsx](../../../mechanisms/apps/web/src/App.tsx.md#output-65ac24c24624)
- [apps/web/src/views/toolTabs.test.ts](../../collaboration/a-machine-is-a-keypair/contract.md#output-379c44af631b)
- [apps/web/src/views/workspaceState.test.ts](../../../mechanisms/apps/web/src/views/workspaceState.ts.md#output-14d3dbc939ff)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
