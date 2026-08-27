//! Reading an index, and refusing to show what it cannot explain.
//!
//! Protects docs/guarantees/editor-intelligence/an-index-answers-in-documents.md
//!
//! The claim worth testing is the one a unit test cannot make: that a
//! reference found in a **generated** file comes back as a line in the
//! **document** that wrote it. That needs a real indexer, a real weave, and
//! real lineage between them.
//!
//! Skipped loudly without `scip-typescript`, which is what
//! `hick index install typescript` fetches.

use std::path::{Path, PathBuf};

const DOC: &str = r#"# Billing

The invoice reference is built here.

<hick:file path="billing.ts">
export function invoiceRef(id: string): string {
  return `INV-${id}`;
}
</hick:file>
"#;

/// 0-based line of `export function invoiceRef` in DOC.
const DEFINITION_LINE: u32 = 5;

/// The indexer, from this repository's own install or from `PATH`.
///
/// A developer runs `hick index install typescript` once at the top of this
/// repo, and a scratch project in a temp directory is nowhere near it —
/// without this the test skips on the machine most likely to be running it,
/// which is how a suite ends up green while testing nothing. The same borrow
/// `live_session.rs` and `live_session_csharp.rs` do for their tools.
fn indexer() -> Option<PathBuf> {
    let ours = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.hick-cache/indexers/node/node_modules/.bin/scip-typescript");
    if ours.is_file() {
        return ours.canonicalize().ok();
    }
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|dir| dir.join("scip-typescript"))
        .find(|c| c.is_file())
}

fn project() -> Option<(tempfile::TempDir, PathBuf)> {
    let dir = tempfile::tempdir().ok()?;
    let root = dir.path().canonicalize().ok()?;
    std::fs::write(root.join("billing.hick"), DOC).ok()?;
    std::fs::write(
        root.join("use.ts"),
        "import { invoiceRef } from \"./billing\";\nexport const first = invoiceRef(\"1\");\n",
    )
    .ok()?;
    std::fs::write(
        root.join("package.json"),
        "{ \"name\": \"t\", \"version\": \"1.0.0\" }\n",
    )
    .ok()?;
    std::fs::write(
        root.join("tsconfig.json"),
        "{ \"compilerOptions\": { \"target\": \"ES2020\", \"module\": \"commonjs\" }, \
         \"include\": [\"billing.ts\", \"use.ts\"] }\n",
    )
    .ok()?;
    Some((dir, root))
}

fn build_index(root: &Path, indexer: &Path) -> bool {
    let out_dir = root.join(".hick-cache/index");
    std::fs::create_dir_all(&out_dir).ok();
    std::process::Command::new(indexer)
        .args(["index", "--output"])
        .arg(out_dir.join("typescript.scip"))
        .current_dir(root)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[tokio::test(flavor = "multi_thread")]
async fn a_reference_in_a_generated_file_comes_back_as_a_line_in_its_document() {
    let Some(indexer) = indexer() else {
        eprintln!("SKIPPED: no scip-typescript (`hick index install typescript`)");
        return;
    };
    let Some((_dir, root)) = project() else {
        eprintln!("SKIPPED: could not make a scratch project");
        return;
    };

    // Weave, so the generated file the indexer will read exists.
    let run = hickory_cli::run_doc(
        &root.join("billing.hick"),
        &[],
        hickory_cli::RunMode::Weave,
        hickory_cli::ExecutorChoice::Local,
    )
    .await
    .expect("the document weaves");
    hickory_cli::write_outputs(&run, None).expect("outputs written");
    assert!(
        root.join("billing.ts").exists(),
        "the weave produced nothing"
    );

    assert!(build_index(&root, &indexer), "the indexer failed");
    hickory_cli::index_read::write_built_from(&root, "typescript", "2026-08-27T00:00:00Z")
        .expect("a build record");

    let index = hickory_cli::index_read::read_index(&hickory_cli::index_read::index_path(
        &root,
        "typescript",
    ))
    .expect("the index reads");
    let raw = hickory_cli::index_read::occurrences(&index, "invoiceRef");
    assert!(
        raw.iter().any(|h| h.path == "billing.ts"),
        "the indexer never saw the generated file: {raw:?}"
    );

    let doc_index =
        hickory_cli::serve::store::DocIndex::scan(&root).expect("an index of documents");
    let generated = hickory_cli::serve::api::generated_outputs(&root, &doc_index);
    // The map is output path -> document **id**, not path. Reading it as a
    // path is silent: the weave fails and every generated hit is dropped as
    // unexplainable, which looks exactly like lineage having nothing to say.
    let doc_id = generated
        .get("billing.ts")
        .expect("billing.ts is known to be generated");
    let doc_path = doc_index.absolute(doc_id).expect("the id resolves");

    let lineage = hickory_cli::output_lineage(&run, "billing.ts").expect("lineage");
    let text = match run.result.files.get("billing.ts").unwrap() {
        hick_exec::node::FileContent::Text(s) => s.clone(),
        hick_exec::node::FileContent::Binary(_) => unreachable!(),
    };
    let source = std::fs::read_to_string(&doc_path).unwrap();

    let found = hickory_cli::index_read::explain(&raw, &generated, |output| {
        (output == "billing.ts").then(|| (lineage.clone(), text.clone(), source.clone()))
    });

    // The claim: the definition, found in a file nobody edits, is reported at
    // the line of the DOCUMENT that wrote it.
    let through_document = found
        .hits
        .iter()
        .find(|h| h.through.as_deref() == Some("billing.ts"))
        .expect("no hit was mapped back into the document");
    assert!(
        through_document.path.ends_with("billing.hick"),
        "{through_document:?}"
    );
    assert_eq!(
        through_document.line, DEFINITION_LINE,
        "the wrong document line: {through_document:?}"
    );
    assert!(
        found
            .hits
            .iter()
            .any(|h| h.path == "use.ts" && h.through.is_none()),
        "the hand-written file's own reference was lost: {:?}",
        found.hits
    );
    assert_eq!(found.unmapped, 0, "something generated went unexplained");
}

#[tokio::test(flavor = "multi_thread")]
async fn an_edited_file_makes_the_index_say_it_is_out_of_date() {
    // An index is a cache and must be marked as one. Staleness is not a
    // second mechanism: it is the same question recordings answer — did an
    // input change — asked with the same tool.
    let Some((_dir, root)) = project() else {
        eprintln!("SKIPPED: could not make a scratch project");
        return;
    };
    std::fs::create_dir_all(root.join(".hick-cache/index")).unwrap();
    // A minimal index naming one file is enough: nothing here reads symbols.
    let mut index = scip::types::Index::new();
    let mut document = scip::types::Document::new();
    document.relative_path = "use.ts".into();
    index.documents.push(document);
    scip::write_message_to_file(
        hickory_cli::index_read::index_path(&root, "typescript"),
        index,
    )
    .unwrap();

    let built =
        hickory_cli::index_read::write_built_from(&root, "typescript", "2026-08-27T00:00:00Z")
            .unwrap();
    assert_eq!(built.inputs.len(), 1);
    assert!(
        built.moved_on(&root).is_empty(),
        "fresh index reported stale"
    );

    std::fs::write(root.join("use.ts"), "// somebody typed\n").unwrap();
    assert_eq!(built.moved_on(&root), vec!["use.ts".to_string()]);
}
