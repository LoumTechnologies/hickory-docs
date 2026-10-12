use crate::{backend::uri_key, position_map::PositionMap};
use std::collections::HashMap;
use tower_lsp::lsp_types::Url;

pub(crate) fn translate_locations(
    value: serde_json::Value,
    index: &HashMap<String, (Url, PositionMap)>,
) -> serde_json::Value {
    use serde_json::Value;

    fn translate_range(range: &Value, map: &PositionMap) -> Option<(Option<String>, Value)> {
        let sl = range.pointer("/start/line")?.as_u64()? as u32;
        let sc = range.pointer("/start/character")?.as_u64()? as u32;
        let el = range.pointer("/end/line")?.as_u64()? as u32;
        let ec = range.pointer("/end/character")?.as_u64()? as u32;
        let (start, sl, sc) = map.source_document(sl, sc)?;
        let (end, el, ec) = map.source_document(el, ec)?;
        if start != end {
            return None;
        }
        Some((
            start,
            serde_json::json!({
                "start": { "line": sl, "character": sc },
                "end": { "line": el, "character": ec },
            }),
        ))
    }

    fn walk(value: &mut Value, index: &HashMap<String, (Url, PositionMap)>) {
        match value {
            Value::Array(items) => {
                for item in items {
                    walk(item, index);
                }
            }
            Value::Object(obj) => {
                // Location: { uri, range }
                let plain = obj
                    .get("uri")
                    .and_then(|u| u.as_str())
                    .and_then(|u| index.get(&uri_key(u)))
                    .and_then(|(hick, map)| {
                        let (document, range) = translate_range(obj.get("range")?, map)?;
                        let uri = document
                            .map(|d| {
                                let mut uri = hick.join(&d).ok()?;
                                uri.set_query(hick.query());
                                Some(uri)
                            })
                            .unwrap_or_else(|| Some(hick.clone()))?;
                        Some((uri.to_string(), range))
                    });
                if let Some((uri, range)) = plain {
                    obj.insert("uri".into(), Value::String(uri));
                    obj.insert("range".into(), range);
                    return;
                }
                // LocationLink: { targetUri, targetRange, targetSelectionRange }
                let link = obj
                    .get("targetUri")
                    .and_then(|u| u.as_str())
                    .and_then(|u| index.get(&uri_key(u)))
                    .and_then(|(hick, map)| {
                        let (document, range) = translate_range(obj.get("targetRange")?, map)?;
                        let uri = document
                            .map(|d| {
                                let mut uri = hick.join(&d).ok()?;
                                uri.set_query(hick.query());
                                Some(uri)
                            })
                            .unwrap_or_else(|| Some(hick.clone()))?;
                        let sel = obj
                            .get("targetSelectionRange")
                            .and_then(|r| translate_range(r, map));
                        Some((uri.to_string(), range, sel.map(|(_, r)| r)))
                    });
                if let Some((uri, range, sel)) = link {
                    obj.insert("targetUri".into(), Value::String(uri));
                    obj.insert("targetRange".into(), range);
                    if let Some(sel) = sel {
                        obj.insert("targetSelectionRange".into(), sel);
                    }
                    return;
                }
                for (_, v) in obj.iter_mut() {
                    walk(v, index);
                }
            }
            _ => {}
        }
    }

    let mut value = value;
    walk(&mut value, index);
    value
}
