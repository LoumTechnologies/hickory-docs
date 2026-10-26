<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-agent/tests/harness_offline.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a249eb0fed42"></a>
## `crates/hickory-agent/tests/harness_offline.rs`

<a id="f-a249eb0fed42-reports_historical_measurements_offline"></a>
### `reports_historical_measurements_offline`

Historical economics reports remain readable after removing the Rust runtime.

<storycode:copy id="f-a249eb0fed42-reports_historical_measurements_offline">
//! Historical economics reports remain readable after removing the Rust runtime.
#[test]
fn reports_historical_measurements_offline() {
    let root = tempfile::tempdir().unwrap();
    let report = hickory_agent::harness::generate_report(root.path()).unwrap();
    assert!(!report.is_empty());
}
</storycode:copy>

### Assemble `crates/hickory-agent/tests/harness_offline.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-agent/tests/harness_offline.rs"><storycode:paste select="#f-a249eb0fed42-reports_historical_measurements_offline"/></storycode:file>

</storycode:doc>
