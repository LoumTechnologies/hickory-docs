<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-agent/tests/harness_offline.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `46936630f0e3613ab0b9e11bc8e6287140404e96287b0ed55905275f8234f85a`.

<book:entry path="crates/hickory-agent/tests/harness_offline.rs">
//! Historical economics reports remain readable after removing the Rust runtime.
#[test]
fn reports_historical_measurements_offline() {
    let root = tempfile::tempdir().unwrap();
    let report = hickory_agent::harness::generate_report(root.path()).unwrap();
    assert!(!report.is_empty());
}
</book:entry>

</book:doc>
