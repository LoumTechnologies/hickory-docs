//! Lineage across `<hick:include>` / `<hick:upstream>`: a span spliced from
//! another file must be attributed to THAT file, and a reverse edit through
//! it must land in that file — byte-exactly — while the including document
//! stays untouched.
//!
//! Before span-file stamping existed, these spans were attributed to the
//! including document with offsets into the included one: a reverse edit was
//! silent corruption of whichever text happened to sit at those offsets.
//!
//! Protects docs/guarantees/lineage/included-spans-name-their-own-file.md

use std::collections::HashMap;
use std::path::Path;

use hickory_cli::{DocRun, ExecutorChoice, RunMode, output_lineage, run_doc};
use hickory_lineage::{OutputEdit, apply_source_edits, map_edits};

fn write_doc(dir: &Path, name: &str, body: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(
        &path,
        format!("<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\n{body}\n</hick:doc>\n"),
    )
    .unwrap();
    path
}

async fn weave(doc_path: &Path) -> DocRun {
    run_doc(doc_path, &[], RunMode::Weave, ExecutorChoice::Local)
        .await
        .expect("weave must succeed")
}

fn output_text(run: &DocRun, path: &str) -> String {
    match run.result.files.get(path).expect("output exists") {
        hick_exec::node::FileContent::Text(s) => s.clone(),
        other => panic!("expected text output, got {other:?}"),
    }
}

/// An included file's `<hick:file>` content is attributed to the included
/// file, and an edit routed through the output lands there.
#[tokio::test]
async fn an_edit_to_included_content_lands_in_the_included_file() {
    let dir = tempfile::tempdir().unwrap();
    let chapter = write_doc(
        dir.path(),
        "chapter.hick",
        "<hick:file path=\"app.py\">def helper():\n    return 41\n</hick:file>",
    );
    let main = write_doc(
        dir.path(),
        "main.hick",
        "Main prose.\n<hick:include file=\"chapter.hick\" />",
    );

    let run = weave(&main).await;
    let content = output_text(&run, "app.py");
    assert!(content.contains("return 41"), "{content}");

    let provenance = output_lineage(&run, "app.py").expect("app.py has lineage");

    // Every editable byte of this output must name chapter.hick, not
    // main.hick — the offsets are chapter.hick's.
    let chapter_canonical = chapter.canonicalize().unwrap();
    let chapter_source = std::fs::read_to_string(&chapter).unwrap();
    let mut saw_chapter = false;
    for p in &provenance {
        if let Some((doc, s, e)) = p.origin.source() {
            assert!(
                Path::new(doc) == chapter_canonical,
                "output bytes {}..{} attributed to {doc}, expected {}",
                p.start,
                p.end,
                chapter_canonical.display()
            );
            // Byte-exact: the claimed span holds exactly the output bytes.
            assert_eq!(
                &chapter_source[s..e],
                &content[p.start..p.end],
                "span {s}..{e} of chapter.hick does not match output {}..{}",
                p.start,
                p.end
            );
            saw_chapter = true;
        }
    }
    assert!(saw_chapter, "no editable provenance at all: {provenance:?}");

    // Route an edit: 41 -> 42.
    let pos = content.find("41").unwrap();
    let edit = OutputEdit {
        start: pos,
        end: pos + 2,
        text: "42".into(),
    };
    let source_edits = map_edits(&content, std::slice::from_ref(&edit), &provenance).unwrap();
    assert!(
        source_edits
            .iter()
            .all(|e| Path::new(&e.doc_path) == chapter_canonical),
        "{source_edits:?}"
    );

    let mut sources = HashMap::new();
    for e in &source_edits {
        sources
            .entry(e.doc_path.clone())
            .or_insert_with(|| std::fs::read_to_string(&e.doc_path).unwrap());
    }
    let updated = apply_source_edits(&sources, &source_edits).unwrap();
    for (doc_path, new_source) in &updated {
        std::fs::write(doc_path, new_source).unwrap();
    }

    // The included file changed; the including document did not.
    assert!(
        std::fs::read_to_string(&chapter)
            .unwrap()
            .contains("return 42"),
        "edit did not land in chapter.hick"
    );
    let main_source = std::fs::read_to_string(&main).unwrap();
    assert!(main_source.contains("Main prose."), "{main_source}");
    assert!(
        !main_source.contains("42"),
        "main.hick was corrupted: {main_source}"
    );

    // And the round trip closes: re-weaving reproduces the edited output.
    let rerun = weave(&main).await;
    assert!(output_text(&rerun, "app.py").contains("return 42"));
}

/// A fragment pasted from an upstream document is attributed to the upstream
/// file, and editing the pasted bytes rewrites that file.
#[tokio::test]
async fn an_edit_to_a_pasted_upstream_fragment_lands_upstream() {
    let dir = tempfile::tempdir().unwrap();
    let notes = write_doc(
        dir.path(),
        "notes.hick",
        "<hick:copy id=\"rule\">Refuse, do not queue.</hick:copy>",
    );
    let main = write_doc(
        dir.path(),
        "main.hick",
        "<hick:upstream file=\"notes.hick\" />\n<hick:file path=\"rules.txt\">\
         <hick:paste select=\"#rule\" /></hick:file>",
    );

    let run = weave(&main).await;
    let content = output_text(&run, "rules.txt");
    assert!(content.contains("Refuse, do not queue."), "{content}");

    let provenance = output_lineage(&run, "rules.txt").unwrap();
    let notes_canonical = notes.canonicalize().unwrap();

    let pos = content.find("queue").unwrap();
    let edit = OutputEdit {
        start: pos,
        end: pos + "queue".len(),
        text: "wait".into(),
    };
    let source_edits = map_edits(&content, std::slice::from_ref(&edit), &provenance).unwrap();
    assert!(
        source_edits
            .iter()
            .all(|e| Path::new(&e.doc_path) == notes_canonical),
        "expected the edit to route to notes.hick: {source_edits:?}"
    );

    let mut sources = HashMap::new();
    for e in &source_edits {
        sources
            .entry(e.doc_path.clone())
            .or_insert_with(|| std::fs::read_to_string(&e.doc_path).unwrap());
    }
    let updated = apply_source_edits(&sources, &source_edits).unwrap();
    for (doc_path, new_source) in &updated {
        std::fs::write(doc_path, new_source).unwrap();
    }
    assert!(
        std::fs::read_to_string(&notes)
            .unwrap()
            .contains("do not wait"),
        "edit did not land upstream"
    );
    assert!(!std::fs::read_to_string(&main).unwrap().contains("wait"));
}

/// A stale-provenance span that would split a UTF-8 character is an error,
/// never a panic (replace_range aborts on a non-boundary offset).
#[test]
fn a_mid_character_source_span_is_an_error_not_a_panic() {
    let mut sources = HashMap::new();
    sources.insert("doc.hick".to_string(), "café".to_string());
    let edits = vec![hickory_lineage::SourceEdit {
        doc_path: "doc.hick".into(),
        span: (3, 4), // inside the two-byte 'é'
        text: "x".into(),
    }];
    let err = apply_source_edits(&sources, &edits).unwrap_err();
    assert!(format!("{err}").contains("character boundaries"), "{err}");
}
