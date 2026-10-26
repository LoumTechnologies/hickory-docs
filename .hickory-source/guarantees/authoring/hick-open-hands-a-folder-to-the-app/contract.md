<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# hick Opens The App While Explicit Verbs Keep Their CLI Behavior

For: Authors editing executable documents and the files those documents produce.

Given an installed desktop app, when `hick` runs without arguments, then it
launches a blank editor window and returns. `hick <path>` and `hick open <path>`
open the named folder or document with an absolute path; a missing path is refused.
`hick open` defaults to the current folder. Existing subcommands and aliases,
`--help`, and `--version` keep their CLI behavior. A command name wins over a
bare folder name; `./test` or `hick open test` disambiguates that folder.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/hick-open-hands-a-folder-to-the-app.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-20cd237e20c3"></a>
## `crates/hickory-cli/src/cli_launch.rs`

Related contracts: [hick Opens The App While Explicit Verbs Keep Their CLI Behavior](contract.md).

<a id="f-20cd237e20c3-arguments"></a>
### `arguments`

Preserve clap's command errors while accepting editor-style path arguments.

<storycode:copy id="f-20cd237e20c3-arguments">
//! Preserve clap's command errors while accepting editor-style path arguments.
use anyhow::Result;
use clap::CommandFactory;
use std::{ffi::OsString, path::Path, process::ExitCode};

pub fn arguments() -> Vec<OsString> {
    normalize(std::env::args_os().collect())
}

</storycode:copy>
<a id="f-20cd237e20c3-normalize"></a>
### `normalize`

<storycode:copy id="f-20cd237e20c3-normalize">
fn normalize(mut args: Vec<OsString>) -> Vec<OsString> {
    if args.len() == 1 {
        args.extend(["open".into(), "--blank-window".into()]);
    } else {
        let first = &args[1];
        let is_command = super::Cli::command()
            .get_subcommands()
            .any(|c| first == c.get_name() || c.get_all_aliases().any(|alias| first == alias));
        let text = first.to_string_lossy();
        // A typo like `hick tes` still gets clap's suggestion. Ambiguous folder
        // names can always be written `./test` or `hick open test`.
        if !is_command
            && !text.starts_with('-')
            && (Path::new(first).exists()
                || text.contains('/')
                || text.contains('\\')
                || text.ends_with(".md"))
        {
            args.insert(1, "open".into());
        }
    }
    args
}

</storycode:copy>
<a id="f-20cd237e20c3-open"></a>
### `open`

<storycode:copy id="f-20cd237e20c3-open">
pub fn open(target: Option<&Path>) -> Result<ExitCode> {
    if let Some(path) = target
        && !path.exists()
    {
        anyhow::bail!(
            "{} does not exist, so there is nothing to open.\n  \
            `hick open` takes a folder or a `.md` file and defaults to the working directory.",
            path.display()
        );
    }
    let app = hickory_cli::open_app::find(|name| std::env::var(name).ok(), |p| p.exists())
        .ok_or_else(|| anyhow::anyhow!(hickory_cli::open_app::not_installed()))?;
    if let Some(path) = target {
        hickory_cli::open_app::open(&app, path)?;
    } else {
        hickory_cli::open_app::open_blank(&app)?;
    }
    Ok(ExitCode::SUCCESS)
}

</storycode:copy>
<a id="f-20cd237e20c3-tests"></a>
### `tests`

<storycode:copy id="f-20cd237e20c3-tests">
#[cfg(test)]
mod tests {
    use super::*;
    // Guarantee: docs/guarantees/authoring/hick-open-hands-a-folder-to-the-app.md
    #[test]
    fn launch_forms_keep_commands_and_errors() {
        for (input, expected) in [
            (vec!["hick"], vec!["hick", "open", "--blank-window"]),
            (vec!["hick", "."], vec!["hick", "open", "."]),
            (
                vec!["hick", "missing.md"],
                vec!["hick", "open", "missing.md"],
            ),
            (vec!["hick", "test"], vec!["hick", "test"]),
            (vec!["hick", "tes"], vec!["hick", "tes"]),
            (vec!["hick", "--help"], vec!["hick", "--help"]),
            (vec!["hick", "--version"], vec!["hick", "--version"]),
        ] {
            assert_eq!(
                normalize(input.into_iter().map(OsString::from).collect()),
                expected.into_iter().map(OsString::from).collect::<Vec<_>>()
            );
        }
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/cli_launch.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/cli_launch.rs"><storycode:paste select="#f-20cd237e20c3-arguments"/><storycode:paste select="#f-20cd237e20c3-normalize"/><storycode:paste select="#f-20cd237e20c3-open"/><storycode:paste select="#f-20cd237e20c3-tests"/></storycode:file>

<a id="output-8fd650059062"></a>
## `crates/hickory-cli/tests/open_app.rs`

Related contracts: [hick Opens The App While Explicit Verbs Keep Their CLI Behavior](contract.md).

<a id="f-8fd650059062-hick"></a>
### `hick`

`hick open` — the CLI handing a folder to the desktop app.
Protects `docs/guarantees/authoring/hick-open-hands-a-folder-to-the-app.md`.
The app itself is a separate artifact and is not built by this suite, so
`HICKORY_DESKTOP` points at a stub that records its arguments. That is the
whole contract worth testing here: which path is passed, in what form, and
whether the terminal comes back.
Every use of `Path` here is inside a `#[cfg(unix)]` item: the stub app is a
`#!/bin/sh` script with a mode bit, which is not a thing on Windows. An
unconditional import is therefore an unused import there, and CI builds with
`-D warnings`.

<storycode:copy id="f-8fd650059062-hick">
//! `hick open` — the CLI handing a folder to the desktop app.
//!
//! Protects `docs/guarantees/authoring/hick-open-hands-a-folder-to-the-app.md`.
//!
//! The app itself is a separate artifact and is not built by this suite, so
//! `HICKORY_DESKTOP` points at a stub that records its arguments. That is the
//! whole contract worth testing here: which path is passed, in what form, and
//! whether the terminal comes back.

// Every use of `Path` here is inside a `#[cfg(unix)]` item: the stub app is a
// `#!/bin/sh` script with a mode bit, which is not a thing on Windows. An
// unconditional import is therefore an unused import there, and CI builds with
// `-D warnings`.
#[cfg(unix)]
use std::path::Path;
use std::process::Command;

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

</storycode:copy>
<a id="f-8fd650059062-stub"></a>
### `stub`

A stand-in for the app: writes its argv to a file and exits.

<storycode:copy id="f-8fd650059062-stub">
/// A stand-in for the app: writes its argv to a file and exits.
#[cfg(unix)]
fn stub(dir: &Path, record: &Path) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt as _;
    let path = dir.join("hickory-desktop");
    std::fs::write(
        &path,
        format!("#!/bin/sh\nprintf '%s' \"$1\" > {}\n", record.display()),
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

</storycode:copy>
<a id="f-8fd650059062-open_hands_the_app_an_absolute_path_and_returns"></a>
### `open_hands_the_app_an_absolute_path_and_returns`

<storycode:copy id="f-8fd650059062-open_hands_the_app_an_absolute_path_and_returns">
#[test]
#[cfg(unix)]
fn open_hands_the_app_an_absolute_path_and_returns() {
    // Absolute because the app is launched detached: one started from a dock
    // or Finder inherits `/`, so a relative path would resolve somewhere
    // nobody meant.
    let dir = tempfile::tempdir().unwrap();
    let notes = dir.path().join("notes");
    std::fs::create_dir_all(&notes).unwrap();
    let record = dir.path().join("argv.txt");
    let app = stub(dir.path(), &record);

    let out = hick()
        .env("HICKORY_DESKTOP", &app)
        .current_dir(&notes)
        .args(["open", "."])
        .output()
        .expect("run hick");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Give the detached child a moment to write.
    for _ in 0..50 {
        if record.exists() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(40));
    }
    let got = std::fs::read_to_string(&record).expect("the app was launched");
    let want = std::fs::canonicalize(&notes).unwrap();
    assert_eq!(Path::new(&got), want, "the app got {got}");
}

</storycode:copy>
<a id="f-8fd650059062-a_single_document_is_passed_through_rather_than_its_folder"></a>
### `a_single_document_is_passed_through_rather_than_its_folder`

<storycode:copy id="f-8fd650059062-a_single_document_is_passed_through_rather_than_its_folder">
#[test]
#[cfg(unix)]
fn a_single_document_is_passed_through_rather_than_its_folder() {
    // The app handles a file target itself — it locks the parent directory
    // and opens the document — so narrowing to the folder here would throw
    // away which document was asked for.
    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("note.md");
    std::fs::write(
        &doc,
        "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\"/>",
    )
    .unwrap();
    let record = dir.path().join("argv.txt");
    let app = stub(dir.path(), &record);

    assert!(
        hick()
            .env("HICKORY_DESKTOP", &app)
            .args(["open"])
            .arg(&doc)
            .output()
            .unwrap()
            .status
            .success()
    );
    for _ in 0..50 {
        if record.exists() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(40));
    }
    let got = std::fs::read_to_string(&record).unwrap();
    assert_eq!(Path::new(&got), std::fs::canonicalize(&doc).unwrap());
}

</storycode:copy>
<a id="f-8fd650059062-a_path_that_does_not_exist_is_refused_before_anything_is_launched"></a>
### `a_path_that_does_not_exist_is_refused_before_anything_is_launched`

<storycode:copy id="f-8fd650059062-a_path_that_does_not_exist_is_refused_before_anything_is_launched">
#[test]
fn a_path_that_does_not_exist_is_refused_before_anything_is_launched() {
    let dir = tempfile::tempdir().unwrap();
    let out = hick()
        .env("HICKORY_DESKTOP", "/nonexistent/hickory-desktop")
        .args(["open"])
        .arg(dir.path().join("no-such-folder"))
        .output()
        .expect("run hick");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(stderr.contains("does not exist"), "{stderr}");
    assert!(
        stderr.contains("defaults to the working directory"),
        "{stderr}"
    );
}

</storycode:copy>
<a id="f-8fd650059062-a_named_app_that_is_not_there_names_it_rather_than_falling_back"></a>
### `a_named_app_that_is_not_there_names_it_rather_than_falling_back`

<storycode:copy id="f-8fd650059062-a_named_app_that_is_not_there_names_it_rather_than_falling_back">
#[test]
fn a_named_app_that_is_not_there_names_it_rather_than_falling_back() {
    // Falling through to a different app than the one somebody named is how
    // you debug the wrong binary for an hour.
    let dir = tempfile::tempdir().unwrap();
    let out = hick()
        .env("HICKORY_DESKTOP", "/opt/nope/hickory-desktop")
        .args(["open"])
        .arg(dir.path())
        .output()
        .expect("run hick");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(stderr.contains("/opt/nope/hickory-desktop"), "{stderr}");
    assert!(stderr.contains("unset it"), "{stderr}");
}

</storycode:copy>
<a id="f-8fd650059062-short_forms_open_a_blank_window_or_the_named_document"></a>
### `short_forms_open_a_blank_window_or_the_named_document`

<storycode:copy id="f-8fd650059062-short_forms_open_a_blank_window_or_the_named_document">
// Guarantee: docs/guarantees/authoring/hick-open-hands-a-folder-to-the-app.md
#[test]
#[cfg(unix)]
fn short_forms_open_a_blank_window_or_the_named_document() {
    for blank in [true, false] {
        let dir = tempfile::tempdir().unwrap();
        let record = dir.path().join("argv.txt");
        let app = stub(dir.path(), &record);
        let doc = dir.path().join("a note.md");
        std::fs::write(&doc, "ordinary markdown").unwrap();
        let mut command = hick();
        command
            .env("HICKORY_DESKTOP", app)
            .env("HICKORY_PROJECT_DIR", "/must-not-open");
        if !blank {
            command.arg(&doc);
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        for _ in 0..50 {
            if record.exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(40));
        }
        let got = std::fs::read_to_string(record).unwrap();
        if blank {
            assert_eq!(got, "--blank-window");
        } else {
            assert_eq!(Path::new(&got), doc.canonicalize().unwrap());
        }
    }
}

</storycode:copy>
<a id="f-8fd650059062-help_and_version_work_without_the_desktop_and_typos_still_get_suggestions"></a>
### `help_and_version_work_without_the_desktop_and_typos_still_get_suggestions`

<storycode:copy id="f-8fd650059062-help_and_version_work_without_the_desktop_and_typos_still_get_suggestions">
#[test]
fn help_and_version_work_without_the_desktop_and_typos_still_get_suggestions() {
    for arg in ["--help", "--version"] {
        assert!(
            hick()
                .env("HICKORY_DESKTOP", "/missing")
                .arg(arg)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    let output = hick().arg("tes").output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("test"));
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/open_app.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/open_app.rs"><storycode:paste select="#f-8fd650059062-hick"/><storycode:paste select="#f-8fd650059062-stub"/><storycode:paste select="#f-8fd650059062-open_hands_the_app_an_absolute_path_and_returns"/><storycode:paste select="#f-8fd650059062-a_single_document_is_passed_through_rather_than_its_folder"/><storycode:paste select="#f-8fd650059062-a_path_that_does_not_exist_is_refused_before_anything_is_launched"/><storycode:paste select="#f-8fd650059062-a_named_app_that_is_not_there_names_it_rather_than_falling_back"/><storycode:paste select="#f-8fd650059062-short_forms_open_a_blank_window_or_the_named_document"/><storycode:paste select="#f-8fd650059062-help_and_version_work_without_the_desktop_and_typos_still_get_suggestions"/></storycode:file>

## Implementation review

### Implementation

- [crates/hickory-cli/src/cli_launch.rs](contract.md#output-20cd237e20c3)

### Tests and fixtures

- [crates/hickory-cli/tests/open_app.rs](contract.md#output-8fd650059062)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
