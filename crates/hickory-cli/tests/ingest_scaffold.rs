//! `hick ingest --from` — the scaffolder's door.
//!
//! Protects `docs/guarantees/authoring/ingest-owns-what-a-scaffolder-wrote.md`
//! and `docs/guarantees/lineage/ingested-bytes-are-not-yours.md`.
//!
//! Drives the real binary against a real run, because the claim being made is
//! about what ends up in the `.hick` file on disk and what the document then
//! produces from it — not about a function in isolation.

use std::path::Path;
use std::process::Command;

fn hick() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_hick"));
    // The local executor keeps the test to one process and no daemon; the
    // door being tested is the volume, which every executor extracts the
    // same way.
    cmd.env("HICKORY_EXECUTOR", "local");
    cmd
}

/// A cell that writes a scaffold into its mount: two source files and a
/// build-output directory the project ignores.
fn doc_source() -> String {
    let write = if cfg!(windows) {
        "mkdir out\\obj & echo hello> out\\main.txt & echo lib> out\\lib.txt & echo junk> out\\obj\\build.log"
    } else {
        "mkdir -p out/obj && printf 'hello\\n' > out/main.txt && printf 'lib\\n' > out/lib.txt && printf 'junk\\n' > out/obj/build.log"
    };
    format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
# A scaffolded app

<hick:container name="sdk" />
<hick:volume name="project" output="app" />

<hick:exec container="sdk" mount="project:out">
<hick:copy id="scaffold">
{write}
</hick:copy>
</hick:exec>
</hick:doc>
"##
    )
}

/// A repository, because the gitignore filter is the repository's own.
fn repo(dir: &Path, gitignore: &str) {
    let git = |args: &[&str]| {
        Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .expect("run git");
    };
    git(&["init", "-q"]);
    git(&["config", "user.email", "t@example.com"]);
    git(&["config", "user.name", "T"]);
    std::fs::write(dir.join(".gitignore"), gitignore).unwrap();
}

#[test]
fn an_ingest_writes_the_runs_files_into_the_document_and_skips_what_git_ignores() {
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path(), "obj/\n");
    let doc = dir.path().join("app.hick");
    std::fs::write(&doc, doc_source()).unwrap();

    let out = hick()
        .args(["ingest", "--from", "#scaffold"])
        .arg(&doc)
        .output()
        .expect("run hick");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(out.status.success(), "ingest failed: {stderr}\n{stdout}");

    let source = std::fs::read_to_string(&doc).unwrap();
    // The element shape is the load-bearing decision: `exec > ingested >
    // file`, never `exec > file`, because `file > exec` already means the
    // opposite. See docs/specs/freeform/owning-what-a-scaffolder-wrote.md.
    assert!(
        source.contains("<hick:ingested from=\"#scaffold\""),
        "{source}"
    );
    assert!(source.contains("sha256=\""), "{source}");
    assert!(source.contains("files=\"2\""), "{source}");
    assert!(source.contains("skipped=\"1\""), "{source}");
    assert!(
        source.contains("<hick:file path=\"app/main.txt\">"),
        "{source}"
    );
    assert!(
        source.contains("<hick:file path=\"app/lib.txt\">"),
        "{source}"
    );
    // Build output the project already ignores must not enter the document
    // that owns the source.
    assert!(!source.contains("path=\"obj/build.log\""), "{source}");
    // The block is INSIDE the cell, not beside it.
    let exec_at = source.find("<hick:exec").unwrap();
    let ingested_at = source.find("<hick:ingested").unwrap();
    let close_at = source.find("</hick:exec>").unwrap();
    assert!(exec_at < ingested_at && ingested_at < close_at, "{source}");

    // And what was skipped is named, not merely counted.
    assert!(stdout.contains("obj/build.log"), "{stdout}");
}

// Protects docs/guarantees/authoring/an-ingest-says-when-it-could-not-check-gitignore.md
#[test]
fn ingesting_outside_a_git_repository_warns_that_gitignore_was_never_consulted() {
    // No `repo()` call: this directory is deliberately not a git repository,
    // which is exactly the case `gitignored()` returns `None` for — "could
    // not check" rather than "checked, found nothing to skip".
    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("app.hick");
    std::fs::write(&doc, doc_source()).unwrap();

    let out = hick()
        .args(["ingest", "--from", "#scaffold"])
        .arg(&doc)
        .output()
        .expect("run hick");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(out.status.success(), "ingest failed: {stderr}\n{stdout}");

    // Filtering never ran, so `obj/build.log` — which a real project would
    // gitignore — is ingested unfiltered rather than silently dropped.
    let source = std::fs::read_to_string(&doc).unwrap();
    assert!(source.contains("skipped=\"0\""), "{source}");
    assert!(source.contains("path=\"app/obj/build.log\""), "{source}");

    // And the reason `skipped="0"` means "never checked" here, not "checked,
    // found nothing" — which the document's own attributes cannot
    // distinguish — is on stderr instead.
    assert!(
        stderr.contains("not a git repository") && stderr.contains(".gitignore filtering"),
        "{stderr}"
    );
}

#[test]
fn the_ingested_document_produces_the_scaffold_from_a_clone() {
    // The whole point of ingest over a `from=` into the transcript cache:
    // the base survives a clone, so a weave — no execution at all — writes
    // the scaffolder's files.
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path(), "obj/\n");
    let doc = dir.path().join("app.hick");
    std::fs::write(&doc, doc_source()).unwrap();
    assert!(
        hick()
            .args(["ingest", "--from", "#scaffold"])
            .arg(&doc)
            .output()
            .expect("run hick")
            .status
            .success()
    );

    let out = hick().arg("weave").arg(&doc).output().expect("run hick");
    assert!(
        out.status.success(),
        "weave failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("app/main.txt")).unwrap(),
        "hello\n"
    );
}

#[test]
fn an_ingested_body_starts_on_its_own_line_without_changing_a_byte() {
    // Legibility, and it costs the file nothing: the break that ends the open
    // tag's line belongs to the tag. See
    // docs/guarantees/language/a-generated-file-starts-at-its-first-byte.md.
    //
    // Before that rule this had to be written inline — a scaffolded file
    // jammed onto the end of its own tag, which is the first thing a reader of
    // the document meets. The byte check below is the half that made it
    // impossible then and makes it safe now.
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path(), "obj/\n");
    let doc = dir.path().join("app.hick");
    std::fs::write(&doc, doc_source()).unwrap();
    assert!(
        hick()
            .args(["ingest", "--from", "#scaffold"])
            .arg(&doc)
            .output()
            .expect("run hick")
            .status
            .success()
    );

    let source = std::fs::read_to_string(&doc).unwrap();
    assert!(
        source.contains("<hick:file path=\"app/main.txt\">\nhello\n</hick:file>"),
        "the body belongs on its own line:\n{source}"
    );

    // And the file the document produces is byte-for-byte what the run wrote,
    // with no newline gained from the formatting above.
    let out = hick().arg("weave").arg(&doc).output().expect("run hick");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("app/main.txt")).unwrap(),
        "hello\n"
    );
}

#[test]
fn ingested_bytes_report_their_run_rather_than_reading_as_yours() {
    // Marking a scaffolder's files `literal` would make forty files of
    // somebody else's code claim to be text you wrote; marking them `exec`
    // would make them synthetic and kill the reverse edit on the very bytes
    // you most want to edit.
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path(), "obj/\n");
    let doc = dir.path().join("app.hick");
    std::fs::write(&doc, doc_source()).unwrap();
    assert!(
        hick()
            .args(["ingest", "--from", "#scaffold"])
            .arg(&doc)
            .output()
            .expect("run hick")
            .status
            .success()
    );

    let out = hick()
        .args(["lineage"])
        .arg(&doc)
        .args(["--output", "app/main.txt"])
        .output()
        .expect("run hick");
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        out.status.success(),
        "lineage failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout.contains("ingested"), "{stdout}");
    assert!(!stdout.contains("literal"), "{stdout}");
    assert!(stdout.contains("run "), "{stdout}");
}

#[test]
fn a_second_ingest_is_refused_and_names_the_recorded_base() {
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path(), "obj/\n");
    let doc = dir.path().join("app.hick");
    std::fs::write(&doc, doc_source()).unwrap();
    assert!(
        hick()
            .args(["ingest", "--from", "#scaffold"])
            .arg(&doc)
            .output()
            .expect("run hick")
            .status
            .success()
    );
    let before = std::fs::read_to_string(&doc).unwrap();

    let out = hick()
        .args(["ingest", "--from", "#scaffold"])
        .arg(&doc)
        .output()
        .expect("run hick");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    // An ingest happens once. The re-ingest merge that used to run here was
    // retired by lenses.md: a scaffold is upgraded through its commit's
    // recipe, and the refusal says where.
    assert!(
        stderr.contains("already has an <hick:ingested> block"),
        "{stderr}"
    );
    assert!(stderr.contains("sha256"), "{stderr}");
    assert!(stderr.contains("history lens"), "{stderr}");
    // Refused means nothing changed.
    assert_eq!(std::fs::read_to_string(&doc).unwrap(), before);
}

#[test]
fn a_binary_the_project_would_keep_is_refused_by_name() {
    // A `hick:file` body is raw bytes under the no-escaping invariant, so
    // there is no encoding to hide a binary in. Counted and named, never
    // silently mangled.
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path(), "");
    let doc = dir.path().join("app.hick");
    let write = if cfg!(windows) {
        // `certutil -decode` is the portable-enough way to land a NUL byte.
        "mkdir out & echo hello> out\\main.txt & copy /b nul out\\logo.png & echo. >> out\\logo.png"
    } else {
        "mkdir -p out && printf 'hello\\n' > out/main.txt && printf '\\000\\377\\376' > out/logo.png"
    };
    std::fs::write(
        &doc,
        format!(
            r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
<hick:container name="sdk" />
<hick:volume name="project" output="app" />
<hick:exec container="sdk" mount="project:out">
<hick:copy id="scaffold">
{write}
</hick:copy>
</hick:exec>
</hick:doc>
"##
        ),
    )
    .unwrap();
    if cfg!(windows) {
        eprintln!("SKIPPED: no portable cmd.exe way to write a non-UTF-8 byte");
        return;
    }

    let before = std::fs::read_to_string(&doc).unwrap();
    let out = hick()
        .args(["ingest", "--from", "#scaffold"])
        .arg(&doc)
        .output()
        .expect("run hick");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(stderr.contains("logo.png"), "{stderr}");
    assert!(stderr.contains("not UTF-8"), "{stderr}");
    // Named AND actionable: the gitignore is the filter.
    assert!(stderr.contains(".gitignore"), "{stderr}");
    assert_eq!(std::fs::read_to_string(&doc).unwrap(), before);
}

#[test]
fn an_id_that_is_not_in_a_cell_is_refused_with_the_ids_that_are() {
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path(), "");
    let doc = dir.path().join("app.hick");
    std::fs::write(&doc, doc_source()).unwrap();

    let out = hick()
        .args(["ingest", "--from", "#nope"])
        .arg(&doc)
        .output()
        .expect("run hick");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(stderr.contains("scaffold"), "{stderr}");
    assert!(stderr.contains("Next step"), "{stderr}");
}

#[test]
fn the_app_shows_the_ingested_files_as_blocks_and_not_as_the_command() {
    // Two failures the app would have had without the block model knowing
    // about `ingested`: the cell's command line would have shown a
    // scaffolder's whole output, and the files the document now owns would
    // have been invisible in the editor.
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path(), "obj/\n");
    let doc = dir.path().join("app.hick");
    std::fs::write(&doc, doc_source()).unwrap();
    assert!(
        hick()
            .args(["ingest", "--from", "#scaffold"])
            .arg(&doc)
            .output()
            .expect("run hick")
            .status
            .success()
    );

    let out = hick()
        .args(["run", "--json"])
        .arg(&doc)
        .output()
        .expect("run hick");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let model: serde_json::Value = serde_json::from_slice(&out.stdout).expect("block model json");
    let text = model.to_string();
    // The files are blocks.
    assert!(text.contains("main.txt"), "{text}");
    assert!(text.contains("lib.txt"), "{text}");
    // And the cell's command is the command, not the scaffold.
    let commands: Vec<&str> = text.matches("mkdir").collect();
    assert!(!commands.is_empty(), "the command survived: {text}");
}

#[test]
fn ingesting_does_not_move_the_files_the_volume_already_placed() {
    // The volume says `output="app"`, so `hick run` writes `app/main.txt`.
    // An ingest must keep writing it there. Stripping the prefix — which this
    // did until 2026-08-24 — meant the act of ingesting silently rearranged
    // the tree it was asked to preserve, and the drift only showed up as a
    // file appearing in a new place on the next run.
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path(), "obj/\n");
    let doc = dir.path().join("app.hick");
    std::fs::write(&doc, doc_source()).unwrap();

    // Where it lands BEFORE the ingest.
    assert!(
        hick()
            .arg("run")
            .arg(&doc)
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(
        dir.path().join("app/main.txt").exists(),
        "the volume writes under app/"
    );

    assert!(
        hick()
            .args(["ingest", "--from", "#scaffold"])
            .arg(&doc)
            .output()
            .unwrap()
            .status
            .success()
    );

    // And after: the same place, from the document's own bytes this time.
    assert!(
        hick()
            .arg("weave")
            .arg(&doc)
            .output()
            .unwrap()
            .status
            .success()
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("app/main.txt")).unwrap(),
        "hello\n"
    );
    assert!(
        !dir.path().join("main.txt").exists(),
        "the ingest moved the file out of its volume's output path"
    );
}

#[test]
fn an_ingest_takes_the_volume_and_not_the_documents_own_files() {
    // A volume declared `output="."` shares its prefix with every other output
    // the document produces. Selecting by prefix swallowed those too, so a
    // document's own `hick:file` blocks got ingested as if a scaffolder had
    // written them.
    let dir = tempfile::tempdir().unwrap();
    repo(dir.path(), "");
    let doc = dir.path().join("app.hick");
    let write = if cfg!(windows) {
        "mkdir out & echo scaffolded> out\\generated.txt"
    } else {
        "mkdir -p out && printf 'scaffolded\\n' > out/generated.txt"
    };
    std::fs::write(
        &doc,
        format!(
            r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
<hick:container name="sdk" />
<hick:volume name="project" output="." />
<hick:file path="mine.txt">I wrote this
</hick:file>
<hick:exec container="sdk" mount="project:out">
<hick:copy id="scaffold">
{write}
</hick:copy>
</hick:exec>
</hick:doc>
"##
        ),
    )
    .unwrap();

    let out = hick()
        .args(["ingest", "--from", "#scaffold"])
        .arg(&doc)
        .output()
        .expect("run hick");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let source = std::fs::read_to_string(&doc).unwrap();
    let ingested_at = source.find("<hick:ingested").unwrap();
    let block = &source[ingested_at..];
    assert!(block.contains("generated.txt"), "{block}");
    // `mine.txt` is the document's own, and must not be inside the block —
    // being ingested would mark bytes the author wrote as somebody else's.
    assert!(
        !block.contains("mine.txt"),
        "the document's own file was ingested: {block}"
    );
    assert!(source.contains("files=\"1\""), "{source}");
}
