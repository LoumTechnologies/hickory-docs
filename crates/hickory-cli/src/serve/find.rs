//! Exhaustive find, and replace, across the folder.
//!
//! Deliberately NOT `/api/search`. That one is ranked — BM25 plus embeddings,
//! top-k — which is the right shape for "where is the thing about invoices"
//! and the wrong shape entirely for replace: a ranked answer is a *sample*,
//! and replacing across a sample silently changes some of the occurrences.
//! This one is exhaustive, ordered by path, and reports when it stopped.
//!
//! ## What replace refuses
//!
//! A **generated** file. Its bytes come from a document, and writing to it
//! either loses the edit at the next weave or fights the up-loop for it. The
//! honest answer is to say so and name the document, so the reader can make
//! the change where it will survive — which is also the change that fixes
//! every other copy of it. See `declared_outputs` in `api.rs`.
//!
//! A **`.hick` document** is fair game: it is source, and a rename inside one
//! is exactly the edit somebody means.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use serde_json::{Value, json};

use super::LocalState;
use super::api::{ApiError, ApiResult};

/// Files bigger than this are skipped. A find that has to read a 200MB build
/// artefact to tell you it is not there has already wasted more of your time
/// than the answer is worth.
const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;

/// Stop after this many matches. A pattern like `e` matches a million times
/// in a real repository, and a listing nobody can read is not an answer.
const MAX_MATCHES: usize = 5_000;

#[derive(Deserialize)]
pub struct FindParams {
    /// The pattern. Literal unless `regex` is set.
    pub q: String,
    #[serde(default)]
    pub regex: bool,
    /// Case-sensitive when set; insensitive is the default, because that is
    /// what someone typing a word into a box means.
    #[serde(default)]
    pub case: bool,
    #[serde(default)]
    pub whole_word: bool,
}

/// Build the matcher these parameters describe.
///
/// Every flag is a change to the PATTERN rather than a branch in the loop, so
/// find and replace cannot end up matching differently — which would be the
/// worst possible bug in a tool like this: a preview that does not describe
/// the write.
fn matcher(params: &FindParams) -> Result<regex::Regex, ApiError> {
    if params.q.is_empty() {
        return Err(ApiError::bad_request("the pattern q= must not be empty"));
    }
    let body = if params.regex {
        params.q.clone()
    } else {
        regex::escape(&params.q)
    };
    let bounded = if params.whole_word {
        format!(r"\b(?:{body})\b")
    } else {
        body
    };
    let cased = if params.case {
        bounded
    } else {
        format!("(?i){bounded}")
    };
    regex::Regex::new(&cased).map_err(|e| {
        ApiError::bad_request(format!(
            "that is not a valid regular expression: {e}\n  \
             Turn the regex option off to search for it literally."
        ))
    })
}

/// One matched line, with enough to render a row and to jump to it.
fn line_hits(regex: &regex::Regex, text: &str, budget: &mut usize) -> Vec<Value> {
    let mut out = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if *budget == 0 {
            break;
        }
        let mut columns = Vec::new();
        for m in regex.find_iter(line) {
            // A zero-width match (`^`, `\b`) would otherwise report every
            // position on the line and replace nothing.
            if m.is_empty() {
                continue;
            }
            columns.push(json!({ "column": m.start(), "length": m.len() }));
            *budget -= 1;
            if *budget == 0 {
                break;
            }
        }
        if !columns.is_empty() {
            out.push(json!({
                "line": index + 1,
                // The whole line, so a result row shows the match in context
                // without a second request per hit.
                "text": line,
                "at": columns,
            }));
        }
    }
    out
}

/// Every text file under the root, in path order, with its contents.
fn walk_text_files(root: &Path) -> Vec<(String, PathBuf, String)> {
    let mut files = Vec::new();
    let walker = ignore::WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .require_git(false)
        .git_global(false)
        .filter_entry(|e| e.file_name() != ".hick-cache" && e.file_name() != "node_modules")
        .build();
    for entry in walker.flatten() {
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        if entry.metadata().map(|m| m.len()).unwrap_or(u64::MAX) > MAX_FILE_BYTES {
            continue;
        }
        let Ok(rel) = entry.path().strip_prefix(root) else {
            continue;
        };
        // Not UTF-8 is not an error: it is a binary file, and a find over the
        // folder should not stop at the first PNG.
        let Ok(text) = std::fs::read_to_string(entry.path()) else {
            continue;
        };
        files.push((
            rel.to_string_lossy().replace('\\', "/"),
            entry.path().to_path_buf(),
            text,
        ));
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    files
}

/// `GET /api/find?q=…` — every match in the folder, in path order.
pub async fn find(
    State(state): State<LocalState>,
    Query(params): Query<FindParams>,
) -> ApiResult<Json<Value>> {
    let regex = matcher(&params)?;
    let root = state.index.root().to_path_buf();
    let generated = generated_map(&state);

    let (files, truncated) = tokio::task::spawn_blocking(move || {
        let mut budget = MAX_MATCHES;
        let mut out = Vec::new();
        for (rel, _, text) in walk_text_files(&root) {
            if budget == 0 {
                return (out, true);
            }
            let hits = line_hits(&regex, &text, &mut budget);
            if hits.is_empty() {
                continue;
            }
            out.push(json!({
                "path": rel,
                "matches": hits,
                // Named here so the UI can grey out what replace will refuse
                // BEFORE anyone presses the button.
                "generated_by": super::api::generated_by(&generated, &rel),
            }));
        }
        (out, budget == 0)
    })
    .await
    .map_err(|e| ApiError::internal(format!("the find task failed: {e}")))?;

    Ok(Json(json!({ "files": files, "truncated": truncated })))
}

#[derive(Deserialize)]
pub struct ReplaceBody {
    #[serde(flatten)]
    pub find: FindParams,
    /// What each match becomes. `$1` and friends work when `regex` is set.
    pub replacement: String,
    /// Only these paths, when given — how "replace in the files I ticked"
    /// is expressed. Absent means every file that matches.
    #[serde(default)]
    pub paths: Option<Vec<String>>,
}

/// `POST /api/find/replace` — rewrite every match, and say what was skipped.
pub async fn replace(
    State(state): State<LocalState>,
    Json(body): Json<ReplaceBody>,
) -> ApiResult<Json<Value>> {
    let regex = matcher(&body.find)?;
    let root = state.index.root().to_path_buf();
    let generated = generated_map(&state);
    let only: Option<std::collections::HashSet<String>> =
        body.paths.map(|list| list.into_iter().collect());
    let replacement = body.replacement.clone();

    let find = body.find.q.clone();
    let result = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let mut changed = Vec::new();
        let mut skipped = Vec::new();
        let mut total = 0usize;
        // Collected first, written second. A replace is the batch writer this
        // whole design is shaped around: forty files in one act, so undoing
        // it is one act too rather than forty reverts done in the right order
        // by hand. Recording per file as we went would produce forty acts and
        // make the undo useless.
        let mut writes: Vec<(std::path::PathBuf, Vec<u8>)> = Vec::new();
        for (rel, absolute, text) in walk_text_files(&root) {
            if only.as_ref().is_some_and(|set| !set.contains(&rel)) {
                continue;
            }
            let count = regex.find_iter(&text).filter(|m| !m.is_empty()).count();
            if count == 0 {
                continue;
            }
            // The refusal, and the reason. Writing here either loses the edit
            // at the next weave or fights the up-loop for it.
            if let Some(doc) = generated.get(&rel) {
                skipped.push(json!({
                    "path": rel,
                    "matches": count,
                    "reason": "generated",
                    "document": doc,
                }));
                continue;
            }
            let next = regex.replace_all(&text, replacement.as_str()).into_owned();
            if next == text {
                continue;
            }
            writes.push((absolute, next.into_bytes()));
            total += count;
            changed.push(json!({ "path": rel, "matches": count }));
        }

        // One act for the whole replace, recorded before any of it lands.
        // This is what closes the sentence at the end of
        // `docs/guarantees/search/find-and-replace-is-exhaustive.md`.
        crate::history::record(
            &root,
            hickory_workspace::history::ActKind::Replace,
            Some(format!("{find} → {replacement}")),
            &writes,
        );
        for (absolute, bytes) in &writes {
            super::store::write_atomic(absolute, bytes)
                .map_err(|e| format!("could not write {}: {e:#}", absolute.display()))?;
        }
        Ok(json!({ "changed": changed, "skipped": skipped, "replacements": total }))
    })
    .await
    .map_err(|e| ApiError::internal(format!("the replace task failed: {e}")))?
    .map_err(ApiError::internal)?;

    Ok(Json(result))
}

/// Output path → the id of the document that writes it.
fn generated_map(state: &LocalState) -> HashMap<String, String> {
    super::api::generated_outputs(state.index.root(), &state.index)
}
