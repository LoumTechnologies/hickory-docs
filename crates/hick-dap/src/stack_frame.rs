use crate::mapping::{Mapping, same_file};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

/// Where execution is, in the document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    pub id: i64,
    pub name: String,
    /// 0-based document line, when this frame is in a file the document
    /// generated. `None` for a library frame, which is shown read-only rather
    /// than pretended to be part of the document.
    pub line: Option<u32>,
    /// The file as the adapter named it, for frames outside the document.
    pub source: Option<String>,
    /// 0-based line in `source`, after mapping to a contributor document when
    /// possible, otherwise the adapter's raw source line.
    pub source_line: Option<u32>,
    /// Exact captured launch/construction source, not frozen-runtime proof.
    /// Unknown external sources never borrow the main document's digest.
    #[serde(default)]
    pub source_hash: Option<String>,
    /// Positive adapter-owned source references cannot borrow filesystem proof.
    #[serde(default)]
    pub source_reference: Option<i64>,
    pub in_document: bool,
    /// Raw adapter coordinates, before document mapping; private capture binds
    /// these independently from the human-facing document coordinates.
    #[serde(default)]
    pub adapter_source: Option<String>,
    #[serde(default)]
    pub adapter_line: Option<u32>,
}

impl Frame {
    pub(super) fn from_response(mapping: &Mapping, body: &Value) -> Result<Vec<Self>> {
        let rows = body
            .get("stackFrames")
            .and_then(Value::as_array)
            .context("Adapter returned no valid stack-frame list")?;
        if rows.len() > 4096 {
            bail!("Adapter stack exceeds the interactive frame budget");
        }
        let mut ids = std::collections::HashSet::new();
        rows.iter()
            .map(|row| {
                let id = row
                    .get("id")
                    .and_then(Value::as_i64)
                    .filter(|id| i32::try_from(*id).is_ok())
                    .context("Adapter returned an invalid stack-frame identity")?;
                if !ids.insert(id) {
                    bail!("Adapter returned duplicate stack-frame identities");
                }
                let name = row
                    .get("name")
                    .and_then(Value::as_str)
                    .context("Adapter returned an unnamed stack frame")?
                    .to_owned();
                let path = row
                    .pointer("/source/path")
                    .and_then(Value::as_str)
                    .map(PathBuf::from);
                let source_reference = match row.pointer("/source/sourceReference") {
                    None => None,
                    Some(value) => {
                        let reference = value
                            .as_i64()
                            .filter(|value| (0..=i32::MAX as i64).contains(value))
                            .context("Adapter returned an invalid source reference")?;
                        (reference > 0).then_some(reference)
                    }
                };
                // Our initialize request uses one-based lines. Zero is unavailable.
                let raw_line = row
                    .get("line")
                    .and_then(Value::as_u64)
                    .and_then(|line| u32::try_from(line).ok())
                    .filter(|line| *line > 0)
                    .map(|line| line - 1);
                let location = path
                    .as_deref()
                    .zip(raw_line)
                    .filter(|_| source_reference.is_none())
                    .and_then(|(path, line)| mapping.source_location(path, line));
                let line = location
                    .as_ref()
                    .filter(|(path, _)| same_file(path, mapping.document()))
                    .map(|(_, line)| *line);
                let source_hash = location
                    .as_ref()
                    .map(|(path, _)| path.as_path())
                    .or(path.as_deref())
                    .filter(|_| source_reference.is_none())
                    .and_then(|source| mapping.source_hash(source));
                Ok(Self {
                    id,
                    name,
                    line,
                    source_hash,
                    source_reference,
                    source_line: location
                        .as_ref()
                        .map(|(_, line)| *line)
                        .or_else(|| path.as_ref().and(raw_line)),
                    adapter_source: path
                        .as_ref()
                        .map(|path| path.to_string_lossy().into_owned()),
                    adapter_line: path.as_ref().and(raw_line),
                    source: location
                        .map(|(path, _)| path)
                        .or(path)
                        .map(|path| path.to_string_lossy().into_owned()),
                    in_document: line.is_some(),
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn frame_identity_and_unavailable_source_are_not_invented() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("main.py");
        std::fs::write(&path, "value=1\n").unwrap();
        let mapping = Mapping::identity(&path);
        let row = json!({"id":0,"name":"main","source":{"path":path},"line":1});
        let frames = Frame::from_response(&mapping, &json!({"stackFrames":[row.clone()]})).unwrap();
        assert_eq!(frames[0].id, 0);
        assert_eq!(frames[0].line, Some(0));
        assert_eq!(
            frames[0].source_hash,
            Some(mapping.observation_source().unwrap().revision)
        );
        let mut virtual_source = row.clone();
        virtual_source["source"]["sourceReference"] = json!(17);
        let frames =
            Frame::from_response(&mapping, &json!({"stackFrames":[virtual_source]})).unwrap();
        assert_eq!(frames[0].source_reference, Some(17));
        assert_eq!(frames[0].source_hash, None);
        assert_eq!(frames[0].line, None);
        assert!(!frames[0].in_document);
        for line in [
            Value::Null,
            json!(0),
            json!(-1),
            json!(true),
            json!(4294967296u64),
        ] {
            let mut unknown = row.clone();
            unknown["id"] = json!(-7);
            unknown["line"] = line;
            let frames = Frame::from_response(&mapping, &json!({"stackFrames":[unknown]})).unwrap();
            assert_eq!(frames[0].id, -7);
            assert_eq!(frames[0].line, None);
            assert_eq!(frames[0].source_line, None);
            assert_eq!(frames[0].adapter_line, None);
        }
        for invalid in [
            json!({}),
            json!({"stackFrames":null}),
            json!({"stackFrames":[row.clone(),row.clone()]}),
            json!({"stackFrames":[{"id":true,"name":"boolean","line":1}]}),
            json!({"stackFrames":[{"id":2147483648i64,"name":"wide","line":1}]}),
        ] {
            assert!(Frame::from_response(&mapping, &invalid).is_err());
        }
        assert!(
            Frame::from_response(&mapping, &json!({"stackFrames":[]}))
                .unwrap()
                .is_empty()
        );
    }
}
