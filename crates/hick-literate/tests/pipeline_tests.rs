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
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "hello world");
}

#[tokio::test]
async fn test_file_with_exec_dry_run_transcript() {
    let src = hick_doc(
        r#"<hick:container name="demo" image="alpine" />
<hick:file path="out.txt"><hick:exec container="demo">echo hi</hick:exec></hick:file>"#,
    );
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert!(
        content.contains("$ echo hi"),
        "expected dry-run transcript with '$ echo hi', got: {content}"
    );
}

#[tokio::test]
async fn test_multi_file_output() {
    let src = hick_doc(
        r#"<hick:file path="a.txt">aaa</hick:file>
<hick:file path="b.txt">bbb</hick:file>
<hick:file path="c.txt">ccc</hick:file>"#,
    );
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    assert_eq!(result.files.len(), 3);
    assert_eq!(result.files.get("a.txt").unwrap(), "aaa");
    assert_eq!(result.files.get("b.txt").unwrap(), "bbb");
    assert_eq!(result.files.get("c.txt").unwrap(), "ccc");
}

#[tokio::test]
async fn test_raw_text_characters() {
    let src = hick_doc(r#"<hick:file path="out.txt">a & b < c > d</hick:file>"#);
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "a & b < c > d");
}

// ---------------------------------------------------------------------------
// Exec show attribute tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_exec_show_command_only() {
    let src = hick_doc(
        r#"<hick:container name="demo" image="alpine" />
<hick:exec container="demo">echo setup</hick:exec>
<hick:file path="out.txt"><hick:exec container="demo" show="command">echo setup</hick:exec></hick:file>"#,
    );
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert!(
        content.contains("$ echo setup"),
        "should show command, got: {content}"
    );
}

#[tokio::test]
async fn test_exec_show_none() {
    let src = hick_doc(
        r#"<hick:container name="demo" image="alpine" />
<hick:exec container="demo">echo setup</hick:exec>
<hick:file path="out.txt">before<hick:exec container="demo" show="none">echo setup</hick:exec>after</hick:file>"#,
    );
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert_eq!(content, "beforeafter", "show=none should render nothing");
}

#[tokio::test]
async fn test_exec_show_output_only() {
    // In dry-run mode, output is always empty, so show="output" produces nothing
    let src = hick_doc(
        r#"<hick:container name="demo" image="alpine" />
<hick:exec container="demo">echo hello</hick:exec>
<hick:file path="out.txt">before<hick:exec container="demo" show="output">echo hello</hick:exec>after</hick:file>"#,
    );
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert_eq!(
        content, "beforeafter",
        "show=output in dry-run should render nothing (no output)"
    );
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
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "3.12");
}

#[tokio::test]
async fn test_cut_paste() {
    let src = hick_doc(concat!(
        r#"<hick:cut id="secret">hidden-value</hick:cut>"#,
        "\n",
        r##"<hick:file path="out.txt"><hick:paste select="#secret" /></hick:file>"##,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "hidden-value");
}

#[tokio::test]
async fn test_paste_missing_selector() {
    let src = hick_doc(
        r##"<hick:file path="out.txt">before<hick:paste select="#nonexistent" />after</hick:file>"##,
    );
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    // Missing paste produces no output, no crash
    assert_eq!(result.files.get("out.txt").unwrap(), "beforeafter");
}

#[tokio::test]
async fn test_copy_paste_across_documents() {
    let doc1 = hick_doc(r#"<hick:copy id="version">2.0.0</hick:copy>"#);
    let doc2 =
        hick_doc(r##"<hick:file path="out.txt"><hick:paste select="#version" /></hick:file>"##);
    let result = hick_literate::run_pipeline(&[("doc1.hick", &doc1), ("doc2.hick", &doc2)], &[])
        .await
        .unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "2.0.0");
}

// ---------------------------------------------------------------------------
// Class-based selector tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_class_selector_single_block() {
    let src = hick_doc(concat!(
        r#"<hick:copy class="imports">import foo;</hick:copy>"#,
        "\n",
        r#"<hick:file path="out.txt"><hick:paste select=".imports" /></hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "import foo;");
}

#[tokio::test]
async fn test_class_selector_multiple_blocks() {
    let src = hick_doc(concat!(
        r#"<hick:copy class="imports">import foo;</hick:copy>"#,
        "\n",
        r#"<hick:copy class="imports">import bar;</hick:copy>"#,
        "\n",
        r#"<hick:copy class="imports">import baz;</hick:copy>"#,
        "\n",
        r#"<hick:file path="out.txt"><hick:paste select=".imports" /></hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    assert_eq!(
        result.files.get("out.txt").unwrap(),
        "import foo;import bar;import baz;"
    );
}

#[tokio::test]
async fn test_class_selector_with_newlines() {
    let src = hick_doc(concat!(
        r#"<hick:copy class="imports">import foo;
</hick:copy>"#,
        "\n",
        r#"<hick:copy class="imports">import bar;
</hick:copy>"#,
        "\n",
        r#"<hick:file path="out.txt"><hick:paste select=".imports" /></hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    assert_eq!(
        result.files.get("out.txt").unwrap(),
        "import foo;\nimport bar;\n"
    );
}

#[tokio::test]
async fn test_class_selector_interleaved_with_id() {
    let src = hick_doc(concat!(
        r#"<hick:copy id="header" class="sections"># Header
</hick:copy>"#,
        "\n",
        r#"<hick:copy class="sections">## Section 1
</hick:copy>"#,
        "\n",
        r#"<hick:copy class="sections">## Section 2
</hick:copy>"#,
        "\n",
        r##"<hick:file path="by-class.md"><hick:paste select=".sections" /></hick:file>"##,
        "\n",
        r##"<hick:file path="by-id.md"><hick:paste select="#header" /></hick:file>"##,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    // Class selector gets all three sections
    assert_eq!(
        result.files.get("by-class.md").unwrap(),
        "# Header\n## Section 1\n## Section 2\n"
    );
    // ID selector gets only the header
    assert_eq!(result.files.get("by-id.md").unwrap(), "# Header\n");
}

#[tokio::test]
async fn test_class_selector_multiple_classes() {
    let src = hick_doc(concat!(
        r#"<hick:copy class="imports deps">import shared;</hick:copy>"#,
        "\n",
        r#"<hick:copy class="imports">import local;</hick:copy>"#,
        "\n",
        r#"<hick:file path="imports.txt"><hick:paste select=".imports" /></hick:file>"#,
        "\n",
        r#"<hick:file path="deps.txt"><hick:paste select=".deps" /></hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    assert_eq!(
        result.files.get("imports.txt").unwrap(),
        "import shared;import local;"
    );
    assert_eq!(result.files.get("deps.txt").unwrap(), "import shared;");
}

#[tokio::test]
async fn test_class_selector_cut_blocks() {
    let src = hick_doc(concat!(
        r#"<hick:cut class="secrets">SECRET1;</hick:cut>"#,
        "\n",
        r#"<hick:cut class="secrets">SECRET2;</hick:cut>"#,
        "\n",
        r#"<hick:file path="out.txt"><hick:paste select=".secrets" /></hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "SECRET1;SECRET2;");
}

#[tokio::test]
async fn test_class_selector_missing() {
    let src = hick_doc(concat!(
        r#"<hick:copy class="imports">import foo;</hick:copy>"#,
        "\n",
        r#"<hick:file path="out.txt">before<hick:paste select=".nonexistent" />after</hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    // Missing class selector produces no output (like missing ID)
    assert_eq!(result.files.get("out.txt").unwrap(), "beforeafter");
}

#[tokio::test]
async fn test_class_selector_across_documents() {
    let doc1 = hick_doc(r#"<hick:copy class="chunks">chunk1;</hick:copy>"#);
    let doc2 = hick_doc(r#"<hick:copy class="chunks">chunk2;</hick:copy>"#);
    let doc3 = hick_doc(r#"<hick:file path="out.txt"><hick:paste select=".chunks" /></hick:file>"#);
    let result = hick_literate::run_pipeline(
        &[
            ("doc1.hick", &doc1),
            ("doc2.hick", &doc2),
            ("doc3.hick", &doc3),
        ],
        &[],
    )
    .await
    .unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "chunk1;chunk2;");
}

#[tokio::test]
async fn test_class_selector_with_substitution() {
    let src = hick_doc(concat!(
        r#"<hick:substitute name="project" pattern="FavoriteApp" value="MyProject" />"#,
        "\n",
        r#"<hick:copy class="imports">import FavoriteApp;</hick:copy>"#,
        "\n",
        r#"<hick:file path="out.txt"><hick:paste select=".imports" /></hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    // Substitution should apply to pasted content
    assert_eq!(result.files.get("out.txt").unwrap(), "import MyProject;");
}

// ---------------------------------------------------------------------------
// Variable / parameter tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_var_val_basic() {
    let src = hick_doc(concat!(
        r#"<hick:var name="version">1.0.0</hick:var>"#,
        "\n",
        r#"<hick:file path="out.txt">v=<hick:val name="version" /></hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "v=1.0.0");
}

#[tokio::test]
async fn test_param_overrides_var() {
    let src = hick_doc(concat!(
        r#"<hick:var name="version">1.0.0</hick:var>"#,
        "\n",
        r#"<hick:file path="out.txt">v=<hick:val name="version" /></hick:file>"#,
    ));
    let params = vec![("version".to_string(), "2.0.0".to_string())];
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &params)
        .await
        .unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "v=2.0.0");
}

#[tokio::test]
async fn test_var_missing_produces_nothing() {
    let src =
        hick_doc(r#"<hick:file path="out.txt">before<hick:val name="missing" />after</hick:file>"#);
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "beforeafter");
}

#[tokio::test]
async fn test_var_across_documents() {
    let doc1 = hick_doc(r#"<hick:var name="shared">hello</hick:var>"#);
    let doc2 = hick_doc(r#"<hick:file path="out.txt"><hick:val name="shared" /></hick:file>"#);
    let result = hick_literate::run_pipeline(&[("doc1.hick", &doc1), ("doc2.hick", &doc2)], &[])
        .await
        .unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "hello");
}

// ---------------------------------------------------------------------------
// Include tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_include_basic() {
    let dir = std::env::temp_dir().join("hick-test-include-basic");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // Write fragment
    std::fs::write(
        dir.join("version.hick"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:copy id="ver">4.0.0</hick:copy>
</hick:doc>"#,
    )
    .unwrap();

    // Write main doc
    let main_src = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:include file="version.hick" />
<hick:file path="out.txt">v=<hick:paste select="#ver" /></hick:file>
</hick:doc>"##;
    let main_path = dir.join("main.hick");
    std::fs::write(&main_path, main_src).unwrap();

    let result = hick_literate::run_pipeline(&[(main_path.to_str().unwrap(), main_src)], &[])
        .await
        .unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "v=4.0.0");

    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn test_include_circular_detected() {
    let dir = std::env::temp_dir().join("hick-test-include-cycle");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    std::fs::write(
        dir.join("a.hick"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:include file="b.hick" />
</hick:doc>"#,
    )
    .unwrap();

    std::fs::write(
        dir.join("b.hick"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:include file="a.hick" />
</hick:doc>"#,
    )
    .unwrap();

    let src = std::fs::read_to_string(dir.join("a.hick")).unwrap();
    let result = hick_literate::run_pipeline(&[(dir.join("a.hick").to_str().unwrap(), &src)], &[]).await;
    assert!(result.is_err(), "circular include should fail");

    let _ = std::fs::remove_dir_all(&dir);
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
    let result = hick_literate::run_pipeline(&[("smoke.hick", &src)], &[])
        .await
        .unwrap();

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
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
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
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "hello from h prefix");
}

#[tokio::test]
async fn test_custom_prefix_cut_paste() {
    let src = h_doc(concat!(
        r#"<h:cut id="ns">http://www.hickorydocs.com/1.0</h:cut>"#,
        "\n",
        r##"<h:file path="out.txt">NS: <h:paste select="#ns" /></h:file>"##,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    assert_eq!(
        result.files.get("out.txt").unwrap(),
        "NS: http://www.hickorydocs.com/1.0"
    );
}

#[tokio::test]
async fn test_custom_prefix_hick_in_text_is_literal() {
    // With h: prefix, <hick:...> in text body is NOT parsed
    let src =
        h_doc(r#"<h:file path="out.txt">Example: <hick:file path="x">y</hick:file></h:file>"#);
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
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
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    assert!(result.containers.contains_key("worker"));
    assert_eq!(result.files.get("out.txt").unwrap(), "done");
}

// ---------------------------------------------------------------------------
// Fork tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fork_creates_container_def_in_dry_run() {
    let src = hick_doc(
        r#"<hick:container name="base" image="alpine">
  <hick:allow network="pypi.org:443" />
  <hick:deny network="*" />
</hick:container>
<hick:exec container="base">echo setup</hick:exec>
<hick:fork from="base" to="analyzer" />
<hick:exec container="analyzer">echo analyze</hick:exec>"#,
    );
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();

    // Fork target should appear as a container definition
    assert!(
        result.containers.contains_key("analyzer"),
        "fork target 'analyzer' should be in container defs"
    );
    // Source container should also be present
    assert!(
        result.containers.contains_key("base"),
        "source container 'base' should be in container defs"
    );
}

#[tokio::test]
async fn test_fork_inherits_source_capabilities() {
    let src = hick_doc(
        r#"<hick:container name="base" image="alpine">
  <hick:allow network="pypi.org:443" />
  <hick:deny network="*" />
</hick:container>
<hick:exec container="base">echo setup</hick:exec>
<hick:fork from="base" to="analyzer" />
<hick:exec container="analyzer">echo analyze</hick:exec>"#,
    );
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();

    let base_caps = result.containers.get("base").unwrap();
    let fork_caps = result.containers.get("analyzer").unwrap();

    // Fork should inherit the source's network rules
    assert_eq!(
        base_caps.network_rules.len(),
        fork_caps.network_rules.len(),
        "fork should inherit source network rules"
    );
    assert_eq!(base_caps.network_rules, fork_caps.network_rules);
}

#[tokio::test]
async fn test_fork_attenuates_capabilities() {
    let src = hick_doc(
        r#"<hick:container name="base" image="alpine">
  <hick:allow network="pypi.org:443" />
</hick:container>
<hick:exec container="base">echo setup</hick:exec>
<hick:fork from="base" to="restricted">
  <hick:deny network="*" />
</hick:fork>
<hick:exec container="restricted">echo analyze</hick:exec>"#,
    );
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();

    let base_caps = result.containers.get("base").unwrap();
    let fork_caps = result.containers.get("restricted").unwrap();

    // Base has 1 allow rule
    assert_eq!(base_caps.network_rules.len(), 1);

    // Fork inherits the allow rule AND adds a deny-all
    assert_eq!(
        fork_caps.network_rules.len(),
        2,
        "fork should have inherited allow + added deny"
    );

    // Check that the deny-all was added
    assert!(
        fork_caps
            .network_rules
            .iter()
            .any(|r| matches!(r, hick_token::NetworkRule::DenyAll)),
        "fork should have deny-all rule"
    );
}

#[tokio::test]
async fn test_fork_with_allow_and_deny_children() {
    let src = hick_doc(
        r#"<hick:container name="base" image="alpine">
  <hick:allow network="pypi.org:443" />
</hick:container>
<hick:exec container="base">echo setup</hick:exec>
<hick:fork from="base" to="scanner">
  <hick:allow network="cve.mitre.org:443" />
  <hick:deny network="*" />
</hick:fork>
<hick:exec container="scanner">echo scan</hick:exec>"#,
    );
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();

    let fork_caps = result.containers.get("scanner").unwrap();

    // Should have: inherited allow(pypi.org:443) + added allow(cve.mitre.org:443) + deny(*)
    assert_eq!(
        fork_caps.network_rules.len(),
        3,
        "fork should have 3 network rules (1 inherited + 2 from fork children)"
    );
}

#[tokio::test]
async fn test_fork_without_declared_source_gets_default_caps() {
    // Source container has no explicit <container> declaration (implicit via exec image attr)
    let src = hick_doc(
        r#"<hick:exec container="base" image="alpine">echo setup</hick:exec>
<hick:fork from="base" to="fork-target" />
<hick:exec container="fork-target">echo hello</hick:exec>"#,
    );
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();

    // Fork target should still appear in container defs with empty capabilities
    assert!(result.containers.contains_key("fork-target"));
    let fork_caps = result.containers.get("fork-target").unwrap();
    assert!(fork_caps.network_rules.is_empty());
    assert!(fork_caps.file_rules.is_empty());
}

#[tokio::test]
async fn test_guide_hick_file_processes() {
    let guide_src = include_str!("../../../docs/hick-guide.hick");
    let result = hick_literate::run_pipeline(&[("hick-guide.hick", guide_src)], &[])
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
        guide.contains("$ cat /out/stamp.txt"),
        "dry-run transcript for guide-verifier should appear in output, got: {guide}"
    );
    assert!(
        guide.contains("echo \"Built by hick\" > /out/stamp.txt"),
        "pasted build-cmd should appear in output"
    );
}

// ---------------------------------------------------------------------------
// Conditional content tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_when_attribute_true() {
    let src = hick_doc(concat!(
        r#"<hick:var name="debug">1</hick:var>"#,
        "\n",
        r#"<hick:container name="dbg" image="alpine" when="debug" />"#,
        "\n",
        r#"<hick:exec container="dbg" when="debug">echo debug</hick:exec>"#,
        "\n",
        r#"<hick:file path="out.txt"><hick:exec container="dbg" /></hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    // Container should exist (when=true)
    assert!(result.containers.contains_key("dbg"));
    // Exec should be in the transcript
    let content = result.files.get("out.txt").unwrap();
    assert!(
        content.contains("$ echo debug"),
        "exec should be included: {content}"
    );
}

#[tokio::test]
async fn test_when_attribute_false() {
    let src = hick_doc(concat!(
        r#"<hick:container name="dbg" image="alpine" when="debug" />"#,
        "\n",
        r#"<hick:exec container="dbg" when="debug">echo debug</hick:exec>"#,
        "\n",
        r#"<hick:file path="out.txt">done</hick:file>"#,
    ));
    // No debug var defined -> when="debug" is false
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    // Container should NOT exist (filtered out)
    assert!(!result.containers.contains_key("dbg"));
}

#[tokio::test]
async fn test_when_tag_true() {
    let src = hick_doc(concat!(
        r#"<hick:var name="mode">prod</hick:var>"#,
        "\n",
        r#"<hick:file path="out.txt">start "#,
        r#"<hick:when test="mode=prod">production content</hick:when>"#,
        r#" end</hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert!(
        content.contains("production content"),
        "when=true should include children: {content}"
    );
}

#[tokio::test]
async fn test_when_tag_false() {
    let src = hick_doc(concat!(
        r#"<hick:var name="mode">dev</hick:var>"#,
        "\n",
        r#"<hick:file path="out.txt">start "#,
        r#"<hick:when test="mode=prod">production content</hick:when>"#,
        r#" end</hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert!(
        !content.contains("production content"),
        "when=false should exclude children: {content}"
    );
}

#[tokio::test]
async fn test_when_on_exec_skips_dag() {
    // When an exec is filtered out, it should not appear in the DAG
    let src = hick_doc(concat!(
        r#"<hick:container name="c" image="alpine" />"#,
        "\n",
        r#"<hick:exec container="c">echo always</hick:exec>"#,
        "\n",
        r#"<hick:exec container="c" when="never_set">echo conditional</hick:exec>"#,
        "\n",
        r#"<hick:file path="out.txt"><hick:exec container="c" /></hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert!(
        content.contains("$ echo always"),
        "unconditional exec should appear: {content}"
    );
    assert!(
        !content.contains("$ echo conditional"),
        "conditional exec should be filtered: {content}"
    );
}

#[tokio::test]
async fn test_when_param_override() {
    // Param should make condition true
    let src = hick_doc(concat!(
        r#"<hick:file path="out.txt">"#,
        r#"<hick:when test="debug">debug info</hick:when>"#,
        r#"done</hick:file>"#,
    ));
    let params = vec![("debug".to_string(), "1".to_string())];
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &params)
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert!(
        content.contains("debug info"),
        "param should enable conditional: {content}"
    );
}

#[tokio::test]
async fn test_when_not_defined() {
    let src = hick_doc(concat!(
        r#"<hick:var name="mode">prod</hick:var>"#,
        "\n",
        r#"<hick:file path="out.txt">"#,
        r#"<hick:when test="!debug">no debug</hick:when>"#,
        r#"<hick:when test="mode">has mode</hick:when>"#,
        r#"</hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert!(
        content.contains("no debug"),
        "!debug should be true: {content}"
    );
    assert!(
        content.contains("has mode"),
        "mode should be defined: {content}"
    );
}

#[tokio::test]
async fn test_when_not_equals() {
    let src = hick_doc(concat!(
        r#"<hick:var name="env">staging</hick:var>"#,
        "\n",
        r#"<hick:file path="out.txt">"#,
        r#"<hick:when test="env!=prod">not prod</hick:when>"#,
        r#"<hick:when test="env!=staging">not staging</hick:when>"#,
        r#"</hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert!(
        content.contains("not prod"),
        "env!=prod should be true: {content}"
    );
    assert!(
        !content.contains("not staging"),
        "env!=staging should be false: {content}"
    );
}

// ---------------------------------------------------------------------------
// Compound boolean condition tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_when_and_condition() {
    let src = hick_doc(concat!(
        r#"<hick:var name="auth">1</hick:var>"#,
        "\n",
        r#"<hick:var name="billing">1</hick:var>"#,
        "\n",
        r#"<hick:file path="out.txt">"#,
        r#"<hick:when test="auth && billing">premium features</hick:when>"#,
        r#"</hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert!(
        content.contains("premium features"),
        "auth && billing should be true: {content}"
    );
}

#[tokio::test]
async fn test_when_and_condition_one_missing() {
    let src = hick_doc(concat!(
        r#"<hick:var name="auth">1</hick:var>"#,
        "\n",
        r#"<hick:file path="out.txt">"#,
        r#"<hick:when test="auth && billing">premium features</hick:when>"#,
        r#"basic</hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert!(
        !content.contains("premium features"),
        "auth && billing should be false when billing missing: {content}"
    );
    assert!(
        content.contains("basic"),
        "basic content should appear: {content}"
    );
}

#[tokio::test]
async fn test_when_or_condition() {
    let src = hick_doc(concat!(
        r#"<hick:var name="admin">1</hick:var>"#,
        "\n",
        r#"<hick:file path="out.txt">"#,
        r#"<hick:when test="auth || admin">has access</hick:when>"#,
        r#"</hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert!(
        content.contains("has access"),
        "auth || admin should be true when admin defined: {content}"
    );
}

#[tokio::test]
async fn test_when_or_condition_both_missing() {
    let src = hick_doc(concat!(
        r#"<hick:file path="out.txt">"#,
        r#"<hick:when test="auth || admin">has access</hick:when>"#,
        r#"denied</hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert!(
        !content.contains("has access"),
        "auth || admin should be false when both missing: {content}"
    );
    assert!(
        content.contains("denied"),
        "denied content should appear: {content}"
    );
}

#[tokio::test]
async fn test_when_not_with_parens() {
    let src = hick_doc(concat!(
        r#"<hick:var name="auth">1</hick:var>"#,
        "\n",
        r#"<hick:var name="billing">1</hick:var>"#,
        "\n",
        r#"<hick:file path="out.txt">"#,
        r#"<hick:when test="!(auth && billing)">not premium</hick:when>"#,
        r#"<hick:when test="auth && billing">premium</hick:when>"#,
        r#"</hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert!(
        !content.contains("not premium"),
        "!(auth && billing) should be false: {content}"
    );
    assert!(
        content.contains("premium"),
        "auth && billing should be true: {content}"
    );
}

#[tokio::test]
async fn test_when_mixed_precedence() {
    // Test that && has higher precedence than ||
    // auth && billing || admin should be true if auth && billing is true
    let src = hick_doc(concat!(
        r#"<hick:var name="auth">1</hick:var>"#,
        "\n",
        r#"<hick:var name="billing">1</hick:var>"#,
        "\n",
        r#"<hick:file path="out.txt">"#,
        r#"<hick:when test="auth && billing || admin">elevated</hick:when>"#,
        r#"</hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert!(
        content.contains("elevated"),
        "auth && billing || admin should be true: {content}"
    );
}

#[tokio::test]
async fn test_when_parens_override_precedence() {
    // (auth || admin) && billing should be false when billing is missing
    let src = hick_doc(concat!(
        r#"<hick:var name="auth">1</hick:var>"#,
        "\n",
        r#"<hick:file path="out.txt">"#,
        r#"<hick:when test="(auth || admin) && billing">premium</hick:when>"#,
        r#"basic</hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert!(
        !content.contains("premium"),
        "(auth || admin) && billing should be false without billing: {content}"
    );
    assert!(content.contains("basic"), "basic should appear: {content}");
}

#[tokio::test]
async fn test_when_equals_with_and() {
    let src = hick_doc(concat!(
        r#"<hick:var name="env">prod</hick:var>"#,
        "\n",
        r#"<hick:var name="debug">1</hick:var>"#,
        "\n",
        r#"<hick:file path="out.txt">"#,
        r#"<hick:when test="env=prod && debug">prod debug mode</hick:when>"#,
        r#"<hick:when test="env=dev && debug">dev debug mode</hick:when>"#,
        r#"</hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert!(
        content.contains("prod debug mode"),
        "env=prod && debug should be true: {content}"
    );
    assert!(
        !content.contains("dev debug mode"),
        "env=dev && debug should be false: {content}"
    );
}

#[tokio::test]
async fn test_when_on_exec_with_compound_condition() {
    let src = hick_doc(concat!(
        r#"<hick:var name="auth">1</hick:var>"#,
        "\n",
        r#"<hick:container name="c" image="alpine" />"#,
        "\n",
        r#"<hick:exec container="c">echo always</hick:exec>"#,
        "\n",
        r#"<hick:exec container="c" when="auth && billing">echo premium</hick:exec>"#,
        "\n",
        r#"<hick:file path="out.txt"><hick:exec container="c" /></hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert!(
        content.contains("$ echo always"),
        "unconditional exec should appear: {content}"
    );
    assert!(
        !content.contains("$ echo premium"),
        "conditional exec should be filtered: {content}"
    );
}

// ---------------------------------------------------------------------------
// Substitution tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_substitute_basic() {
    let src = hick_doc(concat!(
        r#"<hick:substitute name="project" pattern="FavoriteApp" value="MyProject" />"#,
        "\n",
        r#"<hick:file path="out.txt">Welcome to FavoriteApp!</hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert_eq!(content, "Welcome to MyProject!");
}

#[tokio::test]
async fn test_substitute_with_variants() {
    let src = hick_doc(concat!(
        r#"<hick:substitute name="project" pattern="FavoriteApp" value="MyProject" variants="true" />"#,
        "\n",
        r#"<hick:file path="out.txt">"#,
        r#"class FavoriteApp { let favoriteApp = "favorite_app"; }"#,
        r#"</hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    // Should have replaced all case variants
    assert!(
        content.contains("class MyProject"),
        "PascalCase should be replaced: {content}"
    );
    assert!(
        content.contains("let myProject"),
        "camelCase should be replaced: {content}"
    );
    assert!(
        content.contains("my_project"),
        "snake_case should be replaced: {content}"
    );
}

#[tokio::test]
async fn test_substitute_value_from_var() {
    let src = hick_doc(concat!(
        r#"<hick:var name="project">MyProject</hick:var>"#,
        "\n",
        r#"<hick:substitute name="project" pattern="FavoriteApp" />"#,
        "\n",
        r#"<hick:file path="out.txt">Welcome to FavoriteApp!</hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert_eq!(content, "Welcome to MyProject!");
}

#[tokio::test]
async fn test_substitute_value_from_param() {
    let src = hick_doc(concat!(
        r#"<hick:substitute name="project" pattern="FavoriteApp" />"#,
        "\n",
        r#"<hick:file path="out.txt">Welcome to FavoriteApp!</hick:file>"#,
    ));
    let params = vec![("project".to_string(), "MyProject".to_string())];
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &params)
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert_eq!(content, "Welcome to MyProject!");
}

#[tokio::test]
async fn test_path_interpolation_basic() {
    let src = hick_doc(concat!(
        r#"<hick:var name="name">MyApp</hick:var>"#,
        "\n",
        r#"<hick:file path="Sources/{{name}}/App.swift">app code</hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    assert!(
        result.files.contains_key("Sources/MyApp/App.swift"),
        "path should be interpolated: {:?}",
        result.files.keys().collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn test_path_interpolation_with_variant() {
    let src = hick_doc(concat!(
        r#"<hick:var name="name">MyApp</hick:var>"#,
        "\n",
        r#"<hick:file path="{{name:snake_case}}.rs">rust code</hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    assert!(
        result.files.contains_key("my_app.rs"),
        "path should use snake_case: {:?}",
        result.files.keys().collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn test_substitute_in_path() {
    let src = hick_doc(concat!(
        r#"<hick:substitute name="project" pattern="FavoriteApp" value="MyProject" variants="true" />"#,
        "\n",
        r#"<hick:file path="Sources/FavoriteApp/App.swift">code</hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    assert!(
        result.files.contains_key("Sources/MyProject/App.swift"),
        "path substitution should work: {:?}",
        result.files.keys().collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn test_substitute_in_paste() {
    let src = hick_doc(concat!(
        r#"<hick:substitute name="project" pattern="FavoriteApp" value="MyProject" />"#,
        "\n",
        r#"<hick:copy id="template">class FavoriteApp {}</hick:copy>"#,
        "\n",
        r##"<hick:file path="out.txt"><hick:paste select="#template" /></hick:file>"##,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert!(
        content.contains("class MyProject"),
        "substitution should apply to paste: {content}"
    );
}

#[tokio::test]
async fn test_substitute_multiple_patterns() {
    let src = hick_doc(concat!(
        r#"<hick:substitute name="org" pattern="ExampleOrg" value="MyCompany" />"#,
        "\n",
        r#"<hick:substitute name="app" pattern="FavoriteApp" value="MyApp" />"#,
        "\n",
        r#"<hick:file path="out.txt">ExampleOrg presents FavoriteApp</hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    assert_eq!(content, "MyCompany presents MyApp");
}

#[tokio::test]
async fn test_substitute_longer_pattern_first() {
    // If we have "App" and "FavoriteApp", "FavoriteApp" should be matched first
    let src = hick_doc(concat!(
        r#"<hick:substitute name="app" pattern="App" value="Program" />"#,
        "\n",
        r#"<hick:substitute name="favorite" pattern="FavoriteApp" value="MyProject" />"#,
        "\n",
        r#"<hick:file path="out.txt">FavoriteApp is an App</hick:file>"#,
    ));
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    let content = result.files.get("out.txt").unwrap();
    // "FavoriteApp" should become "MyProject", "App" should become "Program"
    assert_eq!(content, "MyProject is an Program");
}

// ---------------------------------------------------------------------------
// Multi-stage pipeline tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_multi_stage_no_hick_output_single_stage() {
    // Pipeline producing no .hick files -> single stage, same as today
    let src = hick_doc(
        r#"<hick:file path="out.txt">hello</hick:file>
<hick:file path="data.json">{"key": "value"}</hick:file>"#,
    );
    let result = hick_literate::run_pipeline_multi_stage(&[("test.hick", &src)], &[], 3)
        .await
        .unwrap();
    assert_eq!(result.files.len(), 2);
    assert_eq!(result.files.get("out.txt").unwrap(), "hello");
    assert_eq!(
        result.files.get("data.json").unwrap(),
        r#"{"key": "value"}"#
    );

    // All files should have HickFile provenance (stage 1)
    for (path, prov) in &result.provenance {
        assert!(
            matches!(prov, hick_store::FileProvenance::HickFile { .. }),
            "expected HickFile provenance for {path}, got {prov:?}"
        );
    }
}

#[tokio::test]
async fn test_multi_stage_two_stages() {
    // Stage 1 produces a .hick file that feeds into stage 2.
    // Use h: prefix for outer doc so that hick: tags inside file body are
    // treated as literal text (not parsed by the stage-1 parser).
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<h:doc xmlns:h="http://www.hickorydocs.com/1.0">
<h:file path="stage1-output.txt">from stage 1</h:file>
<h:file path="generated.hick"><?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="stage2-output.txt">from stage 2</hick:file>
</hick:doc></h:file>
</h:doc>"#;

    let result = hick_literate::run_pipeline_multi_stage(&[("test.hick", &src)], &[], 3)
        .await
        .unwrap();

    // Stage 1 regular file should be present
    assert_eq!(
        result.files.get("stage1-output.txt").unwrap(),
        "from stage 1"
    );

    // Stage 2 output should be present
    assert_eq!(
        result.files.get("stage2-output.txt").unwrap(),
        "from stage 2"
    );

    // .hick file should NOT be in final output (it's intermediate)
    assert!(
        !result.files.contains_key("generated.hick"),
        "intermediate .hick file should not appear in final output"
    );

    // Stage 1 file should have HickFile provenance
    assert!(matches!(
        result.provenance.get("stage1-output.txt").unwrap(),
        hick_store::FileProvenance::HickFile { .. }
    ));

    // Stage 2 file should have GeneratedHick provenance
    assert!(matches!(
        result.provenance.get("stage2-output.txt").unwrap(),
        hick_store::FileProvenance::GeneratedHick { .. }
    ));
}

#[tokio::test]
async fn test_multi_stage_limit_reached_error() {
    // With max_stages=1, any .hick output triggers the limit error.
    let src = hick_doc(r#"<hick:file path="output.hick">placeholder</hick:file>"#);
    let result = hick_literate::run_pipeline_multi_stage(&[("test.hick", &src)], &[], 1).await;
    assert!(result.is_err(), "should error when stage limit reached");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("limit"),
        "error should mention limit, got: {err}"
    );
    assert!(
        err.contains("output.hick"),
        "error should list offending files, got: {err}"
    );
}

#[tokio::test]
async fn test_multi_stage_hick_files_removed_from_output() {
    // Verify that .hick files never appear in final output
    // (they get fed to next stage and removed from results).
    // Use h: prefix so that hick: tags in file body are literal text.
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<h:doc xmlns:h="http://www.hickorydocs.com/1.0">
<h:file path="keep-me.txt">kept</h:file>
<h:file path="intermediate.hick"><?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="generated.txt">from generated hick</hick:file>
</hick:doc></h:file>
</h:doc>"#;

    let result = hick_literate::run_pipeline_multi_stage(&[("test.hick", src)], &[], 3)
        .await
        .unwrap();

    assert!(result.files.contains_key("keep-me.txt"));
    assert!(result.files.contains_key("generated.txt"));
    assert!(
        !result.files.contains_key("intermediate.hick"),
        ".hick files should be removed from final output"
    );
}

// ---------------------------------------------------------------------------
// Feature tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_feature_definition_basic() {
    // Define features and enable one via param
    let src = hick_doc(
        r#"<hick:feature name="auth" description="Authentication support" />
<hick:file path="config.txt">
<hick:when test="auth">auth enabled</hick:when>
<hick:when test="!auth">auth disabled</hick:when>
</hick:file>"#,
    );

    // Enable auth feature
    let result = hick_literate::run_pipeline(
        &[("test.hick", &src)],
        &[("features".to_string(), "auth".to_string())],
    )
    .await
    .unwrap();

    assert_eq!(
        result.files.get("config.txt").unwrap().trim(),
        "auth enabled"
    );
}

#[tokio::test]
async fn test_feature_not_enabled() {
    // Define features but don't enable auth
    let src = hick_doc(
        r#"<hick:feature name="auth" description="Authentication support" />
<hick:file path="config.txt">
<hick:when test="auth">auth enabled</hick:when>
<hick:when test="!auth">auth disabled</hick:when>
</hick:file>"#,
    );

    // Don't enable auth feature
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();

    assert_eq!(
        result.files.get("config.txt").unwrap().trim(),
        "auth disabled"
    );
}

#[tokio::test]
async fn test_feature_with_dependencies() {
    // premium requires auth and billing
    let src = hick_doc(
        r#"<hick:feature name="auth" description="Authentication" />
<hick:feature name="billing" description="Billing" />
<hick:feature name="premium" description="Premium features" requires="auth billing" />
<hick:file path="status.txt">
<hick:when test="auth">auth:yes </hick:when><hick:when test="billing">billing:yes </hick:when><hick:when test="premium">premium:yes</hick:when>
</hick:file>"#,
    );

    // Enable only premium - should auto-enable auth and billing
    let result = hick_literate::run_pipeline(
        &[("test.hick", &src)],
        &[("features".to_string(), "premium".to_string())],
    )
    .await
    .unwrap();

    let content = result.files.get("status.txt").unwrap();
    assert!(
        content.contains("auth:yes"),
        "auth should be enabled transitively"
    );
    assert!(
        content.contains("billing:yes"),
        "billing should be enabled transitively"
    );
    assert!(content.contains("premium:yes"), "premium should be enabled");
}

#[tokio::test]
async fn test_feature_multiple_enabled() {
    // Enable multiple features via comma-separated list
    let src = hick_doc(
        r#"<hick:feature name="auth" description="Authentication" />
<hick:feature name="billing" description="Billing" />
<hick:file path="status.txt">
<hick:when test="auth">auth </hick:when><hick:when test="billing">billing</hick:when>
</hick:file>"#,
    );

    let result = hick_literate::run_pipeline(
        &[("test.hick", &src)],
        &[("features".to_string(), "auth, billing".to_string())],
    )
    .await
    .unwrap();

    let content = result.files.get("status.txt").unwrap();
    assert!(content.contains("auth"), "auth should be enabled");
    assert!(content.contains("billing"), "billing should be enabled");
}

#[tokio::test]
async fn test_feature_unknown_feature_error() {
    // Try to enable a feature that isn't defined
    let src = hick_doc(
        r#"<hick:feature name="auth" description="Authentication" />
<hick:file path="out.txt">test</hick:file>"#,
    );

    let result = hick_literate::run_pipeline(
        &[("test.hick", &src)],
        &[("features".to_string(), "nonexistent".to_string())],
    )
    .await;

    assert!(result.is_err(), "should error on unknown feature");
    let err = match result {
        Err(e) => e.to_string(),
        Ok(_) => panic!("expected error"),
    };
    assert!(
        err.contains("nonexistent"),
        "error should mention the unknown feature"
    );
}

#[tokio::test]
async fn test_feature_circular_dependency_error() {
    // Create circular dependency: a -> b -> c -> a
    let src = hick_doc(
        r#"<hick:feature name="a" description="Feature A" requires="b" />
<hick:feature name="b" description="Feature B" requires="c" />
<hick:feature name="c" description="Feature C" requires="a" />
<hick:file path="out.txt">test</hick:file>"#,
    );

    let result = hick_literate::run_pipeline(
        &[("test.hick", &src)],
        &[("features".to_string(), "a".to_string())],
    )
    .await;

    assert!(result.is_err(), "should error on circular dependency");
    let err = match result {
        Err(e) => e.to_string(),
        Ok(_) => panic!("expected error"),
    };
    assert!(
        err.to_lowercase().contains("circular"),
        "error should mention circular dependency: {err}"
    );
}

#[tokio::test]
async fn test_feature_with_conditional_file() {
    // Use features to conditionally include entire files
    let src = hick_doc(
        r#"<hick:feature name="auth" description="Authentication" />
<hick:file path="main.txt">always present</hick:file>
<hick:when test="auth">
<hick:file path="auth.txt">auth module</hick:file>
</hick:when>"#,
    );

    // Without auth enabled
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    assert!(result.files.contains_key("main.txt"));
    assert!(
        !result.files.contains_key("auth.txt"),
        "auth.txt should not exist without auth feature"
    );

    // With auth enabled
    let result = hick_literate::run_pipeline(
        &[("test.hick", &src)],
        &[("features".to_string(), "auth".to_string())],
    )
    .await
    .unwrap();
    assert!(result.files.contains_key("main.txt"));
    assert!(
        result.files.contains_key("auth.txt"),
        "auth.txt should exist with auth feature"
    );
}

#[tokio::test]
async fn test_feature_with_path_interpolation() {
    // Combine features with path interpolation
    let src = hick_doc(
        r#"<hick:feature name="auth" description="Authentication" />
<hick:var name="module">core</hick:var>
<hick:when test="auth">
<hick:file path="src/{{module}}/auth.txt">auth code</hick:file>
</hick:when>"#,
    );

    let result = hick_literate::run_pipeline(
        &[("test.hick", &src)],
        &[("features".to_string(), "auth".to_string())],
    )
    .await
    .unwrap();

    assert!(result.files.contains_key("src/core/auth.txt"));
    assert_eq!(result.files.get("src/core/auth.txt").unwrap(), "auth code");
}

#[tokio::test]
async fn test_feature_with_substitution() {
    // Combine features with substitution
    let src = hick_doc(
        r#"<hick:feature name="auth" description="Authentication" />
<hick:substitute name="app_name" pattern="MyApp" value="TestProject" variants="true" />
<hick:when test="auth">
<hick:file path="auth.txt">MyApp auth module for my_app users</hick:file>
</hick:when>"#,
    );

    let result = hick_literate::run_pipeline(
        &[("test.hick", &src)],
        &[("features".to_string(), "auth".to_string())],
    )
    .await
    .unwrap();

    let content = result.files.get("auth.txt").unwrap();
    assert!(
        content.contains("TestProject"),
        "should substitute PascalCase"
    );
    assert!(
        content.contains("test_project"),
        "should substitute snake_case"
    );
}

// ---------------------------------------------------------------------------
// Exclusion pattern tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_exclude_basic() {
    // Exclude a specific file
    let src = hick_doc(
        r#"<hick:exclude pattern="TEMPLATE_README.md" />
<hick:file path="README.md">real readme</hick:file>
<hick:file path="TEMPLATE_README.md">template only</hick:file>
<hick:file path="src/main.rs">code</hick:file>"#,
    );

    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();

    assert!(
        result.files.contains_key("README.md"),
        "README.md should exist"
    );
    assert!(
        result.files.contains_key("src/main.rs"),
        "src/main.rs should exist"
    );
    assert!(
        !result.files.contains_key("TEMPLATE_README.md"),
        "TEMPLATE_README.md should be excluded"
    );
}

#[tokio::test]
async fn test_exclude_glob_extension() {
    // Exclude all files with a specific extension
    let src = hick_doc(
        r#"<hick:exclude pattern="*.template-only" />
<hick:file path="config.json">config</hick:file>
<hick:file path="notes.template-only">internal notes</hick:file>
<hick:file path="setup.template-only">setup instructions</hick:file>"#,
    );

    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();

    assert!(result.files.contains_key("config.json"));
    assert!(!result.files.contains_key("notes.template-only"));
    assert!(!result.files.contains_key("setup.template-only"));
}

#[tokio::test]
async fn test_exclude_glob_directory() {
    // Exclude all files under a directory using **
    let src = hick_doc(
        r#"<hick:exclude pattern="docs/internal/**" />
<hick:file path="docs/readme.md">public docs</hick:file>
<hick:file path="docs/internal/dev-notes.md">internal</hick:file>
<hick:file path="docs/internal/setup.md">internal setup</hick:file>
<hick:file path="src/main.rs">code</hick:file>"#,
    );

    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();

    assert!(result.files.contains_key("docs/readme.md"));
    assert!(result.files.contains_key("src/main.rs"));
    assert!(!result.files.contains_key("docs/internal/dev-notes.md"));
    assert!(!result.files.contains_key("docs/internal/setup.md"));
}

#[tokio::test]
async fn test_exclude_multiple_patterns() {
    // Multiple exclusion patterns
    let src = hick_doc(
        r#"<hick:exclude pattern="*.bak" />
<hick:exclude pattern="*.tmp" />
<hick:exclude pattern="TEMPLATE_*" />
<hick:file path="main.rs">code</hick:file>
<hick:file path="main.rs.bak">backup</hick:file>
<hick:file path="cache.tmp">temp</hick:file>
<hick:file path="TEMPLATE_CONFIG.md">template config</hick:file>
<hick:file path="config.md">real config</hick:file>"#,
    );

    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();

    assert!(result.files.contains_key("main.rs"));
    assert!(result.files.contains_key("config.md"));
    assert!(!result.files.contains_key("main.rs.bak"));
    assert!(!result.files.contains_key("cache.tmp"));
    assert!(!result.files.contains_key("TEMPLATE_CONFIG.md"));
}

#[tokio::test]
async fn test_exclude_with_features() {
    // Exclusion patterns work with features
    let src = hick_doc(
        r#"<hick:feature name="auth" description="Authentication" />
<hick:exclude pattern="*.template" />
<hick:file path="app.rs">main app</hick:file>
<hick:file path="SETUP.template">setup instructions</hick:file>
<hick:when test="auth">
<hick:file path="auth.rs">auth code</hick:file>
</hick:when>"#,
    );

    let result = hick_literate::run_pipeline(
        &[("test.hick", &src)],
        &[("features".to_string(), "auth".to_string())],
    )
    .await
    .unwrap();

    assert!(result.files.contains_key("app.rs"));
    assert!(result.files.contains_key("auth.rs"));
    assert!(!result.files.contains_key("SETUP.template"));
}

#[tokio::test]
async fn test_exclude_conditional() {
    // Conditional exclusion based on feature
    let src = hick_doc(
        r#"<hick:feature name="dev" description="Development mode" />
<hick:when test="!dev">
<hick:exclude pattern="*.dev" />
</hick:when>
<hick:file path="app.rs">main app</hick:file>
<hick:file path="debug.dev">dev only debug file</hick:file>"#,
    );

    // Without dev feature - debug.dev should be excluded
    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    assert!(result.files.contains_key("app.rs"));
    assert!(
        !result.files.contains_key("debug.dev"),
        "should exclude .dev files in production"
    );

    // With dev feature - debug.dev should be kept
    let result = hick_literate::run_pipeline(
        &[("test.hick", &src)],
        &[("features".to_string(), "dev".to_string())],
    )
    .await
    .unwrap();
    assert!(result.files.contains_key("app.rs"));
    assert!(
        result.files.contains_key("debug.dev"),
        "should keep .dev files in dev mode"
    );
}

// ---------------------------------------------------------------------------
// Weave (literate programming) tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_weave_basic_output() {
    // Basic weave output with prose and code
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="TEMPLATE.md">

# My Template

This template generates a Rust project.

<hick:file path="src/main.rs">
fn main() {
    println!("Hello!");
}
</hick:file>

The main function prints a greeting.

</hick:doc>"#;

    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();

    // Should produce both the weave file and the actual code file
    assert!(
        result.files.contains_key("TEMPLATE.md"),
        "weave output should exist"
    );
    assert!(
        result.files.contains_key("src/main.rs"),
        "code file should exist"
    );

    let weave = result.files.get("TEMPLATE.md").unwrap();
    assert!(
        weave.contains("# My Template"),
        "weave should contain title"
    );
    assert!(
        weave.contains("This template generates a Rust project."),
        "weave should contain prose"
    );
    assert!(
        weave.contains("### `src/main.rs`"),
        "weave should contain file heading"
    );
    assert!(
        weave.contains("```rust"),
        "weave should have rust code fence"
    );
    assert!(weave.contains("fn main()"), "weave should contain code");
    assert!(
        weave.contains("The main function prints a greeting."),
        "weave should contain trailing prose"
    );
}

#[tokio::test]
async fn test_weave_doc_hidden() {
    // doc-hidden attribute excludes files from weave
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="README.md">

# My Project

<hick:file path="src/main.rs">
fn main() {}
</hick:file>

<hick:file path=".gitignore" doc-hidden="true">
target/
</hick:file>

</hick:doc>"#;

    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();

    // Both files should be generated
    assert!(result.files.contains_key("src/main.rs"));
    assert!(result.files.contains_key(".gitignore"));

    let weave = result.files.get("README.md").unwrap();
    assert!(
        weave.contains("### `src/main.rs`"),
        "visible file should appear in weave"
    );
    assert!(
        !weave.contains(".gitignore"),
        "doc-hidden file should NOT appear in weave"
    );
}

#[tokio::test]
async fn test_weave_variable_interpolation() {
    // Variable interpolation in prose
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="DOCS.md">
<hick:var name="project_name">MyApp</hick:var>
<hick:var name="version">1.0.0</hick:var>

# <hick:val name="project_name" />

Version: <hick:val name="version" />

<hick:file path="VERSION">
<hick:val name="version" />
</hick:file>

</hick:doc>"#;

    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();

    let weave = result.files.get("DOCS.md").unwrap();
    assert!(
        weave.contains("# MyApp"),
        "val in prose should be interpolated"
    );
    assert!(
        weave.contains("Version: 1.0.0"),
        "val in prose should be interpolated"
    );
}

#[tokio::test]
async fn test_weave_conditional_prose() {
    // Conditional prose with <hick:when>
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="README.md">
<hick:var name="auth">1</hick:var>

# My App

<hick:when test="auth">
## Authentication

This app includes authentication support.
</hick:when>

<hick:when test="!auth">
No authentication included.
</hick:when>

<hick:file path="app.rs">
// app code
</hick:file>

</hick:doc>"#;

    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();

    let weave = result.files.get("README.md").unwrap();
    assert!(
        weave.contains("## Authentication"),
        "conditional true content should appear"
    );
    assert!(
        weave.contains("authentication support"),
        "conditional true content should appear"
    );
    assert!(
        !weave.contains("No authentication included"),
        "conditional false content should NOT appear"
    );
}

#[tokio::test]
async fn test_weave_language_detection() {
    // Language detection for various file types
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="LANGS.md">

<hick:file path="app.swift">
import Foundation
</hick:file>

<hick:file path="main.py">
print("hello")
</hick:file>

<hick:file path="config.json">
{}
</hick:file>

<hick:file path="style.css">
body {}
</hick:file>

</hick:doc>"#;

    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();

    let weave = result.files.get("LANGS.md").unwrap();
    assert!(weave.contains("```swift"), "should detect swift");
    assert!(weave.contains("```python"), "should detect python");
    assert!(weave.contains("```json"), "should detect json");
    assert!(weave.contains("```css"), "should detect css");
}

#[tokio::test]
async fn test_weave_no_attribute_no_output() {
    // No weave output when weave attribute is absent
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">

Some prose here.

<hick:file path="out.txt">content</hick:file>

</hick:doc>"#;

    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();

    // Should only have the code file, no weave output
    assert_eq!(result.files.len(), 1, "should only have code file");
    assert!(result.files.contains_key("out.txt"));
}

#[tokio::test]
async fn test_weave_with_substitution() {
    // Substitution should apply to weave prose
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="README.md">
<hick:substitute name="app" pattern="FavoriteApp" value="MyProject" />

# FavoriteApp

Welcome to FavoriteApp!

<hick:file path="app.rs">
// FavoriteApp code
</hick:file>

</hick:doc>"#;

    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();

    let weave = result.files.get("README.md").unwrap();
    assert!(
        weave.contains("# MyProject"),
        "substitution should apply to prose"
    );
    assert!(
        weave.contains("Welcome to MyProject"),
        "substitution should apply to prose"
    );
    assert!(
        weave.contains("// MyProject code"),
        "substitution should apply to code in weave"
    );
}

#[tokio::test]
async fn test_weave_multi_file() {
    // Multiple files in one weave output
    let src = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="TEMPLATE.md">

# Multi-file Template

## Rust Code

<hick:file path="src/lib.rs">
pub fn greet() {}
</hick:file>

## Configuration

<hick:file path="Cargo.toml">
[package]
name = "example"
</hick:file>

</hick:doc>"#;

    let result = hick_literate::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();

    let weave = result.files.get("TEMPLATE.md").unwrap();
    assert!(weave.contains("# Multi-file Template"), "should have title");
    assert!(
        weave.contains("## Rust Code"),
        "should have section headings"
    );
    assert!(
        weave.contains("## Configuration"),
        "should have section headings"
    );
    assert!(
        weave.contains("### `src/lib.rs`"),
        "should have file heading"
    );
    assert!(
        weave.contains("### `Cargo.toml`"),
        "should have file heading"
    );
    assert!(weave.contains("```rust"), "should have rust fence");
    assert!(weave.contains("```toml"), "should have toml fence");
}

// ---------------------------------------------------------------------------
// TASK-20: hick init creates _hick.yml; pipeline show/status; multi-session run
// ---------------------------------------------------------------------------

#[test]
fn test_hick_init_creates_pipeline_config() {
    let dir = tempfile::tempdir().unwrap();
    let config = hick_literate::agents::InitConfig {
        project_dir: dir.path().to_path_buf(),
        no_agents: true,
    };
    let result = hick_literate::agents::init_agents(&config).unwrap();
    assert!(result.pipeline_initialized);
    assert!(dir.path().join("_hick.yml").is_file());
    let content = std::fs::read_to_string(dir.path().join("_hick.yml")).unwrap();
    assert!(content.contains("files:"));
}

#[test]
fn test_hick_init_does_not_overwrite_existing_pipeline_config() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("_hick.yml"),
        "files:\n  - custom.hick\n",
    )
    .unwrap();

    let config = hick_literate::agents::InitConfig {
        project_dir: dir.path().to_path_buf(),
        no_agents: true,
    };
    let result = hick_literate::agents::init_agents(&config).unwrap();
    assert!(!result.pipeline_initialized, "should not overwrite existing _hick.yml");
    let content = std::fs::read_to_string(dir.path().join("_hick.yml")).unwrap();
    assert!(content.contains("custom.hick"), "original content preserved");
}

#[test]
fn test_pipeline_show_no_error_on_empty_config() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("_hick.yml"), "files: []\n").unwrap();
    hick_literate::pipeline::pipeline_show(dir.path()).unwrap();
}

#[test]
fn test_pipeline_show_with_owned_files() {
    let dir = tempfile::tempdir().unwrap();
    let src = hick_doc(
        r#"<hick:file path="src/lib.rs">pub fn lib() {}</hick:file>
<hick:file path="README.md">
# Docs
<hick:paste name="module-list"/>
</hick:file>"#,
    );
    std::fs::write(dir.path().join("_hick.yml"), "files:\n  - main.hick\n").unwrap();
    std::fs::write(dir.path().join("main.hick"), &src).unwrap();
    // Should not error
    hick_literate::pipeline::pipeline_show(dir.path()).unwrap();
}

#[test]
fn test_pipeline_status_classifies_files() {
    let dir = tempfile::tempdir().unwrap();
    let src = hick_doc(r#"<hick:file path="out.txt">hello</hick:file>"#);
    std::fs::write(dir.path().join("_hick.yml"), "files:\n  - main.hick\n").unwrap();
    std::fs::write(dir.path().join("main.hick"), &src).unwrap();
    // Write the output file (pipeline-owned, on disk)
    std::fs::write(dir.path().join("out.txt"), "hello").unwrap();
    // Write an untracked file
    std::fs::write(dir.path().join("extra.txt"), "extra").unwrap();
    // Should not error
    hick_literate::pipeline::pipeline_status(dir.path()).unwrap();
}

/// AC#4: hick run after multiple simulated agent sessions reproduces all files.
///
/// Simulates two sessions by appending <hick:file> elements to the pipeline
/// hick source, then verifies hick run produces all files from both sessions.
#[tokio::test]
async fn test_multi_session_hick_run_reproduces_all_files() {
    // Session 1: create initial pipeline with file a.txt
    let session1 = hick_doc(r#"<hick:file path="a.txt">from-session-1</hick:file>"#);

    let result1 = hick_literate::run_pipeline(&[("session1.hick", &session1)], &[])
        .await
        .unwrap();
    assert_eq!(result1.files.get("a.txt").unwrap(), "from-session-1");

    // Session 2: new hick file appended to the same pipeline adds b.txt
    let session2 = hick_doc(r#"<hick:file path="b.txt">from-session-2</hick:file>"#);

    // Running both session files together simulates running the full pipeline
    // after two agent sessions have each appended their own .hick file.
    let result_combined = hick_literate::run_pipeline(
        &[
            ("session1.hick", &session1),
            ("session2.hick", &session2),
        ],
        &[],
    )
    .await
    .unwrap();

    assert_eq!(
        result_combined.files.get("a.txt").unwrap(),
        "from-session-1",
        "session 1 file reproduced"
    );
    assert_eq!(
        result_combined.files.get("b.txt").unwrap(),
        "from-session-2",
        "session 2 file reproduced"
    );
}
