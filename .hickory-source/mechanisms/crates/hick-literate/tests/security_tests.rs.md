<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-literate/tests/security_tests.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-797343670ac8"></a>
## `crates/hick-literate/tests/security_tests.rs`

<a id="f-797343670ac8-test_authority"></a>
### `test_authority`

Security boundary integration tests: network rules, file rules, token
attenuation/verification, and end-to-end capability parsing.

<storycode:copy id="f-797343670ac8-test_authority">
//! Security boundary integration tests: network rules, file rules, token
//! attenuation/verification, and end-to-end capability parsing.

use std::sync::Arc;

use hick_token::{ContainerCapabilities, NetworkRule, TokenAuthority};

fn test_authority() -> Arc<TokenAuthority> {
    Arc::new(TokenAuthority::new(b"test-root-key-32bytes-long-xxxxx"))
}

</storycode:copy>
<a id="f-797343670ac8-hick_doc"></a>
### `hick_doc`

<storycode:copy id="f-797343670ac8-hick_doc">
fn hick_doc(body: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
{body}
</hick:doc>"#
    )
}

</storycode:copy>
<a id="f-797343670ac8-test_deny_all_blocks_non_whitelisted"></a>
### `test_deny_all_blocks_non_whitelisted`

<storycode:copy id="f-797343670ac8-test_deny_all_blocks_non_whitelisted">
// ---------------------------------------------------------------------------
// Network rule tests
// ---------------------------------------------------------------------------

#[test]
fn test_deny_all_blocks_non_whitelisted() {
    let caps = ContainerCapabilities::new()
        .allow_network("github.com", "443")
        .deny_all_network();

    assert!(caps.check_network("github.com", 443));
    assert!(!caps.check_network("evil.com", 80));
}

</storycode:copy>
<a id="f-797343670ac8-test_no_deny_allows_everything"></a>
### `test_no_deny_allows_everything`

<storycode:copy id="f-797343670ac8-test_no_deny_allows_everything">
#[test]
fn test_no_deny_allows_everything() {
    let caps = ContainerCapabilities::new().allow_network("github.com", "443");

    assert!(caps.check_network("github.com", 443));
    assert!(caps.check_network("anything.com", 9999));
}

</storycode:copy>
<a id="f-797343670ac8-test_deny_all_without_allows"></a>
### `test_deny_all_without_allows`

<storycode:copy id="f-797343670ac8-test_deny_all_without_allows">
#[test]
fn test_deny_all_without_allows() {
    let caps = ContainerCapabilities::new().deny_all_network();

    assert!(!caps.check_network("github.com", 443));
    assert!(!caps.check_network("localhost", 8080));
}

</storycode:copy>
<a id="f-797343670ac8-test_multiple_allows_with_deny"></a>
### `test_multiple_allows_with_deny`

<storycode:copy id="f-797343670ac8-test_multiple_allows_with_deny">
#[test]
fn test_multiple_allows_with_deny() {
    let caps = ContainerCapabilities::new()
        .allow_network("github.com", "443")
        .allow_network("pypi.org", "443")
        .allow_network("registry.npmjs.org", "443")
        .deny_all_network();

    assert!(caps.check_network("github.com", 443));
    assert!(caps.check_network("pypi.org", 443));
    assert!(caps.check_network("registry.npmjs.org", 443));
    assert!(!caps.check_network("evil.com", 80));
    assert!(!caps.check_network("github.com", 80)); // wrong port
}

</storycode:copy>
<a id="f-797343670ac8-test_file_write_enforces_path"></a>
### `test_file_write_enforces_path`

<storycode:copy id="f-797343670ac8-test_file_write_enforces_path">
// ---------------------------------------------------------------------------
// File rule tests
// ---------------------------------------------------------------------------

#[test]
fn test_file_write_enforces_path() {
    let caps = ContainerCapabilities::new().allow_file_write("/output/*");

    assert!(caps.check_file_write("/output/x.txt"));
    assert!(!caps.check_file_write("/etc/passwd"));
}

</storycode:copy>
<a id="f-797343670ac8-test_file_read_enforces_path"></a>
### `test_file_read_enforces_path`

<storycode:copy id="f-797343670ac8-test_file_read_enforces_path">
#[test]
fn test_file_read_enforces_path() {
    let caps = ContainerCapabilities::new().allow_file_read("/input/*");

    assert!(caps.check_file_read("/input/data"));
    assert!(!caps.check_file_read("/secret/key"));
}

</storycode:copy>
<a id="f-797343670ac8-test_write_implies_read"></a>
### `test_write_implies_read`

<storycode:copy id="f-797343670ac8-test_write_implies_read">
#[test]
fn test_write_implies_read() {
    let caps = ContainerCapabilities::new().allow_file_write("/output/*");

    assert!(caps.check_file_read("/output/x.txt"));
}

</storycode:copy>
<a id="f-797343670ac8-test_read_does_not_imply_write"></a>
### `test_read_does_not_imply_write`

<storycode:copy id="f-797343670ac8-test_read_does_not_imply_write">
#[test]
fn test_read_does_not_imply_write() {
    let caps = ContainerCapabilities::new().allow_file_read("/input/*");

    assert!(caps.check_file_read("/input/data"));
    assert!(!caps.check_file_write("/input/data"));
}

</storycode:copy>
<a id="f-797343670ac8-test_attenuation_adds_restrictions"></a>
### `test_attenuation_adds_restrictions`

<storycode:copy id="f-797343670ac8-test_attenuation_adds_restrictions">
// ---------------------------------------------------------------------------
// Token tests
// ---------------------------------------------------------------------------

#[test]
fn test_attenuation_adds_restrictions() {
    let authority = test_authority();
    let caps = ContainerCapabilities::new()
        .allow_network("github.com", "443")
        .allow_network("pypi.org", "443");

    let token = authority.mint("sandbox", &caps).unwrap();

    // Attenuate with deny-all
    let extra = ContainerCapabilities::new().deny_all_network();
    let attenuated = token.attenuate(&extra).unwrap();

    // Attenuated token's capabilities include the deny-all
    let att_caps = attenuated.capabilities();
    assert!(
        att_caps
            .network_rules
            .iter()
            .any(|r| matches!(r, NetworkRule::DenyAll))
    );
    // But still has the original allows
    assert!(att_caps.check_network("github.com", 443));
    assert!(att_caps.check_network("pypi.org", 443));
    // And blocks non-whitelisted
    assert!(!att_caps.check_network("evil.com", 80));
}

</storycode:copy>
<a id="f-797343670ac8-test_attenuated_token_verifies"></a>
### `test_attenuated_token_verifies`

<storycode:copy id="f-797343670ac8-test_attenuated_token_verifies">
#[test]
fn test_attenuated_token_verifies() {
    let authority = test_authority();
    let caps = ContainerCapabilities::new()
        .allow_network("github.com", "443")
        .deny_all_network();

    let token = authority.mint("demo", &caps).unwrap();
    let extra = ContainerCapabilities::new().allow_file_read("/input/*");
    let attenuated = token.attenuate(&extra).unwrap();

    authority.verify(&attenuated).unwrap();
}

</storycode:copy>
<a id="f-797343670ac8-test_wrong_authority_rejects_token"></a>
### `test_wrong_authority_rejects_token`

<storycode:copy id="f-797343670ac8-test_wrong_authority_rejects_token">
#[test]
fn test_wrong_authority_rejects_token() {
    let authority_a = TokenAuthority::new(b"key-aaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    let authority_b = TokenAuthority::new(b"key-bbbbbbbbbbbbbbbbbbbbbbbbbbbb");

    let caps = ContainerCapabilities::new().deny_all_network();
    let token = authority_a.mint("test", &caps).unwrap();

    assert!(
        authority_b.verify(&token).is_err(),
        "token minted by A should not verify with B"
    );
}

</storycode:copy>
<a id="f-797343670ac8-test_container_all_rule_types"></a>
### `test_container_all_rule_types`

Capabilities-from-tags tests (end-to-end parse -> capability check)

<storycode:copy id="f-797343670ac8-test_container_all_rule_types">
// ---------------------------------------------------------------------------
// Capabilities-from-tags tests (end-to-end parse -> capability check)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_container_all_rule_types() {
    let src = hick_doc(
        r#"<hick:container name="full" image="python:3.12">
  <hick:allow network="github.com:443" />
  <hick:allow network="pypi.org:443" />
  <hick:deny network="*" />
  <hick:allow file-read="/input/*" />
  <hick:allow file-write="/output/*" />
  <hick:secret name="API_KEY" from="api-key" />
</hick:container>"#,
    );

    let result = hick_literate::run_pipeline(&[("test.md", &src)], &[])
        .await
        .unwrap();

    let caps = result.containers.get("full").unwrap();

    // Network rules
    assert!(caps.check_network("github.com", 443));
    assert!(caps.check_network("pypi.org", 443));
    assert!(!caps.check_network("evil.com", 80));

    // File rules
    assert!(caps.check_file_read("/input/data.csv"));
    assert!(caps.check_file_write("/output/report.html"));
    assert!(!caps.check_file_write("/input/data.csv"));

    // Secret rules
    assert_eq!(caps.secret_rules.len(), 1);
    assert_eq!(caps.secret_rules[0].env_var, "API_KEY");
    assert_eq!(caps.secret_rules[0].secret_name, "api-key");
}

</storycode:copy>
<a id="f-797343670ac8-test_empty_container_allows_all"></a>
### `test_empty_container_allows_all`

<storycode:copy id="f-797343670ac8-test_empty_container_allows_all">
#[tokio::test]
async fn test_empty_container_allows_all() {
    let src = hick_doc(r#"<hick:container name="open" image="alpine" />"#);

    let result = hick_literate::run_pipeline(&[("test.md", &src)], &[])
        .await
        .unwrap();

    let caps = result.containers.get("open").unwrap();

    // No rules -> default allow for network
    assert!(caps.network_rules.is_empty());
    assert!(caps.check_network("anything.com", 9999));
    // No file rules -> deny (closed-by-default)
    assert!(caps.file_rules.is_empty());
    assert!(!caps.check_file_read("/any/path"));
    assert!(!caps.check_file_write("/any/path"));
}

</storycode:copy>
<a id="f-797343670ac8-test_dag_topological_covers_all_execs"></a>
### `test_dag_topological_covers_all_execs`

<storycode:copy id="f-797343670ac8-test_dag_topological_covers_all_execs">
// ---------------------------------------------------------------------------
// DAG integrity tests
// ---------------------------------------------------------------------------

#[test]
fn test_dag_topological_covers_all_execs() {
    let src = hick_doc(
        r#"<hick:container name="a" image="alpine" />
<hick:container name="b" image="alpine" />
<hick:exec container="a">step 1</hick:exec>
<hick:exec container="a">step 2</hick:exec>
<hick:exec container="b">step 3</hick:exec>
<hick:exec container="b">step 4</hick:exec>"#,
    );
    let doc = hick_lang::parse(&src).unwrap();
    let dag = hick_exec::dag::build_dag(&doc).unwrap();
    let topo = dag.topological_order();
    assert_eq!(
        topo.len(),
        dag.execs.len(),
        "topological order must cover all execs"
    );
}

</storycode:copy>
<a id="f-797343670ac8-test_dag_rejects_broken_paste"></a>
### `test_dag_rejects_broken_paste`

<storycode:copy id="f-797343670ac8-test_dag_rejects_broken_paste">
#[test]
fn test_dag_rejects_broken_paste() {
    // Paste referencing non-existent copy inside an exec:
    // The DAG builder does not currently validate orphan paste selectors as an error;
    // it silently ignores them if they don't match any copy producer inside an exec.
    // This test documents that behaviour: the DAG builds successfully and the paste
    // simply produces no dependency edge.
    let src = hick_doc(
        r##"<hick:exec container="demo" image="alpine"><hick:paste select="#ghost" />echo ok</hick:exec>"##,
    );
    let doc = hick_lang::parse(&src).unwrap();
    let dag = hick_exec::dag::build_dag(&doc).unwrap();
    assert_eq!(dag.execs.len(), 1);
    // No copy/paste edge since the copy doesn't exist
    assert_eq!(dag.edges.len(), 0);
}
</storycode:copy>

### Assemble `crates/hick-literate/tests/security_tests.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/tests/security_tests.rs"><storycode:paste select="#f-797343670ac8-test_authority"/><storycode:paste select="#f-797343670ac8-hick_doc"/><storycode:paste select="#f-797343670ac8-test_deny_all_blocks_non_whitelisted"/><storycode:paste select="#f-797343670ac8-test_no_deny_allows_everything"/><storycode:paste select="#f-797343670ac8-test_deny_all_without_allows"/><storycode:paste select="#f-797343670ac8-test_multiple_allows_with_deny"/><storycode:paste select="#f-797343670ac8-test_file_write_enforces_path"/><storycode:paste select="#f-797343670ac8-test_file_read_enforces_path"/><storycode:paste select="#f-797343670ac8-test_write_implies_read"/><storycode:paste select="#f-797343670ac8-test_read_does_not_imply_write"/><storycode:paste select="#f-797343670ac8-test_attenuation_adds_restrictions"/><storycode:paste select="#f-797343670ac8-test_attenuated_token_verifies"/><storycode:paste select="#f-797343670ac8-test_wrong_authority_rejects_token"/><storycode:paste select="#f-797343670ac8-test_container_all_rule_types"/><storycode:paste select="#f-797343670ac8-test_empty_container_allows_all"/><storycode:paste select="#f-797343670ac8-test_dag_topological_covers_all_execs"/><storycode:paste select="#f-797343670ac8-test_dag_rejects_broken_paste"/></storycode:file>

</storycode:doc>
