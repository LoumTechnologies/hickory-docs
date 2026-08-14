//! `<hick:capture>` end to end, through `hick run`.
//!
//! Protects docs/specs/freeform/literate-debugging.md
//!
//! The value of a capture is that it survives into the woven document, where
//! `<hick:expect>` can pin it. So this drives the real binary and reads the
//! real output file, rather than testing the runner in isolation — that is
//! what `hick-dap/tests/live_capture.rs` does.

use std::path::Path;
use std::process::Command;

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="pricing.md">
# Pricing

<hick:container name="lab" />

<hick:file path="pricing.py">
LINES = [(2, 9.99), (1, 24.50)]


def line_total(quantity, unit_price):
    subtotal = quantity * unit_price
    return subtotal


print(f"total {sum(line_total(q, p) for q, p in LINES):.2f}")
</hick:file>

<hick:exec container="lab">
python3 pricing.py
  <hick:capture at="pricing.py:5" of="quantity, unit_price" />
</hick:exec>
</hick:doc>
"##;

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

fn adapter_available(root: &Path) -> bool {
    hick_dap::discover("python", root).is_some()
}

#[test]
fn a_capture_weaves_the_values_from_inside_the_function() {
    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("pricing.hick");
    std::fs::write(&doc, DOC).unwrap();
    if !adapter_available(dir.path()) {
        eprintln!("SKIPPED: no Python debug adapter (`hick dap install python`)");
        return;
    }

    let out = hick().arg("run").arg(&doc).output().expect("run hick");
    assert!(
        out.status.success(),
        "the run failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let woven = std::fs::read_to_string(dir.path().join("pricing.md")).expect("woven output");
    // The cell's own stdout is still the cell's own stdout.
    assert!(woven.contains("total 44.48"), "{woven}");
    // And the values from inside `line_total`, one row per hit.
    assert!(
        woven.contains("| hit | `quantity` | `unit_price` |"),
        "{woven}"
    );
    assert!(woven.contains("| 1 | `2` |"), "{woven}");
    assert!(woven.contains("| 2 | `1` |"), "{woven}");
}

#[test]
fn a_captured_run_leaves_the_project_alone() {
    // The capture runs the program a second time, under a debugger. That run
    // is a reader: it must not add a file to the project, even though the
    // ordinary cell may.
    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("pricing.hick");
    std::fs::write(&doc, DOC).unwrap();
    if !adapter_available(dir.path()) {
        eprintln!("SKIPPED: no Python debug adapter (`hick dap install python`)");
        return;
    }

    assert!(hick().arg("run").arg(&doc).status().unwrap().success());

    let mut left: Vec<String> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| !name.starts_with('.'))
        .collect();
    left.sort();
    // The document, its woven output, and the file the document generates.
    // Nothing the debug run touched.
    assert_eq!(
        left,
        ["pricing.hick", "pricing.md", "pricing.py"],
        "{left:?}"
    );
}

#[test]
fn a_malformed_capture_is_refused_before_anything_runs() {
    // A capture that cannot mean anything is a parse-time complaint, named
    // with its line — not a table that quietly comes back empty.
    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("bad.hick");
    std::fs::write(
        &doc,
        DOC.replace("at=\"pricing.py:5\"", "at=\"pricing.py\""),
    )
    .unwrap();

    let out = hick().arg("run").arg(&doc).output().expect("run hick");
    assert!(
        !out.status.success(),
        "a malformed capture must fail the run"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("bad.hick"), "{stderr}");
    assert!(stderr.contains("at=\"file.py:14\""), "{stderr}");
    // And it stopped before the cell ran: whatever the weave wrote on the way
    // through, no command produced output into it.
    let woven = std::fs::read_to_string(dir.path().join("pricing.md")).unwrap_or_default();
    assert!(!woven.contains("total 44.48"), "{woven}");
}
