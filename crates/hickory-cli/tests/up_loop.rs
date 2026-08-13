//! `hick up` — the loop that keeps a folder woven and carries edits back.
//!
//! Guarantees protected here:
//! - `docs/guarantees/authoring/an-output-edit-lands-in-its-document.md`
//! - `docs/guarantees/authoring/a-generated-file-refuses-an-edit.md`
//! - `docs/guarantees/authoring/one-loop-owns-a-directory.md`
//!
//! These drive the real binary against a real directory and wait on the
//! filesystem, because that is the whole surface under test: an editor writing
//! a file and a process noticing. An in-process call would test everything
//! except the part that breaks.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

const DOC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="greeter.md">
# Greeter

Prose that belongs to the document.

<hick:file path="greeter.py">
def greet(name):
    return f"Hello, {name}!"


if __name__ == "__main__":
    print(greet("world"))
</hick:file>
</hick:doc>
"#;

/// A `hick up` process that is killed when the test ends, however it ends.
struct Loop {
    child: Child,
    dir: tempfile::TempDir,
}

impl Loop {
    fn start(doc: &str) -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("greeter.hick"), doc).expect("write doc");
        let child = hick()
            .arg("up")
            .arg(dir.path())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn hick up");
        let this = Self { child, dir };
        // The loop is ready once it has written the outputs of the first weave.
        wait_until(&this.path("greeter.py"), |_| true).expect("initial weave");
        this
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.path().join(name)
    }
}

impl Drop for Loop {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Wait for a file to exist and satisfy `pred`, up to a generous timeout.
///
/// Generous because this waits on a debounce plus a weave, and a loaded CI
/// runner is slower than a laptop by more than the margin a tight timeout
/// would leave.
fn wait_until(path: &Path, pred: impl Fn(&str) -> bool) -> Option<String> {
    let deadline = Instant::now() + Duration::from_secs(20);
    let mut last = None;
    while Instant::now() < deadline {
        if let Ok(content) = std::fs::read_to_string(path) {
            if pred(&content) {
                return Some(content);
            }
            last = Some(content);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = last;
    None
}

/// Save a file the way an editor does: write a sibling and rename over the
/// target. Writing in place would test a code path no editor takes.
fn save_atomically(path: &Path, content: &str) {
    let tmp = path.with_extension("editor-tmp");
    std::fs::write(&tmp, content).expect("write temp");
    std::fs::rename(&tmp, path).expect("rename over target");
}

/// Guarantee: `docs/guarantees/authoring/an-output-edit-lands-in-its-document.md`
#[test]
fn an_edit_saved_in_a_woven_file_lands_in_the_document() {
    let up = Loop::start(DOC);
    let output = up.path("greeter.py");

    let woven = std::fs::read_to_string(&output).expect("read output");
    assert!(
        woven.contains("Hello, {name}!"),
        "unexpected weave: {woven}"
    );
    save_atomically(&output, &woven.replace("Hello, {name}!", "Howdy, {name}!"));

    let doc = wait_until(&up.path("greeter.hick"), |c| c.contains("Howdy"))
        .expect("edit should reach the document");
    assert!(
        doc.contains("Howdy, {name}!"),
        "document did not receive the edit: {doc}"
    );
    // The document is still a document: the edit landed inside the block it
    // came from, not appended or pasted over the structure.
    assert!(doc.contains("<hick:file path=\"greeter.py\">"), "{doc}");
    assert!(doc.contains("# Greeter"), "{doc}");
}

/// An edit made in the document appears in the generated file without asking.
///
/// Guarantee: `docs/guarantees/authoring/an-output-edit-lands-in-its-document.md`
#[test]
fn an_edit_saved_in_the_document_reaches_the_woven_file() {
    let up = Loop::start(DOC);

    let doc_path = up.path("greeter.hick");
    let doc = std::fs::read_to_string(&doc_path).expect("read doc");
    save_atomically(
        &doc_path,
        &doc.replace("greet(\"world\")", "greet(\"everyone\")"),
    );

    let output = wait_until(&up.path("greeter.py"), |c| c.contains("everyone"))
        .expect("document edit should reach the output file");
    assert!(output.contains("greet(\"everyone\")"), "{output}");
}

/// Two regions edited in one save must both land. A single edit span wide
/// enough to cover both would also cover what lies between them.
///
/// Guarantee: `docs/guarantees/authoring/an-output-edit-lands-in-its-document.md`
#[test]
fn two_regions_edited_in_one_save_both_land() {
    let up = Loop::start(DOC);
    let output = up.path("greeter.py");

    let woven = std::fs::read_to_string(&output).expect("read output");
    let edited = woven
        .replace("def greet(name):", "def greet(name, punct=\"!\"):")
        .replace("greet(\"world\")", "greet(\"world\", \"?\")");
    save_atomically(&output, &edited);

    let doc = wait_until(&up.path("greeter.hick"), |c| c.contains("punct"))
        .expect("first region should reach the document");
    assert!(doc.contains("punct=\"!\""), "first region missing: {doc}");
    assert!(
        doc.contains("\"world\", \"?\""),
        "second region missing: {doc}"
    );
}

/// Guarantee: `docs/guarantees/authoring/a-generated-file-refuses-an-edit.md`
#[test]
fn a_fully_generated_file_is_read_only_and_restores_a_forced_edit() {
    let up = Loop::start(DOC);
    let woven_markdown = up.path("greeter.md");

    let before = wait_until(&woven_markdown, |c| c.contains("# Greeter"))
        .expect("woven markdown should exist");

    let meta = std::fs::metadata(&woven_markdown).expect("stat");
    assert!(
        meta.permissions().readonly(),
        "a file with no editable byte should be read-only while the loop runs"
    );

    // Force the edit through anyway, the way `:w!` does.
    #[cfg(unix)]
    let perms = {
        use std::os::unix::fs::PermissionsExt;
        std::fs::Permissions::from_mode(0o644)
    };
    #[cfg(not(unix))]
    #[allow(clippy::permissions_set_readonly_false)]
    let perms = {
        let mut p = meta.permissions();
        p.set_readonly(false);
        p
    };
    std::fs::set_permissions(&woven_markdown, perms).expect("chmod");
    save_atomically(&woven_markdown, &before.replace("# Greeter", "# Tampered"));

    let restored = wait_until(&woven_markdown, |c| !c.contains("# Tampered"))
        .expect("the refused edit should be restored");
    assert_eq!(
        restored, before,
        "file should be byte-identical to the weave"
    );

    // The document is untouched: a refusal must not half-apply.
    let doc = std::fs::read_to_string(up.path("greeter.hick")).expect("read doc");
    assert!(!doc.contains("Tampered"), "{doc}");
}

/// Guarantee: `docs/guarantees/authoring/one-loop-owns-a-directory.md`
#[test]
fn a_second_loop_on_the_same_directory_refuses_to_start() {
    let up = Loop::start(DOC);

    let out = hick()
        .arg("up")
        .arg(up.dir.path())
        .output()
        .expect("run second hick up");

    assert!(!out.status.success(), "the second loop should have refused");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("already open in another Hickory Docs process"),
        "the refusal should say why: {stderr}"
    );
    // A refusal an operator can act on names the lock it is talking about.
    assert!(stderr.contains("up.lock"), "{stderr}");
}

/// A directory with no documents is a mistake worth naming, not an empty
/// success that sits there watching nothing.
#[test]
fn an_empty_directory_is_an_error_that_says_so() {
    let dir = tempfile::tempdir().expect("tempdir");
    let out = hick()
        .arg("up")
        .arg(dir.path())
        .output()
        .expect("run hick up");

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("No .hick files found"), "{stderr}");
    // The message every other command gives, with its list of ways out —
    // `up` must not grow a second, thinner one.
    assert!(
        stderr.contains("Add .hick files to the directory"),
        "{stderr}"
    );
}
