<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-literate/tests/support/prediction_privacy.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1f32c10dec2d"></a>
## `crates/hick-literate/tests/support/prediction_privacy.rs`

<a id="f-1f32c10dec2d-assert_withheld"></a>
### `assert_withheld`

Compare JSON values/keys, not random ID substrings (a UUID can contain 160).

<storycode:copy id="f-1f32c10dec2d-assert_withheld">
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
</storycode:copy>

### Assemble `crates/hick-literate/tests/support/prediction_privacy.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/tests/support/prediction_privacy.rs"><storycode:paste select="#f-1f32c10dec2d-assert_withheld"/></storycode:file>

</storycode:doc>
