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
