<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/doc_cli_args.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-ec3736253708"></a>
## `crates/hickory-cli/src/doc_cli_args.rs`

<a id="f-ec3736253708-DocCommand"></a>
### `DocCommand`

<storycode:copy id="f-ec3736253708-DocCommand">
//! Arguments for the document tool CLI.
use clap::Subcommand;
use std::path::PathBuf;

/// The document tool set. Every subcommand opens a session on DOC, runs one
/// tool, and prints the observation.
///
/// The workflow they are built for: read a surface, edit against the hashes
/// you just read, verify. Anchors are content hashes, so an edit against a
/// line that has changed since you read it is refused rather than misapplied
/// — re-read and try again.
#[derive(Subcommand)]
pub(super) enum DocCommand {
    /// Print the document source, every line prefixed with its SHA-256 content
    /// hash. Those hashes are the anchors `edit` takes.
    Read(DocReadArgs),
    /// Print a woven output file, hashline-rendered. `--lineage` also marks
    /// which lines are editable and where each came from in the document.
    ReadOutput(DocReadOutputArgs),
    /// Read any project file, hashline-rendered, read-only — a data export,
    /// a config, a source file. Recorded in the session as context.
    ReadFile(DocReadFileArgs),
    /// Edit an output file; the change is mapped back into the document
    /// byte-exactly through lineage. This is how CODE should be edited.
    EditOutput(DocEditArgs),
    /// Edit the document source directly. This is how STRUCTURE and PROSE
    /// should be edited — and where lineage refusals send you.
    Edit(DocEditArgs),
    /// Execute the document for real (every exec cell, every expectation) and
    /// write its outputs. Run this before calling an edit done.
    Verify(DocVerifyArgs),
}
</storycode:copy>
<a id="f-ec3736253708-DocCommonArgs"></a>
### `DocCommonArgs`

<storycode:copy id="f-ec3736253708-DocCommonArgs">
#[derive(clap::Args)]
pub(super) struct DocCommonArgs {
    /// Parameter overrides, `key=value` (repeatable).
    #[arg(long = "param", value_parser = hick_literate::parse_param, global = true)]
    pub(super) params: Vec<(String, String)>,
    /// Enable `<hick:feature>` flags (comma-separated, repeatable).
    #[arg(long = "features", value_delimiter = ',', global = true)]
    pub(super) features: Vec<String>,
    /// Print `{"tool", "ok", "text"}` instead of the observation alone.
    #[arg(long = "json", global = true)]
    pub(super) json: bool,
    /// Append this call and its result to a `hick:session` document, creating
    /// it if needed. Defaults to `HICKORY_SESSION`; unset, nothing is
    /// recorded. This is how work done by an OUTSIDE agent still produces a
    /// session `hick ingest --from session` can compact.
    #[arg(long = "session", global = true)]
    pub(super) session: Option<PathBuf>,
}
</storycode:copy>
<a id="f-ec3736253708-DocReadArgs"></a>
### `DocReadArgs`

<storycode:copy id="f-ec3736253708-DocReadArgs">
#[derive(clap::Args)]
pub(super) struct DocReadArgs {
    /// The `.md` document. Omit when the working directory holds exactly
    /// one.
    pub(super) doc: Option<PathBuf>,
    /// Read an UPSTREAM document instead (by name or path). The primary
    /// document's `hick:upstream` closure is listed in the output.
    #[arg(long = "upstream")]
    pub(super) upstream: Option<String>,
    #[command(flatten)]
    pub(super) common: DocCommonArgs,
}
</storycode:copy>
<a id="f-ec3736253708-DocReadOutputArgs"></a>
### `DocReadOutputArgs`

<storycode:copy id="f-ec3736253708-DocReadOutputArgs">
#[derive(clap::Args)]
pub(super) struct DocReadOutputArgs {
    /// The `.md` document. Omit when the working directory holds exactly
    /// one.
    pub(super) doc: Option<PathBuf>,
    /// Which output file, as the document names it.
    #[arg(long = "path")]
    pub(super) path: String,
    /// Annotate each range with where it came from and whether it can be
    /// edited through the output. Read this before editing.
    #[arg(long = "lineage")]
    pub(super) lineage: bool,
    #[command(flatten)]
    pub(super) common: DocCommonArgs,
}
</storycode:copy>
<a id="f-ec3736253708-DocReadFileArgs"></a>
### `DocReadFileArgs`

<storycode:copy id="f-ec3736253708-DocReadFileArgs">
#[derive(clap::Args)]
pub(super) struct DocReadFileArgs {
    /// The `.md` document whose session this read belongs to. Omit when
    /// the working directory holds exactly one.
    pub(super) doc: Option<PathBuf>,
    /// The file, relative to the document's directory. Must be inside the
    /// project (the document's git repository, or its directory).
    #[arg(long = "path")]
    pub(super) path: String,
    /// First line to show (1-based).
    #[arg(long = "from")]
    pub(super) from: Option<usize>,
    /// Last line to show (1-based, inclusive).
    #[arg(long = "to")]
    pub(super) to: Option<usize>,
    #[command(flatten)]
    pub(super) common: DocCommonArgs,
}
</storycode:copy>
<a id="f-ec3736253708-DocEditArgs"></a>
### `DocEditArgs`

<storycode:copy id="f-ec3736253708-DocEditArgs">
#[derive(clap::Args)]
pub(super) struct DocEditArgs {
    /// The `.md` document. Omit when the working directory holds exactly
    /// one.
    pub(super) doc: Option<PathBuf>,
    /// Which output file (edit-output only).
    #[arg(long = "path")]
    pub(super) path: Option<String>,
    /// Replace this line run: `hash` for one line, `first..last` for a range.
    #[arg(long = "run")]
    pub(super) run: Option<String>,
    /// Insert BELOW this line instead of replacing anything; `^` inserts at
    /// the top of the file.
    #[arg(long = "after")]
    pub(super) after: Option<String>,
    /// Which occurrence to use when the anchor matches more than one place
    /// (1-based).
    #[arg(long = "occurrence")]
    pub(super) occurrence: Option<usize>,
    /// Whole-surface SHA-256 from the latest read, including range interior.
    #[arg(long)]
    pub(super) snapshot: Option<String>,
    /// Edit an UPSTREAM document instead (edit only).
    #[arg(long = "upstream")]
    pub(super) upstream: Option<String>,
    /// The replacement text. Omit (or pass `-`) to read it from stdin, which
    /// is what code should come through. Pass an empty string to DELETE the
    /// anchored run.
    #[arg(long = "input")]
    pub(super) input: Option<String>,
    #[command(flatten)]
    pub(super) common: DocCommonArgs,
}

</storycode:copy>
<a id="f-ec3736253708-DocVerifyArgs"></a>
### `DocVerifyArgs`

<storycode:copy id="f-ec3736253708-DocVerifyArgs">
#[derive(clap::Args)]
pub(super) struct DocVerifyArgs {
    /// The `.md` document. Omit when the working directory holds exactly
    /// one.
    pub(super) doc: Option<PathBuf>,
    #[command(flatten)]
    pub(super) common: DocCommonArgs,
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/doc_cli_args.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/doc_cli_args.rs"><storycode:paste select="#f-ec3736253708-DocCommand"/><storycode:paste select="#f-ec3736253708-DocCommonArgs"/><storycode:paste select="#f-ec3736253708-DocReadArgs"/><storycode:paste select="#f-ec3736253708-DocReadOutputArgs"/><storycode:paste select="#f-ec3736253708-DocReadFileArgs"/><storycode:paste select="#f-ec3736253708-DocEditArgs"/><storycode:paste select="#f-ec3736253708-DocVerifyArgs"/></storycode:file>

</storycode:doc>
