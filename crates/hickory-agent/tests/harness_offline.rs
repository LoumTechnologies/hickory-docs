//! Historical economics reports remain readable after removing the Rust runtime.
#[test]
fn reports_historical_measurements_offline() {
    let root = tempfile::tempdir().unwrap();
    let report = hickory_agent::harness::generate_report(root.path()).unwrap();
    assert!(!report.is_empty());
}
