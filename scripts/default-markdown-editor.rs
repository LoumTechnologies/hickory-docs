//! Installer-only macOS helper. The Swift bridge calls AppKit's current,
//! asynchronous API: legacy LaunchServices returns before consent completes.
//! Swift ships with the same Xcode tools required to build the desktop app.
use std::process::{Command, ExitCode};

const BRIDGE: &str = r#"
import AppKit
import UniformTypeIdentifiers

let application = URL(fileURLWithPath: CommandLine.arguments.last!).standardizedFileURL
let workspace = NSWorkspace.shared

guard let markdown = UTType(filenameExtension: "md") else {
    fputs("macOS could not resolve the Markdown content type.\n", stderr)
    exit(1)
}
func isDefault() -> Bool {
    workspace.urlForApplication(toOpen: markdown)?.standardizedFileURL == application
}
if isDefault() {
    print("Hickory Docs is already the default editor for .md files.")
    exit(0)
}
print("Making Hickory Docs the default Markdown editor. Confirm if macOS asks.")
// One request for the preferred .md type. The completion runs AFTER any
// system consent prompt, so a pending choice never triggers another request.
workspace.setDefaultApplication(at: application, toOpen: markdown) { error in
    if let error {
        fputs("Could not change the Markdown default: \(error.localizedDescription)\n", stderr)
        exit(1)
    }
    guard isDefault() else {
        fputs("macOS did not retain Hickory Docs as the Markdown default.\n", stderr)
        exit(1)
    }
    print("Hickory Docs is the default editor for .md files.")
    exit(0)
}
RunLoop.main.run()
"#;

fn main() -> ExitCode {
    let Some(application) = std::env::args_os().nth(1) else {
        eprintln!("Expected the installed .app path");
        return ExitCode::FAILURE;
    };
    match Command::new("/usr/bin/swift")
        .args(["-e", BRIDGE])
        .arg(application)
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
