<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Index Answers In Documents

For: Authors using completion, diagnostics and refactoring inside literate source.

Given a project with a SCIP index, when a person asks where a name is used,
then every answer is a place they can actually edit:

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/editor-intelligence/an-index-answers-in-documents.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-b72540443cf7"></a>
## `crates/hickory-cli/src/index_install.rs`

Related contracts: [An Index Answers In Documents](contract.md).

<a id="f-b72540443cf7-INDEXERS_DIR"></a>
### `INDEXERS_DIR`

Which code indexers `hick index install` can fetch, and how one is run.
`docs/specs/freeform/an-index-beside-the-language-server.md`.
**SCIP is in addition to LSP, never in place of it.** A language server
answers about the file you are looking at, including the parts you have
not saved; an index answers about the whole project, as it was when the
index was built. Every editor that has only one of them feels like the
half it has.
## What is built here, and what is deliberately not
Built: the catalogue, the sandboxed install, discovery that prefers a copy
you installed yourself, and **spawning** an indexer to produce a
`index.scip`.
Not built: **nothing reads the index.** Until 2026-08-27 that was blocked
by a contradiction — reading it means linking the `scip` crate, which is
Apache-2.0, permissive but not MIT — and `AGENTS.md`'s heading and its
rule disagreed about whether that was allowed. The heading was amended:
the rule (no copyleft) was always the policy, so Apache-2.0 is allowed and
the crate may be linked. Nothing blocks the reading half now except that
nobody has written it. The indexers themselves never needed the decision:
they are **spawned**, exactly as language servers and debug adapters
already are, and nothing links them.
So `hick index build` produces an index and says, in as many words, that
nothing consumes it yet. That is a strange thing to ship on its own, and
shipping it while claiming navigation had improved would be worse.

<storycode:copy id="f-b72540443cf7-INDEXERS_DIR">
//! Which code indexers `hick index install` can fetch, and how one is run.
//!
//! `docs/specs/freeform/an-index-beside-the-language-server.md`.
//!
//! **SCIP is in addition to LSP, never in place of it.** A language server
//! answers about the file you are looking at, including the parts you have
//! not saved; an index answers about the whole project, as it was when the
//! index was built. Every editor that has only one of them feels like the
//! half it has.
//!
//! ## What is built here, and what is deliberately not
//!
//! Built: the catalogue, the sandboxed install, discovery that prefers a copy
//! you installed yourself, and **spawning** an indexer to produce a
//! `index.scip`.
//!
//! Not built: **nothing reads the index.** Until 2026-08-27 that was blocked
//! by a contradiction — reading it means linking the `scip` crate, which is
//! Apache-2.0, permissive but not MIT — and `AGENTS.md`'s heading and its
//! rule disagreed about whether that was allowed. The heading was amended:
//! the rule (no copyleft) was always the policy, so Apache-2.0 is allowed and
//! the crate may be linked. Nothing blocks the reading half now except that
//! nobody has written it. The indexers themselves never needed the decision:
//! they are **spawned**, exactly as language servers and debug adapters
//! already are, and nothing links them.
//!
//! So `hick index build` produces an index and says, in as many words, that
//! nothing consumes it yet. That is a strange thing to ship on its own, and
//! shipping it while claiming navigation had improved would be worse.

use crate::tool_install::{Catalogue, Installer};

/// Where an indexer is installed, relative to the project root.
pub const INDEXERS_DIR: &str = ".hick-cache/indexers";

</storycode:copy>
<a id="f-b72540443cf7-CATALOGUE"></a>
### `CATALOGUE`

<storycode:copy id="f-b72540443cf7-CATALOGUE">
pub static CATALOGUE: std::sync::LazyLock<Catalogue> = std::sync::LazyLock::new(|| Catalogue {
    what: "code indexer",
    prefix: INDEXERS_DIR,
    command: "hick index install",
    installers: &INSTALLERS,
});

</storycode:copy>
<a id="f-b72540443cf7-plans"></a>
### `plans`

Every language this can install, with whether the machine can.

<storycode:copy id="f-b72540443cf7-plans">
/// Every language this can install, with whether the machine can.
pub fn plans() -> Vec<crate::tool_install::InstallPlan> {
    crate::tool_install::plans(&CATALOGUE)
}

</storycode:copy>
<a id="f-b72540443cf7-install"></a>
### `install`

<storycode:copy id="f-b72540443cf7-install">
/// Install one language's indexer, confined.
pub fn install(root: &std::path::Path, language: &str) -> anyhow::Result<std::path::PathBuf> {
    crate::tool_install::install(
        &CATALOGUE,
        root,
        hick_plugins::registry()
            .interface(language, hick_plugins::InterfaceKind::Indexer)
            .and_then(hick_plugins::Interface::installer)
            .map(|installer| installer.language)
            .unwrap_or(language),
    )
}

</storycode:copy>
<a id="f-b72540443cf7-use_declaration"></a>
### `use_declaration`

How to get an indexer for `language` when discovery finds none.
The same rule the debug adapters learned the hard way: never name a
command that does not exist. Most SCIP indexers are not things this
product installs, and saying so costs a person nothing while a wrong
suggestion costs them a shell round trip.

<storycode:copy id="f-b72540443cf7-use_declaration">
/// How to get an indexer for `language` when discovery finds none.
///
/// The same rule the debug adapters learned the hard way: never name a
/// command that does not exist. Most SCIP indexers are not things this
/// product installs, and saying so costs a person nothing while a wrong
/// suggestion costs them a shell round trip.
pub use hick_plugins::metadata::index_advice as how_to_get;

</storycode:copy>
<a id="f-b72540443cf7-suggests_hick_install"></a>
### `suggests_hick_install`

Whether [`how_to_get`] points at `hick index install`.
For the drift check below, and nothing else.

<storycode:copy id="f-b72540443cf7-suggests_hick_install">
/// Whether [`how_to_get`] points at `hick index install`.
///
/// For the drift check below, and nothing else.
pub fn suggests_hick_install(language: &str) -> bool {
    how_to_get(language).contains("hick index install")
}

</storycode:copy>
<a id="f-b72540443cf7-Recipe"></a>
### `Recipe`

<storycode:copy id="f-b72540443cf7-Recipe">
/// How one indexer is invoked, once found.
pub struct Recipe {
    /// The binary, as discovery finds it.
    pub bin: &'static str,
    /// Arguments before the output path.
    pub args: &'static [&'static str],
    /// The flag that names the output file.
    pub output_flag: &'static str,
}

</storycode:copy>
<a id="f-b72540443cf7-recipe"></a>
### `recipe`

The recipe for a language, or `None` when hick cannot drive its indexer.

<storycode:copy id="f-b72540443cf7-recipe">
/// The recipe for a language, or `None` when hick cannot drive its indexer.
pub fn recipe(language: &str) -> Option<Recipe> {
    match hick_plugins::registry().interface(language, hick_plugins::InterfaceKind::Indexer) {
        Some(hick_plugins::Interface::Indexer {
            bin,
            args,
            output_flag,
            ..
        }) => Some(Recipe {
            bin,
            args,
            output_flag,
        }),
        _ => None,
    }
}

</storycode:copy>
<a id="f-b72540443cf7-discover"></a>
### `discover`

Find an indexer, preferring the copy this project installed.
The same rule as language servers and debug adapters, argued once already:
what is already on the machine wins, nothing is ever installed to satisfy
a lookup, and no network is touched.

<storycode:copy id="f-b72540443cf7-discover">
/// Find an indexer, preferring the copy this project installed.
///
/// The same rule as language servers and debug adapters, argued once already:
/// what is already on the machine wins, nothing is ever installed to satisfy
/// a lookup, and no network is touched.
pub fn discover(language: &str, root: &std::path::Path) -> Option<std::path::PathBuf> {
    let recipe = recipe(language)?;
    let project = root.join(INDEXERS_DIR).join("node/node_modules/.bin");
    for dir in [Some(project), None].into_iter().flatten() {
        let candidate = dir.join(recipe.bin);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|dir| dir.join(recipe.bin))
        .find(|candidate| candidate.is_file())
}

</storycode:copy>
<a id="f-b72540443cf7-tests"></a>
### `tests`

<storycode:copy id="f-b72540443cf7-tests">
#[cfg(test)]
mod tests {
    #[test]
    fn every_command_the_message_names_is_a_command_that_exists() {
        // The drift check the debug adapters grew, and a stronger version of
        // it: the first draft here said `hick index install javascript`,
        // which fails with "no installer for 'javascript'" because the
        // catalogue is keyed by ecosystem. Asserting that SOME install
        // command is named was not enough — the named one has to be real.
        let installable: Vec<&str> = super::INSTALLERS.iter().map(|i| i.language).collect();
        for language in [
            "typescript",
            "javascript",
            "typescriptreact",
            "javascriptreact",
            "python",
        ] {
            let message = super::how_to_get(language);
            let named = message
                .split("hick index install ")
                .nth(1)
                .and_then(|rest| rest.split(['`', ' ', '.']).next())
                .unwrap_or_default();
            assert!(
                installable.contains(&named),
                "`{language}` is told to run `hick index install {named}`, which does not exist"
            );
        }
        for language in ["rust", "csharp", "java", "cobol"] {
            assert!(
                !super::suggests_hick_install(language),
                "`{language}` is not installable and the message offers it anyway"
            );
        }
    }

    #[test]
    fn every_installable_language_has_a_recipe_to_run_it() {
        // Installing an indexer nothing knows how to invoke is a download
        // that does nothing.
        for installer in super::INSTALLERS.iter() {
            assert!(
                super::recipe(installer.language).is_some(),
                "{} can be installed and not run",
                installer.language
            );
        }
    }
}

</storycode:copy>
<a id="f-b72540443cf7-INSTALLERS"></a>
### `INSTALLERS`

<storycode:copy id="f-b72540443cf7-INSTALLERS">
static INSTALLERS: std::sync::LazyLock<Vec<Installer>> = std::sync::LazyLock::new(|| {
    hick_plugins::registry()
        .installers(hick_plugins::InterfaceKind::Indexer)
        .copied()
        .collect()
});
</storycode:copy>

### Assemble `crates/hickory-cli/src/index_install.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/index_install.rs"><storycode:paste select="#f-b72540443cf7-INDEXERS_DIR"/><storycode:paste select="#f-b72540443cf7-CATALOGUE"/><storycode:paste select="#f-b72540443cf7-plans"/><storycode:paste select="#f-b72540443cf7-install"/><storycode:paste select="#f-b72540443cf7-use_declaration"/><storycode:paste select="#f-b72540443cf7-suggests_hick_install"/><storycode:paste select="#f-b72540443cf7-Recipe"/><storycode:paste select="#f-b72540443cf7-recipe"/><storycode:paste select="#f-b72540443cf7-discover"/><storycode:paste select="#f-b72540443cf7-tests"/><storycode:paste select="#f-b72540443cf7-INSTALLERS"/></storycode:file>

<a id="output-22debb023e48"></a>
## `crates/hickory-cli/src/index_read.rs`

Related contracts: [An Index Answers In Documents](contract.md).

<a id="f-22debb023e48-BuiltFrom"></a>
### `BuiltFrom`

Reading an index, and refusing to show what it cannot explain.
`docs/specs/freeform/an-index-beside-the-language-server.md`. The
producing half is [`crate::index_install`]; this is the half that answers
a question with what it produced.
Three rules from that document shape everything here, and each is a
refusal rather than a feature:
**An index is a cache, and is marked as one.** It is stale the moment you
type. It may speed up an answer and may never *be* the answer to a
question about correctness, so every report says when it was built and
whether the files it covers have moved since.
**Positions go through lineage or are not shown.** An index of woven files
holds `(file, line, column)` for files a person never edits. Showing one
without mapping it back to a document span would send them to a file that
regenerates over their edit — the exact failure
`a-generated-file-refuses-an-edit` exists to prevent. A reference in a
generated file that cannot be mapped back is **dropped**, and the count of
dropped ones is reported, because silently showing fewer results than
exist is its own kind of lie.
**The index is never required.** Nothing here is on the path of any
navigation feature that already works.

<storycode:copy id="f-22debb023e48-BuiltFrom">
//! Reading an index, and refusing to show what it cannot explain.
//!
//! `docs/specs/freeform/an-index-beside-the-language-server.md`. The
//! producing half is [`crate::index_install`]; this is the half that answers
//! a question with what it produced.
//!
//! Three rules from that document shape everything here, and each is a
//! refusal rather than a feature:
//!
//! **An index is a cache, and is marked as one.** It is stale the moment you
//! type. It may speed up an answer and may never *be* the answer to a
//! question about correctness, so every report says when it was built and
//! whether the files it covers have moved since.
//!
//! **Positions go through lineage or are not shown.** An index of woven files
//! holds `(file, line, column)` for files a person never edits. Showing one
//! without mapping it back to a document span would send them to a file that
//! regenerates over their edit — the exact failure
//! `a-generated-file-refuses-an-edit` exists to prevent. A reference in a
//! generated file that cannot be mapped back is **dropped**, and the count of
//! dropped ones is reported, because silently showing fewer results than
//! exist is its own kind of lie.
//!
//! **The index is never required.** Nothing here is on the path of any
//! navigation feature that already works.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

/// What an index was built from, written beside it.
///
/// Staleness is not a second mechanism: it is the same question recordings
/// already answer — did an input change — asked with the same tool, a hash of
/// the bytes that went in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltFrom {
    /// RFC 3339, so a report can say "indexed at 14:02" where the age matters.
    pub at: String,
    /// Project-relative path → SHA-256 at index time.
    pub inputs: HashMap<String, String>,
}

</storycode:copy>
<a id="f-22debb023e48-impl-BuiltFrom"></a>
### `impl-BuiltFrom`

<storycode:copy id="f-22debb023e48-impl-BuiltFrom">
impl BuiltFrom {
    /// Which of the indexed files have changed since, in a stable order.
    pub fn moved_on(&self, root: &Path) -> Vec<String> {
        let mut stale: Vec<String> = self
            .inputs
            .iter()
            .filter(|(path, was)| {
                let now = std::fs::read(root.join(path))
                    .ok()
                    .map(|bytes| format!("{:x}", Sha256::digest(&bytes)));
                // A file that has been deleted has certainly moved on.
                now.as_deref() != Some(was.as_str())
            })
            .map(|(path, _)| path.clone())
            .collect();
        stale.sort();
        stale
    }
}

</storycode:copy>
<a id="f-22debb023e48-index_path"></a>
### `index_path`

Where an index and its provenance live for one language.

<storycode:copy id="f-22debb023e48-index_path">
/// Where an index and its provenance live for one language.
pub fn index_path(root: &Path, language: &str) -> PathBuf {
    root.join(".hick-cache/index")
        .join(format!("{language}.scip"))
}

</storycode:copy>
<a id="f-22debb023e48-built_from_path"></a>
### `built_from_path`

<storycode:copy id="f-22debb023e48-built_from_path">
fn built_from_path(root: &Path, language: &str) -> PathBuf {
    root.join(".hick-cache/index")
        .join(format!("{language}.built-from.json"))
}

</storycode:copy>
<a id="f-22debb023e48-write_built_from"></a>
### `write_built_from`

Record what an index covers, right after building it.

<storycode:copy id="f-22debb023e48-write_built_from">
/// Record what an index covers, right after building it.
pub fn write_built_from(root: &Path, language: &str, at: &str) -> Result<BuiltFrom> {
    let index = read_index(&index_path(root, language))?;
    let inputs = index
        .documents
        .iter()
        .filter_map(|doc| {
            let bytes = std::fs::read(root.join(&doc.relative_path)).ok()?;
            Some((
                doc.relative_path.clone(),
                format!("{:x}", Sha256::digest(&bytes)),
            ))
        })
        .collect();
    let built = BuiltFrom {
        at: at.to_string(),
        inputs,
    };
    std::fs::write(
        built_from_path(root, language),
        serde_json::to_vec_pretty(&built)?,
    )?;
    Ok(built)
}

</storycode:copy>
<a id="f-22debb023e48-built_from"></a>
### `built_from`

What an index says it was built from, when it says.

<storycode:copy id="f-22debb023e48-built_from">
/// What an index says it was built from, when it says.
pub fn built_from(root: &Path, language: &str) -> Option<BuiltFrom> {
    let text = std::fs::read_to_string(built_from_path(root, language)).ok()?;
    serde_json::from_str(&text).ok()
}

</storycode:copy>
<a id="f-22debb023e48-read_index"></a>
### `read_index`

<storycode:copy id="f-22debb023e48-read_index">
/// Read a SCIP index off disk.
pub fn read_index(path: &Path) -> Result<scip::types::Index> {
    let bytes = std::fs::read(path).with_context(|| {
        format!(
            "no index at {} — `hick index build` writes one",
            path.display()
        )
    })?;
    <scip::types::Index as protobuf::Message>::parse_from_bytes(&bytes)
        .with_context(|| format!("{} is not a SCIP index", path.display()))
}

</storycode:copy>
<a id="f-22debb023e48-RawHit"></a>
### `RawHit`

One place a name appears, before anything has been mapped.

<storycode:copy id="f-22debb023e48-RawHit">
/// One place a name appears, before anything has been mapped.
#[derive(Debug, Clone)]
pub struct RawHit {
    /// Project-relative, as the indexer recorded it.
    pub path: String,
    /// 0-based.
    pub line: u32,
    pub symbol: String,
    pub is_definition: bool,
}

</storycode:copy>
<a id="f-22debb023e48-occurrences"></a>
### `occurrences`

Every occurrence whose symbol mentions `query`.
Substring rather than exact: a SCIP symbol is a long structured string
(`scip-typescript npm . . src/`a.ts`/invoiceRef().`), and a person types
the name they are looking for, not that.

<storycode:copy id="f-22debb023e48-occurrences">
/// Every occurrence whose symbol mentions `query`.
///
/// Substring rather than exact: a SCIP symbol is a long structured string
/// (`scip-typescript npm . . src/`a.ts`/invoiceRef().`), and a person types
/// the name they are looking for, not that.
pub fn occurrences(index: &scip::types::Index, query: &str) -> Vec<RawHit> {
    let mut hits = Vec::new();
    for document in &index.documents {
        for occurrence in &document.occurrences {
            if !occurrence.symbol.contains(query) {
                continue;
            }
            let Some(line) = occurrence.range.first() else {
                continue;
            };
            hits.push(RawHit {
                path: document.relative_path.clone(),
                line: *line as u32,
                symbol: occurrence.symbol.clone(),
                // `SymbolRole::Definition` is bit 1 of a bitset.
                is_definition: occurrence.symbol_roles & 1 != 0,
            });
        }
    }
    hits.sort_by(|a, b| (&a.path, a.line).cmp(&(&b.path, b.line)));
    hits
}

</storycode:copy>
<a id="f-22debb023e48-Hit"></a>
### `Hit`

One place a name appears, after it has been explained.

<storycode:copy id="f-22debb023e48-Hit">
/// One place a name appears, after it has been explained.
#[derive(Debug, Clone)]
pub struct Hit {
    /// Where a person should be sent — a document when the file is generated,
    /// the file itself when it is not.
    pub path: String,
    /// 0-based line in `path`.
    pub line: u32,
    pub is_definition: bool,
    /// The generated file this came from, when `path` is a document.
    pub through: Option<String>,
}

</storycode:copy>
<a id="f-22debb023e48-Found"></a>
### `Found`

What a lookup found, and what it refused to show.

<storycode:copy id="f-22debb023e48-Found">
/// What a lookup found, and what it refused to show.
#[derive(Debug, Default)]
pub struct Found {
    pub hits: Vec<Hit>,
    /// Occurrences in generated files that lineage could not explain.
    ///
    /// Counted and reported rather than dropped in silence: showing fewer
    /// results than exist, with no sign that anything was left out, is its
    /// own kind of lie.
    pub unmapped: usize,
}

</storycode:copy>
<a id="f-22debb023e48-Lineage"></a>
### `Lineage`

A generated file's lineage, as the index needs it: byte ranges in the
output, and the document byte span each came from.

<storycode:copy id="f-22debb023e48-Lineage">
/// A generated file's lineage, as the index needs it: byte ranges in the
/// output, and the document byte span each came from.
pub type Lineage = Vec<hickory_lineage::Provenance>;

</storycode:copy>
<a id="f-22debb023e48-explain"></a>
### `explain`

Turn raw occurrences into places a person can be sent.
`generated` maps an output path to the document that writes it, and
`lineage_of` produces that output's provenance and current text. Both are
passed in so this is testable without weaving anything.

<storycode:copy id="f-22debb023e48-explain">
/// Turn raw occurrences into places a person can be sent.
///
/// `generated` maps an output path to the document that writes it, and
/// `lineage_of` produces that output's provenance and current text. Both are
/// passed in so this is testable without weaving anything.
pub fn explain(
    raw: &[RawHit],
    generated: &HashMap<String, String>,
    mut lineage_of: impl FnMut(&str) -> Option<(Lineage, String, String)>,
) -> Found {
    let mut found = Found::default();
    for hit in raw {
        let Some(_document) = generated.get(&hit.path) else {
            // Not a generated file: it is a file the person edits, and
            // sending them to it is right.
            found.hits.push(Hit {
                path: hit.path.clone(),
                line: hit.line,
                is_definition: hit.is_definition,
                through: None,
            });
            continue;
        };
        // Generated: it must be explained, or it is not shown.
        let Some((lineage, output_text, doc_text)) = lineage_of(&hit.path) else {
            found.unmapped += 1;
            continue;
        };
        let Some(offset) = line_start(&output_text, hit.line) else {
            found.unmapped += 1;
            continue;
        };
        let Some(origin) = lineage
            .iter()
            .find(|p| p.start <= offset && offset < p.end)
            .and_then(|p| p.origin.location())
        else {
            // Synthetic bytes, or an agent's without a document span. There
            // is nowhere to send anybody, so nobody is sent.
            found.unmapped += 1;
            continue;
        };
        let (doc_path, span_start, _) = origin;
        let Some(doc_line) = line_of(&doc_text, span_start) else {
            found.unmapped += 1;
            continue;
        };
        found.hits.push(Hit {
            path: doc_path.to_string(),
            line: doc_line,
            is_definition: hit.is_definition,
            through: Some(hit.path.clone()),
        });
    }
    // An indexer records several occurrences at one position — a definition
    // and the enclosing range of the thing it defines, for instance — and
    // once they are mapped back to a document line they are the same place
    // said twice. A person reading a list of results wants places, not
    // occurrences.
    found.hits.dedup_by(|a, b| {
        a.path == b.path && a.line == b.line && a.is_definition == b.is_definition
    });
    found
}

</storycode:copy>
<a id="f-22debb023e48-line_start"></a>
### `line_start`

<storycode:copy id="f-22debb023e48-line_start">
/// Byte offset of the start of 0-based `line`.
fn line_start(text: &str, line: u32) -> Option<usize> {
    if line == 0 {
        return Some(0);
    }
    let mut seen = 0;
    for (index, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            seen += 1;
            if seen == line {
                return Some(index + 1);
            }
        }
    }
    None
}

</storycode:copy>
<a id="f-22debb023e48-line_of"></a>
### `line_of`

<storycode:copy id="f-22debb023e48-line_of">
/// 0-based line containing byte `offset`.
fn line_of(text: &str, offset: usize) -> Option<u32> {
    if offset > text.len() {
        return None;
    }
    Some(
        text.as_bytes()[..offset]
            .iter()
            .filter(|b| **b == b'\n')
            .count() as u32,
    )
}

</storycode:copy>
<a id="f-22debb023e48-tests"></a>
### `tests`

<storycode:copy id="f-22debb023e48-tests">
#[cfg(test)]
mod tests {
    use super::*;
    use hickory_lineage::{Origin, Provenance};

    fn generated() -> HashMap<String, String> {
        HashMap::from([("out.ts".to_string(), "doc.md".to_string())])
    }

    #[test]
    fn a_hit_in_a_hand_written_file_is_shown_as_itself() {
        let raw = vec![RawHit {
            path: "src/a.ts".into(),
            line: 3,
            symbol: "x".into(),
            is_definition: false,
        }];
        let found = explain(&raw, &generated(), |_| None);
        assert_eq!(found.hits.len(), 1);
        assert_eq!(found.hits[0].path, "src/a.ts");
        assert!(found.hits[0].through.is_none());
        assert_eq!(found.unmapped, 0);
    }

    #[test]
    fn a_hit_in_a_generated_file_is_shown_at_the_document_that_wrote_it() {
        // The whole point: a reference found in `out.ts` is only useful if it
        // opens the document fragment that contributed those bytes.
        let output = "line zero\nline one\nline two\n";
        let doc = "# A document\n\n<hick:file path=\"out.ts\">\nline zero\nline one\nline two\n</hick:file>\n";
        let body_at = doc.find("line zero").unwrap();
        let lineage = vec![Provenance {
            start: 0,
            end: output.len(),
            origin: Origin::Literal {
                doc_path: "doc.md".into(),
                span: (body_at, doc.len()),
            },
        }];
        let raw = vec![RawHit {
            path: "out.ts".into(),
            line: 1,
            symbol: "x".into(),
            is_definition: true,
        }];
        let found = explain(&raw, &generated(), |_| {
            Some((lineage.clone(), output.to_string(), doc.to_string()))
        });
        assert_eq!(found.unmapped, 0);
        assert_eq!(found.hits.len(), 1);
        assert_eq!(found.hits[0].path, "doc.md");
        assert_eq!(found.hits[0].through.as_deref(), Some("out.ts"));
        assert!(found.hits[0].is_definition);
        // Line 3 of the document is where the block's body starts.
        assert_eq!(found.hits[0].line, 3);
    }

    #[test]
    fn a_generated_hit_with_no_lineage_is_not_shown_and_is_counted() {
        // Showing it would send somebody to edit a file that regenerates over
        // them. Dropping it silently would show fewer results than exist with
        // no sign anything was left out.
        let raw = vec![RawHit {
            path: "out.ts".into(),
            line: 1,
            symbol: "x".into(),
            is_definition: false,
        }];
        let found = explain(&raw, &generated(), |_| None);
        assert!(found.hits.is_empty());
        assert_eq!(found.unmapped, 1);
    }

    #[test]
    fn synthetic_bytes_have_nowhere_to_send_anybody() {
        let output = "generated header\nreal line\n";
        let lineage = vec![Provenance {
            start: 0,
            end: output.len(),
            origin: Origin::Synthetic,
        }];
        let raw = vec![RawHit {
            path: "out.ts".into(),
            line: 0,
            symbol: "x".into(),
            is_definition: false,
        }];
        let found = explain(&raw, &generated(), |_| {
            Some((lineage.clone(), output.to_string(), String::new()))
        });
        assert!(found.hits.is_empty());
        assert_eq!(found.unmapped, 1);
    }

    #[test]
    fn line_arithmetic_survives_the_ends() {
        let text = "a\nbb\nccc";
        assert_eq!(line_start(text, 0), Some(0));
        assert_eq!(line_start(text, 1), Some(2));
        assert_eq!(line_start(text, 2), Some(5));
        assert_eq!(line_start(text, 9), None);
        assert_eq!(line_of(text, 0), Some(0));
        assert_eq!(line_of(text, 2), Some(1));
        assert_eq!(line_of(text, 5), Some(2));
        assert_eq!(line_of(text, 999), None);
    }

    #[test]
    fn a_changed_file_is_what_makes_an_index_stale() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.ts"), "original\n").unwrap();
        let built = BuiltFrom {
            at: "2026-08-27T14:02:00Z".into(),
            inputs: HashMap::from([(
                "a.ts".to_string(),
                format!("{:x}", Sha256::digest(b"original\n")),
            )]),
        };
        assert!(built.moved_on(dir.path()).is_empty());
        std::fs::write(dir.path().join("a.ts"), "edited\n").unwrap();
        assert_eq!(built.moved_on(dir.path()), vec!["a.ts".to_string()]);
        // A deleted file has certainly moved on.
        std::fs::remove_file(dir.path().join("a.ts")).unwrap();
        assert_eq!(built.moved_on(dir.path()), vec!["a.ts".to_string()]);
    }
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/index_read.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/index_read.rs"><storycode:paste select="#f-22debb023e48-BuiltFrom"/><storycode:paste select="#f-22debb023e48-impl-BuiltFrom"/><storycode:paste select="#f-22debb023e48-index_path"/><storycode:paste select="#f-22debb023e48-built_from_path"/><storycode:paste select="#f-22debb023e48-write_built_from"/><storycode:paste select="#f-22debb023e48-built_from"/><storycode:paste select="#f-22debb023e48-read_index"/><storycode:paste select="#f-22debb023e48-RawHit"/><storycode:paste select="#f-22debb023e48-occurrences"/><storycode:paste select="#f-22debb023e48-Hit"/><storycode:paste select="#f-22debb023e48-Found"/><storycode:paste select="#f-22debb023e48-Lineage"/><storycode:paste select="#f-22debb023e48-explain"/><storycode:paste select="#f-22debb023e48-line_start"/><storycode:paste select="#f-22debb023e48-line_of"/><storycode:paste select="#f-22debb023e48-tests"/></storycode:file>

<a id="output-38661048fddf"></a>
## `crates/hickory-cli/tests/index_reads_back_into_documents.rs`

Related contracts: [An Index Answers In Documents](contract.md).

<a id="f-38661048fddf-DOC"></a>
### `DOC`

Reading an index, and refusing to show what it cannot explain.
Protects docs/guarantees/editor-intelligence/an-index-answers-in-documents.md
The claim worth testing is the one a unit test cannot make: that a
reference found in a **generated** file comes back as a line in the
**document** that wrote it. That needs a real indexer, a real weave, and
real lineage between them.
Skipped loudly without `scip-typescript`, which is what
`hick index install typescript` fetches.

<storycode:copy id="f-38661048fddf-DOC">
//! Reading an index, and refusing to show what it cannot explain.
//!
//! Protects docs/guarantees/editor-intelligence/an-index-answers-in-documents.md
//!
//! The claim worth testing is the one a unit test cannot make: that a
//! reference found in a **generated** file comes back as a line in the
//! **document** that wrote it. That needs a real indexer, a real weave, and
//! real lineage between them.
//!
//! Skipped loudly without `scip-typescript`, which is what
//! `hick index install typescript` fetches.

use std::path::{Path, PathBuf};

const DOC: &str = r#"# Billing

The invoice reference is built here.

<hick:file path="billing.ts">
export function invoiceRef(id: string): string {
  return `INV-${id}`;
}
</hick:file>
"#;

</storycode:copy>
<a id="f-38661048fddf-DEFINITION_LINE"></a>
### `DEFINITION_LINE`

0-based line of `export function invoiceRef` in DOC.

<storycode:copy id="f-38661048fddf-DEFINITION_LINE">
/// 0-based line of `export function invoiceRef` in DOC.
const DEFINITION_LINE: u32 = 5;

</storycode:copy>
<a id="f-38661048fddf-indexer"></a>
### `indexer`

The indexer, from this repository's own install or from `PATH`.
A developer runs `hick index install typescript` once at the top of this
repo, and a scratch project in a temp directory is nowhere near it —
without this the test skips on the machine most likely to be running it,
which is how a suite ends up green while testing nothing. The same borrow
`live_session.rs` and `live_session_csharp.rs` do for their tools.

<storycode:copy id="f-38661048fddf-indexer">
/// The indexer, from this repository's own install or from `PATH`.
///
/// A developer runs `hick index install typescript` once at the top of this
/// repo, and a scratch project in a temp directory is nowhere near it —
/// without this the test skips on the machine most likely to be running it,
/// which is how a suite ends up green while testing nothing. The same borrow
/// `live_session.rs` and `live_session_csharp.rs` do for their tools.
fn indexer() -> Option<PathBuf> {
    let ours = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.hick-cache/indexers/node/node_modules/.bin/scip-typescript");
    if ours.is_file() {
        return ours.canonicalize().ok();
    }
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|dir| dir.join("scip-typescript"))
        .find(|c| c.is_file())
}

</storycode:copy>
<a id="f-38661048fddf-project"></a>
### `project`

<storycode:copy id="f-38661048fddf-project">
fn project() -> Option<(tempfile::TempDir, PathBuf)> {
    let dir = tempfile::tempdir().ok()?;
    let root = dir.path().canonicalize().ok()?;
    std::fs::write(root.join("billing.md"), DOC).ok()?;
    std::fs::write(
        root.join("use.ts"),
        "import { invoiceRef } from \"./billing\";\nexport const first = invoiceRef(\"1\");\n",
    )
    .ok()?;
    std::fs::write(
        root.join("package.json"),
        "{ \"name\": \"t\", \"version\": \"1.0.0\" }\n",
    )
    .ok()?;
    std::fs::write(
        root.join("tsconfig.json"),
        "{ \"compilerOptions\": { \"target\": \"ES2020\", \"module\": \"commonjs\" }, \
         \"include\": [\"billing.ts\", \"use.ts\"] }\n",
    )
    .ok()?;
    Some((dir, root))
}

</storycode:copy>
<a id="f-38661048fddf-build_index"></a>
### `build_index`

<storycode:copy id="f-38661048fddf-build_index">
fn build_index(root: &Path, indexer: &Path) -> bool {
    let out_dir = root.join(".hick-cache/index");
    std::fs::create_dir_all(&out_dir).ok();
    std::process::Command::new(indexer)
        .args(["index", "--output"])
        .arg(out_dir.join("typescript.scip"))
        .current_dir(root)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

</storycode:copy>
<a id="f-38661048fddf-a_reference_in_a_generated_file_comes_back_as_a_line_in_its_document"></a>
### `a_reference_in_a_generated_file_comes_back_as_a_line_in_its_document`

<storycode:copy id="f-38661048fddf-a_reference_in_a_generated_file_comes_back_as_a_line_in_its_document">
#[tokio::test(flavor = "multi_thread")]
async fn a_reference_in_a_generated_file_comes_back_as_a_line_in_its_document() {
    let Some(indexer) = indexer() else {
        eprintln!("SKIPPED: no scip-typescript (`hick index install typescript`)");
        return;
    };
    let Some((_dir, root)) = project() else {
        eprintln!("SKIPPED: could not make a scratch project");
        return;
    };

    // Weave, so the generated file the indexer will read exists.
    let run = hickory_cli::run_doc(
        &root.join("billing.md"),
        &[],
        hickory_cli::RunMode::Weave,
        hickory_cli::ExecutorChoice::Local,
    )
    .await
    .expect("the document weaves");
    hickory_cli::write_outputs(&run, None).expect("outputs written");
    assert!(
        root.join("billing.ts").exists(),
        "the weave produced nothing"
    );

    assert!(build_index(&root, &indexer), "the indexer failed");
    hickory_cli::index_read::write_built_from(&root, "typescript", "2026-08-27T00:00:00Z")
        .expect("a build record");

    let index = hickory_cli::index_read::read_index(&hickory_cli::index_read::index_path(
        &root,
        "typescript",
    ))
    .expect("the index reads");
    let raw = hickory_cli::index_read::occurrences(&index, "invoiceRef");
    assert!(
        raw.iter().any(|h| h.path == "billing.ts"),
        "the indexer never saw the generated file: {raw:?}"
    );

    let doc_index =
        hickory_cli::serve::store::DocIndex::scan(&root).expect("an index of documents");
    let generated = hickory_cli::serve::api::generated_outputs(&root, &doc_index);
    // The map is output path -> document **id**, not path. Reading it as a
    // path is silent: the weave fails and every generated hit is dropped as
    // unexplainable, which looks exactly like lineage having nothing to say.
    let doc_id = generated
        .get("billing.ts")
        .expect("billing.ts is known to be generated");
    let doc_path = doc_index.absolute(doc_id).expect("the id resolves");

    let lineage = hickory_cli::output_lineage(&run, "billing.ts").expect("lineage");
    let text = match run.result.files.get("billing.ts").unwrap() {
        hick_exec::node::FileContent::Text(s) => s.clone(),
        hick_exec::node::FileContent::Binary(_) => unreachable!(),
    };
    let source = std::fs::read_to_string(&doc_path).unwrap();

    let found = hickory_cli::index_read::explain(&raw, &generated, |output| {
        (output == "billing.ts").then(|| (lineage.clone(), text.clone(), source.clone()))
    });

    // The claim: the definition, found in a file nobody edits, is reported at
    // the line of the DOCUMENT that wrote it.
    let through_document = found
        .hits
        .iter()
        .find(|h| h.through.as_deref() == Some("billing.ts"))
        .expect("no hit was mapped back into the document");
    assert!(
        through_document.path.ends_with("billing.md"),
        "{through_document:?}"
    );
    assert_eq!(
        through_document.line, DEFINITION_LINE,
        "the wrong document line: {through_document:?}"
    );
    assert!(
        found
            .hits
            .iter()
            .any(|h| h.path == "use.ts" && h.through.is_none()),
        "the hand-written file's own reference was lost: {:?}",
        found.hits
    );
    assert_eq!(found.unmapped, 0, "something generated went unexplained");
}

</storycode:copy>
<a id="f-38661048fddf-an_edited_file_makes_the_index_say_it_is_out_of_date"></a>
### `an_edited_file_makes_the_index_say_it_is_out_of_date`

<storycode:copy id="f-38661048fddf-an_edited_file_makes_the_index_say_it_is_out_of_date">
#[tokio::test(flavor = "multi_thread")]
async fn an_edited_file_makes_the_index_say_it_is_out_of_date() {
    // An index is a cache and must be marked as one. Staleness is not a
    // second mechanism: it is the same question recordings answer — did an
    // input change — asked with the same tool.
    let Some((_dir, root)) = project() else {
        eprintln!("SKIPPED: could not make a scratch project");
        return;
    };
    std::fs::create_dir_all(root.join(".hick-cache/index")).unwrap();
    // A minimal index naming one file is enough: nothing here reads symbols.
    let mut index = scip::types::Index::new();
    let mut document = scip::types::Document::new();
    document.relative_path = "use.ts".into();
    index.documents.push(document);
    scip::write_message_to_file(
        hickory_cli::index_read::index_path(&root, "typescript"),
        index,
    )
    .unwrap();

    let built =
        hickory_cli::index_read::write_built_from(&root, "typescript", "2026-08-27T00:00:00Z")
            .unwrap();
    assert_eq!(built.inputs.len(), 1);
    assert!(
        built.moved_on(&root).is_empty(),
        "fresh index reported stale"
    );

    std::fs::write(root.join("use.ts"), "// somebody typed\n").unwrap();
    assert_eq!(built.moved_on(&root), vec!["use.ts".to_string()]);
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/index_reads_back_into_documents.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/index_reads_back_into_documents.rs"><storycode:paste select="#f-38661048fddf-DOC"/><storycode:paste select="#f-38661048fddf-DEFINITION_LINE"/><storycode:paste select="#f-38661048fddf-indexer"/><storycode:paste select="#f-38661048fddf-project"/><storycode:paste select="#f-38661048fddf-build_index"/><storycode:paste select="#f-38661048fddf-a_reference_in_a_generated_file_comes_back_as_a_line_in_its_document"/><storycode:paste select="#f-38661048fddf-an_edited_file_makes_the_index_say_it_is_out_of_date"/></storycode:file>

## Implementation review

### Implementation

- [crates/hickory-cli/src/index_install.rs](contract.md#output-b72540443cf7)
- [crates/hickory-cli/src/index_read.rs](contract.md#output-22debb023e48)

### Tests and fixtures

- [crates/hickory-cli/tests/index_reads_back_into_documents.rs](contract.md#output-38661048fddf)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
