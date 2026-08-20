//! Display maths, and what a reader with no hick installed sees.
//!
//! Protects docs/guarantees/authoring/an-equation-renders-and-shows-its-source.md
//!
//! The app typesets a `<hick:math>` block in place. The woven markdown is the
//! other half of that promise: an equation that only existed inside our own
//! editor would be worth less than the image someone would otherwise paste.

use std::process::Command;

fn hick() -> Command {
    Command::new(env!("CARGO_BIN_EXE_hick"))
}

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="physics.md">
# Energy

The relation itself:

<hick:math>
e = mc^2
</hick:math>
</hick:doc>
"##;

fn weave(source: &str) -> String {
    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("physics.hick");
    std::fs::write(&doc, source).unwrap();
    let out = hick().arg("weave").arg(&doc).output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    std::fs::read_to_string(dir.path().join("physics.md")).unwrap()
}

#[test]
fn maths_weaves_into_the_dollar_block_every_renderer_understands() {
    // `$$…$$` is what GitHub, an editor preview, and every static-site
    // generator already draw. A ```latex fence would show a reader the
    // SOURCE of an equation where they expected the equation.
    let woven = weave(DOC);
    assert!(woven.contains("$$"), "no display-maths block in:\n{woven}");
    assert!(woven.contains("e = mc^2"), "maths body missing:\n{woven}");
}

#[test]
fn the_tag_is_source_and_does_not_survive_the_weave() {
    let woven = weave(DOC);
    assert!(
        !woven.contains("hick:math"),
        "tag leaked into the weave:\n{woven}"
    );
}

#[test]
fn the_prose_around_an_equation_is_untouched() {
    // The equation is a paragraph among paragraphs, not a section break.
    let woven = weave(DOC);
    assert!(woven.contains("# Energy"), "heading missing:\n{woven}");
    assert!(
        woven.contains("The relation itself:"),
        "prose missing:\n{woven}"
    );
}
