//! `hick census`: the report — arguments, table, footer.
//!
//! The counting is `hickory_cli::census`, which is in the library so a test
//! and the MCP surface can ask the same question. This file is the printing
//! only, and it is its own file for the reason `bin_equiv.rs` and
//! `bin_lineage.rs` are: `main.rs` is at its length baseline
//! (`scripts/check-file-length.sh`).

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use hickory_cli::census::{Census, DocumentCensus, FileCensus, LineTotals};

#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum By {
    /// One row per produced file (the default).
    File,
    /// One row per producing document, with its own prose/code/tag counts.
    Document,
}

#[derive(clap::Args)]
pub struct CensusArgs {
    /// The folder to walk, or a single `.hick` document. Defaults to the
    /// working directory.
    #[arg(default_value = ".")]
    path: PathBuf,
    /// Emit the whole report as one JSON object instead of a table.
    ///
    /// Key order is stable, so a `hick:exec` cell that runs
    /// `hick census --json .` can pin the result with a `hick:expect`.
    #[arg(long)]
    json: bool,
    /// Group by producing document instead of by produced file.
    #[arg(long = "by", value_enum, default_value = "file")]
    by: By,
    /// List only files carrying generated or cell-output lines — the files
    /// that are not, or not entirely, yours.
    #[arg(long = "only-generated")]
    only_generated: bool,
}

pub async fn cmd_census(args: CensusArgs) -> Result<ExitCode> {
    let census = hickory_cli::census::census(&args.path).await?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&census)?);
        return Ok(ExitCode::SUCCESS);
    }
    match args.by {
        By::File => print_files(&census, args.only_generated),
        By::Document => print_documents(&census),
    }
    print_folders(&census);
    print_footer(&census);
    Ok(ExitCode::SUCCESS)
}

/// One row per produced file: what is on disk, and who to change to change it.
fn print_files(census: &Census, only_generated: bool) {
    let rows: Vec<&FileCensus> = census
        .files
        .iter()
        .filter(|f| !only_generated || f.totals.generated + f.totals.cell_output > 0)
        .collect();
    if rows.is_empty() {
        println!("no produced files under {}", census.root);
        println!();
        return;
    }
    let width = column_width(rows.iter().map(|f| f.path.as_str()));
    println!(
        "{:<width$}  {:>8}  {:>9}  {:>11}  generator",
        "file", "authored", "generated", "cell-output"
    );
    for file in rows {
        println!(
            "{:<width$}  {:>8}  {:>9}  {:>11}  {}",
            elide(&file.path, width),
            file.totals.authored,
            file.totals.generated,
            file.totals.cell_output,
            file.generator(),
        );
    }
    println!();
    if only_generated {
        println!(
            "Listed the {} of {} produced file(s) carrying a line nobody typed.",
            census
                .files
                .iter()
                .filter(|f| f.totals.generated + f.totals.cell_output > 0)
                .count(),
            census.files.len()
        );
        println!();
    }
}

/// One row per document: what it produced, and what it is in its own lines.
fn print_documents(census: &Census) {
    if census.documents.is_empty() {
        println!("no documents under {}", census.root);
        println!();
        return;
    }
    let width = column_width(census.documents.iter().map(|d| d.path.as_str()));
    println!(
        "{:<width$}  {:>8}  {:>9}  {:>11}  {:>5}  {:>6}  {:>5}  {:>3}",
        "document", "authored", "generated", "cell-output", "files", "prose", "code", "tag"
    );
    for doc in &census.documents {
        print_document_row(doc, width);
    }
    println!();
}

fn print_document_row(doc: &DocumentCensus, width: usize) {
    println!(
        "{:<width$}  {:>8}  {:>9}  {:>11}  {:>5}  {:>6}  {:>5}  {:>3}",
        elide(&doc.path, width),
        doc.produced.authored,
        doc.produced.generated,
        doc.produced.cell_output,
        doc.files.len(),
        doc.source.prose,
        doc.source.fragment,
        doc.source.tag,
    );
}

/// Totals per top-level folder, then everything.
fn print_folders(census: &Census) {
    let width = census
        .folders
        .iter()
        .map(|f| f.folder.len())
        .chain(std::iter::once("TOTAL".len()))
        .max()
        .unwrap_or(5)
        .clamp(8, 40);
    println!(
        "{:<width$}  {:>5}  {:>8}  {:>17}  {:>18}  {:>19}",
        "folder", "files", "lines", "authored", "generated", "cell-output"
    );
    for folder in &census.folders {
        print_totals_row(&folder.folder, folder.files, &folder.totals, width);
    }
    print_totals_row("TOTAL", census.files.len(), &census.totals, width);
    println!();
}

fn print_totals_row(label: &str, files: usize, totals: &LineTotals, width: usize) {
    println!(
        "{:<width$}  {:>5}  {:>8}  {:>9} {:>6.1}%  {:>9} {:>6.1}%  {:>10} {:>6.1}%",
        elide(label, width),
        files,
        totals.lines,
        totals.authored,
        totals.authored_pct(),
        totals.generated,
        totals.generated_pct(),
        totals.cell_output,
        totals.cell_output_pct(),
    );
}

/// What the census could not count, and what the documents themselves are.
///
/// Both belong at the bottom and both belong in the report rather than in
/// this file's head: a percentage of "the code" that quietly left out half
/// the repository is the failure mode this footer exists to prevent.
fn print_footer(census: &Census) {
    let source = &census.document_totals;
    println!(
        "{} document(s), {} line(s) of them: {} prose, {} fragment (code), {} tag.",
        census.documents.len(),
        source.lines,
        source.prose,
        source.fragment,
        source.tag,
    );
    println!(
        "{} tracked file(s) are not produced by any document.",
        census.unproduced
    );
    println!(
        "  Not counted above, along with everything under `.hick-cache/` and \
         everything git ignores."
    );
    if !census.empty_volumes.is_empty() {
        println!(
            "{} output volume(s) had nothing on disk to count:",
            census.empty_volumes.len()
        );
        for volume in &census.empty_volumes {
            println!("  {volume}");
        }
        println!(
            "  A weave never runs a cell, so a volume's files are counted \
             where the last run left them. Run the document to put them there."
        );
    }
    println!(
        "Woven, never run: every count above is byte lineage, except \
         cell-output, which is whole files attributed to the cell that \
         mounts their volume."
    );
}

/// A column just wide enough for its longest value, within reason.
fn column_width<'a>(values: impl Iterator<Item = &'a str>) -> usize {
    values.map(str::len).max().unwrap_or(8).clamp(8, 64)
}

/// Trim from the LEFT, keeping the file name: two generated files in deep
/// folders differ at the end of their paths, not the start.
fn elide(text: &str, width: usize) -> String {
    match text.len() > width {
        false => text.to_string(),
        true => format!("…{}", &text[text.len() - (width - 1)..]),
    }
}
