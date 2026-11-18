use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::Path,
};
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    from: usize,
    to: usize,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    version: u32,
    path: String,
    source: String,
    selection: Selection,
    #[serde(default)]
    variable_context: Option<Value>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanRequest {
    version: u32,
    path: String,
    source: String,
    selection: Selection,
    catalog_fingerprint: String,
    template_id: String,
    #[serde(default)]
    variable_context: Option<Value>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Template {
    id: String,
    label: String,
    description: String,
    language: String,
    syntax: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    version: u32,
    templates: Vec<Template>,
}
fn hash(bytes: impl AsRef<[u8]>) -> String {
    hex::encode(Sha256::digest(bytes.as_ref()))
}
fn boundary(source: &str, offset: usize) -> Result<usize, String> {
    let mut utf16 = 0;
    for (byte, c) in source.char_indices() {
        if utf16 == offset {
            if source[..byte].ends_with('\r') && source[byte..].starts_with('\n') {
                return Err("Selection splits a CRLF line ending".into());
            }
            return Ok(byte);
        }
        utf16 += c.len_utf16();
    }
    if utf16 == offset {
        Ok(source.len())
    } else {
        Err("Selection is outside source or splits a Unicode scalar".into())
    }
}
fn validate(context: &Context) -> Result<(usize, usize), String> {
    if !matches!(context.version, 1 | 2) {
        return Err("Unsupported template protocol. Update app and engine together.".into());
    }
    if context.version == 1 && context.variable_context.is_some() {
        return Err("Template protocol 1 cannot carry a variable snapshot".into());
    }
    if context.version == 2 {
        variable_snapshot(context)?;
    }
    crate::launch_profiles::inventory::validate_workspace_path(&context.path)?;
    if context.source.len() > 8 * 1024 * 1024 {
        return Err("Template source exceeds 8 MiB".into());
    }
    if context.selection.from > context.selection.to {
        return Err("Template selection is reversed".into());
    }
    Ok((
        boundary(&context.source, context.selection.from)?,
        boundary(&context.source, context.selection.to)?,
    ))
}
fn builtin() -> Vec<Template> {
    [
        (
            "hickory.file",
            "File block",
            "Construct a file from authored source",
            "markdown",
            "<h:file xmlns:h=\"http://www.hickorydocs.com/1.0\" path=\"${1:path}\">\n${2:content}\n</h:file>$0",
        ),
        (
            "hickory.copy",
            "Named mechanism",
            "Declare a reusable implementation fragment",
            "markdown",
            "<h:copy xmlns:h=\"http://www.hickorydocs.com/1.0\" id=\"${1:name}\">\n${2:implementation}\n</h:copy>$0",
        ),
        (
            "python.function",
            "Function",
            "Function with linked name and body fields",
            "python",
            "def ${1:name}(${2:args}):\n    ${3:pass}\n$0",
        ),
        (
            "python.test",
            "Test function",
            "A pytest declaration; execution requires a test provider",
            "python",
            "def test_${1:behavior}():\n    ${2:assert True}\n$0",
        ),
        (
            "rust.function",
            "Function",
            "Rust function skeleton",
            "rust",
            "fn ${1:name}(${2}) {\n    ${3:todo!()}\n}\n$0",
        ),
        (
            "typescript.function",
            "Function",
            "TypeScript function skeleton",
            "typescript",
            "function ${1:name}(${2}) {\n    ${3}\n}\n$0",
        ),
        (
            "javascript.function",
            "Function",
            "JavaScript function skeleton",
            "javascript",
            "function ${1:name}(${2}) {\n    ${3}\n}\n$0",
        ),
        (
            "go.function",
            "Function",
            "Go function skeleton",
            "go",
            "func ${1:name}(${2}) {\n\t${3}\n}\n$0",
        ),
        (
            "csharp.method",
            "Method",
            "C# method skeleton",
            "csharp",
            "${1:public} ${2:void} ${3:Method}(${4})\n{\n    ${5}\n}\n$0",
        ),
    ]
    .into_iter()
    .map(|(id, label, description, language, syntax)| Template {
        id: id.into(),
        label: label.into(),
        description: description.into(),
        language: language.into(),
        syntax: syntax.into(),
    })
    .collect()
}
fn load(root: &Path) -> Result<Vec<(Template, &'static str)>, String> {
    let mut entries: Vec<_> = builtin()
        .into_iter()
        .map(|entry| (entry, "builtin"))
        .collect();
    let path = root.join(".hickory/templates.json");
    match std::fs::symlink_metadata(&path) {
        Ok(_) => {
            let canonical = path
                .canonicalize()
                .map_err(|e| format!("Cannot resolve template catalog: {e}"))?;
            if !canonical.starts_with(root) {
                return Err("Template catalog escapes the workspace".into());
            }
            let file = std::fs::File::open(canonical)
                .map_err(|e| format!("Cannot read template catalog: {e}"))?;
            let mut bytes = Vec::new();
            file.take(1_048_577)
                .read_to_end(&mut bytes)
                .map_err(|e| e.to_string())?;
            if bytes.len() > 1_048_576 {
                return Err("Template catalog exceeds 1 MiB".into());
            }
            let manifest: Manifest = serde_json::from_slice(&bytes)
                .map_err(|e| format!("Invalid .hickory/templates.json: {e}"))?;
            if manifest.version != 1 || manifest.templates.len() > 256 {
                return Err("Unsupported or oversized template catalog".into());
            }
            entries.extend(
                manifest
                    .templates
                    .into_iter()
                    .map(|entry| (entry, "workspace")),
            );
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("Cannot inspect template catalog: {error}")),
    }
    let mut ids = BTreeSet::new();
    for (entry, _) in &entries {
        if entry.id.is_empty()
            || entry.id.len() > 128
            || !entry
                .id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
            || !ids.insert(entry.id.clone())
        {
            return Err("Template IDs must be unique bounded ASCII names".into());
        }
        if entry.label.is_empty()
            || entry.label.len() > 256
            || entry.description.len() > 2048
            || entry.language.is_empty()
            || entry.language.len() > 64
            || entry.syntax.len() > 262_144
        {
            return Err(format!("Template {} exceeds catalog limits", entry.id));
        }
    }
    Ok(entries)
}
fn variables(
    root: &Path,
    context: &Context,
    from: usize,
    to: usize,
) -> BTreeMap<String, Option<String>> {
    let path = root.join(&context.path);
    let line_start = context.source[..from].rfind('\n').map_or(0, |i| i + 1);
    let line_end = context.source[from..]
        .find('\n')
        .map_or(context.source.len(), |i| from + i);
    let line = context.source[..from]
        .bytes()
        .filter(|b| *b == b'\n')
        .count();
    [
        (
            "TM_SELECTED_TEXT",
            Some(context.source[from..to].to_string()),
        ),
        (
            "TM_CURRENT_LINE",
            Some(
                context.source[line_start..line_end]
                    .trim_end_matches('\r')
                    .to_string(),
            ),
        ),
        ("TM_LINE_INDEX", Some(line.to_string())),
        ("TM_LINE_NUMBER", Some((line + 1).to_string())),
        (
            "TM_FILENAME",
            path.file_name()
                .and_then(|p| p.to_str())
                .map(str::to_string),
        ),
        (
            "TM_FILENAME_BASE",
            path.file_stem()
                .and_then(|p| p.to_str())
                .map(str::to_string),
        ),
        (
            "TM_DIRECTORY",
            path.parent().and_then(|p| p.to_str()).map(str::to_string),
        ),
        ("TM_FILEPATH", path.to_str().map(str::to_string)),
        ("RELATIVE_FILEPATH", Some(context.path.clone())),
        (
            "WORKSPACE_NAME",
            root.file_name()
                .and_then(|p| p.to_str())
                .map(str::to_string),
        ),
        ("WORKSPACE_FOLDER", root.to_str().map(str::to_string)),
    ]
    .into_iter()
    .map(|(key, value)| (key.into(), value))
    .collect()
}
fn contextual_syntax(
    template: &Template,
    context: &Context,
    from: usize,
) -> Result<String, String> {
    let start = context.source[..from].rfind('\n').map_or(0, |at| at + 1);
    let prefix = &context.source[start..from];
    let indent = if prefix.bytes().all(|byte| matches!(byte, b' ' | b'\t')) {
        prefix
    } else {
        ""
    };
    // Adjust literal template lines; a selected-text variable keeps its own
    // captured bytes instead of being independently reformatted.
    let extra = template
        .syntax
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        .checked_mul(indent.len())
        .and_then(|bytes| bytes.checked_add(template.syntax.len()));
    if !extra.is_some_and(|bytes| bytes <= 262_144) {
        return Err("Template indentation exceeds the shared snippet input limit".into());
    }
    Ok(template.syntax.replace('\n', &format!("\n{indent}")))
}
fn variable_snapshot(
    context: &Context,
) -> Result<hick_lsp::snippet_context::SnippetVariableSnapshot, String> {
    let raw = context
        .variable_context
        .as_ref()
        .ok_or("Template protocol 2 requires a captured variable context")?;
    let object = raw
        .as_object()
        .ok_or("Malformed template variable snapshot")?;
    if object.len() != 3
        || !["clock", "random_hex", "clipboard"]
            .iter()
            .all(|key| object.contains_key(*key))
    {
        return Err("Unsupported template variable snapshot fields".into());
    }
    hick_lsp::snippet_context::SnippetVariableSnapshot::read(
        &raw["clock"],
        &raw["random_hex"],
        &raw["clipboard"],
    )
}
fn template_variables(
    root: &Path,
    context: &Context,
    syntax: &str,
    language: &str,
    from: usize,
    to: usize,
) -> Result<BTreeMap<String, Option<String>>, String> {
    let snapshot = variable_snapshot(context)?;
    let requested = hick_lsp::snippet::requested_variables(syntax)?;
    let path = root.join(&context.path);
    let origin = hick_lsp::snippet_context::SnippetVariableContext {
        source: &context.source,
        from,
        to,
        path: &path,
        workspace: Some(root),
        language: Some(language),
    };
    let presence = hick_lsp::snippet_context::variable_presence(&origin, &snapshot, &requested)?;
    let requested = hick_lsp::snippet::effective_requested_variables(syntax, &presence)?;
    hick_lsp::snippet_context::variables(&origin, &snapshot, &requested)
}
fn expanded_template(
    root: &Path,
    context: &Context,
    syntax: &str,
    language: &str,
    from: usize,
    to: usize,
) -> Result<hick_lsp::snippet::SnippetPlan, String> {
    if context.version == 1 {
        return hick_lsp::snippet::expand_snippet(syntax, &variables(root, context, from, to));
    }
    let values = template_variables(root, context, syntax, language, from, to)?;
    hick_lsp::snippet::expand_snippet_v2(syntax, &values)
}
fn prepare(
    root: &Path,
    context: &Context,
    selected: Option<(&str, &str)>,
) -> Result<Value, String> {
    let (from, to) = validate(context)?;
    let root = root
        .canonicalize()
        .map_err(|e| format!("Workspace unavailable: {e}"))?;
    // Paths name source drafts, never instructions or process arguments.
    let language = hick_plugins::routing::language_id(&context.path).unwrap_or("unknown");
    let entries = load(&root)?;
    let fingerprint =
        hash(serde_json::to_vec(&(1, language, &entries)).map_err(|e| e.to_string())?);
    let mut value = json!({"version":1,"template_protocol":context.version,"path":context.path,"selection":context.selection,"source_hash":hash(context.source.as_bytes()),"workspace_id":hash(root.as_os_str().as_encoded_bytes()),"language":language,"catalog_fingerprint":fingerprint,"notices":["Templates edit the retained source draft only. They do not run commands, save files or establish semantic correctness.","Finish composition and choose one selection before applying."]});
    if context.version == 2 {
        value["variable_context"] = json!(context.variable_context);
        value["source_files_modified"] = json!(false);
        value["notices"] = json!([
            "Templates edit the retained source draft only. They do not run commands, save files or establish semantic correctness.",
            "Finish composition and choose one selection before applying."
        ]);
    }
    if let Some((id, expected)) = selected {
        if expected != fingerprint {
            return Err("Template catalog changed. Refresh and review the template again.".into());
        }
        let (entry, origin) = entries
            .iter()
            .find(|(entry, _)| entry.id == id && entry.language == language)
            .ok_or_else(|| {
                "Template is unavailable for this source language. Refresh the catalog.".to_string()
            })?;
        let syntax = contextual_syntax(entry, context, from)?;
        let plan = expanded_template(&root, context, &syntax, language, from, to)?;
        value["available"] = json!(true);
        value["template_id"] = json!(id);
        value["origin"] = json!(origin);
        value["replacement"] = json!(context.selection);
        if context.version == 2 {
            let snapshot = variable_snapshot(context)?;
            let values = template_variables(&root, context, &syntax, language, from, to)?;
            let requested = hick_lsp::snippet::effective_requested_variables(&syntax, &values)?;
            if requested.contains("CLIPBOARD") && snapshot.clipboard.is_none() {
                value["available"] = json!(false);
                value["context_request"] = json!({"version":1,"variables":["CLIPBOARD"]});
                value["reason"] = json!("This template needs a clipboard reading before preview.");
                return Ok(value);
            }
            if requested.contains("CLIPBOARD") && snapshot.clipboard == Some(None) {
                value["context_unavailable"] = json!(["CLIPBOARD"]);
                value["notices"].as_array_mut().unwrap().push(json!(
                    "Clipboard unavailable; its fallback or empty text was used."
                ));
            }
        }
        value["snippet"] = plan.metadata(&syntax);
    } else {
        let entries:Vec<_>=entries.iter().filter(|(entry,_)|entry.language==language).map(|(entry,origin)|{let result=contextual_syntax(entry,context,from).and_then(|syntax|expanded_template(&root,context,&syntax,language,from,to));json!({"id":entry.id,"label":entry.label,"description":entry.description,"language":entry.language,"origin":origin,"available":result.is_ok(),"reason":result.err()})}).collect();
        value["available"] = json!(entries.iter().any(|entry| entry["available"] == true));
        value["entries"] = json!(entries);
    }
    Ok(value)
}
fn admitted(
    state: &LocalState,
    context: &Context,
    selected: Option<(&str, &str)>,
) -> Result<Value, String> {
    validate(context)?;
    let root = state
        .index
        .root()
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let path = root.join(&context.path);
    // A new draft may lack a leaf, but its nearest existing ancestor must
    // establish workspace confinement. An inaccessible or dangling identity
    // never gains permission merely because a read failed.
    let mut ancestor = path.as_path();
    loop {
        match std::fs::symlink_metadata(ancestor) {
            Ok(_) => {
                let resolved = ancestor
                    .canonicalize()
                    .map_err(|error| format!("Cannot establish source identity: {error}"))?;
                if !resolved.starts_with(&root) {
                    return Err("Source path escapes the workspace".into());
                }
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                ancestor = ancestor
                    .parent()
                    .ok_or("Source has no workspace ancestor")?;
            }
            Err(error) => return Err(format!("Cannot inspect source identity: {error}")),
        }
    }
    let claims = super::api::generated_outputs(&root, &state.index);
    if let Some(owner) = super::api::generated_by(&claims, &context.path) {
        return Err(format!(
            "This source is generated by {owner}. Insert a template in its authored construction instead."
        ));
    }
    for (generated, owner) in &claims {
        if same_file::is_same_file(&path, root.join(generated)).unwrap_or(false) {
            return Err(format!(
                "This source aliases an output generated by {owner}. Insert a template in its authored construction instead."
            ));
        }
    }
    prepare(&root, context, selected)
}
pub async fn catalog(
    State(state): State<LocalState>,
    Json(context): Json<Context>,
) -> ApiResult<Json<Value>> {
    tokio::task::spawn_blocking(move || admitted(&state, &context, None))
        .await
        .map_err(|e| ApiError::internal(format!("Template catalog failed: {e}")))?
        .map(Json)
        .map_err(ApiError::unprocessable)
}
pub async fn plan(
    State(state): State<LocalState>,
    Json(request): Json<PlanRequest>,
) -> ApiResult<Json<Value>> {
    tokio::task::spawn_blocking(move || {
        let context = Context {
            version: request.version,
            path: request.path,
            source: request.source,
            selection: request.selection,
            variable_context: request.variable_context,
        };
        admitted(
            &state,
            &context,
            Some((&request.template_id, &request.catalog_fingerprint)),
        )
    })
    .await
    .map_err(|e| ApiError::internal(format!("Template planning failed: {e}")))?
    .map(Json)
    .map_err(ApiError::unprocessable)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn context() -> Context {
        Context {
            version: 1,
            variable_context: None,
            path: "part.py".into(),
            source: "😀 old\r\n".into(),
            selection: Selection { from: 3, to: 6 },
        }
    }
    #[test]
    fn catalog_and_plan_capture_exact_unicode_selection_without_writes() {
        let root = tempfile::tempdir().unwrap();
        let context = context();
        let catalog = prepare(root.path(), &context, None).unwrap();
        assert_eq!(catalog["language"], "python");
        assert_eq!(catalog["source_hash"], hash(context.source.as_bytes()));
        let plan = prepare(
            root.path(),
            &context,
            Some((
                "python.function",
                catalog["catalog_fingerprint"].as_str().unwrap(),
            )),
        )
        .unwrap();
        assert_eq!(plan["replacement"], json!({"from":3,"to":6}));
        assert_eq!(plan["snippet"]["text"], "def name(args):\n    pass\n");
        assert!(
            plan["snippet"]["fields"]
                .as_array()
                .unwrap()
                .iter()
                .any(|field| field["index"] == 0)
        );
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    }
    #[test]
    fn protocol_two_context_and_derived_fields_keep_one_snapshot_and_demand_only_clipboard() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join(".hickory")).unwrap();
        let syntax = r"${TM_FILENAME}|${CURRENT_YEAR}|${RANDOM_HEX}|${1:${TM_SELECTED_TEXT}}|$1|${1/(.*)/${1:/upcase}/}$0";
        let manifest = json!({"version":1,"templates":[{"id":"context.vars","label":"Variables","description":"","language":"python","syntax":syntax},{"id":"context.clip","label":"Clipboard","description":"","language":"python","syntax":"${CLIPBOARD:fallback}$0"},{"id":"context.unused","label":"Unused","description":"","language":"python","syntax":"${TM_SELECTED_TEXT:${CLIPBOARD}}$0"}]});
        std::fs::write(
            root.path().join(".hickory/templates.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        let mut context = context();
        context.version = 2;
        context.variable_context = Some(
            json!({"clock":{"unix_millis":1709228107000i64,"utc_offset_minutes":330},"random_hex":"000102030405060708090a0b0c0d0e0f","clipboard":null}),
        );
        let catalog = prepare(root.path(), &context, None).unwrap();
        let fingerprint = catalog["catalog_fingerprint"].as_str().unwrap();
        let plan = prepare(root.path(), &context, Some(("context.vars", fingerprint))).unwrap();
        assert_eq!(plan["snippet"]["text"], "part.py|2024|000102|old|old|OLD");
        assert_eq!(plan["snippet"]["version"], 2);
        assert_eq!(plan["source_files_modified"], false);
        assert_eq!(
            plan["variable_context"],
            context.variable_context.clone().unwrap()
        );
        let unused = prepare(root.path(), &context, Some(("context.unused", fingerprint))).unwrap();
        assert!(unused.get("context_request").is_none());
        assert_eq!(unused["snippet"]["text"], "old");
        let demand = prepare(root.path(), &context, Some(("context.clip", fingerprint))).unwrap();
        assert_eq!(demand["available"], false);
        assert_eq!(
            demand["context_request"],
            json!({"version":1,"variables":["CLIPBOARD"]})
        );
        context.variable_context.as_mut().unwrap()["clipboard"] =
            json!({"available":false,"text":null});
        let denied = prepare(root.path(), &context, Some(("context.clip", fingerprint))).unwrap();
        assert_eq!(denied["snippet"]["text"], "fallback");
        assert_eq!(denied["context_unavailable"], json!(["CLIPBOARD"]));
        assert_eq!(
            denied["catalog_fingerprint"],
            catalog["catalog_fingerprint"]
        );
    }
    #[test]
    fn unused_known_defaults_do_not_allocate_excessive_context_values() {
        let root = tempfile::tempdir().unwrap();
        let mut context = context();
        context.version = 2;
        context.variable_context = Some(json!({"clock":null,"random_hex":null,"clipboard":null}));
        context.source = "x".repeat(1_048_577);
        context.selection = Selection { from: 0, to: 0 };
        let plan = expanded_template(
            root.path(),
            &context,
            "${TM_FILENAME:${TM_CURRENT_LINE}}$0",
            "python",
            0,
            0,
        )
        .unwrap();
        assert_eq!(plan.text, "part.py");
    }
    #[test]
    fn literal_template_lines_follow_the_existing_indent() {
        let root = tempfile::tempdir().unwrap();
        let context = Context {
            version: 1,
            variable_context: None,
            path: "part.py".into(),
            source: "    ".into(),
            selection: Selection { from: 4, to: 4 },
        };
        let catalog = prepare(root.path(), &context, None).unwrap();
        let plan = prepare(
            root.path(),
            &context,
            Some((
                "python.function",
                catalog["catalog_fingerprint"].as_str().unwrap(),
            )),
        )
        .unwrap();
        assert_eq!(
            plan["snippet"]["text"],
            "def name(args):\n        pass\n    "
        );
        assert_eq!(
            plan["snippet"]["syntax"],
            "def ${1:name}(${2:args}):\n        ${3:pass}\n    $0"
        );
        let large = Context {
            source: " ".repeat(262_144),
            selection: Selection {
                from: 262_144,
                to: 262_144,
            },
            ..context.clone()
        };
        let unavailable = prepare(root.path(), &large, None).unwrap();
        assert_eq!(unavailable["available"], false);
        assert!(
            prepare(
                root.path(),
                &large,
                Some((
                    "python.function",
                    unavailable["catalog_fingerprint"].as_str().unwrap()
                ))
            )
            .unwrap_err()
            .contains("input limit")
        );
    }
    #[test]
    fn workspace_data_templates_recheck_catalog_and_reject_conflicting_ids() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join(".hickory")).unwrap();
        let manifest = root.path().join(".hickory/templates.json");
        let context = context();
        let before = prepare(root.path(), &context, None).unwrap();
        std::fs::write(&manifest,r#"{"version":1,"templates":[{"id":"project.function","label":"Project","description":"selected text","language":"python","syntax":"${TM_SELECTED_TEXT} ${TM_FILENAME} $0"}]}"#).unwrap();
        assert!(
            prepare(
                root.path(),
                &context,
                Some((
                    "python.function",
                    before["catalog_fingerprint"].as_str().unwrap()
                ))
            )
            .unwrap_err()
            .contains("catalog changed")
        );
        let catalog = prepare(root.path(), &context, None).unwrap();
        let plan = prepare(
            root.path(),
            &context,
            Some((
                "project.function",
                catalog["catalog_fingerprint"].as_str().unwrap(),
            )),
        )
        .unwrap();
        assert_eq!(plan["origin"], "workspace");
        assert_eq!(plan["snippet"]["text"], "old part.py ");
        std::fs::write(manifest,r#"{"version":1,"templates":[{"id":"python.function","label":"Shadow","description":"","language":"python","syntax":"bad"}]}"#).unwrap();
        assert!(
            prepare(root.path(), &context, None)
                .unwrap_err()
                .contains("unique")
        );
    }
    #[test]
    fn invalid_positions_protocols_and_manifest_shapes_never_return_partial_plans() {
        let root = tempfile::tempdir().unwrap();
        for selection in [
            Selection { from: 1, to: 2 },
            Selection { from: 7, to: 7 },
            Selection { from: 9, to: 9 },
            Selection { from: 5, to: 2 },
        ] {
            let mut context = context();
            context.selection = selection;
            assert!(prepare(root.path(), &context, None).is_err());
        }
        let mut context = context();
        context.version = 2;
        assert!(prepare(root.path(), &context, None).is_err());
        assert!(serde_json::from_value::<PlanRequest>(json!({"version":1,"path":"a.py","source":"","selection":{"from":0,"to":0},"template_id":"x","catalog_fingerprint":"x","command":"arbitrary"})).is_err());
    }
}
#[cfg(test)]
mod refusal_tests {
    use super::*;
    #[test]
    fn unsupported_syntax_is_visible_without_a_partial_insertion() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join(".hickory")).unwrap();
        std::fs::write(root.path().join(".hickory/templates.json"),r#"{"version":1,"templates":[{"id":"project.transform","label":"Transform","description":"Requires unsupported syntax","language":"python","syntax":"${1/name/other/}"}]}"#).unwrap();
        let context = Context {
            version: 1,
            variable_context: None,
            path: "a.py".into(),
            source: String::new(),
            selection: Selection { from: 0, to: 0 },
        };
        let catalog = prepare(root.path(), &context, None).unwrap();
        let entry = catalog["entries"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["id"] == "project.transform")
            .unwrap();
        assert_eq!(entry["available"], false);
        assert!(entry["reason"].as_str().unwrap().contains("transforms"));
        assert!(
            prepare(
                root.path(),
                &context,
                Some((
                    "project.transform",
                    catalog["catalog_fingerprint"].as_str().unwrap()
                ))
            )
            .is_err()
        );
    }
    #[cfg(unix)]
    #[test]
    fn catalog_symlinks_cannot_borrow_a_foreign_workspace() {
        let root = tempfile::tempdir().unwrap();
        let foreign = tempfile::tempdir().unwrap();
        std::fs::write(foreign.path().join("templates.json"), "not even read").unwrap();
        std::fs::create_dir(root.path().join(".hickory")).unwrap();
        std::os::unix::fs::symlink(
            foreign.path().join("templates.json"),
            root.path().join(".hickory/templates.json"),
        )
        .unwrap();
        let canonical = root.path().canonicalize().unwrap();
        assert!(load(&canonical).err().unwrap().contains("escapes"));
    }
}
