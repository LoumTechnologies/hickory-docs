//! EditSession behavior: edit_output through lineage, refusal routing,
//! edit_doc escalation, verify, and the staleness-impossible property.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use hickory_agent::hashline::line_hash;
use hickory_agent::{EditSession, ToolInvocation, execute_tool};
use hickory_executor::{Executor, LocalExecutor};

/// Copy blocks pasted once (editable through the output) and twice (the
/// refusal-routing case), plus a real exec + expectation for verify.
const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:copy id="greet">fn greet() {
    println!("hello");
}
</hick:copy>
<hick:copy id="dup">const SHARED: u8 = 1;
</hick:copy>
<hick:file path="gen.rs">// header
<hick:paste select="#greet" />// mid
<hick:paste select="#dup" />// tail
<hick:paste select="#dup" /></hick:file>
<hick:container name="c" image="alpine" />
<hick:exec container="c">
echo hi
<hick:expect match="exact">hi
</hick:expect>
</hick:exec>
</hick:doc>
"##;

fn write_doc(dir: &Path, source: &str) -> PathBuf {
    let path = dir.join("doc.hick");
    std::fs::write(&path, source).unwrap();
    path
}

fn executor() -> Arc<dyn Executor> {
    Arc::new(LocalExecutor::new().unwrap())
}

fn inv(name: &str, args: &[(&str, &str)], input: Option<&str>) -> ToolInvocation {
    ToolInvocation {
        name: name.to_string(),
        args: args
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        input: input.map(str::to_string),
        raw_xml: format!("<hick:tool name=\"{name}\"></hick:tool>"),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn read_output_with_lineage_marks_editability() {
    let dir = tempfile::tempdir().unwrap();
    let doc_path = write_doc(dir.path(), DOC);
    let mut session = EditSession::open(&doc_path, &[]).await.unwrap();

    let out = execute_tool(
        &mut session,
        executor(),
        &inv(
            "read_output",
            &[("path", "gen.rs"), ("with_lineage", "true")],
            None,
        ),
    )
    .await;
    assert!(out.ok, "{}", out.text);
    // Hashline prefix on every content line.
    assert!(
        out.text
            .contains(&format!("{}|// header", line_hash("// header")))
    );
    // Lineage annotations name both editable and doc-routed ranges.
    assert!(
        out.text.contains("editable via edit_output"),
        "{}",
        out.text
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn edit_output_maps_through_lineage_and_reweaves() {
    let dir = tempfile::tempdir().unwrap();
    let doc_path = write_doc(dir.path(), DOC);
    let mut session = EditSession::open(&doc_path, &[]).await.unwrap();

    let target = line_hash("    println!(\"hello\");");
    let out = execute_tool(
        &mut session,
        executor(),
        &inv(
            "edit_output",
            &[("path", "gen.rs"), ("run", &target)],
            Some("    println!(\"hi there\");"),
        ),
    )
    .await;
    assert!(out.ok, "{}", out.text);
    // The DOCUMENT was updated byte-exactly and written to disk.
    let doc_source = std::fs::read_to_string(&doc_path).unwrap();
    assert!(doc_source.contains("println!(\"hi there\");"));
    assert!(!doc_source.contains("println!(\"hello\");"));
    // The result carries the re-hashed edited region.
    assert!(
        out.text
            .contains(&format!("{}|", line_hash("    println!(\"hi there\");"))),
        "{}",
        out.text
    );
    // The fresh weave reflects the edit.
    let read = execute_tool(
        &mut session,
        executor(),
        &inv("read_output", &[("path", "gen.rs")], None),
    )
    .await;
    assert!(read.text.contains("hi there"), "{}", read.text);
}

#[tokio::test(flavor = "multi_thread")]
async fn duplicated_paste_refusal_routes_to_edit_doc() {
    let dir = tempfile::tempdir().unwrap();
    let doc_path = write_doc(dir.path(), DOC);
    let mut session = EditSession::open(&doc_path, &[]).await.unwrap();

    let dup = line_hash("const SHARED: u8 = 1;");

    // Two occurrences of the pasted line: without an occurrence index the
    // anchor is ambiguous, and the error lists the candidates.
    let ambiguous = execute_tool(
        &mut session,
        executor(),
        &inv(
            "edit_output",
            &[("path", "gen.rs"), ("run", &dup)],
            Some("const SHARED: u8 = 2;"),
        ),
    )
    .await;
    assert!(!ambiguous.ok);
    assert!(ambiguous.text.contains("ambiguous"), "{}", ambiguous.text);
    assert!(
        ambiguous.text.contains("occurrence 2"),
        "{}",
        ambiguous.text
    );

    // With the occurrence index, lineage refuses (shared source bytes) and
    // the refusal names the source location and routes to edit_doc.
    let refused = execute_tool(
        &mut session,
        executor(),
        &inv(
            "edit_output",
            &[("path", "gen.rs"), ("run", &dup), ("occurrence", "2")],
            Some("const SHARED: u8 = 2;"),
        ),
    )
    .await;
    assert!(!refused.ok);
    assert!(
        refused.text.contains("routing, not failure"),
        "{}",
        refused.text
    );
    assert!(refused.text.contains("edit_doc"), "{}", refused.text);
    assert!(
        refused.text.contains("doc.hick"),
        "refusal must name the source document: {}",
        refused.text
    );
    // Nothing changed.
    assert_eq!(std::fs::read_to_string(&doc_path).unwrap(), DOC);

    // Follow the pointer: edit_doc on the shared source block fixes BOTH
    // occurrences on re-weave.
    // (In the document the copy tag and its content share a line, so the
    // anchor comes from read_doc's hashes, not the output's.)
    let doc_line = r#"<hick:copy id="dup">const SHARED: u8 = 1;"#;
    let fixed = execute_tool(
        &mut session,
        executor(),
        &inv(
            "edit_doc",
            &[("run", &line_hash(doc_line))],
            Some(r#"<hick:copy id="dup">const SHARED: u8 = 2;"#),
        ),
    )
    .await;
    assert!(fixed.ok, "{}", fixed.text);
    let read = execute_tool(
        &mut session,
        executor(),
        &inv("read_output", &[("path", "gen.rs")], None),
    )
    .await;
    assert_eq!(
        read.text.matches("const SHARED: u8 = 2;").count(),
        2,
        "{}",
        read.text
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn synthetic_range_refusal_routes_to_edit_doc() {
    // A paste with a separator: the separator bytes are synthetic.
    let doc = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:copy id="a" class="f">alpha</hick:copy>
<hick:copy id="b" class="f">beta</hick:copy>
<hick:file path="out.txt"><hick:paste select=".f" separator=" | " /></hick:file>
</hick:doc>
"##;
    let dir = tempfile::tempdir().unwrap();
    let doc_path = write_doc(dir.path(), doc);
    let mut session = EditSession::open(&doc_path, &[]).await.unwrap();

    // The whole line "alpha | beta" crosses the synthetic separator.
    let out = execute_tool(
        &mut session,
        executor(),
        &inv(
            "edit_output",
            &[("path", "out.txt"), ("run", &line_hash("alpha | beta"))],
            Some("gamma | delta"),
        ),
    )
    .await;
    assert!(!out.ok);
    assert!(out.text.contains("synthetic"), "{}", out.text);
    assert!(out.text.contains("edit_doc"), "{}", out.text);
}

#[tokio::test(flavor = "multi_thread")]
async fn external_mutation_is_absorbed_or_reported_never_misapplied() {
    let dir = tempfile::tempdir().unwrap();
    let doc_path = write_doc(dir.path(), DOC);
    let mut session = EditSession::open(&doc_path, &[]).await.unwrap();

    // Mutate the document externally (as another writer would).
    let mutated = DOC.replace("// header", "// HEADER");
    std::fs::write(&doc_path, &mutated).unwrap();

    // 1. An edit anchored on the NEW content succeeds: the session detects
    //    the content mismatch and re-weaves automatically before resolving.
    let out = execute_tool(
        &mut session,
        executor(),
        &inv(
            "edit_output",
            &[("path", "gen.rs"), ("run", &line_hash("// HEADER"))],
            Some("// header v2"),
        ),
    )
    .await;
    assert!(out.ok, "{}", out.text);
    assert!(
        std::fs::read_to_string(&doc_path)
            .unwrap()
            .contains("// header v2")
    );

    // 2. Mutate again; an edit anchored on content that no longer exists is
    //    a structured error naming the external change — never a misapplied
    //    edit.
    let mutated2 = std::fs::read_to_string(&doc_path)
        .unwrap()
        .replace("// header v2", "// header v3");
    std::fs::write(&doc_path, &mutated2).unwrap();
    let stale = execute_tool(
        &mut session,
        executor(),
        &inv(
            "edit_output",
            &[("path", "gen.rs"), ("run", &line_hash("// header v2"))],
            Some("// nope"),
        ),
    )
    .await;
    assert!(!stale.ok);
    assert!(stale.text.contains("changed on disk"), "{}", stale.text);
    assert!(stale.text.contains("matches no lines"), "{}", stale.text);
    // The document still carries the external edit, untouched.
    assert_eq!(std::fs::read_to_string(&doc_path).unwrap(), mutated2);
}

#[tokio::test(flavor = "multi_thread")]
async fn insertion_through_doc_and_output() {
    let dir = tempfile::tempdir().unwrap();
    let doc_path = write_doc(dir.path(), DOC);
    let mut session = EditSession::open(&doc_path, &[]).await.unwrap();

    // Doc-side pure insert below the greet println.
    let out = execute_tool(
        &mut session,
        executor(),
        &inv(
            "edit_doc",
            &[("after", &line_hash("    println!(\"hello\");"))],
            Some("    println!(\"world\");"),
        ),
    )
    .await;
    assert!(out.ok, "{}", out.text);
    let doc_source = std::fs::read_to_string(&doc_path).unwrap();
    assert!(doc_source.contains("println!(\"hello\");\n    println!(\"world\");"));

    // Output-side insert follows lineage's neighbor attachment.
    let out2 = execute_tool(
        &mut session,
        executor(),
        &inv(
            "edit_output",
            &[
                ("path", "gen.rs"),
                ("after", &line_hash("    println!(\"world\");")),
            ],
            Some("    // inserted via output"),
        ),
    )
    .await;
    assert!(out2.ok, "{}", out2.text);
    assert!(
        std::fs::read_to_string(&doc_path)
            .unwrap()
            .contains("// inserted via output")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn verify_executes_and_reports_expectations() {
    let dir = tempfile::tempdir().unwrap();
    let doc_path = write_doc(dir.path(), DOC);
    let mut session = EditSession::open(&doc_path, &[]).await.unwrap();

    let pass = execute_tool(&mut session, executor(), &inv("verify", &[], None)).await;
    assert!(pass.ok, "{}", pass.text);
    assert!(pass.text.starts_with("PASS"), "{}", pass.text);
    // Outputs were written next to the document.
    assert!(dir.path().join("gen.rs").is_file());

    // Break the expectation via edit_doc, verify must FAIL.
    let broke = execute_tool(
        &mut session,
        executor(),
        &inv(
            "edit_doc",
            &[("run", &line_hash("echo hi"))],
            Some("echo bye"),
        ),
    )
    .await;
    assert!(broke.ok, "{}", broke.text);
    let fail = execute_tool(&mut session, executor(), &inv("verify", &[], None)).await;
    assert!(!fail.ok);
    assert!(fail.text.starts_with("FAIL"), "{}", fail.text);
    assert!(fail.text.contains("expectation"), "{}", fail.text);
}
