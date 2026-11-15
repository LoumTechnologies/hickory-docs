use super::*;
use crate::{child_lsp::ChildLspHandle, provider_artifacts::ProviderArtifacts};
use serde_json::{Value, json};
use std::time::{Duration, Instant};
#[derive(Clone)]
struct HeldCompletion {
    original: Value,
    handle: ChildLspHandle,
    resolve: bool,
    virtual_uri: Url,
    origin: Url,
    position: Position,
}
pub(super) struct PreparedCompletions {
    items: ProviderArtifacts<HeldCompletion>,
}
impl Default for PreparedCompletions {
    fn default() -> Self {
        Self {
            items: ProviderArtifacts::new(4096, Duration::from_secs(120)),
        }
    }
}
fn completion_error(message: impl Into<String>) -> tower_lsp::jsonrpc::Error {
    tower_lsp::jsonrpc::Error::invalid_params(message.into())
}
fn completion_items(value: &mut Value) -> Result<&mut Vec<Value>> {
    if value.is_array() {
        return value
            .as_array_mut()
            .ok_or_else(|| completion_error("Malformed completion array"));
    }
    let defaults = value.get("itemDefaults").cloned();
    if let Some(defaults) = &defaults {
        if !defaults.is_object() {
            return Err(completion_error("Malformed completion defaults"));
        }
    }
    let items = value
        .get_mut("items")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| completion_error("Malformed completion list"))?;
    for item in items.iter_mut() {
        let object = item
            .as_object_mut()
            .ok_or_else(|| completion_error("Malformed completion item"))?;
        if let Some(defaults) = &defaults {
            for key in [
                "commitCharacters",
                "insertTextFormat",
                "insertTextMode",
                "data",
            ] {
                if !object.contains_key(key) {
                    if let Some(value) = defaults.get(key) {
                        object.insert(key.into(), value.clone());
                    }
                }
            }
            if !object.contains_key("textEdit") {
                if let Some(range) = defaults.get("editRange") {
                    let text = object
                        .get("textEditText")
                        .or_else(|| object.get("insertText"))
                        .or_else(|| object.get("label"))
                        .and_then(Value::as_str)
                        .ok_or_else(|| completion_error("Missing default completion text"))?;
                    let edit = if range.get("start").is_some() {
                        json!({"range":range,"newText":text})
                    } else if range.get("insert").is_some() && range.get("replace").is_some() {
                        json!({"insert":range["insert"],"replace":range["replace"],"newText":text})
                    } else {
                        return Err(completion_error("Malformed default completion range"));
                    };
                    object.insert("textEdit".into(), edit);
                }
            }
        }
    }
    Ok(items)
}
fn map_completion_item(
    original: &Value,
    primary: &Url,
    map: &PositionMap,
    origin: &Url,
    position: Position,
) -> Result<Value> {
    fn project(raw: &Value, primary: &Url, map: &PositionMap, origin: &Url) -> Result<Range> {
        let range: Range = serde_json::from_value(raw.clone())
            .map_err(|_| completion_error("Malformed completion range"))?;
        let (owner,range)=map.source_edit_range(range).ok_or_else(||completion_error("Completion edit crosses source fragments. Request a reviewed source change instead."))?;
        let mut uri = match owner {
            Some(path) => primary
                .join(&path)
                .map_err(|_| completion_error("Invalid completion source identity"))?,
            None => primary.clone(),
        };
        uri.set_query(primary.query());
        if &uri != origin {
            return Err(completion_error(
                "Completion edit belongs to another authored source. Request a reviewed source change instead.",
            ));
        }
        Ok(range)
    }
    fn edit(value: &mut Value, primary: &Url, map: &PositionMap, origin: &Url) -> Result<()> {
        let object = value
            .as_object_mut()
            .ok_or_else(|| completion_error("Malformed completion edit"))?;
        let original = object
            .get("range")
            .or_else(|| object.get("replace"))
            .cloned()
            .ok_or_else(|| completion_error("Missing completion range"))?;
        let raw: Range = serde_json::from_value(original)
            .map_err(|_| completion_error("Malformed completion range"))?;
        let text = object
            .get("newText")
            .and_then(Value::as_str)
            .ok_or_else(|| completion_error("Missing completion text"))?
            .to_owned();
        let simple = object.contains_key("range");
        if !simple && (!object.contains_key("insert") || !object.contains_key("replace")) {
            return Err(completion_error("Malformed insert/replace completion"));
        }
        for key in if simple {
            vec!["range"]
        } else {
            vec!["insert", "replace"]
        } {
            let range = project(
                object
                    .get(key)
                    .ok_or_else(|| completion_error("Missing completion range"))?,
                primary,
                map,
                origin,
            )?;
            object.insert(key.into(), json!(range));
        }
        let indent = if map.is_identity() || map.is_project() {
            0
        } else {
            map.mappings
                .iter()
                .find(|entry| entry.virtual_line == raw.start.line)
                .map(|entry| entry.column_offset.max(0) as usize)
                .unwrap_or(0)
        };
        object.insert(
            "newText".into(),
            json!(reindent(&text, indent, raw.end.character == 0)),
        );
        Ok(())
    }
    if original.as_object().is_none() || original.get("label").and_then(Value::as_str).is_none() {
        return Err(completion_error("Malformed completion item"));
    }
    if let Some(edit) = original.get("textEdit") {
        if edit.get("range").is_some()
            && (edit.get("insert").is_some() || edit.get("replace").is_some())
        {
            return Err(completion_error("Ambiguous completion edit"));
        }
        let keys = if edit.get("range").is_some() {
            vec!["range"]
        } else {
            vec!["insert", "replace"]
        };
        for key in keys {
            let range: Range = serde_json::from_value(
                edit.get(key)
                    .cloned()
                    .ok_or_else(|| completion_error("Missing completion range"))?,
            )
            .map_err(|_| completion_error("Malformed completion range"))?;
            if range.start.line != position.line
                || range.end.line != position.line
                || range.start > position
                || range.end < position
            {
                return Err(completion_error(
                    "Completion replacement must contain its original cursor on one line",
                ));
            }
        }
        if let (Some(insert), Some(replace)) = (edit.get("insert"), edit.get("replace")) {
            let insert: Range = serde_json::from_value(insert.clone())
                .map_err(|_| completion_error("Malformed insertion range"))?;
            let replace: Range = serde_json::from_value(replace.clone())
                .map_err(|_| completion_error("Malformed replacement range"))?;
            if insert.start != replace.start || insert.end > replace.end {
                return Err(completion_error(
                    "Incompatible insert/replace completion ranges",
                ));
            }
        }
    }
    let mut mapped = original.clone();
    if let Some(value) = mapped.get_mut("textEdit") {
        edit(value, primary, map, origin)?;
    }
    if let Some(values) = mapped.get_mut("additionalTextEdits") {
        for value in values
            .as_array_mut()
            .ok_or_else(|| completion_error("Malformed additional completion edits"))?
        {
            edit(value, primary, map, origin)?;
        }
    }
    Ok(mapped)
}
impl HickBackend {
    pub(super) async fn provider_completion(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        let revision = self.provider_source_revision().await;
        let source = params.text_document_position;
        let origin = source.text_document.uri;
        let Some((virtual_uri, language, position)) =
            self.virtual_target(&origin, source.position).await
        else {
            return Ok(None);
        };
        let Some(root) = self.root_of(&virtual_uri).await else {
            return Ok(None);
        };
        let (handle, resolve) = {
            let dispatcher = self.dispatcher.lock().await;
            let Ok(handle) = dispatcher.get_child(&language, &root) else {
                return Ok(None);
            };
            let resolve = dispatcher
                .server_capabilities(&language, &root)
                .and_then(|caps| caps.pointer("/completionProvider/resolveProvider"))
                .and_then(Value::as_bool)
                == Some(true);
            (handle.clone(), resolve)
        };
        self.require_provider_revision(&revision).await?;
        let mut request = json!({"textDocument":{"uri":virtual_uri},"position":position});
        if let Some(context) = params.context {
            request["context"] = json!(context);
        }
        let mut result = handle
            .request("textDocument/completion", request)
            .await
            .map_err(child_request_error)?;
        self.require_provider_revision(&revision).await?;
        if result.is_null() {
            return Ok(None);
        }
        let index = self.index_snapshot().await;
        let (primary, map) = index
            .get(&uri_key(virtual_uri.as_str()))
            .ok_or_else(|| completion_error("Completion source mapping changed. Request again."))?;
        let items = completion_items(&mut result)?;
        if items.len() > 2048 {
            return Err(completion_error(
                "More than 2,048 completion items. Type a longer prefix.",
            ));
        }
        let originals = items.clone();
        let mapped = originals
            .iter()
            .map(|item| map_completion_item(item, primary, map, &origin, position))
            .collect::<Result<Vec<_>>>()?;
        self.require_provider_revision(&revision).await?;
        let mut held = self.prepared_completions.lock().await;
        for ((item, original), mapped) in items.iter_mut().zip(originals).zip(mapped) {
            let token = held.items.insert(
                Instant::now(),
                revision.clone(),
                HeldCompletion {
                    original,
                    handle: handle.clone(),
                    resolve,
                    virtual_uri: virtual_uri.clone(),
                    origin: origin.clone(),
                    position,
                },
            );
            *item = mapped;
            item["data"] = json!({"hickory_completion":token});
        }
        drop(held);
        self.require_provider_revision(&revision).await?;
        serde_json::from_value(result)
            .map(Some)
            .map_err(|_| completion_error("Malformed mapped completion response"))
    }
    pub(super) async fn resolve_provider_completion(
        &self,
        item: CompletionItem,
    ) -> Result<CompletionItem> {
        let token = item
            .data
            .as_ref()
            .and_then(|data| data.get("hickory_completion"))
            .and_then(Value::as_str)
            .ok_or_else(|| completion_error("Unknown completion. Request completions again."))?;
        let revision = self.provider_source_revision().await;
        let held = self
            .prepared_completions
            .lock()
            .await
            .items
            .get(Instant::now(), token, &revision)
            .ok_or_else(|| {
                completion_error("Completion expired or source changed. Request completions again.")
            })?;
        let mut original = held.original;
        if held.resolve {
            let resolved = held
                .handle
                .request("completionItem/resolve", original.clone())
                .await
                .map_err(child_request_error)?;
            self.require_provider_revision(&revision).await?;
            let fields = resolved
                .as_object()
                .ok_or_else(|| completion_error("Provider did not resolve this completion"))?;
            let object = original
                .as_object_mut()
                .ok_or_else(|| completion_error("Malformed held completion"))?;
            for (key, value) in fields {
                object.insert(key.clone(), value.clone());
            }
        }
        let index = self.index_snapshot().await;
        let (primary, map) = index
            .get(&uri_key(held.virtual_uri.as_str()))
            .ok_or_else(|| completion_error("Completion source mapping changed. Request again."))?;
        let mut mapped = map_completion_item(&original, primary, map, &held.origin, held.position)?;
        let snippet = completion_snippet_data(&mapped, &held.origin, held.position, map)?;
        mapped["data"] = json!({"hickory_completion":token});
        if let Some(snippet) = snippet {
            mapped["data"]["hickory_snippet"] = snippet;
        }
        self.require_provider_revision(&revision).await?;
        serde_json::from_value(mapped)
            .map_err(|_| completion_error("Malformed resolved completion"))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completion_defaults_expand_before_retaining_the_provider_item() {
        let range = json!({"start":{"line":0,"character":0},"end":{"line":0,"character":3}});
        let mut response = json!({"isIncomplete":false,"itemDefaults":{"editRange":range,"insertTextFormat":2,"data":{"provider":"default"}},"items":[{"label":"display","textEditText":"${1:value}","data":{"provider":"specific"}},{"label":"fallback"}]});
        let items = completion_items(&mut response).unwrap();
        assert_eq!(items[0]["textEdit"]["newText"], "${1:value}");
        assert_eq!(items[0]["data"]["provider"], "specific");
        assert_eq!(items[1]["data"]["provider"], "default");
        assert_eq!(items[1]["textEdit"]["range"], range);
        assert_eq!(items[1]["insertTextFormat"], 2);
    }
    #[test]
    fn completion_projection_maps_known_edits_without_rewriting_opaque_data() {
        let origin = Url::parse("file:///book.md").unwrap();
        let map = PositionMap::build(&[crate::virtual_file::VirtualSegment {
            text: "import\nold\n".into(),
            source_span: None,
            source_line: 6,
            source_column: 2,
        }]);
        let original = json!({"label":"new","textEdit":{"range":{"start":{"line":1,"character":0},"end":{"line":1,"character":3}},"newText":"new\nnext"},"additionalTextEdits":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":"import other\n"}],"data":{"range":{"start":{"line":91,"character":0},"end":{"line":91,"character":1}}}});
        let mapped =
            map_completion_item(&original, &origin, &map, &origin, Position::new(1, 2)).unwrap();
        assert_eq!(
            mapped["textEdit"]["range"]["start"],
            json!({"line":6,"character":2})
        );
        assert_eq!(mapped["textEdit"]["newText"], "new\n  next");
        assert_eq!(
            mapped["additionalTextEdits"][0]["range"]["start"]["line"],
            5
        );
        assert_eq!(mapped["data"], original["data"]);
        assert_eq!(original["textEdit"]["range"]["start"]["line"], 1);
        assert!(
            map_completion_item(
                &original,
                &origin,
                &map,
                &Url::parse("file:///other.md").unwrap(),
                Position::new(1, 2)
            )
            .is_err()
        );
    }
    #[test]
    fn completion_ranges_cannot_escape_the_cursor_or_swallow_source_fragments() {
        let origin = Url::parse("file:///book.md").unwrap();
        let map = PositionMap::build(&[
            crate::virtual_file::VirtualSegment {
                text: "old\n".into(),
                source_span: None,
                source_line: 1,
                source_column: 0,
            },
            crate::virtual_file::VirtualSegment {
                text: "next\n".into(),
                source_span: None,
                source_line: 10,
                source_column: 0,
            },
        ]);
        let base = json!({"label":"new","textEdit":{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":3}},"newText":"new"}});
        assert!(map_completion_item(&base, &origin, &map, &origin, Position::new(1, 1)).is_err());
        let mut crossing = base.clone();
        crossing["additionalTextEdits"] = json!([{"range":{"start":{"line":0,"character":0},"end":{"line":1,"character":3}},"newText":"replacement"}]);
        assert!(
            map_completion_item(&crossing, &origin, &map, &origin, Position::new(0, 1)).is_err()
        );
        let mut backwards = base;
        backwards["textEdit"] = json!({"insert":{"start":{"line":0,"character":0},"end":{"line":0,"character":3}},"replace":{"start":{"line":0,"character":0},"end":{"line":0,"character":2}},"newText":"replacement"});
        assert!(
            map_completion_item(&backwards, &origin, &map, &origin, Position::new(0, 1)).is_err()
        );
    }
}
fn completion_snippet_data(
    item: &Value,
    origin: &Url,
    position: Position,
    map: &PositionMap,
) -> Result<Option<Value>> {
    if item.get("insertTextFormat").and_then(Value::as_u64) != Some(2) {
        return Ok(None);
    }
    let syntax = item
        .get("textEdit")
        .and_then(|edit| edit.get("newText"))
        .or_else(|| item.get("insertText"))
        .or_else(|| item.get("label"))
        .and_then(Value::as_str)
        .ok_or_else(|| completion_error("Missing snippet insertion text"))?;
    let mut variables: std::collections::BTreeMap<String, Option<String>> = [
        "TM_SELECTED_TEXT",
        "TM_CURRENT_LINE",
        "TM_CURRENT_WORD",
        "TM_LINE_INDEX",
        "TM_LINE_NUMBER",
        "TM_FILENAME",
        "TM_FILENAME_BASE",
        "TM_DIRECTORY",
        "TM_FILEPATH",
        "RELATIVE_FILEPATH",
        "WORKSPACE_NAME",
        "WORKSPACE_FOLDER",
        "CLIPBOARD",
        "CURRENT_YEAR",
        "CURRENT_YEAR_SHORT",
        "CURRENT_MONTH",
        "CURRENT_MONTH_NAME",
        "CURRENT_MONTH_NAME_SHORT",
        "CURRENT_DATE",
        "CURRENT_DAY_NAME",
        "CURRENT_DAY_NAME_SHORT",
        "CURRENT_HOUR",
        "CURRENT_MINUTE",
        "CURRENT_SECOND",
        "CURRENT_SECONDS_UNIX",
        "CURRENT_TIMEZONE_OFFSET",
        "RANDOM",
        "RANDOM_HEX",
        "UUID",
        "BLOCK_COMMENT_START",
        "BLOCK_COMMENT_END",
        "LINE_COMMENT",
    ]
    .into_iter()
    .map(|name| (name.into(), None))
    .collect();
    if let Ok(path) = origin.to_file_path() {
        for (name, value) in [
            ("TM_FILEPATH", Some(path.as_path())),
            ("TM_DIRECTORY", path.parent()),
        ] {
            if let Some(value) = value.and_then(|path| path.to_str()) {
                variables.insert(name.into(), Some(value.into()));
            }
        }
        for (name, value) in [
            ("TM_FILENAME", path.file_name()),
            ("TM_FILENAME_BASE", path.file_stem()),
        ] {
            if let Some(value) = value.and_then(|value| value.to_str()) {
                variables.insert(name.into(), Some(value.into()));
            }
        }
    }
    if let Some((_, range)) = map.source_edit_range(Range::new(position, position)) {
        variables.insert("TM_LINE_INDEX".into(), Some(range.start.line.to_string()));
        variables.insert(
            "TM_LINE_NUMBER".into(),
            Some((u64::from(range.start.line) + 1).to_string()),
        );
    }
    let plan = crate::snippet::expand_snippet(syntax, &variables).map_err(completion_error)?;
    Ok(Some(
        json!({"version":1,"syntax":syntax,"text":plan.text,"fields":plan.fields.into_iter().map(|field|json!({"index":field.index,"from":field.from,"to":field.to,"choices":field.choices})).collect::<Vec<_>>()}),
    ))
}
#[cfg(test)]
mod snippet_wire_tests {
    use super::*;
    #[test]
    fn resolved_snippet_metadata_uses_authored_coordinates_and_never_provider_metadata() {
        let item = json!({"label":"function","insertTextFormat":2,"insertText":"${1:😀 value} = $1; ${TM_FILENAME} ${TM_LINE_NUMBER}$0","data":{"hickory_snippet":{"text":"forged"}}});
        let plan = completion_snippet_data(
            &item,
            &Url::parse("file:///work/a%20file.py").unwrap(),
            Position::new(2, 1),
            &PositionMap::identity(4),
        )
        .unwrap()
        .unwrap();
        assert_eq!(plan["text"], "😀 value = 😀 value; a file.py 3");
        assert_eq!(plan["fields"][0]["to"], 8);
        assert_eq!(plan["syntax"], item["insertText"]);
        let unsupported =
            json!({"label":"unsupported","insertTextFormat":2,"insertText":"${1/x/y/}"});
        assert!(
            completion_snippet_data(
                &unsupported,
                &Url::parse("file:///a.py").unwrap(),
                Position::new(0, 0),
                &PositionMap::identity(4)
            )
            .is_err()
        );
    }
}
