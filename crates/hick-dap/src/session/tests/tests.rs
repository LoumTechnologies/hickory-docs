use super::*;

#[test]
fn capabilities_are_read_from_the_adapters_own_answer() {
    // Every control is gated on one of these, so reading them wrongly
    // means offering a button that does nothing.
    let caps = Capabilities::from_initialize(&json!({
        "supportsConditionalBreakpoints": true,
        "supportsRestartFrame": true,
        "supportsSetVariable": true,
        "exceptionBreakpointFilters": [
            { "filter": "raised", "label": "Raised Exceptions" },
            { "filter": "uncaught", "label": "Uncaught Exceptions" }
        ]
    }));
    assert!(caps.conditional_breakpoints);
    assert!(caps.restart_frame);
    assert!(caps.set_variable);
    // Absent means false, never "probably supported".
    assert!(!caps.step_back);
    assert!(!caps.log_points);
    assert_eq!(caps.exception_filters.len(), 2);
    assert_eq!(caps.exception_filters[0].id, "raised");
}

#[test]
fn step_verbs_map_to_the_protocols_own_names() {
    assert_eq!(Step::Over.command(), "next");
    assert_eq!(Step::In.command(), "stepIn");
    assert_eq!(Step::Out.command(), "stepOut");
    assert_eq!(Step::Continue.command(), "continue");
    assert_eq!(Step::DropFrame.command(), "restartFrame");
    assert_eq!(Step::Back.command(), "stepBack");
}

#[test]
fn launch_arguments_can_be_overridden_per_language() {
    // `justMyCode` is right for Python and meaningless elsewhere, so the
    // defaults have to be overridable rather than final.
    let mut base = json!({ "program": "a.py", "justMyCode": true });
    merge(
        &mut base,
        &json!({ "justMyCode": false, "args": ["--fast"] }),
    );
    assert_eq!(base["justMyCode"], json!(false));
    assert_eq!(base["args"], json!(["--fast"]));
    assert_eq!(base["program"], json!("a.py"));
}

#[test]
fn a_document_line_maps_into_the_file_it_generates() {
    let source = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"o.md\">\n\
             # Title\n\n\
             <hick:file path=\"app.py\">\n\
             def total(x):\n\
             \x20   return x * 2\n\
             </hick:file>\n\
             </hick:doc>\n";
    let dir = tempfile::tempdir().unwrap();
    let mapping = Mapping::for_document(Path::new("d.md"), source, dir.path()).expect("maps");

    // The mapping and the bytes are one decision, so they are asserted
    // together: whatever `weave_into` writes is what the map describes.
    // A block's text begins with the newline after its opening tag, and
    // the file is written WITHOUT it — the engine's own weave writes no
    // such line, and a leading blank line moves whatever has to be at
    // byte 0 (a `#!`, a BOM, an XML declaration). Getting either half
    // wrong on its own puts every breakpoint one line from where it was
    // asked for.
    let written = crate::weave_into(source, dir.path()).expect("weaves");
    assert_eq!(written.len(), 1);
    let bytes = std::fs::read_to_string(&written[0]).unwrap();
    assert!(bytes.starts_with("def total(x):"), "{bytes:?}");

    // `def total(x):` is document line 5 (0-based), and line ZERO of the
    // file that was just written.
    let (path, line) = mapping.to_generated(5).expect("the code line maps");
    assert!(path.ends_with("app.py"), "{path:?}");
    assert_eq!(line, 0);
    assert_eq!(bytes.lines().nth(line as usize), Some("def total(x):"));

    // And back again, which is what a stack frame needs.
    assert_eq!(mapping.to_document(&path, 0), Some(5));
    assert_eq!(mapping.to_document(&path, 1), Some(6));
}

#[test]
fn a_prose_line_maps_to_nothing() {
    // A breakpoint on a heading is not a breakpoint the adapter can bind,
    // and saying so beats sending it and reporting `verified: false`
    // without explanation.
    let source = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"o.md\">\n\
             # Just prose here\n\
             </hick:doc>\n";
    let dir = tempfile::tempdir().unwrap();
    let mapping = Mapping::for_document(Path::new("d.md"), source, dir.path()).unwrap();
    assert_eq!(mapping.to_generated(2), None);
}

#[test]
fn a_plain_file_maps_every_line_to_itself() {
    // Protects docs/guarantees/debugging/a-plain-file-has-the-same-debugger.md
    //
    // `src/main.rs` is not woven: the breakpoint goes on the line it was
    // asked for, in the file itself, and a frame in that file comes back
    // on the same line. A frame in ANOTHER file of the project is not
    // in this one — but it keeps its source, so the app can open it.
    let dir = tempfile::tempdir().unwrap();
    let main = dir.path().join("main.rs");
    let lib = dir.path().join("lib.rs");
    std::fs::write(&main, "fn main() {}\n").unwrap();
    std::fs::write(&lib, "pub fn f() {}\n").unwrap();
    let mapping = Mapping::identity(&main);
    assert!(mapping.is_identity());
    assert_eq!(mapping.to_generated(7), Some((main.clone(), 7)));
    assert_eq!(mapping.to_document(&main, 7), Some(7));
    // The adapter's own spelling of the same file still counts.
    let spelled = dir.path().join(".").join("main.rs");
    assert_eq!(mapping.to_document(&spelled, 3), Some(3));
    assert_eq!(mapping.to_document(&lib, 3), None);
    assert_eq!(mapping.generated_files(), vec![main]);
}
