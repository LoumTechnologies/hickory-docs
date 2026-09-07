//! `hick census` counts what was not typed.
//!
//! Guarantee: docs/guarantees/lineage/a-census-counts-what-was-not-typed.md
//!
//! Drives the real binary against a document holding one of each thing the
//! census has to tell apart: a literal `hick:file`, a `hick:file` that is a
//! pasted fragment, a `hick:file` whose bytes are an exec's output
//! (`show="output"`), and a cell that writes into an output volume. The
//! document is RUN once with `--cache` so the recording exists, and the
//! census then weaves it — never running a cell — which is the whole claim.

use std::path::Path;
use std::process::Command;

fn hick() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_hick"));
    cmd.env("HICKORY_EXECUTOR", "local");
    cmd
}

/// Four produced files, one of each origin the census separates.
///
/// `typed.py` is literal text; `pasted.py` is a `hick:copy` fragment pasted
/// verbatim, which lineage reports as `paste` and the census counts as
/// authored for the same reason it counts literal text — a person typed
/// those bytes, once, somewhere in the document. `ran.txt` is exec output.
/// The last cell writes `made.txt` into the `built` volume, which lands in
/// `generated/` and has no byte lineage at all.
const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="census.md">
# A census fixture

One paragraph of prose, typed by a person.

<hick:container name="sh" image="host" />
<hick:volume name="built" output="generated" />

<hick:copy id="greeting" language="python">
def greet():
    return "hello"
</hick:copy>

<hick:file path="typed.py">
def typed():
    return 1
</hick:file>

<hick:file path="pasted.py"><hick:paste select="#greeting" /></hick:file>

<hick:file path="ran.txt"><hick:exec container="sh" show="output">printf 'one\ntwo\nthree\n'</hick:exec></hick:file>

<hick:exec container="sh" mount="built:out">mkdir -p out && printf 'a\nb\n' > out/made.txt</hick:exec>
</hick:doc>
"##;

fn setup(dir: &Path) {
    std::fs::write(dir.join("census.hick"), DOC).expect("write doc");
    let run = hick()
        .arg("run")
        .arg("--cache")
        .arg(dir.join("census.hick"))
        .output()
        .expect("run hick");
    assert!(
        run.status.success(),
        "hick run failed:\n{}",
        String::from_utf8_lossy(&run.stderr)
    );
}

fn census_text(dir: &Path) -> String {
    let out = hick()
        .arg("census")
        .arg("--json")
        .arg(dir)
        .output()
        .expect("run hick census");
    assert!(
        out.status.success(),
        "hick census failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf-8")
}

fn census_json(dir: &Path) -> serde_json::Value {
    serde_json::from_str(&census_text(dir)).expect("census emits one JSON object")
}

fn file<'a>(census: &'a serde_json::Value, path: &str) -> &'a serde_json::Value {
    census
        .get("files")
        .and_then(|f| f.as_array())
        .expect("files array")
        .iter()
        .find(|f| f["path"] == path)
        .unwrap_or_else(|| panic!("no census row for {path} in {census:#}"))
}

#[test]
fn literal_bytes_are_authored_and_exec_output_is_not() {
    let dir = tempfile::tempdir().expect("tempdir");
    setup(dir.path());
    let census = census_json(dir.path());

    // Literal text: every line is the author's, including the blank one.
    let typed = file(&census, "typed.py");
    assert_eq!(typed["kind"], "tangled");
    assert_eq!(typed["generated"], 0, "typed.py: {typed:#}");
    assert_eq!(typed["cell_output"], 0);
    assert!(typed["authored"].as_u64().expect("authored") >= 2);

    // A pasted fragment is bytes a person typed, in another element.
    let pasted = file(&census, "pasted.py");
    assert_eq!(pasted["generated"], 0, "pasted.py: {pasted:#}");
    assert!(pasted["authored"].as_u64().expect("authored") >= 2);

    // Exec output: nobody typed any of it.
    let ran = file(&census, "ran.txt");
    assert_eq!(ran["authored"], 0, "ran.txt: {ran:#}");
    assert_eq!(ran["cell_output"], 0);
    assert_eq!(ran["generated"], ran["lines"], "ran.txt: {ran:#}");
    assert!(ran["generated"].as_u64().expect("generated") >= 3);
}

#[test]
fn a_volume_file_is_the_cells_and_is_counted_apart() {
    let dir = tempfile::tempdir().expect("tempdir");
    setup(dir.path());
    let census = census_json(dir.path());

    let made = file(&census, "generated/made.txt");
    assert_eq!(made["kind"], "cell-output");
    assert_eq!(made["authored"], 0);
    assert_eq!(made["generated"], 0);
    assert_eq!(made["cell_output"], 2, "generated/made.txt: {made:#}");
    assert_eq!(made["volume"], "built");
    assert_eq!(made["cells"], serde_json::json!(["sh"]));
    assert_eq!(made["document"], "census.hick");
}

#[test]
fn the_weave_mixes_typed_prose_with_the_weavers_own_lines() {
    let dir = tempfile::tempdir().expect("tempdir");
    setup(dir.path());
    let census = census_json(dir.path());

    // The woven markdown holds the author's prose AND the fences, headings
    // and transcript the weaver drew — so both columns are non-zero, which
    // is the mixed-line rule doing its job.
    let woven = file(&census, "census.md");
    assert_eq!(woven["kind"], "woven");
    assert!(woven["authored"].as_u64().expect("authored") > 0);
    assert!(woven["generated"].as_u64().expect("generated") > 0);
    assert_eq!(
        woven["lines"].as_u64().expect("lines"),
        woven["authored"].as_u64().expect("authored")
            + woven["generated"].as_u64().expect("generated")
    );
}

#[test]
fn the_json_shape_is_stable_enough_to_pin() {
    let dir = tempfile::tempdir().expect("tempdir");
    setup(dir.path());
    let text = census_text(dir.path());
    let census: serde_json::Value = serde_json::from_str(&text).expect("one JSON object");

    // Key ORDER, not just presence: a `hick:exec` cell pins this output with
    // a `hick:expect`, so a reordering is a breaking change. Read off the
    // emitted text, because serde_json's own `Value` sorts keys.
    let mut at = 0usize;
    for key in [
        "\n  \"root\":",
        "\n  \"documents\":",
        "\n  \"files\":",
        "\n  \"folders\":",
        "\n  \"totals\":",
        "\n  \"document_totals\":",
        "\n  \"unproduced\":",
        "\n  \"empty_volumes\":",
    ] {
        let found = text
            .find(key)
            .unwrap_or_else(|| panic!("census --json has no top-level {key}:\n{text}"));
        assert!(found > at, "top-level keys are out of order at {key}");
        at = found;
    }

    let totals = &census["totals"];
    let sum: u64 = ["authored", "generated", "cell_output"]
        .iter()
        .map(|k| totals[*k].as_u64().expect("count"))
        .sum();
    assert_eq!(totals["lines"].as_u64().expect("lines"), sum);

    // Every produced file is charged to exactly one folder total.
    let folders: u64 = census["folders"]
        .as_array()
        .expect("folders")
        .iter()
        .map(|f| f["files"].as_u64().expect("files"))
        .sum();
    assert_eq!(
        folders,
        census["files"].as_array().expect("files").len() as u64
    );

    // The document's own lines are reported apart from what it produced.
    let doc = &census["documents"][0];
    assert_eq!(doc["path"], "census.hick");
    assert!(doc["prose"].as_u64().expect("prose") > 0);
    assert!(
        doc["fragment"].as_u64().expect("fragment") > 0,
        "the fixture has code in it: {doc:#}"
    );
    assert!(doc["tag"].as_u64().expect("tag") > 0);
    assert_eq!(
        doc["lines"].as_u64().expect("lines"),
        doc["prose"].as_u64().expect("prose")
            + doc["fragment"].as_u64().expect("fragment")
            + doc["tag"].as_u64().expect("tag")
    );
}

#[test]
fn only_generated_hides_the_files_that_are_entirely_yours() {
    let dir = tempfile::tempdir().expect("tempdir");
    setup(dir.path());
    let out = hick()
        .arg("census")
        .arg("--only-generated")
        .arg(dir.path())
        .output()
        .expect("run hick census");
    assert!(out.status.success());
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("ran.txt"), "{text}");
    assert!(text.contains("generated/made.txt"), "{text}");
    assert!(
        !text.lines().any(|l| l.starts_with("typed.py")),
        "typed.py has no generated line, so it is not listed:\n{text}"
    );
}
