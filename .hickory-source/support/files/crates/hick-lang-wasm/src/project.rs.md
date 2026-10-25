<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hick-lang-wasm/src/project.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `3c717c2cc36487d56bf2fa1872fa616e17744e668815bda1d3aa6599571c53b1`.

<book:entry path="crates/hick-lang-wasm/src/project.rs">
use wasm_bindgen::prelude::*;

pub(crate) fn read_tree(input_json: &str) -> Result<hick_lang::ProjectTree, String> {
    let input: Vec<serde_json::Value> =
        serde_json::from_str(input_json).map_err(|e| format!("input project: {e}"))?;
    let mut tree = hick_lang::ProjectTree::default();
    for item in input {
        let path = item["path"].as_str().ok_or("entry requires path")?;
        if tree.get(path).is_some() {
            return Err(format!("duplicate input path: {path}"));
        }
        let bytes: Vec<u8> =
            serde_json::from_value(item["bytes"].clone()).map_err(|e| e.to_string())?;
        let object = item.as_object().ok_or("project entry must be an object")?;
        if object
            .keys()
            .any(|key| !["path", "bytes", "kind", "executable", "origins"].contains(&key.as_str()))
        {
            return Err(format!("unsupported input entry metadata for {path}"));
        }
        let kind_name = match object.get("kind") {
            None => "file",
            Some(value) => value.as_str().ok_or("input kind must be a string")?,
        };
        let kind = match kind_name {
            "file" => hick_lang::EntryKind::File,
            "symlink" => hick_lang::EntryKind::Symlink,
            other => return Err(format!("unsupported input kind: {other}")),
        };
        let executable = match object.get("executable") {
            None => false,
            Some(value) => value
                .as_bool()
                .ok_or("input executable metadata must be a boolean")?,
        };
        let origins = match object.get("origins") {
            None => vec![],
            Some(value) => {
                serde_json::from_value(value.clone()).map_err(|e| format!("input origins: {e}"))?
            }
        };
        tree.insert(
            path,
            hick_lang::ProjectEntry {
                bytes: bytes.into(),
                kind,
                executable,
                origins,
            },
        )?;
    }
    Ok(tree)
}

pub(crate) fn serialize(tree: &hick_lang::ProjectTree) -> Vec<serde_json::Value> {
    tree.entries().iter().map(|(path,e)|serde_json::json!({"path":path,"bytes":e.bytes.as_ref(),"kind":match e.kind {hick_lang::EntryKind::File=>"file",hick_lang::EntryKind::Symlink=>"symlink"},"executable":e.executable,"origins":e.origins})).collect()
}

/// Exact shared patch regeneration; every downstream act is checked before returning source.
#[wasm_bindgen]
pub fn regenerate_project_patch(
    entry: &str,
    sources_json: &str,
    input_json: &str,
    id: &str,
    new_json: &str,
) -> String {
    let result = (|| {
        let sources =
            serde_json::from_str(sources_json).map_err(|e| format!("source closure: {e}"))?;
        hick_lang::regenerate_patch(
            entry,
            &sources,
            &read_tree(input_json)?,
            id,
            &read_tree(new_json)?,
        )
    })();
    match result {
        Ok(sources) => serde_json::json!({"version":1,"sources":sources,"error":null}).to_string(),
        Err(error) => serde_json::json!({"version":1,"sources":null,"error":error}).to_string(),
    }
}

/// One deterministic transition renderer for all comparison surfaces.
#[wasm_bindgen]
pub fn project_transition(old_json: &str, new_json: &str) -> String {
    let result = (|| hick_lang::unified_transition(&read_tree(old_json)?, &read_tree(new_json)?))();
    match result {
        Ok(diff) => serde_json::json!({"version":1,"diff":diff,"error":null}).to_string(),
        Err(error) => serde_json::json!({"version":1,"diff":null,"error":error}).to_string(),
    }
}

#[wasm_bindgen]
pub fn reverse_project_edit(
    entry: &str,
    sources_json: &str,
    input_json: &str,
    endpoint_json: &str,
    edit_json: &str,
) -> String {
    let result = (|| {
        let sources =
            serde_json::from_str(sources_json).map_err(|e| format!("source closure: {e}"))?;
        let edit: serde_json::Value =
            serde_json::from_str(edit_json).map_err(|e| format!("edit: {e}"))?;
        let range: (usize, usize) =
            serde_json::from_value(edit["range"].clone()).map_err(|e| e.to_string())?;
        hick_lang::reverse_project_edit(
            entry,
            &sources,
            &read_tree(input_json)?,
            &read_tree(endpoint_json)?,
            edit["path"].as_str().ok_or("edit requires path")?,
            range,
            edit["replacement"]
                .as_str()
                .ok_or("edit requires replacement")?,
        )
    })();
    match result {
        Ok(sources) => serde_json::json!({"version":1,"sources":sources,"error":null}).to_string(),
        Err(error) => serde_json::json!({"version":1,"sources":null,"error":error}).to_string(),
    }
}

#[wasm_bindgen]
pub fn project_position(local_json: &str, endpoint_json: &str, path: &str, byte: usize) -> String {
    let result = (|| {
        hick_lang::project_byte_location(
            &read_tree(local_json)?,
            &read_tree(endpoint_json)?,
            path,
            byte,
        )
    })();
    match result {
        Ok(location) => {
            serde_json::json!({"version":1,"location":location,"error":null}).to_string()
        }
        Err(error) => serde_json::json!({"version":1,"location":null,"error":error}).to_string(),
    }
}

/// The same exact-source selection used by native document launches.
#[wasm_bindgen]
pub fn latest_debug_project(versions_json: &str, displayed_json: &str, path: &str) -> String {
    let result = (|| {
        let versions: Vec<serde_json::Value> =
            serde_json::from_str(versions_json).map_err(|e| format!("project versions: {e}"))?;
        let versions = versions
            .iter()
            .map(|v| read_tree(&v.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        hick_lang::latest_debug_project(&versions, &read_tree(displayed_json)?, path)
    })();
    match result {
        Ok(selection) => {
            serde_json::json!({"version":1,"selection":selection,"error":null}).to_string()
        }
        Err(error) => serde_json::json!({"version":1,"selection":null,"error":error}).to_string(),
    }
}
</book:entry>

</book:doc>
