//! Project-wide search over a hick folder, shaped like
//! [semble](https://github.com/MinishLab/semble): ask in natural language or
//! code, get ranked chunks with exact file and line, plus "find code related
//! to this file:line". First-party, no third-party CLI.
//!
//! Two rankers, one seam:
//!
//! - **Lexical** — BM25 over identifier-aware tokens (camelCase and
//!   snake_case split). Always available, fully offline, needs nothing.
//! - **Semantic** — Model2Vec static embeddings (the engine semble uses),
//!   loaded from a model folder on disk. Optional: when the folder is
//!   absent, search is lexical and says nothing about it. Nothing here
//!   touches the network — downloading a model is the CLI's explicit,
//!   never-automatic job, the same doctrine as `hick lsp install`.
//!
//! When both rankers run, results are fused with reciprocal-rank fusion, so
//! neither scale has to be calibrated against the other.
//!
//! The index lives in `.hick-cache/search/index.json`, keyed by content
//! hash: an unchanged file is never re-chunked or re-embedded, so the first
//! search pays for the build and the rest are incremental — semble's cached
//! index, implemented here.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};
use model2vec_rs::model::StaticModel;
use serde::{Deserialize, Serialize};

/// Lines per chunk. Big enough that a function usually fits, small enough
/// that a hit points somewhere, not at a whole file.
const CHUNK_LINES: usize = 40;
/// Overlap between consecutive chunks, so a match spanning a boundary is
/// still inside one chunk.
const CHUNK_OVERLAP: usize = 8;
/// Files larger than this are skipped: they are generated blobs or data, and
/// a chunk inside one is rarely what anyone is searching for.
const MAX_FILE_BYTES: u64 = 512 * 1024;
/// Reciprocal-rank-fusion constant (the standard 60 from the RRF paper).
const RRF_K: f32 = 60.0;

/// One search hit: a chunk of one file, with 1-based line bounds.
#[derive(Debug, Clone, Serialize)]
pub struct Hit {
    /// Path relative to the searched root.
    pub path: String,
    pub start_line: usize,
    pub end_line: usize,
    /// Fused score; comparable only within one result list.
    pub score: f32,
    /// The chunk's text, for preview. Callers may truncate.
    pub snippet: String,
}

/// Where the optional embedding model lives for a project root.
///
/// Under `.hick-cache` so `hick init`'s gitignore entry already covers it,
/// and per-project so deleting a project leaves nothing behind.
pub fn model_dir(root: &Path) -> PathBuf {
    root.join(".hick-cache").join("models").join("embed")
}

/// Whether the model folder holds what [`StaticModel`] needs to load.
pub fn model_available(root: &Path) -> bool {
    let dir = model_dir(root);
    ["model.safetensors", "tokenizer.json", "config.json"]
        .iter()
        .all(|f| dir.join(f).is_file())
}

fn index_path(root: &Path) -> PathBuf {
    root.join(".hick-cache").join("search").join("index.json")
}

/// One indexed chunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Chunk {
    start_line: usize,
    end_line: usize,
    text: String,
    /// Embedding under the current model; absent when indexed without one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    embedding: Option<Vec<f32>>,
}

/// Everything remembered about one file.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileEntry {
    /// FNV-1a of the file bytes — the staleness key.
    content_hash: u64,
    /// Whether `embedding`s were computed for this file's chunks.
    embedded: bool,
    chunks: Vec<Chunk>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Index {
    /// rel path → entry.
    files: HashMap<String, FileEntry>,
}

/// A ready-to-query search engine over one project root.
pub struct SearchEngine {
    root: PathBuf,
    model: Option<StaticModel>,
    index: Index,
}

impl SearchEngine {
    /// Open (and refresh) the index for `root`. Walks the tree, re-chunks
    /// and re-embeds only files whose bytes changed, and persists the
    /// result. Loads the embedding model iff its folder is populated.
    pub fn open(root: &Path) -> Result<Self> {
        let root = root
            .canonicalize()
            .with_context(|| format!("search root {} does not exist", root.display()))?;
        let model = if model_available(&root) {
            let dir = model_dir(&root);
            Some(
                StaticModel::from_pretrained(dir.to_string_lossy().as_ref(), None, None, None)
                    .with_context(|| {
                        format!(
                            "the embedding model under {} failed to load.\n  \
                             It may be a partial download — remove the folder and re-run \
                             `hick search --install-model`, or search without it (lexical \
                             search needs no model).",
                            dir.display()
                        )
                    })?,
            )
        } else {
            None
        };

        let mut engine = SearchEngine {
            index: load_index(&root),
            root,
            model,
        };
        engine.refresh()?;
        Ok(engine)
    }

    /// Whether queries will be ranked semantically as well as lexically.
    pub fn semantic(&self) -> bool {
        self.model.is_some()
    }

    /// Bring the index up to date with the tree, persisting only if
    /// something actually changed. Public so a long-lived holder (the local
    /// server) can reuse one engine across queries instead of re-parsing the
    /// index per request.
    pub fn refresh(&mut self) -> Result<()> {
        let mut fresh: HashMap<String, FileEntry> = HashMap::new();
        let mut to_embed: Vec<String> = Vec::new();
        let mut changed = false;

        for rel in walk_files(&self.root) {
            let Ok(bytes) = std::fs::read(self.root.join(&rel)) else {
                continue;
            };
            if looks_binary(&bytes) {
                continue;
            }
            let Ok(text) = String::from_utf8(bytes) else {
                continue;
            };
            let hash = fnv1a(text.as_bytes());
            match self.index.files.remove(&rel) {
                // Unchanged, and its embedding state matches what the loaded
                // model can promise: keep it as-is.
                Some(entry)
                    if entry.content_hash == hash && entry.embedded == self.model.is_some() =>
                {
                    fresh.insert(rel, entry);
                }
                _ => {
                    changed = true;
                    fresh.insert(
                        rel.clone(),
                        FileEntry {
                            content_hash: hash,
                            embedded: false,
                            chunks: chunk(&text),
                        },
                    );
                    if self.model.is_some() {
                        to_embed.push(rel);
                    }
                }
            }
        }
        // Anything left behind was deleted from the tree.
        changed |= !self.index.files.is_empty();

        if let Some(model) = &self.model {
            for rel in to_embed {
                let entry = fresh.get_mut(&rel).expect("just inserted");
                let texts: Vec<String> = entry.chunks.iter().map(|c| c.text.clone()).collect();
                let embeddings = model.encode(&texts);
                for (chunk, embedding) in entry.chunks.iter_mut().zip(embeddings) {
                    chunk.embedding = Some(embedding);
                }
                entry.embedded = true;
            }
        }

        self.index = Index { files: fresh };
        if changed { self.persist() } else { Ok(()) }
    }

    fn persist(&self) -> Result<()> {
        let path = index_path(&self.root);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("could not create {}", parent.display()))?;
        }
        let json = serde_json::to_vec(&self.index)?;
        std::fs::write(&path, json)
            .with_context(|| format!("could not write the search index {}", path.display()))?;
        Ok(())
    }

    /// Rank every chunk against `query` and return the best `top_k`.
    pub fn search(&self, query: &str, top_k: usize) -> Vec<Hit> {
        let chunks = self.all_chunks();
        if chunks.is_empty() {
            return Vec::new();
        }

        let lexical = bm25_ranking(query, &chunks);
        let semantic = self.model.as_ref().map(|model| {
            let query_embedding = model
                .encode(std::slice::from_ref(&query.to_string()))
                .into_iter()
                .next()
                .unwrap_or_default();
            cosine_ranking(&query_embedding, &chunks)
        });

        fuse(&chunks, lexical, semantic, top_k)
    }

    /// Chunks similar to the chunk containing `line` (1-based) of `rel_path`
    /// — semble's `find_related`. The anchor chunk itself is excluded.
    pub fn related(&self, rel_path: &str, line: usize, top_k: usize) -> Result<Vec<Hit>> {
        let entry = self.index.files.get(rel_path).ok_or_else(|| {
            anyhow::anyhow!(
                "{rel_path} is not in the search index.\n  \
                 The index covers text files under {} that git does not ignore. \
                 Check the path is relative to that root.",
                self.root.display()
            )
        })?;
        let anchor = entry
            .chunks
            .iter()
            .find(|c| c.start_line <= line && line <= c.end_line)
            .or(entry.chunks.last())
            .ok_or_else(|| anyhow::anyhow!("{rel_path} has no indexed content"))?;

        // The anchor's text IS the query; ask for extra results so dropping
        // overlapping anchor-adjacent chunks still leaves top_k.
        let hits = self.search(&anchor.text, top_k + 4);
        Ok(hits
            .into_iter()
            .filter(|h| {
                // Inclusive bounds on both sides, so the anchor chunk and
                // anything overlapping it are dropped.
                !(h.path == rel_path
                    && h.start_line <= anchor.end_line
                    && anchor.start_line <= h.end_line)
            })
            .take(top_k)
            .collect())
    }

    fn all_chunks(&self) -> Vec<(&str, &Chunk)> {
        let mut sorted: Vec<&String> = self.index.files.keys().collect();
        sorted.sort();
        sorted
            .into_iter()
            .flat_map(|rel| {
                self.index.files[rel]
                    .chunks
                    .iter()
                    .map(move |c| (rel.as_str(), c))
            })
            .collect()
    }
}

/// Walk `root` the way ripgrep would: honouring `.gitignore`, skipping
/// hidden files and `.hick-cache`, keeping only plausible text files.
fn walk_files(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let walker = ignore::WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        // Honour .gitignore files even when the root is not (yet) a git
        // repository — the intent of the file is the same either way.
        .require_git(false)
        .git_global(false)
        .filter_entry(|e| e.file_name() != ".hick-cache")
        .build();
    for entry in walker.flatten() {
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        if entry
            .metadata()
            .map(|m| m.len() > MAX_FILE_BYTES)
            .unwrap_or(true)
        {
            continue;
        }
        if let Ok(rel) = entry.path().strip_prefix(root) {
            // Forward slashes even on Windows: the rel path is an index key
            // and a display string, not an OS path.
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
    out.sort();
    out
}

/// A NUL in the first 4 KiB is as good a binary test as `grep` uses.
fn looks_binary(bytes: &[u8]) -> bool {
    bytes.iter().take(4096).any(|&b| b == 0)
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Split text into overlapping line windows with 1-based line bounds.
fn chunk(text: &str) -> Vec<Chunk> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return Vec::new();
    }
    let stride = CHUNK_LINES - CHUNK_OVERLAP;
    let mut chunks = Vec::new();
    let mut start = 0usize;
    loop {
        let end = (start + CHUNK_LINES).min(lines.len());
        let body = lines[start..end].join("\n");
        if !body.trim().is_empty() {
            chunks.push(Chunk {
                start_line: start + 1,
                end_line: end,
                text: body,
                embedding: None,
            });
        }
        if end == lines.len() {
            break;
        }
        start += stride;
    }
    chunks
}

/// Identifier-aware tokens: lowercase, split on non-alphanumerics AND on
/// camelCase humps, so "DocIndex" is findable as "doc index".
fn tokens(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for raw in text.split(|c: char| !c.is_alphanumeric()) {
        if raw.is_empty() {
            continue;
        }
        let mut word = String::new();
        let chars: Vec<char> = raw.chars().collect();
        for (i, &c) in chars.iter().enumerate() {
            let hump = c.is_uppercase()
                && i > 0
                && (chars[i - 1].is_lowercase()
                    || chars.get(i + 1).is_some_and(|n| n.is_lowercase()));
            if hump && !word.is_empty() {
                out.push(std::mem::take(&mut word));
            }
            word.extend(c.to_lowercase());
        }
        if !word.is_empty() {
            out.push(word);
        }
    }
    out.retain(|t| t.len() >= 2);
    out
}

/// Chunk indices ranked by BM25 (best first), scoreless chunks omitted.
fn bm25_ranking(query: &str, chunks: &[(&str, &Chunk)]) -> Vec<usize> {
    let query_tokens = tokens(query);
    if query_tokens.is_empty() {
        return Vec::new();
    }
    let docs: Vec<Vec<String>> = chunks.iter().map(|(_, c)| tokens(&c.text)).collect();
    let n = docs.len() as f32;
    let avg_len = (docs.iter().map(Vec::len).sum::<usize>() as f32 / n).max(1.0);

    let mut df: HashMap<&str, usize> = HashMap::new();
    for doc in &docs {
        let mut seen: Vec<&str> = doc.iter().map(String::as_str).collect();
        seen.sort_unstable();
        seen.dedup();
        for t in seen {
            *df.entry(t).or_default() += 1;
        }
    }

    let (k1, b) = (1.2f32, 0.75f32);
    let mut scored: Vec<(usize, f32)> = docs
        .iter()
        .enumerate()
        .map(|(i, doc)| {
            let len = doc.len() as f32;
            let mut score = 0.0;
            for qt in &query_tokens {
                let tf = doc.iter().filter(|t| *t == qt).count() as f32;
                if tf == 0.0 {
                    continue;
                }
                let dfq = *df.get(qt.as_str()).unwrap_or(&0) as f32;
                let idf = ((n - dfq + 0.5) / (dfq + 0.5) + 1.0).ln();
                score += idf * (tf * (k1 + 1.0)) / (tf + k1 * (1.0 - b + b * len / avg_len));
            }
            (i, score)
        })
        .filter(|(_, s)| *s > 0.0)
        .collect();
    scored.sort_by(|a, b| b.1.total_cmp(&a.1));
    scored.into_iter().map(|(i, _)| i).collect()
}

/// Chunk indices ranked by cosine similarity (best first).
fn cosine_ranking(query: &[f32], chunks: &[(&str, &Chunk)]) -> Vec<usize> {
    let mut scored: Vec<(usize, f32)> = chunks
        .iter()
        .enumerate()
        .filter_map(|(i, (_, c))| c.embedding.as_ref().map(|e| (i, cosine(query, e))))
        .collect();
    scored.sort_by(|a, b| b.1.total_cmp(&a.1));
    scored.into_iter().map(|(i, _)| i).collect()
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}

/// Reciprocal-rank fusion of the available rankings.
fn fuse(
    chunks: &[(&str, &Chunk)],
    lexical: Vec<usize>,
    semantic: Option<Vec<usize>>,
    top_k: usize,
) -> Vec<Hit> {
    let mut score: HashMap<usize, f32> = HashMap::new();
    for ranking in std::iter::once(lexical).chain(semantic) {
        for (rank, idx) in ranking.into_iter().enumerate() {
            *score.entry(idx).or_default() += 1.0 / (RRF_K + rank as f32 + 1.0);
        }
    }
    let mut ranked: Vec<(usize, f32)> = score.into_iter().collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    ranked
        .into_iter()
        .take(top_k)
        .map(|(i, s)| {
            let (path, chunk) = chunks[i];
            Hit {
                path: path.to_string(),
                start_line: chunk.start_line,
                end_line: chunk.end_line,
                score: s,
                snippet: chunk.text.clone(),
            }
        })
        .collect()
}

fn load_index(root: &Path) -> Index {
    // A cache that fails to load is an empty cache: the refresh rebuilds it.
    std::fs::read(index_path(root))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

/// The files a model install must produce, with their upstream names — one
/// list shared by the CLI's downloader and [`model_available`].
pub const MODEL_FILES: &[&str] = &["model.safetensors", "tokenizer.json", "config.json"];

/// The default model repo the CLI installs from. potion-base-8M is
/// MinishLab's recommended general-purpose static model: ~30 MB on disk,
/// CPU-only, no runtime beyond this crate.
pub const DEFAULT_MODEL_REPO: &str = "minishlab/potion-base-8M";

/// Validate a `file:line` reference for `related` lookups.
pub fn parse_file_line(spec: &str) -> Result<(String, usize)> {
    let Some((file, line)) = spec.rsplit_once(':') else {
        bail!(
            "expected FILE:LINE (e.g. src/app.py:42), got '{spec}'.\n  \
             The line number anchors which part of the file to find related code for."
        );
    };
    let line: usize = line.parse().with_context(|| {
        format!("'{line}' is not a line number — expected FILE:LINE, e.g. src/app.py:42")
    })?;
    Ok((file.to_string(), line))
}

// Protects docs/guarantees/search/search-is-offline-by-default.md
#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &Path, rel: &str, text: &str) {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    #[test]
    fn tokens_split_identifiers() {
        assert_eq!(
            tokens("DocIndex.path_of"),
            vec!["doc", "index", "path", "of"]
        );
        assert_eq!(tokens("HTTPServer"), vec!["http", "server"]);
    }

    #[test]
    fn chunks_carry_one_based_line_bounds_and_overlap() {
        let text = (1..=100)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let chunks = chunk(&text);
        assert_eq!(chunks[0].start_line, 1);
        assert_eq!(chunks[0].end_line, 40);
        assert_eq!(chunks[1].start_line, 33);
        assert_eq!(chunks.last().unwrap().end_line, 100);
    }

    #[test]
    fn lexical_search_finds_the_right_file() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "a/parser.rs",
            "fn parse_document(input: &str) {}\n",
        );
        write(
            dir.path(),
            "b/render.rs",
            "fn render_output(html: &str) {}\n",
        );
        let engine = SearchEngine::open(dir.path()).unwrap();
        assert!(!engine.semantic());
        let hits = engine.search("parse a document", 5);
        assert_eq!(hits[0].path, "a/parser.rs");
        assert_eq!(hits[0].start_line, 1);
    }

    #[test]
    fn the_index_is_cached_and_survives_reopen() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "x.txt", "alpha beta gamma\n");
        let _ = SearchEngine::open(dir.path()).unwrap();
        let cache = index_path(&dir.path().canonicalize().unwrap());
        assert!(
            cache.is_file(),
            "index not persisted at {}",
            cache.display()
        );
        let engine = SearchEngine::open(dir.path()).unwrap();
        assert_eq!(engine.search("alpha", 3).len(), 1);
    }

    #[test]
    fn gitignored_files_stay_out_of_the_index() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), ".gitignore", "generated.log\n");
        write(dir.path(), "kept.txt", "needle here\n");
        write(dir.path(), "generated.log", "needle here too\n");
        let engine = SearchEngine::open(dir.path()).unwrap();
        let hits = engine.search("needle", 10);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].path, "kept.txt");
    }

    #[test]
    fn related_excludes_the_anchor_chunk() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "a.rs", "fn compute_totals(rows: &[Row]) {}\n");
        write(
            dir.path(),
            "b.rs",
            "fn compute_grand_totals(rows: &[Row]) {}\n",
        );
        let engine = SearchEngine::open(dir.path()).unwrap();
        let hits = engine.related("a.rs", 1, 5).unwrap();
        assert!(hits.iter().all(|h| h.path != "a.rs"));
        assert_eq!(hits[0].path, "b.rs");
    }

    #[test]
    fn file_line_specs_parse_and_fail_helpfully() {
        assert_eq!(
            parse_file_line("src/x.py:42").unwrap(),
            ("src/x.py".into(), 42)
        );
        let err = parse_file_line("src/x.py").unwrap_err().to_string();
        assert!(err.contains("FILE:LINE"), "{err}");
    }
}
