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

// ---------------------------------------------------------------------------
// renderer="graph": the scene you can drag.
// Protects docs/guarantees/authoring/a-drawn-diagram-is-document-text.md and
// docs/guarantees/authoring/a-derived-diagram-keeps-your-layout.md.
// ---------------------------------------------------------------------------

const GRAPH_DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="arch.md">
# Layers

<hick:diagram renderer="graph" asserts="#no-back-edges">
{
  "nodes": [
    {"id": "api", "label": "API server"},
    {"id": "store", "label": "Postgres", "shape": "cylinder"}
  ],
  "edges": [
    {"from": "api", "to": "store", "label": "SQL"}
  ],
  "layout": {
    "api": {"x": 40, "y": 30, "w": 160, "h": 64},
    "store": {"x": 40, "y": 190, "w": 160, "h": 64}
  }
}
</hick:diagram>

<hick:exec id="no-back-edges" container="sh">
echo 0
<hick:expect match="exact">0
</hick:expect>
</hick:exec>
</hick:doc>
"##;

#[test]
fn a_graph_scene_weaves_to_the_picture_that_was_drawn() {
    // The scene's JSON is for the editor; the weave is for a reader with no
    // hick installed. The woven form is an SVG the weave itself draws — the
    // author's positions, sizes, shapes and colours — because a mermaid
    // downgrade re-laid the diagram out and looked like a different drawing.
    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("arch.hick");
    std::fs::write(&doc, GRAPH_DOC).unwrap();

    let out = hick().arg("weave").arg(&doc).output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let woven = std::fs::read_to_string(dir.path().join("arch.md")).unwrap();
    assert!(
        woven.contains("![diagram](diagram-"),
        "no image reference in:\n{woven}"
    );
    assert!(!woven.contains("hick:diagram"), "tag leaked:\n{woven}");

    // The referenced SVG is a real written file holding the real drawing.
    let name = woven
        .split("![diagram](")
        .nth(1)
        .and_then(|rest| rest.split(')').next())
        .expect("an image reference");
    let svg = std::fs::read_to_string(dir.path().join(name)).expect("the SVG file exists");
    assert!(svg.starts_with("<svg"), "{svg}");
    assert!(svg.contains(">Postgres</text>"), "label missing:\n{svg}");
    assert!(svg.contains(">SQL</text>"), "edge label missing:\n{svg}");
    // The author's layout is IN the picture: the store sits at y=190.
    assert!(svg.contains("190"), "positions dropped:\n{svg}");
}

#[test]
fn a_scene_that_does_not_parse_weaves_as_its_json_and_is_warned_about() {
    // Never eat the author's bytes: an unparseable scene weaves as what it
    // is, and the warning names the problem before anything runs.
    let source = GRAPH_DOC.replace(r##""nodes": ["##, r##""nodes": [ BROKEN"##);
    let warnings = hickory_cli::diagram_assertion_warnings(&parse(&source));
    assert!(
        warnings.iter().any(|w| w.contains("not a scene")),
        "{warnings:?}"
    );

    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("arch.hick");
    std::fs::write(&doc, &source).unwrap();
    let out = hick().arg("weave").arg(&doc).output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let woven = std::fs::read_to_string(dir.path().join("arch.md")).unwrap();
    assert!(woven.contains("```json"), "raw body not shown:\n{woven}");
    assert!(woven.contains("BROKEN"), "author's bytes eaten:\n{woven}");
}

#[test]
fn a_well_formed_scene_with_a_broken_reference_is_warned_about() {
    let source = GRAPH_DOC.replace(r##""to": "store""##, r##""to": "ghost""##);
    let warnings = hickory_cli::diagram_assertion_warnings(&parse(&source));
    assert!(
        warnings.iter().any(|w| w.contains("'ghost'")),
        "{warnings:?}"
    );
}

#[test]
fn a_derived_scene_takes_its_topology_from_a_fragment_and_keeps_its_own_layout() {
    // The generator owns the topology (a copy fragment, pinnable by the cell
    // that wrote it); the person owns the layout. The weave sees them joined.
    let source = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="arch.md">
<hick:copy id="arch-topology">{"nodes": [{"id": "api"}, {"id": "store"}], "edges": [{"from": "api", "to": "store"}]}</hick:copy>

<hick:diagram renderer="graph">
{
  "topology": <hick:paste select="#arch-topology" />,
  "layout": {"api": {"x": 0, "y": 0}}
}
</hick:diagram>
</hick:doc>
"##;
    let dir = tempfile::tempdir().unwrap();
    let doc = dir.path().join("arch.hick");
    std::fs::write(&doc, source).unwrap();
    let out = hick().arg("weave").arg(&doc).output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let woven = std::fs::read_to_string(dir.path().join("arch.md")).unwrap();
    assert!(
        woven.contains("![diagram](diagram-"),
        "no image reference in:\n{woven}"
    );
    let name = woven
        .split("![diagram](")
        .nth(1)
        .and_then(|rest| rest.split(')').next())
        .expect("an image reference");
    let svg = std::fs::read_to_string(dir.path().join(name)).expect("the SVG file exists");
    assert!(
        svg.contains(">api</text>") && svg.contains(">store</text>"),
        "pasted topology missing:\n{svg}"
    );
}
