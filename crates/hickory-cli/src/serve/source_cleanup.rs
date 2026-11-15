//! Explicit cleanup produces ordinary source edits; it never saves or invokes a provider.
use serde::Deserialize;
use serde_json::{Value, json};
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    TrimTrailingHorizontal,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub uri: String,
    pub source: String,
    pub profile: Profile,
}
pub fn prepare(request: Request) -> Result<Value, String> {
    if request.source.len() > 8 * 1024 * 1024 {
        return Err(
            "Cleanup source exceeds 8 MiB. Narrow the source before preparing again.".into(),
        );
    }
    super::resource_changes::targets(&json!({"changes":{request.uri.clone():[]}}))?;
    let mut edits = vec![];
    match request.profile {
        Profile::TrimTrailingHorizontal => {
            for (line, row) in request.source.split_inclusive('\n').enumerate() {
                let body = if let Some(row) = row.strip_suffix('\n') {
                    row.strip_suffix('\r').unwrap_or(row)
                } else {
                    row
                };
                let trimmed = body.trim_end_matches([' ', '\t']);
                if trimmed.len() != body.len() {
                    if edits.len() >= 10_000 {
                        return Err("Cleanup exceeds 10,000 edits. Narrow the source before preparing again.".into());
                    }
                    edits.push(json!({"range":{"start":{"line":line,"character":trimmed.encode_utf16().count()},"end":{"line":line,"character":body.encode_utf16().count()}},"newText":""}));
                }
            }
        }
    }
    Ok(json!({"changes":{request.uri:edits}}))
}
pub async fn inspect(
    axum::Json(request): axum::Json<Request>,
) -> super::api::ApiResult<axum::Json<Value>> {
    tokio::task::spawn_blocking(move || prepare(request))
        .await
        .map_err(|error| {
            super::api::ApiError::internal(format!(
                "Cleanup preparation failed: {error}. Prepare again."
            ))
        })?
        .map(axum::Json)
        .map_err(super::api::ApiError::unprocessable)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn edits_preserve_unicode_line_endings_indentation_and_absence_of_a_final_newline() {
        let edit = prepare(Request {
            uri: "hick:///a.py".into(),
            source: "\t😀 value \t\r\n  keep\nlast \t".into(),
            profile: Profile::TrimTrailingHorizontal,
        })
        .unwrap();
        assert_eq!(
            edit,
            json!({"changes":{"hick:///a.py":[
                {"range":{"start":{"line":0,"character":9},"end":{"line":0,"character":11}},"newText":""},
                {"range":{"start":{"line":2,"character":4},"end":{"line":2,"character":6}},"newText":""}
            ]}})
        );
    }
    #[test]
    fn a_noop_is_empty_and_invalid_scope_or_excess_edits_refuse_the_whole_proposal() {
        let request = |uri: &str, source: String| Request {
            uri: uri.into(),
            source,
            profile: Profile::TrimTrailingHorizontal,
        };
        assert_eq!(
            prepare(request("hick:///a.py", "  keep\r\n\u{a0}".into())).unwrap(),
            json!({"changes":{"hick:///a.py":[]}})
        );
        assert!(prepare(request("file:///outside.py", "x ".into())).is_err());
        assert!(prepare(request("hick:///.git/config", "x ".into())).is_err());
        assert!(prepare(request("hick:///a.py", "x \n".repeat(10_001))).is_err());
    }
}
