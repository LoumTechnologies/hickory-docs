<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Plan reusable source templates without publishing edits

For maintainers connecting reusable generation to the existing snippet and coordinated draft mechanisms. The [frontend contract](../frontends/contract.md) requires a usable host path; a catalog or a pure plan is not that path by itself.

## Capture a source draft and exact UTF-16 selection

A template plan edits no file. Its versioned input retains a source draft and one ordered selection. Convert UTF-16 positions to UTF-8 only at scalar boundaries, and refuse positions inside CRLF. The host independently retains the actual editor and workspace lease; matching text alone never authorizes applying an old plan.

Production protocol 2 carries a client-captured variable snapshot and echoes it with every catalog, plan and demanded context response. Its clock and seed remain fixed through Preview and Apply. Clipboard access is requested only for the selected reachable declaration; denial is retained rather than replaced with server state. Protocol 1 remains a legacy pure-policy seam and does not enable computed transforms or supplied variable context. The catalog fingerprint identifies declaration content independently of the snapshot, so a demanded clipboard reading does not invalidate the same reviewed catalog.

<ui:copy id="source-template-inputs">
use super::{
    api::{ApiError, ApiResult},
    LocalState,
};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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
    #[serde(default)]
    reading_context: Option<TemplateReadingContext>,
    #[serde(default)]
    transform_program_versions: Option<Vec<u32>>,
    #[serde(default)]
    carets: Option<TemplateCarets>,
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
    #[serde(default)]
    reading_context: Option<TemplateReadingContext>,
    #[serde(default)]
    transform_program_versions: Option<Vec<u32>>,
    #[serde(default)]
    carets: Option<TemplateCarets>,
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
    template_programs(context)?;
    validate_template_carets(context)?;
    if context.version == 1 && context.reading_context.is_some() {
        return Err("Template protocol1 does not support captured construction contexts".into());
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
</ui:copy>

## Capture one complete ordered caret roster

A coordinated template never becomes independent row requests. Keep every authored UTF-16 interval, the primary row and explicit continuation mode with one source, entry proof and contributor lease. Initial overlapping or duplicate selections refuse; only a live guarded continuation may retain coincident empty logical slots. Index all requested scalar/CRLF boundaries once, without rescanning a large source per caret. A shared expansion budget follows variable presence, effective demand, cursor values and final rendering across the whole catalog or plan.

<ui:copy id="source-template-batch-values">
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateCarets {
    version: u32,
    selections: Vec<Selection>,
    primary: usize,
    #[serde(default)]
    continuing: bool,
}
fn validate_template_carets<'a>(
    context: &'a Context,
) -> Result<Option<&'a TemplateCarets>, String> {
    let Some(carets) = &context.carets else {
        return Ok(None);
    };
    if context.version != 2
        || carets.version != 1
        || !(2..=256).contains(&carets.selections.len())
        || carets.primary >= carets.selections.len()
        || carets.selections[carets.primary].from != context.selection.from
        || carets.selections[carets.primary].to != context.selection.to
    {
        return Err("The complete template selection roster or primary is invalid".into());
    }
    for selected in &carets.selections {
        if selected.from > selected.to {
            return Err("Template caret interval is reversed".into());
        }
    }
    let mut ordered: Vec<_> = carets.selections.iter().collect();
    ordered.sort_by_key(|selected| (selected.from, selected.to));
    for pair in ordered.windows(2) {
        let (before, after) = (pair[0], pair[1]);
        if before.to > after.from
            || before.from == after.from
                && before.to == after.to
                && (!carets.continuing || before.from != before.to)
        {
            return Err(
                "Overlapping template selections or duplicate initial carets are unsupported"
                    .into(),
            );
        }
    }
    Ok(Some(carets))
}
fn template_selection_offsets(context: &Context) -> Result<BTreeMap<usize, usize>, String> {
    if context.source.len() > 8 * 1024 * 1024 {
        return Err("Template source exceeds its eight MiB reading budget".into());
    }
    let carets = validate_template_carets(context)?;
    let selections = carets
        .map(|value| value.selections.as_slice())
        .unwrap_or(std::slice::from_ref(&context.selection));
    if selections.iter().any(|range| range.from > range.to) {
        return Err("Template selection is reversed".into());
    }
    let wanted: BTreeSet<_> = selections
        .iter()
        .flat_map(|range| [range.from, range.to])
        .collect();
    let mut wanted = wanted.into_iter().peekable();
    let mut offsets = BTreeMap::new();
    let mut units = 0usize;
    for (byte, ch) in context.source.char_indices() {
        if wanted.peek().is_some_and(|offset| *offset < units) {
            return Err("Template selection splits a Unicode scalar".into());
        }
        if wanted.peek() == Some(&units) {
            if context.source[..byte].ends_with('\r') && ch == '\n' {
                return Err("Template selection splits a CRLF line ending".into());
            }
            offsets.insert(units, byte);
            wanted.next();
        }
        units += ch.len_utf16();
    }
    if wanted.peek() == Some(&units) {
        offsets.insert(units, context.source.len());
        wanted.next();
    }
    if wanted.next().is_some() {
        return Err("Template selection is outside source or splits a Unicode scalar".into());
    }
    Ok(offsets)
}
fn bind_template_cursor(
    values: &mut BTreeMap<String, Option<String>>,
    requested: &BTreeSet<String>,
    cursor: usize,
    budget: &mut hick_lsp::snippet::SnippetExpansionBudget,
) -> Result<(), String> {
    if cursor >= 256 {
        return Err("Template cursor index exceeds the captured roster".into());
    }
    // The rebinding allocates its own bounded numeric literals after the
    // context reader's default values; charge those allocations as well.
    fn digits(number: usize) -> usize {
        if number < 10 {
            1
        } else if number < 100 {
            2
        } else {
            3
        }
    }
    for (name, number) in [("CURSOR_INDEX", cursor), ("CURSOR_NUMBER", cursor + 1)] {
        if requested.contains(name) {
            budget.charge_context_bytes(digits(number))?;
        }
    }
    hick_lsp::snippet_context::bind_cursor_variables(values, requested, cursor)
}
fn template_row_variables(
    root: &Path,
    context: &Context,
    syntax: &str,
    language: &str,
    from: usize,
    to: usize,
    cursor: usize,
    budget: &mut hick_lsp::snippet::SnippetExpansionBudget,
) -> Result<(BTreeMap<String, Option<String>>, bool), String> {
    let snapshot = variable_snapshot(context)?;
    let path = root.join(&context.path);
    let origin = hick_lsp::snippet_context::SnippetVariableContext {
        source: &context.source,
        from,
        to,
        path: &path,
        workspace: Some(root),
        language: Some(language),
    };
    let requested = hick_lsp::snippet::requested_variables_with_budget(syntax, budget)?;
    let mut presence = hick_lsp::snippet_context::variable_presence_with_budget(
        &origin, &snapshot, &requested, budget,
    )?;
    bind_template_cursor(&mut presence, &requested, cursor, budget)?;
    let effective =
        hick_lsp::snippet::effective_requested_variables_with_budget(syntax, &presence, budget)?;
    let mut values =
        hick_lsp::snippet_context::variables_with_budget(&origin, &snapshot, &effective, budget)?;
    bind_template_cursor(&mut values, &effective, cursor, budget)?;
    Ok((values, effective.contains("CLIPBOARD")))
}
struct TemplateRowExpansion {
    selection: Selection,
    syntax: String,
    values: BTreeMap<String, Option<String>>,
    clipboard: bool,
}
fn template_expansion_rows(
    root: &Path,
    context: &Context,
    entry: &Template,
    language: &str,
    carets: &TemplateCarets,
    graph: &PreparedTemplateGraph<'_>,
    budget: &mut hick_lsp::snippet::SnippetExpansionBudget,
) -> Result<Vec<TemplateRowExpansion>, String> {
    carets
        .selections
        .iter()
        .enumerate()
        .map(|(index, selection)| {
            let (from, to) = graph.byte_range(selection)?;
            let syntax = contextual_syntax(entry, context, from)?;
            let (values, clipboard) =
                template_row_variables(root, context, &syntax, language, from, to, index, budget)?;
            Ok(TemplateRowExpansion {
                selection: selection.clone(),
                syntax,
                values,
                clipboard,
            })
        })
        .collect()
}
fn materialize_template_rows(
    rows: &[TemplateRowExpansion],
    budget: &mut hick_lsp::snippet::SnippetExpansionBudget,
) -> Result<Vec<Value>, String> {
    rows.iter().map(|row| {
        let plan = hick_lsp::snippet::expand_snippet_v2_with_budget(&row.syntax, &row.values, budget)?;
        Ok(json!({"selection":row.selection,"replacement":row.selection,"snippet":plan.metadata(&row.syntax)}))
    }).collect()
}
</ui:copy>

## Keep reusable definitions as bounded data

Built-in templates work without a language-server connection. Workspace definitions live in `.hickory/templates.json` with `version: 1` and a `templates` list; every entry has `id`, `label`, `description`, `language` and `syntax`. IDs cannot shadow a built-in or another entry. Templates contain snippet syntax, never commands. A malformed catalog fails independently of readable source. Resolve its physical identity inside this workspace, bound its bytes and retain explicit unsupported-syntax reasons.

Capture one immutable compiler graph from all supplied contributor text, with confined saved-source fallback and explicit deletions. Keep source/namespace ownership separate from language: newly generated targets and aliases refuse even if the editor has not received newer ownership metadata. A positively bound canonical entry may be acquired once; missing or stale entry authority never guesses a filename. The prepared graph is reusable across an ordered caret batch without reloading its closure.

<ui:copy id="embedded-template-captured-graph">
#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateReadingContext {
    version: u32,
    #[serde(default)]
    project_entry: Option<String>,
    #[serde(default)]
    project_endpoint: Option<String>,
    #[serde(default)]
    sources: BTreeMap<String, String>,
    #[serde(default)]
    unaddressed_sources: BTreeMap<String, String>,
    #[serde(default)]
    missing_sources: BTreeSet<String>,
    #[serde(default)]
    entry_selection: Option<Value>,
}
struct CapturedTemplateLanguage {
    language: hick_lsp::source_context::SourceLanguageContext,
    binding: Value,
    needs_entry: bool,
}
struct PreparedTemplateGraph<'a> {
    context: &'a Context,
    reader: hick_lsp::source_context::PreparedSourceLanguageContext,
    explicit: Option<String>,
    contributes: bool,
    hashes: BTreeMap<String, String>,
    graph_fingerprint: String,
    needs_entry: bool,
    byte_offsets: BTreeMap<usize, usize>,
}
impl<'a> PreparedTemplateGraph<'a> {
    fn capture(root: &Path, context: &'a Context, scope: Option<&str>) -> Result<Self, String> {
        validate(context)?;
        let byte_offsets = template_selection_offsets(context)?;
        let mut supplied = BTreeMap::new();
        let mut missing = BTreeSet::new();
        let reading = context.reading_context.as_ref();
        if let Some(reading) = reading {
            if reading.version != 1 {
                return Err("Unsupported source-language reading context".into());
            }
            if reading
                .sources
                .len()
                .saturating_add(reading.unaddressed_sources.len())
                > 4096
                || reading
                    .sources
                    .values()
                    .map(String::len)
                    .sum::<usize>()
                    .saturating_add(
                        reading
                            .unaddressed_sources
                            .values()
                            .map(String::len)
                            .sum::<usize>(),
                    )
                    > 32 * 1024 * 1024
                || reading.missing_sources.len() > 4096
                || reading
                    .missing_sources
                    .iter()
                    .map(String::len)
                    .sum::<usize>()
                    > 1024 * 1024
            {
                return Err("Template contributor capture exceeds its interactive budget".into());
            }
            for text in reading.unaddressed_sources.values() {
                if text.len() > 8 * 1024 * 1024
                    || super::resource_inputs::contains_construction(text)?
                {
                    return Err("An unaddressed retained construction has no source namespace authority; preserve it and select its authored context".into());
                }
            }
            for (path, text) in &reading.sources {
                let normalized = super::source_tests::relative(Path::new(""), path)?;
                if normalized != *path
                    || text.len() > 8 * 1024 * 1024
                    || supplied.insert(normalized, text.clone()).is_some()
                {
                    return Err("Template source capture has ambiguous paths or exceeds8MiB".into());
                }
            }
            for path in &reading.missing_sources {
                let normalized = super::source_tests::relative(Path::new(""), path)?;
                if normalized != *path || supplied.contains_key(path) {
                    return Err("A template contributor is both supplied and deleted".into());
                }
                missing.insert(normalized);
            }
        }
        if supplied
            .get(&context.path)
            .is_some_and(|source| source != &context.source)
            || missing.contains(&context.path)
        {
            return Err("Original template source disagrees with the contributor capture".into());
        }
        supplied.insert(context.path.clone(), context.source.clone());
        let markdown = hick_plugins::routing::language_id(&context.path) == Some("markdown");
        let reader = hick_lsp::source_context::PreparedSourceLanguageContext::new(
            &context.path,
            &context.source,
            None,
        )?;
        let explicit = reading
            .and_then(|value| value.project_entry.clone())
            .or_else(|| reader.declares_project().then(|| context.path.clone()));
        if reading
            .and_then(|value| value.project_endpoint.as_ref())
            .is_some()
            && explicit.is_none()
        {
            return Err("A selected endpoint requires its original construction entry".into());
        }
        let mut compiled_sources = BTreeSet::new();
        let mut observed = BTreeMap::from([(context.path.clone(), context.source.clone())]);
        let mut state = None;
        let mut logical_origin = context.path.clone();
        let mut contributes = false;
        let mut construction_base = std::path::PathBuf::new();
        if let Some(entry) = &explicit {
            let entry = super::source_tests::relative(Path::new(""), entry)?;
            let base = Path::new(&entry).parent().unwrap_or(Path::new(""));
            let name = Path::new(&entry)
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or("Construction entry has no UTF-8 filename")?;
            let captured = std::cell::RefCell::new((BTreeMap::new(), 0usize));
            let sources = hick_lang::load_project_sources(name, &|logical| {
                let path = super::source_tests::relative(base, logical)?;
                if missing.contains(&path) {
                    return Err(format!(
                        "Construction still references deleted source {path}"
                    ));
                }
                let text = match supplied.get(&path) {
                    Some(text) => text.clone(),
                    None => super::source_tests::saved_source(root, &path)?,
                };
                let mut values = captured.borrow_mut();
                if !values.0.contains_key(&path) {
                    values.1 += text.len();
                    if values.0.len() >= 10000 || values.1 > 128 * 1024 * 1024 {
                        return Err(
                            "Template construction closure exceeds its interactive budget".into(),
                        );
                    }
                    values.0.insert(path, text.clone());
                }
                Ok(text)
            })?;
            observed = captured.into_inner().0;
            compiled_sources = observed.keys().cloned().collect();
            contributes = observed.contains_key(&context.path);
            construction_base = base.to_path_buf();
            if contributes {
                logical_origin = Path::new(&context.path)
                    .strip_prefix(base)
                    .map_err(|_| "Template source is outside its selected construction namespace")?
                    .to_str()
                    .ok_or("Template source identity is not UTF-8")?
                    .replace('\\', "/");
            } else {
                observed.insert(context.path.clone(), context.source.clone());
            }
            let endpoint = hick_lsp::project::ProjectEndpoint::parse(
                reading
                    .and_then(|value| value.project_endpoint.as_deref())
                    .unwrap_or("new"),
            )?;
            state = Some(
                hick_lsp::document::HickDocumentState::from_project_at(
                    name,
                    &sources,
                    &hick_lang::ProjectTree::default(),
                    &endpoint,
                )
                .map_err(|error| {
                    format!("Template construction language is unavailable: {error}")
                })?,
            );
        }
        let needs_entry = reading.is_some()
            && explicit.is_none()
            && reading
                .and_then(|value| value.entry_selection.as_ref())
                .is_none();
        let mut claims = std::collections::HashMap::new();
        let mut ownership_saved_bytes = 0usize;
        for (path, text) in &supplied {
            if !super::resource_inputs::contains_construction(text)? {
                continue;
            }
            let doc = hick_lang::parse(text).map_err(|error| error.to_string())?;
            if state.is_some() && !compiled_sources.contains(path) {
                let changed = match std::fs::symlink_metadata(root.join(path)) {
                    Ok(_) => {
                        let saved = super::source_tests::saved_source(root, path)?;
                        ownership_saved_bytes = ownership_saved_bytes
                            .checked_add(saved.len())
                            .ok_or("Ownership source budget overflow")?;
                        if ownership_saved_bytes > 128 * 1024 * 1024 {
                            return Err(
                                "Ownership source reads exceed their interactive budget".into()
                            );
                        }
                        saved != *text
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
                    Err(error) => {
                        return Err(format!("Retained source ownership is unavailable: {error}"));
                    }
                };
                if changed {
                    return Err(format!(
                        "Retained construction {path} is outside the captured ownership context"
                    ));
                }
            }
            if state.is_none() && !doc.find_tags("project").is_empty() && !needs_entry {
                return Err(
                    "Choose the retained construction entry before applying a template".into(),
                );
            }
            for tag in doc.find_tags("file") {
                if tag.get_attribute("path").is_none_or(|value| {
                    super::api::join_under(path.rsplit_once('/').map_or("", |(dir, _)| dir), value)
                        .is_none()
                }) {
                    return Err("A dynamic or unknown construction output cannot grant template write authority".into());
                }
            }
            for target in super::api::declared_outputs(path, text) {
                claims.insert(target, path.clone());
            }
        }
        if let Some(tree) = state
            .as_ref()
            .and_then(|state| state.selected_project.as_ref())
        {
            for target in tree.entries().keys() {
                claims.insert(
                    super::source_tests::relative(&construction_base, target)?,
                    explicit.clone().unwrap(),
                );
            }
        }
        if let Some(owner) = super::api::generated_by(&claims, &context.path) {
            return Err(format!(
                "Captured construction {owner} generates this source; insert the template in its authored owner"
            ));
        }
        for (target, owner) in &claims {
            if !target.ends_with('/')
                && same_file::is_same_file(root.join(&context.path), root.join(target))
                    .unwrap_or(false)
            {
                return Err(format!("This source aliases a captured output of {owner}"));
            }
        }
        let project = state
            .as_ref()
            .filter(|_| contributes)
            .and_then(|state| state.selected_project.as_ref())
            .map(|tree| (tree, logical_origin.as_str()));
        let reader = reader.with_project(project);
        // Every supplied source participates in ownership, even when not a compiler contributor.
        observed.extend(supplied);
        let hashes: BTreeMap<_, _> = observed
            .iter()
            .map(|(path, text)| (path.clone(), hash(text.as_bytes())))
            .collect();
        if hashes.len() > 10000 {
            return Err(
                "Template source fingerprint exceeds its interactive metadata budget".into(),
            );
        }
        let unaddressed: BTreeMap<_, _> = reading
            .into_iter()
            .flat_map(|reading| reading.unaddressed_sources.iter())
            .map(|(id, text)| (id.clone(), hash(text.as_bytes())))
            .collect();
        let graph_fingerprint = hash(
            serde_json::to_vec(&(
                1,
                &context.path,
                &explicit,
                reading.and_then(|value| value.project_endpoint.as_ref()),
                reading.and_then(|value| value.entry_selection.as_ref()),
                &hashes,
                &missing,
                &unaddressed,
                scope,
            ))
            .map_err(|error| error.to_string())?,
        );
        Ok(Self {
            context,
            reader,
            explicit,
            contributes,
            hashes,
            graph_fingerprint,
            needs_entry,
            byte_offsets,
        })
    }
    fn byte_range(&self, selection: &Selection) -> Result<(usize, usize), String> {
        let admitted = self.context.carets.as_ref().map_or_else(
            || {
                self.context.selection.from == selection.from
                    && self.context.selection.to == selection.to
            },
            |carets| {
                carets
                    .selections
                    .iter()
                    .any(|row| row.from == selection.from && row.to == selection.to)
            },
        );
        if !admitted {
            return Err("The selection is outside the original captured template roster".into());
        }
        Ok((
            *self
                .byte_offsets
                .get(&selection.from)
                .ok_or("Original template start offset is unavailable")?,
            *self
                .byte_offsets
                .get(&selection.to)
                .ok_or("Original template end offset is unavailable")?,
        ))
    }
    fn language_context(&self, selection: &Selection) -> Result<CapturedTemplateLanguage, String> {
        self.read_language_context(selection, true)
    }
    fn light_language_context(
        &self,
        selection: &Selection,
    ) -> Result<CapturedTemplateLanguage, String> {
        self.read_language_context(selection, false)
    }
    fn read_language_context(
        &self,
        selection: &Selection,
        include_hashes: bool,
    ) -> Result<CapturedTemplateLanguage, String> {
        if selection.from > selection.to {
            return Err("Template selection is reversed".into());
        }
        let (from, to) = self.byte_range(selection)?;
        if self.reader.requires_construction_scope(from, to)
            && self.explicit.is_some()
            && !self.contributes
        {
            return Err("The original embedded source does not contribute to its selected construction entry".into());
        }
        let mut language = self.reader.language(from, to)?;
        if self.needs_entry {
            language.language = None;
            language.scope = "entry-required";
            language.reason=Some("Read the current construction entry before associating this source with a language".into());
        }
        let fingerprint = hash(
            serde_json::to_vec(&(
                1,
                &self.graph_fingerprint,
                selection,
                language.scope,
                &language.language,
                &language.languages,
                &language.targets,
                &language.reason,
            ))
            .map_err(|error| error.to_string())?,
        );
        let mut binding = json!({"version":1,"scope":language.scope,"language":language.language,"languages":language.languages,"targets":language.targets,"reason":language.reason,"project_entry":self.explicit,"graph_fingerprint":self.graph_fingerprint,"fingerprint":fingerprint});
        if include_hashes {
            binding["source_hashes"] = json!(self.hashes);
        }
        if serde_json::to_vec(&binding)
            .map_err(|error| error.to_string())?
            .len()
            > 1024 * 1024
        {
            return Err("Template language reading exceeds its interactive metadata budget".into());
        }
        Ok(CapturedTemplateLanguage {
            language,
            binding,
            needs_entry: self.needs_entry,
        })
    }
}
fn capture_template_language(
    root: &Path,
    context: &Context,
    scope: Option<&str>,
) -> Result<CapturedTemplateLanguage, String> {
    PreparedTemplateGraph::capture(root, context, scope)?.language_context(&context.selection)
}

fn template_programs(context: &Context) -> Result<Vec<u32>, String> {
    let programs = context
        .transform_program_versions
        .clone()
        .unwrap_or_else(|| vec![2]);
    if programs.is_empty()
        || programs.len() > 8
        || programs.iter().any(|version| *version == 0)
        || programs.iter().copied().collect::<BTreeSet<_>>().len() != programs.len()
    {
        return Err("Malformed supported transform program versions".into());
    }
    if !programs.contains(&2) {
        return Err("This template provider currently requires transform program version2".into());
    }
    Ok(programs)
}
</ui:copy>

<ui:copy id="source-template-catalog">
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
</ui:copy>

## Expand from captured context before review

Derive the source language through the existing routing authority. Hash the exact reusable catalog, then recompute that fingerprint when the user requests a plan. Expand selection, filename, path and line variables through the shared snippet mechanism. Unknown variables and unsupported transforms retain that mechanism's explicit behavior; this catalog does not establish complete snippet syntax, embedded-language contexts or multi-caret support. The reply echoes the workspace, source hash, selection, template and replacement so every host can refuse stale observations before its existing coordinated draft transaction.

<ui:copy id="source-template-planning">
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
    prepare_for_scope(root, context, selected, None)
}
fn prepare_for_scope(
    root: &Path,
    context: &Context,
    selected: Option<(&str, &str)>,
    scope: Option<&str>,
) -> Result<Value, String> {
    let (from, to) = validate(context)?;
    if context.carets.is_some() {
        return prepare_caret_templates(root, context, selected, scope);
    }
    let root = root
        .canonicalize()
        .map_err(|e| format!("Workspace unavailable: {e}"))?;
    // Paths name source drafts, never instructions or process arguments.
    let captured = if context.version == 2 {
        Some(capture_template_language(&root, context, scope)?)
    } else {
        None
    };
    let language = captured
        .as_ref()
        .and_then(|reading| reading.language.language.as_deref())
        .unwrap_or_else(|| {
            if context.version == 1 {
                hick_plugins::routing::language_id(&context.path).unwrap_or("unknown")
            } else {
                "unknown"
            }
        });
    let entries = load(&root)?;
    let programs = template_programs(context)?;
    let fingerprint = hash(
        serde_json::to_vec(&(
            1,
            language,
            &entries,
            captured
                .as_ref()
                .map(|reading| &reading.binding["fingerprint"]),
            &programs,
            scope,
        ))
        .map_err(|error| error.to_string())?,
    );
    let mut value = json!({"version":1,"template_protocol":context.version,"path":context.path,"selection":context.selection,"source_hash":hash(context.source.as_bytes()),"workspace_id":hash(root.as_os_str().as_encoded_bytes()),"language":language,"catalog_fingerprint":fingerprint,"notices":["Templates edit the retained source draft only. They do not run commands, save files or establish semantic correctness.","Finish composition and choose one selection before applying."]});
    if context.version == 2 {
        value["variable_context"] = json!(context.variable_context);
        value["source_files_modified"] = json!(false);
        value["notices"] = json!([
            "Templates edit the retained source draft only. They do not run commands, save files or establish semantic correctness.",
            "Finish composition and choose one selection before applying."
        ]);
    }
    if context.transform_program_versions.is_some() {
        value["transform_program_versions"] = json!(programs);
    }
    if let Some(captured) = &captured {
        value["source_context"] = captured.binding.clone();
        if captured.needs_entry {
            if selected.is_some() {
                return Err(
                    "Read the original construction entry and refresh its template catalog".into(),
                );
            }
            value["available"] = json!(false);
            value["entries"] = json!([]);
            value["reading_context_request"] = json!({"version":1,"kind":"construction_entry"});
            value["reason"] = json!(captured.language.reason);
            return Ok(value);
        }
        if captured.language.language.is_none() {
            if selected.is_some() {
                return Err(captured
                    .language
                    .reason
                    .clone()
                    .unwrap_or("No unambiguous language context is available".into()));
            }
            value["available"] = json!(false);
            value["entries"] = json!([]);
            value["reason"] = json!(captured.language.reason);
            return Ok(value);
        }
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
</ui:copy>

## Keep provider reads independent of publication

The HTTP adapters run catalog reads and ownership analysis on a blocking worker. A declared generated file refuses template planning; its authored construction remains the editable target. The plan is still only a read observation: current source ownership, editor identity, composition, selection and write permission must pass again at host application, and normal guarded saving remains separate.

## Review every caret expansion against one prepared graph

Load the manifest and compiler closure once. Share graph hashes at the top level and return lightweight position readings per row. A mixed or unknown language makes the entire choice unavailable. Preview demands clipboard once if any effective row needs it, then repeats the same clock, seed, clipboard and original roster. Plans contain the complete expansion array, never a primary-only fallback. Exhausting the shared work budget refuses the reading rather than presenting the remaining entries as unsupported syntax.

<ui:copy id="source-template-batch-plans">
fn prepare_caret_templates(
    root: &Path,
    context: &Context,
    selected: Option<(&str, &str)>,
    scope: Option<&str>,
) -> Result<Value, String> {
    let carets = validate_template_carets(context)?.ok_or("Template caret roster is missing")?;
    let root = root
        .canonicalize()
        .map_err(|error| format!("Workspace unavailable: {error}"))?;
    let graph = PreparedTemplateGraph::capture(&root, context, scope)?;
    let entries = load(&root)?;
    let programs = template_programs(context)?;
    let mut readings = Vec::with_capacity(carets.selections.len());
    let mut languages = BTreeSet::new();
    let mut targets = BTreeSet::new();
    let mut reason = None;
    let mut metadata_bytes = 0usize;
    for selection in &carets.selections {
        let captured = graph.light_language_context(selection)?;
        languages.extend(captured.language.languages.iter().cloned());
        targets.extend(captured.language.targets.iter().cloned());
        if let Some(message) = &captured.language.reason {
            reason.get_or_insert_with(|| message.clone());
        }
        let reading = json!({"selection":selection,"context":captured.binding});
        metadata_bytes = metadata_bytes
            .checked_add(
                serde_json::to_vec(&reading)
                    .map_err(|error| error.to_string())?
                    .len(),
            )
            .ok_or("Template reading metadata overflow")?;
        if metadata_bytes > 1024 * 1024 || targets.len() > 1024 {
            return Err(
                "The complete template caret reading exceeds its interactive metadata budget"
                    .into(),
            );
        }
        readings.push(reading);
    }
    if languages.len() != 1 {
        reason.get_or_insert_with(|| {
            "The captured carets do not share one known source language".into()
        });
    }
    let language = if reason.is_none() && languages.len() == 1 {
        languages.iter().next().cloned()
    } else {
        None
    };
    let source_fingerprint = hash(
        serde_json::to_vec(&(1, &graph.graph_fingerprint, carets, &readings))
            .map_err(|error| error.to_string())?,
    );
    let binding = json!({"version":1,"scope":"caret-batch","language":language,"languages":languages,"targets":targets,"reason":reason,"project_entry":graph.explicit,"source_hashes":graph.hashes,"graph_fingerprint":graph.graph_fingerprint,"fingerprint":source_fingerprint});
    if serde_json::to_vec(&binding)
        .map_err(|error| error.to_string())?
        .len()
        > 1024 * 1024
    {
        return Err("The captured template graph exceeds its interactive metadata budget".into());
    }
    let fingerprint = hash(
        serde_json::to_vec(&(
            1,
            "caret-batch",
            &binding["fingerprint"],
            &entries,
            &programs,
            scope,
        ))
        .map_err(|error| error.to_string())?,
    );
    let mut value = json!({"version":1,"template_protocol":2,"template_batch_protocol":1,"path":context.path,"selection":context.selection,"carets":carets,"source_hash":hash(context.source.as_bytes()),"workspace_id":hash(root.as_os_str().as_encoded_bytes()),"language":language.as_deref().unwrap_or("unknown"),"catalog_fingerprint":fingerprint,"variable_context":context.variable_context,"source_files_modified":false,"source_context":binding,"readings":readings,"notices":["Every captured template insertion is reviewed and applied together through source Undo/Redo. No file is saved and no command runs.","Finish composition before applying the complete selection roster."]});
    if context.transform_program_versions.is_some() {
        value["transform_program_versions"] = json!(programs);
    }
    if graph.needs_entry {
        if selected.is_some() {
            return Err(
                "Read the original construction entry and refresh all template caret contexts"
                    .into(),
            );
        }
        value["available"] = json!(false);
        value["entries"] = json!([]);
        value["reading_context_request"] = json!({"version":1,"kind":"construction_entry"});
        return Ok(value);
    }
    let Some(language) = language else {
        if selected.is_some() {
            return Err(
                reason.unwrap_or("The complete template caret language is unavailable".into())
            );
        }
        value["available"] = json!(false);
        value["entries"] = json!([]);
        value["reason"] = json!(reason);
        return Ok(value);
    };
    let mut budget = hick_lsp::snippet::SnippetExpansionBudget::default();
    if let Some((id, expected)) = selected {
        if expected != fingerprint {
            return Err(
                "The original template caret catalog changed. Refresh and review the whole batch."
                    .into(),
            );
        }
        let (entry, origin) = entries
            .iter()
            .find(|(entry, _)| entry.id == id && entry.language == language)
            .ok_or("Template is unavailable for the complete caret language")?;
        let rows = template_expansion_rows(
            &root,
            context,
            entry,
            &language,
            carets,
            &graph,
            &mut budget,
        )?;
        let clipboard = rows.iter().any(|row| row.clipboard);
        value["template_id"] = json!(id);
        value["origin"] = json!(origin);
        let snapshot = variable_snapshot(context)?;
        if clipboard && snapshot.clipboard.is_none() {
            value["available"] = json!(false);
            value["context_request"] = json!({"version":1,"variables":["CLIPBOARD"]});
            value["reason"]=json!("At least one captured caret needs the clipboard before the whole batch can be previewed.");
            return Ok(value);
        }
        let expansions = materialize_template_rows(&rows, &mut budget)?;
        if clipboard && snapshot.clipboard == Some(None) {
            value["context_unavailable"] = json!(["CLIPBOARD"]);
            value["notices"].as_array_mut().unwrap().push(json!(
                "Clipboard unavailable; affected caret defaults used their fallback or empty text."
            ));
        }
        value["available"] = json!(true);
        value["expansions"] = json!(expansions);
    } else {
        let mut choices = Vec::new();
        for (entry, origin) in entries
            .iter()
            .filter(|(entry, _)| entry.language == language)
        {
            let outcome = template_expansion_rows(
                &root,
                context,
                entry,
                &language,
                carets,
                &graph,
                &mut budget,
            )
            .and_then(|rows| materialize_template_rows(&rows, &mut budget));
            if budget.exhausted() {
                return Err("The complete template caret catalog exceeds its aggregate work or expansion budget. Choose fewer carets or a smaller catalog.".into());
            }
            choices.push(json!({"id":entry.id,"label":entry.label,"description":entry.description,"language":entry.language,"origin":origin,"available":outcome.is_ok(),"reason":outcome.err()}));
        }
        value["available"] = json!(choices.iter().any(|entry| entry["available"] == true));
        value["entries"] = json!(choices);
    }
    Ok(value)
}
</ui:copy>

<ui:copy id="source-template-http">
fn validate_template_entry(state: &LocalState, context: &Context) -> Result<(), String> {
    if let Some(reading) = &context.reading_context {
        if let Some(raw) = &reading.entry_selection {
            let proof: super::workspace_entry_authority::EntrySelection =
                serde_json::from_value(raw.clone()).map_err(|error| {
                    format!("Malformed original construction entry proof: {error}")
                })?;
            if proof.entry.path != reading.project_entry {
                return Err(
                    "The captured entry proof does not match the selected construction entry"
                        .into(),
                );
            }
            super::workspace_entry_authority::validate_entry_selection(state, &proof)?;
        }
    }
    Ok(())
}
fn admitted(
    state: &LocalState,
    context: &Context,
    selected: Option<(&str, &str)>,
) -> Result<Value, String> {
    validate(context)?;
    let scope = state.resource_root.check(state.index.root())?.to_owned();
    validate_template_entry(state, context)?;
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
    let value = prepare_for_scope(&root, context, selected, Some(&scope))?;
    validate_template_entry(state, context)?;
    if state.resource_root.check(state.index.root())? != scope {
        return Err(
            "Original template workspace changed while reading its language context".into(),
        );
    }
    Ok(value)
}
pub async fn catalog(
    State(state): State<LocalState>,
    Json(context): Json<Context>,
) -> ApiResult<Json<Value>> {
    state
        .resource_root
        .ready()
        .await
        .map_err(ApiError::unprocessable)?;
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
    state
        .resource_root
        .ready()
        .await
        .map_err(ApiError::unprocessable)?;
    tokio::task::spawn_blocking(move || {
        let context = Context {
            version: request.version,
            path: request.path,
            source: request.source,
            selection: request.selection,
            variable_context: request.variable_context,
            reading_context: request.reading_context,
            transform_program_versions: request.transform_program_versions,
            carets: request.carets,
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
</ui:copy>

## Prove refusal without writing source

The focused witnesses exercise Unicode selection, retained variables, immutable catalog fingerprints, conflicting IDs, unsupported syntax and foreign catalog symlinks. They assert that planning does not create a file. Host tests separately exercise the actual floating preview, existing snippet fields and undo path; these pure witnesses do not establish native input or accessibility.

<ui:copy id="source-template-witness">
#[cfg(test)]
mod tests {
    use super::*;
    fn context() -> Context {
        Context {
            version: 1,
            variable_context: None,
            reading_context: None,
            transform_program_versions: None,
            carets: None,
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
        assert!(plan["snippet"]["fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field["index"] == 0));
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
            reading_context: None,
            transform_program_versions: None,
            carets: None,
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
        assert!(prepare(
            root.path(),
            &large,
            Some((
                "python.function",
                unavailable["catalog_fingerprint"].as_str().unwrap()
            ))
        )
        .unwrap_err()
        .contains("input limit"));
    }
    #[test]
    fn workspace_data_templates_recheck_catalog_and_reject_conflicting_ids() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join(".hickory")).unwrap();
        let manifest = root.path().join(".hickory/templates.json");
        let context = context();
        let before = prepare(root.path(), &context, None).unwrap();
        std::fs::write(&manifest,r#"{"version":1,"templates":[{"id":"project.function","label":"Project","description":"selected text","language":"python","syntax":"${TM_SELECTED_TEXT} ${TM_FILENAME} $0"}]}"#).unwrap();
        assert!(prepare(
            root.path(),
            &context,
            Some((
                "python.function",
                before["catalog_fingerprint"].as_str().unwrap()
            ))
        )
        .unwrap_err()
        .contains("catalog changed"));
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
        assert!(prepare(root.path(), &context, None)
            .unwrap_err()
            .contains("unique"));
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
            reading_context: None,
            transform_program_versions: None,
            carets: None,
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
        assert!(prepare(
            root.path(),
            &context,
            Some((
                "project.transform",
                catalog["catalog_fingerprint"].as_str().unwrap()
            ))
        )
        .is_err());
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
</ui:copy>

The embedded service checks below use real captured compiler closures and dirty overrides, including unchanged authored disks, same/mixed language occurrences, stale catalog refusal, deleted inputs and new generated ownership. These are focused model checks; connected browser/native flows remain separate acceptance.

<ui:copy id="embedded-template-service-witness">
#[cfg(test)]
mod embedded_template_tests {
    use super::*;
    fn doc(body: &str) -> String {
        format!("\x3ch:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">{body}\x3c/h:doc>")
    }
    fn entry(target: &str) -> String {
        doc(&format!(
            "\x3ch:project sources=\"part.md\"/>\x3ch:file path=\"{target}\">\x3ch:paste select=\"#body\"/>\x3c/h:file>"
        ))
    }
    fn setup() -> (tempfile::TempDir, Context) {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join(".hickory")).unwrap();
        std::fs::write(root.path().join("project.md"), entry("saved.py")).unwrap();
        let saved = doc("\x3ch:copy id=\"body\">saved\x3c/h:copy>");
        std::fs::write(root.path().join("part.md"), saved).unwrap();
        std::fs::write(root.path().join(".hickory/templates.json"),serde_json::to_vec(&json!({"version":1,"templates":[{"id":"workspace.context","label":"Context","description":"Bound language comments and fields","language":"python","syntax":"${LINE_COMMENT}${TM_FILENAME}|${1:${TM_SELECTED_TEXT}}|$1|${1/(.*)/${1:/upcase}/}$0"}]})).unwrap()).unwrap();
        let source = doc("\x3ch:copy id=\"body\">old\x3c/h:copy>");
        let from = source.find("old").unwrap();
        let context = Context {
            version: 2,
            path: "part.md".into(),
            source: source.clone(),
            selection: Selection { from, to: from + 3 },
            variable_context: Some(
                json!({"clock":{"unix_millis":1704067200000i64,"utc_offset_minutes":0},"random_hex":"0123456789abcdef0123456789abcdef","clipboard":null}),
            ),
            reading_context: Some(TemplateReadingContext {
                version: 1,
                project_entry: Some("project.md".into()),
                sources: BTreeMap::from([
                    ("project.md".into(), entry("draft.py")),
                    ("part.md".into(), source),
                ]),
                ..Default::default()
            }),
            transform_program_versions: Some(vec![2]),
            carets: None,
        };
        (root, context)
    }
    #[test]
    fn dirty_external_fragment_gets_language_comments_and_transform_without_materializing() {
        let (root, context) = setup();
        let before = std::fs::read(root.path().join("project.md")).unwrap();
        let graph =
            PreparedTemplateGraph::capture(root.path(), &context, Some("same-root")).unwrap();
        let first = graph.language_context(&context.selection).unwrap();
        let second = graph.language_context(&context.selection).unwrap();
        assert_eq!(first.binding, second.binding);
        let catalog = prepare(root.path(), &context, None).unwrap();
        assert_eq!(catalog["language"], "python");
        assert_eq!(
            catalog["source_context"]["source_hashes"]["project.md"],
            hash(entry("draft.py").as_bytes())
        );
        let plan = prepare(
            root.path(),
            &context,
            Some((
                "workspace.context",
                catalog["catalog_fingerprint"].as_str().unwrap(),
            )),
        )
        .unwrap();
        assert_eq!(plan["snippet"]["text"], "#part.md|old|old|OLD");
        assert_eq!(plan["source_files_modified"], false);
        assert_eq!(plan["transform_program_versions"], json!([2]));
        assert_eq!(
            std::fs::read(root.path().join("project.md")).unwrap(),
            before
        );
        assert!(!root.path().join("draft.py").exists());
    }
    #[test]
    fn mixed_language_reuse_and_changed_original_graph_never_rebind_review() {
        let (root, mut context) = setup();
        let catalog = prepare(root.path(), &context, None).unwrap();
        let fingerprint = catalog["catalog_fingerprint"].as_str().unwrap().to_owned();
        let mixed = entry("draft.py").replace(
            "\x3c/h:doc>",
            "\x3ch:file path=\"other.js\">\x3ch:paste select=\"#body\"/>\x3c/h:file>\x3c/h:doc>",
        );
        context
            .reading_context
            .as_mut()
            .unwrap()
            .sources
            .insert("project.md".into(), mixed);
        let unavailable = prepare(root.path(), &context, None).unwrap();
        assert_eq!(unavailable["available"], false);
        assert_eq!(unavailable["entries"], json!([]));
        assert!(prepare(
            root.path(),
            &context,
            Some(("workspace.context", &fingerprint))
        )
        .is_err());
    }
    #[test]
    fn missing_contributors_and_new_dirty_generated_claims_refuse_whole_plan() {
        let (root, mut context) = setup();
        context
            .reading_context
            .as_mut()
            .unwrap()
            .missing_sources
            .insert("part.md".into());
        assert!(prepare(root.path(), &context, None).is_err());
        context
            .reading_context
            .as_mut()
            .unwrap()
            .missing_sources
            .clear();
        context.path = "draft.py".into();
        context.source = "old".into();
        context.selection = Selection { from: 0, to: 3 };
        context
            .reading_context
            .as_mut()
            .unwrap()
            .sources
            .insert("draft.py".into(), "old".into());
        assert!(prepare(root.path(), &context, None)
            .unwrap_err()
            .contains("generates"));
    }
    #[test]
    fn canonical_entry_acquisition_is_separate_from_unaddressed_ownership_and_programs() {
        let (root, mut context) = setup();
        context.reading_context.as_mut().unwrap().project_entry = None;
        context
            .reading_context
            .as_mut()
            .unwrap()
            .sources
            .remove("project.md");
        let request = prepare(root.path(), &context, None).unwrap();
        assert_eq!(request["available"], false);
        assert_eq!(
            request["reading_context_request"],
            json!({"version":1,"kind":"construction_entry"})
        );
        context
            .reading_context
            .as_mut()
            .unwrap()
            .unaddressed_sources
            .insert(
                "opaque-editor".into(),
                doc("\x3ch:file path=\"draft.py\">unknown\x3c/h:file>"),
            );
        assert!(prepare(root.path(), &context, None).is_err());
        context
            .reading_context
            .as_mut()
            .unwrap()
            .unaddressed_sources
            .clear();
        context.transform_program_versions = Some(vec![3]);
        assert!(prepare(root.path(), &context, None).is_err());
        context.transform_program_versions = Some(vec![2, 2]);
        assert!(prepare(root.path(), &context, None).is_err());
    }
}
</ui:copy>

## Verify whole-roster preparation before enabling host Apply

The focused cases below check immutable row order, primary selection and indexed scalar/CRLF boundaries; common clock/seed with per-row cursor and selected values; one effective clipboard demand with explicit denial; changed secondary carets; mixed languages; and aggregate refusal without disk changes. These are service policy tests. They do not establish a usable multi-caret host path until the browser and native adapters prepare the pure whole-group plan and pass their actual Apply/history flows.

<ui:copy id="source-template-batch-witness">
#[cfg(test)]
mod caret_template_tests {
    use super::*;
    fn fixture(syntax: &str) -> (tempfile::TempDir, Context) {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join(".hickory")).unwrap();
        std::fs::write(root.path().join(".hickory/templates.json"), serde_json::to_vec(&json!({"version":1,"templates":[{"id":"workspace.carets","label":"Captured carets","description":"One reviewed selection roster","language":"python","syntax":syntax}]})).unwrap()).unwrap();
        let source = "left right".to_string();
        std::fs::write(root.path().join("main.py"), &source).unwrap();
        let context = Context {
            version: 2,
            path: "main.py".into(),
            source,
            selection: Selection { from: 0, to: 4 },
            variable_context: Some(
                json!({"clock":{"unix_millis":1704067200000i64,"utc_offset_minutes":0},"random_hex":"0123456789abcdef0123456789abcdef","clipboard":null}),
            ),
            reading_context: None,
            transform_program_versions: Some(vec![2]),
            carets: Some(TemplateCarets {
                version: 1,
                selections: vec![Selection { from: 5, to: 10 }, Selection { from: 0, to: 4 }],
                primary: 1,
                continuing: false,
            }),
        };
        (root, context)
    }
    fn planned(root: &Path, context: &Context, catalog: &Value) -> Result<Value, String> {
        prepare(
            root,
            context,
            Some((
                "workspace.carets",
                catalog["catalog_fingerprint"].as_str().unwrap(),
            )),
        )
    }
    #[test]
    fn ordered_carets_share_clock_seed_graph_and_keep_distinct_source_values() {
        let (root,context)=fixture("${CURSOR_INDEX}|${CURSOR_NUMBER}|${CURRENT_YEAR}|${RANDOM_HEX}|${1:${TM_SELECTED_TEXT}}|$1|${1/(.*)/${1:/upcase}/}$0");
        let before = std::fs::read(root.path().join("main.py")).unwrap();
        let catalog = prepare(root.path(), &context, None).unwrap();
        let plan = planned(root.path(), &context, &catalog).unwrap();
        assert_eq!(catalog["template_batch_protocol"], 1);
        assert_eq!(catalog["carets"], json!(context.carets));
        assert_eq!(plan["variable_context"], json!(context.variable_context));
        assert_eq!(plan["source_files_modified"], false);
        assert!(plan.get("snippet").is_none());
        assert!(plan.get("replacement").is_none());
        assert_eq!(
            plan["expansions"][0]["snippet"]["text"],
            "0|1|2024|012345|right|right|RIGHT"
        );
        assert_eq!(
            plan["expansions"][1]["snippet"]["text"],
            "1|2|2024|012345|left|left|LEFT"
        );
        assert_eq!(
            plan["expansions"][0]["replacement"],
            json!({"from":5,"to":10})
        );
        assert_eq!(
            plan["expansions"][1]["replacement"],
            json!({"from":0,"to":4})
        );
        assert_eq!(
            plan["source_context"]["source_hashes"]["main.py"],
            hash(context.source.as_bytes())
        );
        for reading in plan["readings"].as_array().unwrap() {
            assert!(reading["context"].get("source_hashes").is_none());
            assert_eq!(
                reading["context"]["graph_fingerprint"],
                plan["source_context"]["graph_fingerprint"]
            );
        }
        assert_eq!(std::fs::read(root.path().join("main.py")).unwrap(), before);
    }
    #[test]
    fn one_effective_clipboard_demand_and_denial_cover_the_complete_original_roster() {
        let (root, mut context) = fixture("${TM_SELECTED_TEXT:${CLIPBOARD:fallback}}$0");
        context.source = "old ".into();
        context.selection = Selection { from: 0, to: 3 };
        context.carets = Some(TemplateCarets {
            version: 1,
            selections: vec![context.selection.clone(), Selection { from: 4, to: 4 }],
            primary: 0,
            continuing: false,
        });
        // Empty selected text is still a known value: its unused default
        // must not acquire clipboard data for either logical slot.
        let unused_catalog = prepare(root.path(), &context, None).unwrap();
        let unused = planned(root.path(), &context, &unused_catalog).unwrap();
        assert!(unused.get("context_request").is_none());
        assert_eq!(unused["expansions"][0]["snippet"]["text"], "old");
        assert_eq!(unused["expansions"][1]["snippet"]["text"], "");
        let manifest = root.path().join(".hickory/templates.json");
        let mut declaration: Value =
            serde_json::from_slice(&std::fs::read(&manifest).unwrap()).unwrap();
        declaration["templates"][0]["syntax"] =
            json!("${TM_SELECTED_TEXT}|${CLIPBOARD:fallback}$0");
        std::fs::write(&manifest, serde_json::to_vec(&declaration).unwrap()).unwrap();
        let catalog = prepare(root.path(), &context, None).unwrap();
        let demand = planned(root.path(), &context, &catalog).unwrap();
        assert_eq!(
            demand["context_request"],
            json!({"version":1,"variables":["CLIPBOARD"]})
        );
        assert_eq!(demand["carets"], json!(context.carets));
        assert!(demand.get("expansions").is_none());
        context.variable_context.as_mut().unwrap()["clipboard"] =
            json!({"available":false,"text":null});
        let denied = planned(root.path(), &context, &catalog).unwrap();
        assert_eq!(denied["expansions"][0]["snippet"]["text"], "old|fallback");
        assert_eq!(denied["expansions"][1]["snippet"]["text"], "|fallback");
        assert_eq!(denied["context_unavailable"], json!(["CLIPBOARD"]));
        assert_eq!(planned(root.path(), &context, &catalog).unwrap(), denied);
    }
    #[test]
    fn secondary_roster_changes_cannot_reuse_a_reviewed_catalog() {
        let (root, mut context) = fixture("${CURSOR_NUMBER}:${1:${TM_SELECTED_TEXT}}$0");
        let catalog = prepare(root.path(), &context, None).unwrap();
        context.carets.as_mut().unwrap().selections[0] = Selection { from: 6, to: 10 };
        assert!(planned(root.path(), &context, &catalog).is_err());
        context.carets.as_mut().unwrap().primary = 0;
        assert!(prepare(root.path(), &context, None).is_err());
    }
    #[test]
    fn scalar_crlf_offsets_are_indexed_once_and_coincident_slots_require_continuation() {
        let (_, mut context) = fixture("$0");
        context.source = "😀\r\nx".into();
        context.selection = Selection { from: 0, to: 2 };
        context.carets = Some(TemplateCarets {
            version: 1,
            selections: vec![context.selection.clone(), Selection { from: 4, to: 5 }],
            primary: 0,
            continuing: false,
        });
        assert_eq!(
            template_selection_offsets(&context).unwrap(),
            BTreeMap::from([(0, 0), (2, 4), (4, 6), (5, 7)])
        );
        context.carets.as_mut().unwrap().selections[1] = Selection { from: 1, to: 2 };
        assert!(template_selection_offsets(&context).is_err());
        context.carets.as_mut().unwrap().selections[1] = Selection { from: 3, to: 5 };
        assert!(template_selection_offsets(&context).is_err());
        context.source.clear();
        context.selection = Selection { from: 0, to: 0 };
        context.carets.as_mut().unwrap().selections =
            vec![context.selection.clone(), context.selection.clone()];
        assert!(template_selection_offsets(&context).is_err());
        context.carets.as_mut().unwrap().continuing = true;
        assert_eq!(
            template_selection_offsets(&context).unwrap(),
            BTreeMap::from([(0, 0)])
        );
        context.carets.as_mut().unwrap().selections = vec![context.selection.clone(); 257];
        assert!(template_selection_offsets(&context).is_err());
    }
    #[test]
    fn different_embedded_languages_refuse_the_whole_choice() {
        let (root, mut context) = fixture("${1:${TM_SELECTED_TEXT}}$0");
        context.path = "notes.md".into();
        context.source = "```python\nold\n```\n```javascript\nnew\n```\n".into();
        let first = context.source.find("old").unwrap();
        let second = context.source.find("new").unwrap();
        context.selection = Selection {
            from: first,
            to: first + 3,
        };
        context.carets = Some(TemplateCarets {
            version: 1,
            selections: vec![
                context.selection.clone(),
                Selection {
                    from: second,
                    to: second + 3,
                },
            ],
            primary: 0,
            continuing: false,
        });
        let catalog = prepare(root.path(), &context, None).unwrap();
        assert_eq!(catalog["available"], false);
        assert_eq!(catalog["entries"], json!([]));
        assert_eq!(
            catalog["source_context"]["languages"],
            json!(["javascript", "python"])
        );
        assert!(planned(root.path(), &context, &catalog).is_err());
    }
    #[test]
    fn aggregate_variable_and_expansion_budget_refuses_before_any_partial_plan() {
        let (root, mut context) = fixture("${TM_SELECTED_TEXT}$0");
        context.source = format!("{} {}", "a".repeat(600000), "b".repeat(600000));
        context.selection = Selection {
            from: 0,
            to: 600000,
        };
        context.carets = Some(TemplateCarets {
            version: 1,
            selections: vec![
                context.selection.clone(),
                Selection {
                    from: 600001,
                    to: 1200001,
                },
            ],
            primary: 0,
            continuing: false,
        });
        let before = std::fs::read(root.path().join("main.py")).unwrap();
        let error = prepare(root.path(), &context, None).unwrap_err();
        assert!(error.contains("aggregate"), "{error}");
        assert_eq!(std::fs::read(root.path().join("main.py")).unwrap(), before);
    }
}
</ui:copy>

<ui:file path="crates/hickory-cli/src/serve/source_templates.rs"><ui:paste select="#source-template-inputs"/><ui:paste select="#source-template-batch-values"/><ui:paste select="#embedded-template-captured-graph"/><ui:paste select="#source-template-catalog"/><ui:paste select="#source-template-planning"/><ui:paste select="#source-template-batch-plans"/><ui:paste select="#source-template-http"/><ui:paste select="#source-template-witness"/><ui:paste select="#embedded-template-service-witness"/><ui:paste select="#source-template-batch-witness"/></ui:file>
## Exercise the provider boundary against a real engine

This isolated HTTP witness reads a custom template, repeats an unchanged plan,
then revokes its definition. It refuses a generated file, a physical alias and a
not-yet-created path below an escaped directory. Its assertions retain exact
authored bytes; no renderer test substitutes for these engine ownership checks.

<ui:copy id="source-template-http-witness">
"""Focused real-engine read-only template planning and authority refusals."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import urllib.error
import urllib.request
ROOT=Path(__file__).resolve().parents[1]
def main():
 spec=importlib.util.spec_from_file_location('template_fixture',ROOT/'scripts/test-simple-browser.py');fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix='hickory-template-service-') as temporary:
  root=Path(temporary)/'workspace';root.mkdir();(root/'.git').mkdir();(root/'.hickory').mkdir()
  source='😀 old\r\n';(root/'source.py').write_bytes(source.encode())
  generated='print(1)\n';(root/'generated.py').write_text(generated)
  author='<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n<h:file path="generated.py">\n'+generated+'</h:file>\n</h:doc>\n';(root/'author.md').write_text(author)
  manifest=root/'.hickory/templates.json';definition={'version':1,'templates':[{'id':'workspace.selected','label':'Selection','description':'Captured selection and line','language':'python','syntax':'${TM_SELECTED_TEXT}:${TM_LINE_NUMBER}$0'}]};manifest.write_text(json.dumps(definition))
  engine=fixture.start([os.environ.get('HICKORY_SIMPLE_TEST_ENGINE',str(ROOT/'target/source-edit-focus/debug/examples/engine_client')),str(root),'window','http://127.0.0.1:1'],env={**os.environ,'HICKORY_STATE_DIR':str(Path(temporary)/'state')})
  try:
   origin=fixture.origin(engine)
   def ask(route,body,rejected=False):
    request=urllib.request.Request(origin+'/api/templates/'+route,data=json.dumps(body).encode(),headers={'Content-Type':'application/json'})
    try:
     with urllib.request.urlopen(request,timeout=30) as response:value=json.load(response)
    except urllib.error.HTTPError as error:
     assert rejected,(error.code,error.read().decode());return error.read().decode()
    assert not rejected,value
    return value
   context={'version':1,'path':'source.py','source':source,'selection':{'from':3,'to':6}}
   catalog=ask('catalog',context);assert catalog['template_protocol']==1 and catalog['available'];assert catalog['source_hash']==hashlib.sha256(source.encode()).hexdigest();assert any(entry['id']=='workspace.selected' and entry['origin']=='workspace' for entry in catalog['entries'])
   request={**context,'catalog_fingerprint':catalog['catalog_fingerprint'],'template_id':'workspace.selected'}
   plan=ask('plan',request);assert plan['snippet']['text']=='old:1' and plan['replacement']==context['selection'];assert ask('plan',request)==plan
   # Revoking a definition after preview cannot silently accept a different plan.
   definition['templates'][0]['syntax']='changed $0';manifest.write_text(json.dumps(definition));assert 'catalog changed' in ask('plan',request,True)
   definition['templates'].append({'id':'workspace.empty_choices','label':'Bounded empty choices','description':'Must refuse mirrored metadata amplification','language':'python','syntax':'${1|'+','*4095+'|}'+'$1'*8})
   manifest.write_text(json.dumps(definition));bounded=ask('catalog',context)
   excessive=next(entry for entry in bounded['entries'] if entry['id']=='workspace.empty_choices');assert not excessive['available'] and 'metadata' in excessive['reason']
   ask('plan',{**context,'catalog_fingerprint':bounded['catalog_fingerprint'],'template_id':'workspace.empty_choices'},True)
   for target in ['generated.py','../outside.py']:
    ask('catalog',{**context,'path':target},True)
   if os.name!='nt':
    (root/'alias.py').symlink_to(root/'generated.py');assert 'aliases' in ask('catalog',{**context,'path':'alias.py'},True)
    foreign=Path(temporary)/'foreign';foreign.mkdir();(root/'escaped').symlink_to(foreign,target_is_directory=True);assert 'escapes' in ask('catalog',{**context,'path':'escaped/new.py'},True)
   ask('plan',{**request,'command':'no executable template commands'},True)
   unknown=ask('catalog',{**context,'path':'plain.unknown'});assert not unknown['available'] and unknown['entries']==[]
   assert (root/'source.py').read_bytes()==source.encode() and (root/'author.md').read_text()==author and (root/'generated.py').read_text()==generated
   print('PASS actual template catalog/plan, unchanged repeat, revoked catalog, generated/alias/escape refusals and unchanged authors')
  finally:fixture.stop(engine)
if __name__=='__main__':main()
</ui:copy>
<ui:file path="scripts/test-source-template-service.py"><ui:paste select="#source-template-http-witness"/></ui:file>
</ui:doc>
