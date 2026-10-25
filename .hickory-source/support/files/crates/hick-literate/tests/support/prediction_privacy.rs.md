<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-literate/tests/support/prediction_privacy.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `1d7003530de67acb85271b574a2d4b5b0cd37b71f4a78e2235eaddfa49305173`.

<book:entry path="crates/hick-literate/tests/support/prediction_privacy.rs">
//! Compare JSON values/keys, not random ID substrings (a UUID can contain 160).
pub fn assert_withheld(value: &impl serde::Serialize, answer: serde_json::Value) {
    fn inspect(value: &serde_json::Value, answer: &serde_json::Value) {
        assert_ne!(value, answer, "Actual answer appeared in public feedback");
        match value {
            serde_json::Value::Object(fields) => {
                for (key, value) in fields {
                    assert!(
                        ![
                            "actual",
                            "evidence",
                            "captures",
                            "transcripts",
                            "files",
                            "stdout",
                            "stderr"
                        ]
                        .contains(&key.as_str())
                    );
                    inspect(value, answer);
                }
            }
            serde_json::Value::Array(values) => {
                for value in values {
                    inspect(value, answer);
                }
            }
            _ => {}
        }
    }
    inspect(&serde_json::to_value(value).unwrap(), &answer);
}
</book:entry>

</book:doc>
