//! Diagrams, and the assertions that keep them honest.
//!
//! Protects docs/guarantees/authoring/a-diagram-names-what-proves-it.md
//!
//! A diagram is the one claim in a repository that nothing re-reads, so the
//! two things worth testing are: it weaves into something a reader can
//! actually see, and the binding between the picture and the cell that proves
//! it cannot rot silently.

use std::process::Command;

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="arch.md">
# Layers

<hick:diagram renderer="mermaid" asserts="#no-back-edges">
flowchart TD
  api --> core
  core --> store
</hick:diagram>

<hick:exec id="no-back-edges" container="sh">
echo 0
<hick:expect match="exact">0
</hick:expect>
</hick:exec>
</hick:doc>
"##;

fn parse(source: &str) -> hick_lang::HickDocument {
    hick_lang::parse(source).expect("the document parses")
}

#[test]
fn a_diagram_weaves_into_a_fence_its_renderer_understands() {
    // The woven markdown is what a reader opens on GitHub, with no hick
    // installed and no notebook. If the picture only existed inside our own
    // editor, the document would be worth less than the drawing it replaced.
    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("arch.hick");
    std::fs::write(&doc, DOC).unwrap();

    let out = hick().arg("weave").arg(&doc).output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let woven = std::fs::read_to_string(dir.path().join("arch.md")).unwrap();
    assert!(woven.contains("```mermaid"), "no fence in:\n{woven}");
    assert!(
        woven.contains("api --> core"),
        "diagram body missing:\n{woven}"
    );
    // The tag itself is source, not output — it must not survive the weave.
    assert!(
        !woven.contains("hick:diagram"),
        "tag leaked into the weave:\n{woven}"
    );
}

#[test]
fn the_renderer_names_the_fence_so_a_new_one_is_a_value_not_a_migration() {
    let source = DOC.replace(r##"renderer="mermaid""##, r##"renderer="d3""##);
    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("arch.hick");
    std::fs::write(&doc, &source).unwrap();

    hick().arg("weave").arg(&doc).output().unwrap();
    let woven = std::fs::read_to_string(dir.path().join("arch.md")).unwrap();
    assert!(woven.contains("```d3"), "renderer ignored:\n{woven}");
}

#[test]
fn a_diagram_proved_by_a_cell_that_exists_warns_about_nothing() {
    assert!(
        hickory_cli::diagram_assertion_warnings(&parse(DOC)).is_empty(),
        "a correctly bound diagram should be quiet"
    );
}

#[test]
fn renaming_the_cell_without_the_reference_is_caught() {
    // The ordinary way a proof gets unhooked from the thing it proved.
    let source = DOC.replace(r##"id="no-back-edges""##, r##"id="layering""##);
    let warnings = hickory_cli::diagram_assertion_warnings(&parse(&source));
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("#no-back-edges"), "{}", warnings[0]);
    assert!(
        warnings[0].contains("does not exist"),
        "the warning should say the check is missing, not merely that something is odd: {}",
        warnings[0]
    );
}

#[test]
fn a_diagram_nothing_checks_says_so_once() {
    let source = DOC.replace(r##" asserts="#no-back-edges""##, "");
    let warnings = hickory_cli::diagram_assertion_warnings(&parse(&source));
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("asserts nothing"), "{}", warnings[0]);
}

#[test]
fn an_unchecked_diagram_still_weaves() {
    // Warning, never error: whether a picture needs proof is the author's
    // call, and a tool that refused to weave a sketch would teach people to
    // draw where it cannot see.
    let source = DOC.replace(r##" asserts="#no-back-edges""##, "");
    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("arch.hick");
    std::fs::write(&doc, &source).unwrap();

    let out = hick().arg("weave").arg(&doc).output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        std::fs::read_to_string(dir.path().join("arch.md"))
            .unwrap()
            .contains("```mermaid")
    );
}

#[test]
fn a_selector_that_is_not_an_id_is_named_as_the_mistake_it_is() {
    let source = DOC.replace(
        r##"asserts="#no-back-edges""##,
        r##"asserts="no-back-edges""##,
    );
    let warnings = hickory_cli::diagram_assertion_warnings(&parse(&source));
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("`#id`"), "{}", warnings[0]);
}

#[test]
fn several_assertions_are_all_checked() {
    let source = DOC.replace(
        r##"asserts="#no-back-edges""##,
        r##"asserts="#no-back-edges #also-missing""##,
    );
    let warnings = hickory_cli::diagram_assertion_warnings(&parse(&source));
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("#also-missing"), "{}", warnings[0]);
}
