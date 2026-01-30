//! Pipeline integration tests: validate file outputs, copy/paste, DAG
//! structure, and full end-to-end behaviour using inline `.hick` content.

use hick_exec::dag;

fn hick_doc(body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
{body}
</hick:doc>"#
    )
}

// ---------------------------------------------------------------------------
// File output tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_simple_file_output() {
    let src = hick_doc(r#"<hick:file path="out.txt">hello world</hick:file>"#);
    let result = hick::run_pipeline(&[("test.hick", &src)]).await.unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "hello world");
}

#[tokio::test]
async fn test_file_with_exec_placeholder() {
    let src = hick_doc(
        r#"<hick:container name="demo" image="alpine" />
<hick:file path="out.txt"><hick:exec container="demo">echo hi</hick:exec></hick:file>"#,
    );
    let result = hick::run_pipeline(&[("test.hick", &src)]).await.unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert!(
        content.contains("[exec in container 'demo']"),
        "expected exec placeholder, got: {content}"
    );
}

#[tokio::test]
async fn test_multi_file_output() {
    let src = hick_doc(
        r#"<hick:file path="a.txt">aaa</hick:file>
<hick:file path="b.txt">bbb</hick:file>
<hick:file path="c.txt">ccc</hick:file>"#,
    );
    let result = hick::run_pipeline(&[("test.hick", &src)]).await.unwrap();
    assert_eq!(result.files.len(), 3);
    assert_eq!(result.files.get("a.txt").unwrap(), "aaa");
    assert_eq!(result.files.get("b.txt").unwrap(), "bbb");
    assert_eq!(result.files.get("c.txt").unwrap(), "ccc");
}

#[tokio::test]
async fn test_raw_text_characters() {
    let src = hick_doc(r#"<hick:file path="out.txt">a & b < c > d</hick:file>"#);
    let result = hick::run_pipeline(&[("test.hick", &src)]).await.unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "a & b < c > d");
}

// ---------------------------------------------------------------------------
// Copy / paste tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_copy_paste() {
    let src = hick_doc(concat!(
        r#"<hick:copy id="v">3.12</hick:copy>"#,
        "\n",
        r##"<hick:file path="out.txt"><hick:paste select="#v" /></hick:file>"##,
    ));
    let result = hick::run_pipeline(&[("test.hick", &src)]).await.unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "3.12");
}

#[tokio::test]
async fn test_cut_paste() {
    let src = hick_doc(concat!(
        r#"<hick:cut id="secret">hidden-value</hick:cut>"#,
        "\n",
        r##"<hick:file path="out.txt"><hick:paste select="#secret" /></hick:file>"##,
    ));
    let result = hick::run_pipeline(&[("test.hick", &src)]).await.unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "hidden-value");
}

#[tokio::test]
async fn test_paste_missing_selector() {
    let src = hick_doc(
        r##"<hick:file path="out.txt">before<hick:paste select="#nonexistent" />after</hick:file>"##,
    );
    let result = hick::run_pipeline(&[("test.hick", &src)]).await.unwrap();
    // Missing paste produces no output, no crash
    assert_eq!(result.files.get("out.txt").unwrap(), "beforeafter");
}

#[tokio::test]
async fn test_copy_paste_across_documents() {
    let doc1 = hick_doc(r#"<hick:copy id="version">2.0.0</hick:copy>"#);
    let doc2 =
        hick_doc(r##"<hick:file path="out.txt"><hick:paste select="#version" /></hick:file>"##);
    let result = hick::run_pipeline(&[("doc1.hick", &doc1), ("doc2.hick", &doc2)])
        .await
        .unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "2.0.0");
}

// ---------------------------------------------------------------------------
// DAG structure tests
// ---------------------------------------------------------------------------

#[test]
fn test_dag_single_exec() {
    let src = hick_doc(r#"<hick:exec container="demo" image="alpine">echo hello</hick:exec>"#);
    let doc = hick_lang::parse(&src).unwrap();
    let dag = dag::build_dag(&doc).unwrap();
    assert_eq!(dag.execs.len(), 1);
    assert_eq!(dag.roots.len(), 1);
    assert_eq!(dag.edges.len(), 0);
}

#[test]
fn test_dag_sequential_same_container() {
    let src = hick_doc(
        r#"<hick:exec container="demo" image="alpine">step 1</hick:exec>
<hick:exec container="demo">step 2</hick:exec>"#,
    );
    let doc = hick_lang::parse(&src).unwrap();
    let dag = dag::build_dag(&doc).unwrap();
    assert_eq!(dag.execs.len(), 2);
    assert_eq!(dag.edges.len(), 1);
    assert_eq!(dag.edges[0].from, dag::ExecId(0));
    assert_eq!(dag.edges[0].to, dag::ExecId(1));
}

#[test]
fn test_dag_parallel_different_containers() {
    let src = hick_doc(
        r#"<hick:container name="a" image="alpine" />
<hick:container name="b" image="alpine" />
<hick:exec container="a">echo a</hick:exec>
<hick:exec container="b">echo b</hick:exec>"#,
    );
    let doc = hick_lang::parse(&src).unwrap();
    let dag = dag::build_dag(&doc).unwrap();
    assert_eq!(dag.execs.len(), 2);
    assert_eq!(dag.edges.len(), 0);
    assert_eq!(dag.roots.len(), 2);
}

#[test]
fn test_dag_volume_flow() {
    let src = hick_doc(
        r#"<hick:container name="writer" image="alpine" />
<hick:container name="reader" image="alpine" />
<hick:volume name="shared" />
<hick:exec container="writer" mount="shared:/output">echo data > /output/file.txt</hick:exec>
<hick:exec container="reader" mount="shared:/input">cat /input/file.txt</hick:exec>"#,
    );
    let doc = hick_lang::parse(&src).unwrap();
    let dag = dag::build_dag(&doc).unwrap();
    assert_eq!(dag.execs.len(), 2);
    let volume_edge = dag
        .edges
        .iter()
        .any(|e| matches!(&e.reason, dag::DependencyReason::VolumeFlow { .. }));
    assert!(volume_edge, "expected a VolumeFlow edge");
}

#[test]
fn test_dag_fork_dependency() {
    let src = hick_doc(
        r#"<hick:container name="base" image="ubuntu:22.04" />
<hick:exec container="base">apt-get update</hick:exec>
<hick:fork from="base" to="analyzer" />
<hick:exec container="analyzer">./run-analysis</hick:exec>"#,
    );
    let doc = hick_lang::parse(&src).unwrap();
    let dag = dag::build_dag(&doc).unwrap();
    assert_eq!(dag.execs.len(), 2);
    let fork_edge = dag
        .edges
        .iter()
        .any(|e| matches!(&e.reason, dag::DependencyReason::Fork { .. }));
    assert!(fork_edge, "expected a Fork edge");
}

// ---------------------------------------------------------------------------
// Full pipeline smoke test
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_full_pipeline_smoke() {
    let src = hick_doc(concat!(
        r#"<hick:container name="build" image="python:3.12">
  <hick:allow network="pypi.org:443" />
  <hick:deny network="*" />
</hick:container>

<hick:copy id="version">1.0.0</hick:copy>

<hick:exec container="build">pip install requests</hick:exec>

"#,
        r##"<hick:file path="manifest.txt">Version: <hick:paste select="#version" />
Container: build
</hick:file>"##,
    ));
    let result = hick::run_pipeline(&[("smoke.hick", &src)]).await.unwrap();

    // File output exists and has pasted version
    let manifest = result.files.get("manifest.txt").unwrap();
    assert!(manifest.contains("1.0.0"), "expected version in manifest");
    assert!(
        manifest.contains("Container: build"),
        "expected static text in manifest"
    );

    // Container was collected
    assert!(result.containers.contains_key("build"));
}

#[tokio::test]
async fn test_no_file_outputs() {
    let src = hick_doc(
        r#"<hick:container name="worker" image="alpine" />
<hick:exec container="worker">echo working</hick:exec>"#,
    );
    let result = hick::run_pipeline(&[("test.hick", &src)]).await.unwrap();
    assert!(result.files.is_empty());
}

// ---------------------------------------------------------------------------
// Custom prefix tests
// ---------------------------------------------------------------------------

fn h_doc(body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<h:doc xmlns:h="http://www.hickorydocs.com/1.0">
{body}
</h:doc>"#
    )
}

#[tokio::test]
async fn test_custom_prefix_file_output() {
    let src = h_doc(r#"<h:file path="out.txt">hello from h prefix</h:file>"#);
    let result = hick::run_pipeline(&[("test.hick", &src)]).await.unwrap();
    assert_eq!(
        result.files.get("out.txt").unwrap(),
        "hello from h prefix"
    );
}

#[tokio::test]
async fn test_custom_prefix_cut_paste() {
    let src = h_doc(concat!(
        r#"<h:cut id="ns">http://www.hickorydocs.com/1.0</h:cut>"#,
        "\n",
        r##"<h:file path="out.txt">NS: <h:paste select="#ns" /></h:file>"##,
    ));
    let result = hick::run_pipeline(&[("test.hick", &src)]).await.unwrap();
    assert_eq!(
        result.files.get("out.txt").unwrap(),
        "NS: http://www.hickorydocs.com/1.0"
    );
}

#[tokio::test]
async fn test_custom_prefix_hick_in_text_is_literal() {
    // With h: prefix, <hick:...> in text body is NOT parsed
    let src = h_doc(
        r#"<h:file path="out.txt">Example: <hick:file path="x">y</hick:file></h:file>"#,
    );
    let result = hick::run_pipeline(&[("test.hick", &src)]).await.unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert!(
        content.contains("<hick:file"),
        "expected literal <hick:file in output, got: {content}"
    );
}

#[tokio::test]
async fn test_custom_prefix_containers() {
    let src = h_doc(
        r#"<h:container name="worker" image="alpine">
  <h:allow network="github.com:443" />
  <h:deny network="*" />
</h:container>
<h:file path="out.txt">done</h:file>"#,
    );
    let result = hick::run_pipeline(&[("test.hick", &src)]).await.unwrap();
    assert!(result.containers.contains_key("worker"));
    assert_eq!(result.files.get("out.txt").unwrap(), "done");
}

#[tokio::test]
async fn test_guide_hick_file_processes() {
    let guide_src = include_str!("../../../docs/hick-guide.hick");
    let result = hick::run_pipeline(&[("hick-guide.hick", guide_src)])
        .await
        .unwrap();

    let guide = result.files.get("docs/hick-guide.md").unwrap();
    assert!(
        guide.contains("# The Hick Language"),
        "guide should contain title"
    );
    assert!(
        guide.contains("http://www.hickorydocs.com/1.0"),
        "pasted namespace URI should appear in guide"
    );
    // Verify hick: examples are literal text, not parsed
    assert!(
        guide.contains("<hick:container"),
        "guide should contain literal <hick:container examples"
    );
    assert!(
        guide.contains("<hick:exec"),
        "guide should contain literal <hick:exec examples"
    );

    // Verify live container demo
    assert!(
        result.containers.contains_key("guide-builder"),
        "guide-builder container should be defined"
    );
    assert!(
        result.containers.contains_key("guide-verifier"),
        "guide-verifier container should be defined"
    );
    assert!(
        guide.contains("[exec in container 'guide-verifier']"),
        "exec placeholder should appear in output"
    );
    assert!(
        guide.contains("echo \"Built by hick\" > /out/stamp.txt"),
        "pasted build-cmd should appear in output"
    );
}
