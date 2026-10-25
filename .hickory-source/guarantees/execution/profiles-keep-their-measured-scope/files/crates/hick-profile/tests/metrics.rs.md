<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-profile/tests/metrics.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Profiles keep their measured scope](../../../../contract.md)

Captured SHA-256: `edfe505ebe49f2ca4ae938057628208bc00275bb97193d4d39ac91cb5ccd282f`.

<book:entry path="crates/hick-profile/tests/metrics.rs">
// Guarantee: docs/guarantees/execution/profiles-keep-their-measured-scope.md
use hick_profile::parse;
use serde_json::json;
#[test]
fn sampled_weights_preserve_threads_and_do_not_invent_calls() {
    let raw=json!({"shared":{"frames":[{"name":"outer","file":"app.rs","line":1},{"name":"inner","file":"app.rs","line":2}]},"profiles":[{"type":"sampled","name":"main","unit":"milliseconds","startValue":0,"endValue":5,"samples":[[0,1],[0]],"weights":[3,2]},{"type":"sampled","name":"worker","unit":"milliseconds","startValue":0,"endValue":2,"samples":[[1]],"weights":[2]}]}).to_string();
    let r = parse("speedscope", &raw).unwrap();
    assert_eq!(r.threads.len(), 2);
    let main = &r.threads[0];
    assert_eq!(main.unit, "seconds");
    assert_eq!(main.measured_weight, 0.005);
    let outer = main.rows.iter().find(|r| r.frame.name == "outer").unwrap();
    assert_eq!(outer.self_weight, 0.002);
    assert_eq!(outer.inclusive_weight, 0.005);
    assert_eq!(outer.total_calls, None);
    assert_eq!(outer.samples, Some(2));
    assert_eq!(r.raw, raw);
}
#[test]
fn recursion_does_not_multiply_inclusive_stack_weight() {
    let raw=json!({"shared":{"frames":[{"name":"recursive"}]},"profiles":[{"type":"sampled","unit":"seconds","startValue":0,"endValue":1,"samples":[[0,0,0]],"weights":[1]}]}).to_string();
    let r = parse("speedscope", &raw).unwrap();
    let row = &r.threads[0].rows[0];
    assert_eq!(row.self_weight, 1.);
    assert_eq!(row.inclusive_weight, 1.);
    assert_eq!(row.samples, Some(1));
}
#[test]
fn evented_profiles_keep_self_and_overlapping_inclusive_costs_separate() {
    let raw=json!({"shared":{"frames":[{"name":"outer"},{"name":"inner"}]},"profiles":[{"type":"evented","unit":"seconds","startValue":0,"endValue":5,"events":[{"type":"O","at":0,"frame":0},{"type":"O","at":1,"frame":1},{"type":"C","at":4,"frame":1},{"type":"C","at":5,"frame":0}]}]}).to_string();
    let r = parse("speedscope", &raw).unwrap();
    let t = &r.threads[0];
    assert_eq!(t.measured_weight, 5.);
    let outer = t.rows.iter().find(|r| r.frame.name == "outer").unwrap();
    assert_eq!(outer.self_weight, 2.);
    assert_eq!(outer.inclusive_weight, 5.);
    assert_eq!(outer.samples, None);
}
#[test]
fn v8_uses_microsecond_deltas_and_zero_based_locations() {
    let raw=json!({"nodes":[{"id":1,"callFrame":{"functionName":"(root)","url":"","lineNumber":-1},"children":[2]},{"id":2,"callFrame":{"functionName":"hot","url":"file:///app.ts","lineNumber":4,"columnNumber":2,"scriptId":"1"}}],"startTime":100,"endTime":3100,"samples":[2,2],"timeDeltas":[1000,2000]}).to_string();
    let r = parse("chrome-cpu", &raw).unwrap();
    let hot = r.threads[0]
        .rows
        .iter()
        .find(|r| r.frame.name == "hot")
        .unwrap();
    assert_eq!(hot.frame.line, Some(5));
    assert_eq!(hot.frame.column, Some(3));
    assert_eq!(hot.self_weight, 0.003);
    assert_eq!(hot.total_calls, None);
}
#[test]
fn missing_v8_deltas_remain_samples_not_estimated_time() {
    let r=parse("chrome-cpu",r#"{"nodes":[{"id":1,"callFrame":{"functionName":"hot"}}],"startTime":0,"endTime":100,"samples":[1]}"#).unwrap();
    assert_eq!(r.threads[0].unit, "samples");
    assert_eq!(r.threads[0].rows[0].self_weight, 1.);
}
#[test]
fn cprofile_retains_recursion_counts_and_inclusive_times() {
    let raw=json!({"elapsed_seconds":2,"functions":[{"name":"recursive","file":"app.py","line":3,"total_calls":9,"primitive_calls":1,"self_seconds":0.2,"cumulative_seconds":1.8}]}).to_string();
    let r = parse("cprofile-json", &raw).unwrap();
    let row = &r.threads[0].rows[0];
    assert_eq!(row.total_calls, Some(9));
    assert_eq!(row.primitive_calls, Some(1));
    assert_eq!(row.samples, None);
    assert_eq!(row.inclusive_weight, 1.8);
}
#[test]
fn malformed_or_unknown_profiles_never_become_zero_cost() {
    for (format, raw) in [
        ("other", "{}"),
        (
            "chrome-cpu",
            r#"{"nodes":[],"startTime":0,"endTime":1,"samples":[2],"timeDeltas":[1]}"#,
        ),
        (
            "speedscope",
            r#"{"shared":{"frames":[]},"profiles":[{"type":"sampled","unit":"seconds","startValue":0,"endValue":1,"samples":[[]]}]}"#,
        ),
        (
            "speedscope",
            r#"{"shared":{"frames":[{"name":"a"}]},"profiles":[{"type":"evented","unit":"seconds","startValue":0,"endValue":1,"events":[{"type":"C","frame":0,"at":0}]}]}"#,
        ),
    ] {
        assert!(parse(format, raw).is_err());
    }
}
#[test]
fn processed_firefox_preserves_symbolicated_rust_locations_and_sample_units() {
    let raw=json!({"threads":[{"name":"main","tid":7,"stringArray":["crate::hot","src/main.rs"],"funcTable":{"name":[0],"fileName":[1],"lineNumber":[12],"columnNumber":[1]},"frameTable":{"func":[0]},"stackTable":{"frame":[0],"prefix":[null]},"samples":{"stack":[0,0],"time":[1,3],"weightType":"samples","weight":[1,2]}}]}).to_string();
    let r = parse("firefox", &raw).unwrap();
    let t = &r.threads[0];
    assert_eq!(t.unit, "samples");
    assert_eq!(t.measured_weight, 3.);
    assert_eq!(t.rows[0].frame.file.as_deref(), Some("src/main.rs"));
    assert_eq!(t.rows[0].frame.line, Some(12));
    assert_eq!(t.rows[0].total_calls, None);
}
#[test]
fn unconverted_firefox_profiles_are_unavailable_instead_of_zero() {
    assert!(parse("firefox", r#"{"threads":[{"samples":{}}]}"#).is_err());
}
</book:entry>

</book:doc>
