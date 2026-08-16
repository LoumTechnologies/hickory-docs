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

/// A document whose weave carries nothing a person wrote.
///
/// Prose in the weave is the document's prose and maps back to it, so a
/// document WITH prose produces a mixed file — writable, protected per range.
/// This one has none: its woven markdown is a generated heading and a fenced
/// copy of a generated file, and every byte of it is synthetic.
const GENERATED_ONLY: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="greeter.md"><hick:file path="greeter.py"><hick:val name="greeting" /></hick:file></hick:doc>
"#;

/// A `hick up` process that is killed when the test ends, however it ends.
struct Loop {
    child: Child,
    dir: tempfile::TempDir,
    /// The child's stderr.
    ///
    /// Captured, not discarded: every wait in this file times out the same
    /// way, so a child that died on startup and a child that is merely slow
    /// produce the identical panic unless its own message is in it. This cost
    /// a real debugging session — `hick up` was exiting immediately with
    /// "Too many open files" (the inotify instance limit), and all five
    /// waiting tests reported only "initial weave".
    ///
    /// It lives outside the watched directory on purpose. A log file *inside*
    /// it would be written by the very loop that is watching it, and the loop
    /// reacting to its own logging is a feedback loop the test would then be
    /// measuring.
    log: tempfile::NamedTempFile,
}

impl Loop {
    fn start(doc: &str) -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("greeter.hick"), doc).expect("write doc");
        let log = tempfile::NamedTempFile::new().expect("stderr capture");
        let sink = log.reopen().expect("reopen stderr capture");
        let child = hick()
            .arg("up")
            .arg(dir.path())
            .stdout(Stdio::null())
            .stderr(Stdio::from(sink))
            .spawn()
            .expect("spawn hick up");
        let mut this = Self { child, dir, log };
        // The loop is ready once it has written the outputs of the first weave.
        if wait_until(&this.path("greeter.py"), |_| true).is_none() {
            panic!("{}", this.diagnose("hick up never wrote its first weave"));
        }
        this
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.path().join(name)
    }

    /// Wait for `path` to satisfy `pred`, failing with the child's own
    /// output rather than a bare timeout message.
    fn wait_for(&mut self, path: &Path, pred: impl Fn(&str) -> bool, what: &str) -> String {
        match wait_until(path, pred) {
            Some(content) => content,
            None => panic!("{}", self.diagnose(what)),
        }
    }

    /// Explain a timeout with what the child process actually said and
    /// whether it is even still running.
    fn diagnose(&mut self, what: &str) -> String {
        let status = match self.child.try_wait() {
            Ok(Some(status)) => format!("the process had already exited ({status})"),
            Ok(None) => "the process was still running".to_string(),
            Err(e) => format!("could not check whether the process was running: {e}"),
        };
        let stderr = std::fs::read_to_string(self.log.path()).unwrap_or_default();
        let stderr = if stderr.trim().is_empty() {
            "  (it printed nothing)".to_string()
        } else {
            stderr
                .lines()
                .map(|l| format!("  {l}"))
                .collect::<Vec<_>>()
                .join("\n")
        };
        format!("{what} within the timeout, and {status}.\nIts stderr:\n{stderr}")
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
    let mut up = Loop::start(DOC);
    let output = up.path("greeter.py");

    let woven = std::fs::read_to_string(&output).expect("read output");
    assert!(
        woven.contains("Hello, {name}!"),
        "unexpected weave: {woven}"
    );
    save_atomically(&output, &woven.replace("Hello, {name}!", "Howdy, {name}!"));

    let doc_path = up.path("greeter.hick");
    let doc = up.wait_for(
        &doc_path,
        |c| c.contains("Howdy"),
        "the edit saved in the output never reached the document",
    );
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
    let mut up = Loop::start(DOC);

    let doc_path = up.path("greeter.hick");
    let doc = std::fs::read_to_string(&doc_path).expect("read doc");
    save_atomically(
        &doc_path,
        &doc.replace("greet(\"world\")", "greet(\"everyone\")"),
    );

    let output_path = up.path("greeter.py");
    let output = up.wait_for(
        &output_path,
        |c| c.contains("everyone"),
        "the edit saved in the document never reached the output file",
    );
    assert!(output.contains("greet(\"everyone\")"), "{output}");
}

/// Two regions edited in one save must both land. A single edit span wide
/// enough to cover both would also cover what lies between them.
///
/// Guarantee: `docs/guarantees/authoring/an-output-edit-lands-in-its-document.md`
#[test]
fn two_regions_edited_in_one_save_both_land() {
    let mut up = Loop::start(DOC);
    let output = up.path("greeter.py");

    let woven = std::fs::read_to_string(&output).expect("read output");
    let edited = woven
        .replace("def greet(name):", "def greet(name, punct=\"!\"):")
        .replace("greet(\"world\")", "greet(\"world\", \"?\")");
    save_atomically(&output, &edited);

    let doc_path = up.path("greeter.hick");
    let doc = up.wait_for(
        &doc_path,
        |c| c.contains("punct"),
        "the first of the two edited regions never reached the document",
    );
    assert!(doc.contains("punct=\"!\""), "first region missing: {doc}");
    assert!(
        doc.contains("\"world\", \"?\""),
        "second region missing: {doc}"
    );
}

/// Guarantee: `docs/guarantees/authoring/a-generated-file-refuses-an-edit.md`
///
/// Property 1 of that guarantee: a file mixing document text with generated
/// text stays WRITABLE, because marking it read-only would block the edits
/// that are legal. Woven markdown is exactly such a file — its prose is the
/// document's prose — so this is the case that says so.
#[test]
fn a_file_mixing_prose_with_generated_text_stays_writable() {
    let up = Loop::start(DOC);
    let woven_markdown = up.path("greeter.md");
    let meta = std::fs::metadata(&woven_markdown).expect("stat");
    assert!(
        !meta.permissions().readonly(),
        "woven markdown carries the document's prose, which is editable"
    );
}

/// Guarantee: `docs/guarantees/authoring/a-generated-file-refuses-an-edit.md`
#[test]
fn a_fully_generated_file_is_read_only_and_restores_a_forced_edit() {
    let mut up = Loop::start(GENERATED_ONLY);
    let woven_markdown = up.path("greeter.md");

    let before = up.wait_for(
        &woven_markdown,
        |c| c.contains("greeter.py"),
        "the woven markdown was never written",
    );

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
    save_atomically(
        &woven_markdown,
        &before.replace("greeter.py", "tampered.py"),
    );

    let restored = up.wait_for(
        &woven_markdown,
        |c| !c.contains("tampered.py"),
        "the forced edit to a fully generated file was never restored",
    );
    assert_eq!(
        restored, before,
        "file should be byte-identical to the weave"
    );

    // The document is untouched: a refusal must not half-apply.
    let doc = std::fs::read_to_string(up.path("greeter.hick")).expect("read doc");
    assert!(!doc.contains("tampered"), "{doc}");
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
