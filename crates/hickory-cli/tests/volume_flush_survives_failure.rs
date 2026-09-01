//! An earlier cell's real output volume survives a LATER cell's failure.
//!
//! Guarantee:
//! docs/guarantees/execution/an-earlier-cells-output-survives-a-later-cells-failure.md
//!
//! Before this existed, output-volume files were flushed into
//! `PipelineResult::files` only once, after the WHOLE exec loop finished —
//! so a cell that failed after an earlier cell had already, genuinely
//! succeeded left that earlier cell's real output invisible on disk
//! anywhere at all. Drives the real binary because the claim is about what
//! actually reaches disk, not about a function in isolation.

use std::process::Command;

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

fn doc() -> &'static str {
    r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
<hick:container name="c" />
<hick:volume name="out" output="app" />

<hick:exec container="c" mount="out:proj">
mkdir -p proj && printf 'real output\n' > proj/file.txt
</hick:exec>

<hick:exec container="c">
exit 1
</hick:exec>
</hick:doc>
"##
}

#[test]
fn an_earlier_cells_real_output_is_mirrored_even_though_the_run_fails() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("report.hick"), doc()).expect("write doc");

    let out = hick()
        .arg("run")
        .arg(dir.path().join("report.hick"))
        .env("HICKORY_EXECUTOR", "local")
        .output()
        .expect("run hick");
    assert!(
        !out.status.success(),
        "the second cell's exit 1 must fail the run"
    );

    // The real output tree never gets cell 1's file: the whole run failed,
    // so `write_outputs_detailed` — the authoritative write — never ran.
    assert!(
        !dir.path().join("app/file.txt").exists(),
        "the real output tree must not gain a file from a run that failed"
    );

    // But the incremental mirror does — cell 1 genuinely succeeded, and its
    // real output was known before cell 2 ever ran.
    let mirrored = dir.path().join(".hick-cache/last-run/app/file.txt");
    let content = std::fs::read_to_string(&mirrored).unwrap_or_else(|e| {
        panic!(
            "cell 1's real output should be mirrored at {}: {e}",
            mirrored.display()
        )
    });
    assert_eq!(content, "real output\n");
}

/// The mirror is best-effort visibility, never a second source of truth: a
/// FULLY successful run still writes the real output tree the normal way,
/// and the mirror must not be mistaken for it.
#[test]
fn a_fully_successful_run_writes_the_real_tree_not_only_the_mirror() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ok_doc = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
<hick:container name="c" />
<hick:volume name="out" output="app" />

<hick:exec container="c" mount="out:proj">
mkdir -p proj && printf 'real output\n' > proj/file.txt
</hick:exec>
</hick:doc>
"##;
    std::fs::write(dir.path().join("report.hick"), ok_doc).expect("write doc");

    let out = hick()
        .arg("run")
        .arg(dir.path().join("report.hick"))
        .env("HICKORY_EXECUTOR", "local")
        .output()
        .expect("run hick");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    assert_eq!(
        std::fs::read_to_string(dir.path().join("app/file.txt")).unwrap(),
        "real output\n",
        "the real output tree must still be written on success"
    );
}
