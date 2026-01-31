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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[])
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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[])
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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[])
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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[])
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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[])
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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[])
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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[])
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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[])
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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "hidden-value");
}

#[tokio::test]
async fn test_paste_missing_selector() {
    let src = hick_doc(
        r##"<hick:file path="out.txt">before<hick:paste select="#nonexistent" />after</hick:file>"##,
    );
    let result = hick::run_pipeline(&[("test.hick", &src)], &[])
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
    let result = hick::run_pipeline(&[("doc1.hick", &doc1), ("doc2.hick", &doc2)], &[])
        .await
        .unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "2.0.0");
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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[])
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
    let result = hick::run_pipeline(&[("test.hick", &src)], &params)
        .await
        .unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "v=2.0.0");
}

#[tokio::test]
async fn test_var_missing_produces_nothing() {
    let src =
        hick_doc(r#"<hick:file path="out.txt">before<hick:val name="missing" />after</hick:file>"#);
    let result = hick::run_pipeline(&[("test.hick", &src)], &[])
        .await
        .unwrap();
    assert_eq!(result.files.get("out.txt").unwrap(), "beforeafter");
}

#[tokio::test]
async fn test_var_across_documents() {
    let doc1 = hick_doc(r#"<hick:var name="shared">hello</hick:var>"#);
    let doc2 = hick_doc(r#"<hick:file path="out.txt"><hick:val name="shared" /></hick:file>"#);
    let result = hick::run_pipeline(&[("doc1.hick", &doc1), ("doc2.hick", &doc2)], &[])
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

    let result = hick::run_pipeline(&[(main_path.to_str().unwrap(), main_src)], &[])
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
    let result = hick::run_pipeline(&[(dir.join("a.hick").to_str().unwrap(), &src)], &[]).await;
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
    let result = hick::run_pipeline(&[("smoke.hick", &src)], &[])
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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[])
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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[])
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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[])
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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[])
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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[])
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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[]).await.unwrap();

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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[]).await.unwrap();

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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[]).await.unwrap();

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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[]).await.unwrap();

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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[]).await.unwrap();

    // Fork target should still appear in container defs with empty capabilities
    assert!(result.containers.contains_key("fork-target"));
    let fork_caps = result.containers.get("fork-target").unwrap();
    assert!(fork_caps.network_rules.is_empty());
    assert!(fork_caps.file_rules.is_empty());
}

#[tokio::test]
async fn test_guide_hick_file_processes() {
    let guide_src = include_str!("../../../docs/hick-guide.hick");
    let result = hick::run_pipeline(&[("hick-guide.hick", guide_src)], &[])
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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[])
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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[])
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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[])
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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[])
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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[])
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
    let result = hick::run_pipeline(&[("test.hick", &src)], &params)
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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[])
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
    let result = hick::run_pipeline(&[("test.hick", &src)], &[])
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
