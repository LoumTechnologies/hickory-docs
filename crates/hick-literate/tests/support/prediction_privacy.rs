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
