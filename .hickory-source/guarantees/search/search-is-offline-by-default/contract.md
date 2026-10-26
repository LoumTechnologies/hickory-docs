<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Search Works Offline; Only An Explicit Install Touches The Network

For: Authors finding and replacing project text without missing results.

Given a machine with no network access, when `hick search "query"` (or the
app's search panel, or the MCP `search` tool) runs, then it answers from a
locally built index with lexical (BM25) ranking, builds and caches that index
under `.hick-cache/search/`, and never attempts a network request.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/search/search-is-offline-by-default.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-7cc0c55f9b04"></a>
## `apps/web/src/lib/workspaceSearch.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Search Works Offline; Only An Explicit Install Touches The Network](contract.md).

<a id="f-7cc0c55f9b04-hit"></a>
### `hit`

<storycode:copy id="f-7cc0c55f9b04-hit">
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { describe, expect, it, vi } from "vitest";
import { openPlainSearchFiles, registerSearchEditor, searchWorkspace } from "./workspaceSearch";

const hit = (path: string, snippet: string) => ({ path, snippet, start_line: 1, end_line: 1, score: 1 });

</storycode:copy>
<a id="f-7cc0c55f9b04-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-7cc0c55f9b04-expression_statement">
describe("searching open files and the folder", () => {
  it("searches every open file without a folder", async () => {
    const found = await searchWorkspace("invoice", 20, [
      { path: "/notes/one.md", content: "Introduction\nInvoice pending" },
      { path: "/elsewhere/two.txt", content: "Invoice paid" },
    ]);
    expect(found.hits.map((h) => h.path).sort()).toEqual(["/elsewhere/two.txt", "/notes/one.md"]);
    expect(found.hits.find((h) => h.path === "/notes/one.md")?.start_line).toBe(2);
  });

  it("includes folder hits and open files outside the folder", async () => {
    const folder = vi.fn().mockResolvedValue({ semantic: true, hits: [hit("closed.md", "invoice")] });
    const found = await searchWorkspace("invoice", 20, [{ path: "/outside/open.txt", content: "invoice" }], folder);
    expect(found.hits.map((h) => h.path)).toEqual(["/outside/open.txt", "closed.md"]);
    expect(found.semantic).toBe(true);
    expect(folder).toHaveBeenCalledWith("invoice", 50);
  });

  it("replaces disk hits with unsaved contents, even when the new contents no longer match", async () => {
    const folder = async () => ({ semantic: false, hits: [hit("open.md", "old invoice"), hit("closed.md", "invoice")] });
    const found = await searchWorkspace("invoice", 20, [{ path: "/project/open.md", content: "No bills here" }], folder);
    expect(found.hits.map((h) => h.path)).toEqual(["closed.md"]);
    const updated = await searchWorkspace("invoice", 20, [{ path: "/project/open.md", content: "New invoice" }], folder);
    expect(updated.hits.filter((h) => h.path.endsWith("open.md"))).toEqual([hit("/project/open.md", "New invoice")].map((h) => ({ ...h, score: expect.any(Number) })));
  });

  it("keeps an external file distinct from a folder file with the same name", async () => {
    const folder = async () => ({ semantic: false, hits: [hit("open.md", "folder invoice")] });
    const found = await searchWorkspace("invoice", 20, [
      { path: "/outside/open.md", content: "external invoice" },
    ], folder, "/project");
    expect(found.hits.map((h) => h.path)).toEqual(["/outside/open.md", "open.md"]);
  });

  it("reads live editor contents and drops closed editors", () => {
    const view = new EditorView({ state: EditorState.create({ doc: "saved" }) });
    const unregister = registerSearchEditor("open.txt", view);
    try {
      view.dispatch({ changes: { from: 0, to: 5, insert: "unsaved invoice" } });
      expect(openPlainSearchFiles()).toContainEqual({ path: "open.txt", content: "unsaved invoice" });
      unregister();
      expect(openPlainSearchFiles()).toEqual([]);
    } finally {
      unregister();
      view.destroy();
    }
  });

  it("deduplicates open paths and respects the result limit", async () => {
    const found = await searchWorkspace("invoice", 1, [
      { path: "/project/open.md", content: "invoice" },
      { path: "open.md", content: "invoice" },
    ]);
    expect(found.hits).toHaveLength(1);
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/workspaceSearch.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/workspaceSearch.test.ts"><storycode:paste select="#f-7cc0c55f9b04-hit"/><storycode:paste select="#f-7cc0c55f9b04-expression_statement"/></storycode:file>

<a id="output-4c0f66567348"></a>
## `crates/hick-search/src/lib.rs`

Related contracts: [Search Works Offline; Only An Explicit Install Touches The Network](contract.md).

<a id="f-4c0f66567348-CHUNK_LINES"></a>
### `CHUNK_LINES`

Project-wide search over a hick folder, shaped like
semble: ask in natural language or
code, get ranked chunks with exact file and line, plus "find code related
to this file:line". First-party, no third-party CLI.
Two rankers, one seam:
- **Lexical** — BM25 over identifier-aware tokens (camelCase and
snake_case split). Always available, fully offline, needs nothing.
- **Semantic** — Model2Vec static embeddings (the engine semble uses),
loaded from a model folder on disk. Optional: when the folder is
absent, search is lexical and says nothing about it. Nothing here
touches the network — downloading a model is the CLI's explicit,
never-automatic job, the same doctrine as `hick lsp install`.
When both rankers run, results are fused with reciprocal-rank fusion, so
neither scale has to be calibrated against the other.
The index lives in `.hick-cache/search/index.json`, keyed by content
hash: an unchanged file is never re-chunked or re-embedded, so the first
search pays for the build and the rest are incremental — semble's cached
index, implemented here.

<storycode:copy id="f-4c0f66567348-CHUNK_LINES">
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
</storycode:copy>
<a id="f-4c0f66567348-CHUNK_OVERLAP"></a>
### `CHUNK_OVERLAP`

Overlap between consecutive chunks, so a match spanning a boundary is
still inside one chunk.

<storycode:copy id="f-4c0f66567348-CHUNK_OVERLAP">
/// Overlap between consecutive chunks, so a match spanning a boundary is
/// still inside one chunk.
const CHUNK_OVERLAP: usize = 8;
</storycode:copy>
<a id="f-4c0f66567348-MAX_FILE_BYTES"></a>
### `MAX_FILE_BYTES`

Files larger than this are skipped: they are generated blobs or data, and
a chunk inside one is rarely what anyone is searching for.

<storycode:copy id="f-4c0f66567348-MAX_FILE_BYTES">
/// Files larger than this are skipped: they are generated blobs or data, and
/// a chunk inside one is rarely what anyone is searching for.
const MAX_FILE_BYTES: u64 = 512 * 1024;
</storycode:copy>
<a id="f-4c0f66567348-MIN_COMPLETION_PREFIX"></a>
### `MIN_COMPLETION_PREFIX`

Below this many characters, every identifier in the project matches.

<storycode:copy id="f-4c0f66567348-MIN_COMPLETION_PREFIX">
/// Below this many characters, every identifier in the project matches.
const MIN_COMPLETION_PREFIX: usize = 2;

</storycode:copy>
<a id="f-4c0f66567348-SEMANTIC_WEIGHT"></a>
### `SEMANTIC_WEIGHT`

How much a chunk's closeness to the current context lifts its tokens.
Chosen so semantics REORDER a frequency ranking rather than replace it: a
token used fifty times still beats one used twice in a nearby chunk, which
is right, because the common name is usually the one wanted. Turning this
up makes the list follow the cursor around and stop being predictable.

<storycode:copy id="f-4c0f66567348-SEMANTIC_WEIGHT">
/// How much a chunk's closeness to the current context lifts its tokens.
///
/// Chosen so semantics REORDER a frequency ranking rather than replace it: a
/// token used fifty times still beats one used twice in a nearby chunk, which
/// is right, because the common name is usually the one wanted. Turning this
/// up makes the list follow the cursor around and stop being predictable.
const SEMANTIC_WEIGHT: f32 = 2.0;

</storycode:copy>
<a id="f-4c0f66567348-Suggestion"></a>
### `Suggestion`

One completion drawn from the project's own text.

<storycode:copy id="f-4c0f66567348-Suggestion">
/// One completion drawn from the project's own text.
#[derive(Debug, Clone, Serialize)]
pub struct Suggestion {
    /// The identifier to insert.
    pub text: String,
    /// Where it was first seen — `path:line`, for the popup's second line.
    pub detail: String,
    /// Comparable only within one list.
    pub score: f32,
    /// Whether the embedding model contributed to the ranking. False means
    /// frequency alone, which is still useful and still offline — and the UI
    /// says so rather than claiming more than happened.
    pub semantic: bool,
}

</storycode:copy>
<a id="f-4c0f66567348-identifiers"></a>
### `identifiers`

The identifier-shaped tokens of a piece of text.
Deliberately language-agnostic: a letter or underscore followed by letters,
digits and underscores. That is the identifier rule of nearly every
language this app runs, and being approximately right for all of them is
worth more here than being exactly right for one.

<storycode:copy id="f-4c0f66567348-identifiers">
/// The identifier-shaped tokens of a piece of text.
///
/// Deliberately language-agnostic: a letter or underscore followed by letters,
/// digits and underscores. That is the identifier rule of nearly every
/// language this app runs, and being approximately right for all of them is
/// worth more here than being exactly right for one.
fn identifiers(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|token| {
            !token.is_empty()
                && token.len() >= MIN_COMPLETION_PREFIX
                && token
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_alphabetic() || c == '_')
        })
}

</storycode:copy>
<a id="f-4c0f66567348-RRF_K"></a>
### `RRF_K`

Reciprocal-rank-fusion constant (the standard 60 from the RRF paper).

<storycode:copy id="f-4c0f66567348-RRF_K">
/// Reciprocal-rank-fusion constant (the standard 60 from the RRF paper).
const RRF_K: f32 = 60.0;

</storycode:copy>
<a id="f-4c0f66567348-Hit"></a>
### `Hit`

One search hit: a chunk of one file, with 1-based line bounds.

<storycode:copy id="f-4c0f66567348-Hit">
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

</storycode:copy>
<a id="f-4c0f66567348-model_dir"></a>
### `model_dir`

Where the optional embedding model lives for a project root.
Under `.hick-cache` so `hick init`'s gitignore entry already covers it,
and per-project so deleting a project leaves nothing behind.

<storycode:copy id="f-4c0f66567348-model_dir">
/// Where the optional embedding model lives for a project root.
///
/// Under `.hick-cache` so `hick init`'s gitignore entry already covers it,
/// and per-project so deleting a project leaves nothing behind.
pub fn model_dir(root: &Path) -> PathBuf {
    root.join(".hick-cache").join("models").join("embed")
}

</storycode:copy>
<a id="f-4c0f66567348-model_available"></a>
### `model_available`

Whether the model folder holds what [`StaticModel`] needs to load.

<storycode:copy id="f-4c0f66567348-model_available">
/// Whether the model folder holds what [`StaticModel`] needs to load.
pub fn model_available(root: &Path) -> bool {
    let dir = model_dir(root);
    ["model.safetensors", "tokenizer.json", "config.json"]
        .iter()
        .all(|f| dir.join(f).is_file())
}

</storycode:copy>
<a id="f-4c0f66567348-index_path"></a>
### `index_path`

<storycode:copy id="f-4c0f66567348-index_path">
fn index_path(root: &Path) -> PathBuf {
    root.join(".hick-cache").join("search").join("index.json")
}

</storycode:copy>
<a id="f-4c0f66567348-Chunk"></a>
### `Chunk`

<storycode:copy id="f-4c0f66567348-Chunk">
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

</storycode:copy>
<a id="f-4c0f66567348-FileEntry"></a>
### `FileEntry`

<storycode:copy id="f-4c0f66567348-FileEntry">
/// Everything remembered about one file.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileEntry {
    /// FNV-1a of the file bytes — the staleness key.
    content_hash: u64,
    /// Whether `embedding`s were computed for this file's chunks.
    embedded: bool,
    chunks: Vec<Chunk>,
}

</storycode:copy>
<a id="f-4c0f66567348-Index"></a>
### `Index`

<storycode:copy id="f-4c0f66567348-Index">
#[derive(Debug, Default, Serialize, Deserialize)]
struct Index {
    /// rel path → entry.
    files: HashMap<String, FileEntry>,
}

</storycode:copy>
<a id="f-4c0f66567348-SearchEngine"></a>
### `SearchEngine`

A ready-to-query search engine over one project root.

<storycode:copy id="f-4c0f66567348-SearchEngine">
/// A ready-to-query search engine over one project root.
pub struct SearchEngine {
    root: PathBuf,
    model: Option<StaticModel>,
    index: Index,
}

</storycode:copy>
<a id="f-4c0f66567348-impl-SearchEngine"></a>
### `impl-SearchEngine`

<storycode:copy id="f-4c0f66567348-impl-SearchEngine">
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

    /// Completions drawn from this project's own text.
    ///
    /// # What this is, and what it is not
    ///
    /// It is not a language model, and this file is careful never to imply
    /// one. It is retrieval: the identifiers this codebase actually uses,
    /// ranked by how often they appear and — when the embedding model is
    /// installed — by how close their surroundings are to what is being
    /// typed right now.
    ///
    /// That is a genuinely different kind of answer from a language server's,
    /// which is the whole reason it is worth showing beside one. An LSP knows
    /// what is *in scope* and what its type is; it has no opinion about
    /// whether this codebase calls the thing `cfg`, `config` or `settings`.
    /// This does, and knows nothing about types. Neither subsumes the other,
    /// which is why the popup has to say which is speaking.
    ///
    /// Degrades in one step: with no model installed the ranking is frequency
    /// alone, which is still useful and still offline. `semantic` on each
    /// suggestion says which happened, so the UI never claims more than it
    /// did.
    pub fn completions(&self, prefix: &str, context: &str, top_k: usize) -> Vec<Suggestion> {
        if prefix.len() < MIN_COMPLETION_PREFIX {
            // Below this every identifier in the project matches, which is a
            // list nobody reads and a request nobody meant.
            return Vec::new();
        }
        let chunks = self.all_chunks();
        if chunks.is_empty() {
            return Vec::new();
        }

        // How close each chunk is to what is being typed. Without a model
        // every chunk is equally close, which reduces the ranking to
        // frequency — the honest fallback rather than a worse guess.
        let affinity: Vec<f32> = match &self.model {
            Some(model) => {
                let query = model
                    .encode(std::slice::from_ref(&context.to_string()))
                    .into_iter()
                    .next()
                    .unwrap_or_default();
                chunks
                    .iter()
                    .map(|(_, chunk)| {
                        chunk
                            .embedding
                            .as_ref()
                            .map(|e| cosine(&query, e).max(0.0))
                            .unwrap_or(0.0)
                    })
                    .collect()
            }
            None => vec![0.0; chunks.len()],
        };

        // token -> (score, where it was first seen)
        let mut scores: HashMap<String, (f32, String)> = HashMap::new();
        for (index, (path, chunk)) in chunks.iter().enumerate() {
            // A chunk's affinity lifts every token in it. `1.0 +` so that
            // with no model the weight is exactly one and the ranking is a
            // pure count.
            let weight = 1.0 + affinity[index] * SEMANTIC_WEIGHT;
            for token in identifiers(&chunk.text) {
                if token.len() <= prefix.len() || !token.starts_with(prefix) {
                    continue;
                }
                let entry = scores
                    .entry(token.to_string())
                    .or_insert_with(|| (0.0, format!("{path}:{}", chunk.start_line)));
                entry.0 += weight;
            }
        }

        let mut out: Vec<Suggestion> = scores
            .into_iter()
            .map(|(text, (score, where_))| Suggestion {
                text,
                detail: where_,
                score,
                semantic: self.model.is_some(),
            })
            .collect();
        // Score first, then alphabetical, so a tie is stable between runs —
        // a completion list that reshuffles on every keystroke is one nobody
        // can build muscle memory against.
        out.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.text.cmp(&b.text))
        });
        out.truncate(top_k);
        out
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

</storycode:copy>
<a id="f-4c0f66567348-walk_files"></a>
### `walk_files`

Walk `root` the way ripgrep would: honouring `.gitignore`, skipping
hidden files and `.hick-cache`, keeping only plausible text files.

<storycode:copy id="f-4c0f66567348-walk_files">
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

</storycode:copy>
<a id="f-4c0f66567348-looks_binary"></a>
### `looks_binary`

A NUL in the first 4 KiB is as good a binary test as `grep` uses.

<storycode:copy id="f-4c0f66567348-looks_binary">
/// A NUL in the first 4 KiB is as good a binary test as `grep` uses.
fn looks_binary(bytes: &[u8]) -> bool {
    bytes.iter().take(4096).any(|&b| b == 0)
}

</storycode:copy>
<a id="f-4c0f66567348-fnv1a"></a>
### `fnv1a`

<storycode:copy id="f-4c0f66567348-fnv1a">
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

</storycode:copy>
<a id="f-4c0f66567348-chunk"></a>
### `chunk`

Split text into overlapping line windows with 1-based line bounds.

<storycode:copy id="f-4c0f66567348-chunk">
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

</storycode:copy>
<a id="f-4c0f66567348-tokens"></a>
### `tokens`

Identifier-aware tokens: lowercase, split on non-alphanumerics AND on
camelCase humps, so "DocIndex" is findable as "doc index".

<storycode:copy id="f-4c0f66567348-tokens">
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

</storycode:copy>
<a id="f-4c0f66567348-bm25_ranking"></a>
### `bm25_ranking`

Chunk indices ranked by BM25 (best first), scoreless chunks omitted.

<storycode:copy id="f-4c0f66567348-bm25_ranking">
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

</storycode:copy>
<a id="f-4c0f66567348-cosine_ranking"></a>
### `cosine_ranking`

Chunk indices ranked by cosine similarity (best first).

<storycode:copy id="f-4c0f66567348-cosine_ranking">
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

</storycode:copy>
<a id="f-4c0f66567348-cosine"></a>
### `cosine`

<storycode:copy id="f-4c0f66567348-cosine">
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

</storycode:copy>
<a id="f-4c0f66567348-fuse"></a>
### `fuse`

Reciprocal-rank fusion of the available rankings.

<storycode:copy id="f-4c0f66567348-fuse">
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

</storycode:copy>
<a id="f-4c0f66567348-load_index"></a>
### `load_index`

<storycode:copy id="f-4c0f66567348-load_index">
fn load_index(root: &Path) -> Index {
    // A cache that fails to load is an empty cache: the refresh rebuilds it.
    std::fs::read(index_path(root))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

</storycode:copy>
<a id="f-4c0f66567348-MODEL_FILES"></a>
### `MODEL_FILES`

The files a model install must produce, with their upstream names — one
list shared by the CLI's downloader and [`model_available`].

<storycode:copy id="f-4c0f66567348-MODEL_FILES">
/// The files a model install must produce, with their upstream names — one
/// list shared by the CLI's downloader and [`model_available`].
pub const MODEL_FILES: &[&str] = &["model.safetensors", "tokenizer.json", "config.json"];

</storycode:copy>
<a id="f-4c0f66567348-DEFAULT_MODEL_REPO"></a>
### `DEFAULT_MODEL_REPO`

The default model repo the CLI installs from. potion-base-8M is
MinishLab's recommended general-purpose static model: ~30 MB on disk,
CPU-only, no runtime beyond this crate.

<storycode:copy id="f-4c0f66567348-DEFAULT_MODEL_REPO">
/// The default model repo the CLI installs from. potion-base-8M is
/// MinishLab's recommended general-purpose static model: ~30 MB on disk,
/// CPU-only, no runtime beyond this crate.
pub const DEFAULT_MODEL_REPO: &str = "minishlab/potion-base-8M";

</storycode:copy>
<a id="f-4c0f66567348-parse_file_line"></a>
### `parse_file_line`

Validate a `file:line` reference for `related` lookups.

<storycode:copy id="f-4c0f66567348-parse_file_line">
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

</storycode:copy>
<a id="f-4c0f66567348-tests"></a>
### `tests`

Protects docs/guarantees/search/search-is-offline-by-default.md

<storycode:copy id="f-4c0f66567348-tests">
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
    fn lexical_search_finds_case_insensitive_parts_of_identifiers_in_hick_files() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "notes/program.md",
            "<hick:file path=\"Program.cs\">\nConsole.WriteLine(\"hello\");\n</hick:file>\n",
        );

        let engine = SearchEngine::open(dir.path()).unwrap();
        for query in ["write", "Write"] {
            let hits = engine.search(query, 5);
            assert!(
                hits.iter().any(|hit| hit.path == "notes/program.md"),
                "{query:?} did not find the hick:file body: {hits:#?}"
            );
        }
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

</storycode:copy>
<a id="f-4c0f66567348-completion_tests"></a>
### `completion_tests`

<storycode:copy id="f-4c0f66567348-completion_tests">
#[cfg(test)]
mod completion_tests {
    use super::*;

    fn project(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for (path, body) in files {
            let full = dir.path().join(path);
            if let Some(parent) = full.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(full, body).unwrap();
        }
        dir
    }

    fn engine(dir: &tempfile::TempDir) -> SearchEngine {
        SearchEngine::open(dir.path()).expect("the index opens")
    }

    #[test]
    fn identifiers_are_split_the_way_nearly_every_language_agrees() {
        let found: Vec<&str> = identifiers("fn read_config(path: &Path) -> Config { }").collect();
        assert!(found.contains(&"read_config"), "{found:?}");
        assert!(found.contains(&"Config"), "{found:?}");
        // Not a keyword filter — `fn` is two characters and survives, which
        // is fine: a two-character prefix will not match it anyway.
        assert!(!found.contains(&"&"), "{found:?}");
    }

    #[test]
    fn a_token_that_starts_with_a_digit_is_not_an_identifier() {
        let found: Vec<&str> = identifiers("x = 3px + total").collect();
        assert!(!found.contains(&"3px"), "{found:?}");
        assert!(found.contains(&"total"), "{found:?}");
    }

    #[test]
    fn a_prefix_too_short_to_narrow_anything_returns_nothing() {
        // Every identifier in the project matches one letter; that is a list
        // nobody reads and a request nobody meant.
        let dir = project(&[("a.py", "total_units = 1\n")]);
        assert!(engine(&dir).completions("t", "", 10).is_empty());
    }

    #[test]
    fn suggestions_come_from_the_projects_own_text() {
        // The whole point: the names THIS codebase uses, which a language
        // server has no opinion about.
        let dir = project(&[(
            "a.py",
            "total_units = 1\ntotal_revenue = 2\nunrelated = 3\n",
        )]);
        let found = engine(&dir).completions("tot", "", 10);
        let names: Vec<&str> = found.iter().map(|s| s.text.as_str()).collect();
        assert!(names.contains(&"total_units"), "{names:?}");
        assert!(names.contains(&"total_revenue"), "{names:?}");
        assert!(!names.contains(&"unrelated"), "{names:?}");
    }

    #[test]
    fn the_prefix_itself_is_not_suggested() {
        // Completing `total` to `total` is not a completion.
        let dir = project(&[("a.py", "total = 1\ntotal_units = 2\n")]);
        let found = engine(&dir).completions("total", "", 10);
        assert!(found.iter().all(|s| s.text != "total"), "{found:?}");
    }

    #[test]
    fn the_more_a_name_is_used_the_higher_it_ranks() {
        // With no model this is the whole ranking, and it is the right one:
        // the common name is usually the one wanted.
        let dir = project(&[(
            "a.py",
            "widget_count\nwidget_count\nwidget_count\nwidget_total\n",
        )]);
        let found = engine(&dir).completions("widget", "", 10);
        assert_eq!(found[0].text, "widget_count", "{found:?}");
    }

    #[test]
    fn a_tie_is_broken_alphabetically_so_the_list_does_not_reshuffle() {
        // A completion list that reorders on every keystroke is one nobody
        // can build muscle memory against.
        let dir = project(&[("a.py", "alpha_one\nalpha_two\n")]);
        let found = engine(&dir).completions("alpha", "", 10);
        assert_eq!(
            found.iter().map(|s| s.text.as_str()).collect::<Vec<_>>(),
            vec!["alpha_one", "alpha_two"]
        );
    }

    #[test]
    fn a_suggestion_says_where_it_came_from() {
        let dir = project(&[("src/app.py", "total_units = 1\n")]);
        let found = engine(&dir).completions("tot", "", 10);
        assert!(found[0].detail.contains("src/app.py"), "{found:?}");
    }

    #[test]
    fn without_the_model_it_says_so_rather_than_claiming_semantics() {
        // The UI must never claim more than happened.
        let dir = project(&[("a.py", "total_units = 1\n")]);
        let engine = engine(&dir);
        assert!(!engine.semantic());
        assert!(
            engine
                .completions("tot", "", 10)
                .iter()
                .all(|s| !s.semantic)
        );
    }

    #[test]
    fn an_empty_project_suggests_nothing_rather_than_failing() {
        let dir = project(&[]);
        assert!(engine(&dir).completions("tot", "", 10).is_empty());
    }
}
</storycode:copy>

### Assemble `crates/hick-search/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-search/src/lib.rs"><storycode:paste select="#f-4c0f66567348-CHUNK_LINES"/><storycode:paste select="#f-4c0f66567348-CHUNK_OVERLAP"/><storycode:paste select="#f-4c0f66567348-MAX_FILE_BYTES"/><storycode:paste select="#f-4c0f66567348-MIN_COMPLETION_PREFIX"/><storycode:paste select="#f-4c0f66567348-SEMANTIC_WEIGHT"/><storycode:paste select="#f-4c0f66567348-Suggestion"/><storycode:paste select="#f-4c0f66567348-identifiers"/><storycode:paste select="#f-4c0f66567348-RRF_K"/><storycode:paste select="#f-4c0f66567348-Hit"/><storycode:paste select="#f-4c0f66567348-model_dir"/><storycode:paste select="#f-4c0f66567348-model_available"/><storycode:paste select="#f-4c0f66567348-index_path"/><storycode:paste select="#f-4c0f66567348-Chunk"/><storycode:paste select="#f-4c0f66567348-FileEntry"/><storycode:paste select="#f-4c0f66567348-Index"/><storycode:paste select="#f-4c0f66567348-SearchEngine"/><storycode:paste select="#f-4c0f66567348-impl-SearchEngine"/><storycode:paste select="#f-4c0f66567348-walk_files"/><storycode:paste select="#f-4c0f66567348-looks_binary"/><storycode:paste select="#f-4c0f66567348-fnv1a"/><storycode:paste select="#f-4c0f66567348-chunk"/><storycode:paste select="#f-4c0f66567348-tokens"/><storycode:paste select="#f-4c0f66567348-bm25_ranking"/><storycode:paste select="#f-4c0f66567348-cosine_ranking"/><storycode:paste select="#f-4c0f66567348-cosine"/><storycode:paste select="#f-4c0f66567348-fuse"/><storycode:paste select="#f-4c0f66567348-load_index"/><storycode:paste select="#f-4c0f66567348-MODEL_FILES"/><storycode:paste select="#f-4c0f66567348-DEFAULT_MODEL_REPO"/><storycode:paste select="#f-4c0f66567348-parse_file_line"/><storycode:paste select="#f-4c0f66567348-tests"/><storycode:paste select="#f-4c0f66567348-completion_tests"/></storycode:file>

<a id="output-4bc9d64cc4ce"></a>
## `crates/hickory-cli/src/search_install.rs`

Related contracts: [Search Works Offline; Only An Explicit Install Touches The Network](contract.md).

<a id="f-4bc9d64cc4ce-install_model"></a>
### `install_model`

`hick search --install-model` — the one place search touches the network.
Same doctrine as `hick lsp install`: fetching things is never automatic.
Search works offline (lexically) from the first run; this download is an
explicit upgrade to semantic ranking, and everything lands under
`.hick-cache/` which `hick init` already keeps out of git.

<storycode:copy id="f-4bc9d64cc4ce-install_model">
//! `hick search --install-model` — the one place search touches the network.
//!
//! Same doctrine as `hick lsp install`: fetching things is never automatic.
//! Search works offline (lexically) from the first run; this download is an
//! explicit upgrade to semantic ranking, and everything lands under
//! `.hick-cache/` which `hick init` already keeps out of git.

use std::path::Path;

use anyhow::{Context as _, Result, bail};

/// Download the default Model2Vec model into the project's model folder.
/// Idempotent: files already present are kept, a partial download never
/// replaces a good file (temp + rename).
pub async fn install_model(root: &Path) -> Result<()> {
    let dir = hick_search::model_dir(root);
    std::fs::create_dir_all(&dir).with_context(|| format!("could not create {}", dir.display()))?;

    let client = reqwest::Client::new();
    for file in hick_search::MODEL_FILES {
        let target = dir.join(file);
        if target.is_file() {
            println!("  {file} — already present");
            continue;
        }
        let url = format!(
            "https://huggingface.co/{}/resolve/main/{file}",
            hick_search::DEFAULT_MODEL_REPO
        );
        println!("  fetching {url}");
        let response = client.get(&url).send().await.with_context(|| {
            format!(
                "could not reach huggingface.co to fetch {file}.\n  \
                 Search still works without the model (lexical ranking); re-run \
                 `hick search --install-model` when you are online."
            )
        })?;
        if !response.status().is_success() {
            bail!(
                "downloading {file} failed: HTTP {} from {url}\n  \
                 If the model repo moved, this binary is out of date — check for a \
                 newer release. Search still works without the model.",
                response.status()
            );
        }
        let bytes = response.bytes().await.context("download interrupted")?;
        let tmp = dir.join(format!("{file}.partial"));
        std::fs::write(&tmp, &bytes)
            .with_context(|| format!("could not write {}", tmp.display()))?;
        std::fs::rename(&tmp, &target)
            .with_context(|| format!("could not move {} into place", tmp.display()))?;
        println!("  {file} — {} KiB", bytes.len() / 1024);
    }

    println!(
        "Model installed under {}. Searches in this project now rank \
         semantically as well as lexically.",
        dir.display()
    );
    Ok(())
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/search_install.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/search_install.rs"><storycode:paste select="#f-4bc9d64cc4ce-install_model"/></storycode:file>

## Implementation review

### Implementation

- [crates/hick-search/src/lib.rs](contract.md#output-4c0f66567348)
- [crates/hickory-cli/src/main.rs](../../../mechanisms/crates/hickory-cli/src/main.rs.md#output-5607f2721a2d)
- [crates/hickory-cli/src/mcp.rs](../../../mechanisms/crates/hickory-cli/src/mcp.rs.md#output-5cbce5960257)
- [crates/hickory-cli/src/search_install.rs](contract.md#output-4bc9d64cc4ce)
- [crates/hickory-cli/src/serve/api.rs](../../../mechanisms/crates/hickory-cli/src/serve/api.rs.md#output-e71b04ae5298)

### Tests and fixtures

- [apps/web/src/lib/workspaceSearch.test.ts](contract.md#output-7cc0c55f9b04)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
