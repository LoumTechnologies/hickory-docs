//! `hick lineage`: the byte-precise Provenance[] of one generated output.
//!
//! Its own file for the reason `bin_equiv.rs` is: the argument struct, the
//! replay path (`--at`, `--history`) and the printing belong together, and
//! `main.rs` is at its length baseline (`scripts/check-file-length.sh`).

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context as _, Result};
use hickory_cli::{DocRun, ExecutorChoice, RunMode, run_doc};

#[derive(clap::Args)]
pub struct LineageArgs {
    /// A `.hick` document.
    doc: PathBuf,
    /// The generated output file to trace (its `<hick:file path>` value).
    #[arg(long = "output")]
    output: String,
    /// Parameter overrides, `key=value` (repeatable).
    #[arg(long = "param", value_parser = hick_literate::parse_param)]
    params: Vec<(String, String)>,
    /// Emit the Provenance[] JSON on stdout instead of a summary.
    #[arg(long)]
    json: bool,
    /// Replay: report the lineage the document had AT this commit.
    ///
    /// The old document is read out of git — nothing is checked out — and
    /// woven without executing anything, so the answer is recomputed exactly
    /// rather than recalled from a store. It gives the state *at* a commit,
    /// never the thread between two of them.
    ///
    /// Replay works back to the last grammar change: an older document is
    /// woven by today's binary, and this product reserves the right to change
    /// the grammar. Past that boundary the tool says so in those words.
    #[arg(long = "at", value_name = "COMMIT")]
    at: Option<String>,
    /// List the commits that touched this document, newest first, and stop.
    /// The values `--at` takes.
    #[arg(long)]
    history: bool,
}
/// The document's repository root and its repository-relative path, or a
/// refusal that says which of the two is missing.
pub(crate) fn doc_in_repo(doc: &std::path::Path) -> Result<(PathBuf, String)> {
    let abs = std::fs::canonicalize(doc)
        .with_context(|| format!("could not resolve {}", doc.display()))?;
    let dir = abs.parent().unwrap_or(std::path::Path::new("."));
    let root = hickory_cli::replay::git_root(dir).ok_or_else(|| {
        anyhow::anyhow!(
            "{} is not inside a git repository, and replay reads old documents \
             out of git.\n  \
             Next step: run `git init` in the folder, or drop `--at` to see the \
             lineage of the document as it stands.",
            doc.display()
        )
    })?;
    let root = std::fs::canonicalize(&root).unwrap_or(root);
    let rel = abs
        .strip_prefix(&root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .map_err(|_| {
            anyhow::anyhow!(
                "{} is not under the repository at {}",
                doc.display(),
                root.display()
            )
        })?;
    Ok((root, rel))
}

/// `hick lineage --history` — the commits `--at` accepts.
fn cmd_lineage_history(args: &LineageArgs) -> Result<ExitCode> {
    let (root, rel) = doc_in_repo(&args.doc)?;
    let commits = hickory_cli::replay::history(&root, &rel, 200);
    if commits.is_empty() {
        println!(
            "no commits touch {rel} — which is entirely normal for a document \
             that has not been committed yet. Replay has nowhere to go until \
             there is a commit to go to."
        );
        return Ok(ExitCode::SUCCESS);
    }
    for c in &commits {
        println!("{}  {}  {}", c.short, c.author, c.subject);
    }
    Ok(ExitCode::SUCCESS)
}

/// Weave the document as it stood at `commit`, or report the boundary.
async fn replay_run(args: &LineageArgs, commit: &str) -> Result<Option<DocRun>> {
    let (root, rel) = doc_in_repo(&args.doc)?;
    // The name the document had THEN, which a rename makes different from
    // the name it has now.
    let rel_then = hickory_cli::replay::history(&root, &rel, 500)
        .into_iter()
        .find(|c| c.sha.starts_with(commit) || c.short.starts_with(commit))
        .map(|c| c.path)
        .unwrap_or(rel.clone());
    match hickory_cli::replay::replay_at(&root, &rel_then, commit).await? {
        hickory_cli::replay::Replay::Woven(run) => Ok(Some(*run)),
        hickory_cli::replay::Replay::GrammarBoundary { commit, detail } => {
            eprintln!(
                "{}",
                hickory_cli::replay::grammar_boundary_message(&commit, &rel_then, &detail)
            );
            Ok(None)
        }
    }
}

pub async fn cmd_lineage(args: LineageArgs) -> Result<ExitCode> {
    if args.history {
        return cmd_lineage_history(&args);
    }
    let run = match &args.at {
        Some(commit) => match replay_run(&args, commit).await? {
            Some(run) => run,
            None => return Ok(ExitCode::from(1)),
        },
        None => {
            run_doc(
                &args.doc,
                &args.params,
                RunMode::Weave,
                ExecutorChoice::Local,
            )
            .await?
        }
    };
    // A file a cell wrote into an output volume is not woven, so it has no
    // byte-precise lineage — but it has a provenance, and answering "no such
    // output" to a file the document plainly produces is the wrong answer to
    // the right question. Reported first, and reported as a success: the
    // question was answerable.
    if !run.result.provenance_maps.contains_key(&args.output)
        && let Some(generated) = hickory_cli::generated_file(&run, &args.output)
    {
        eprintln!(
            "{} -> {}: generated, not woven",
            args.doc.display(),
            args.output
        );
        let where_from = if generated.cells.is_empty() {
            format!("output volume `{}`", generated.volume)
        } else {
            format!(
                "cell{} {} (line{} {}) into output volume `{}`",
                if generated.cells.len() == 1 { "" } else { "s" },
                generated
                    .cells
                    .iter()
                    .map(|c| format!("'{c}'"))
                    .collect::<Vec<_>>()
                    .join(", "),
                if generated.lines.len() == 1 { "" } else { "s" },
                generated
                    .lines
                    .iter()
                    .map(usize::to_string)
                    .collect::<Vec<_>>()
                    .join(", "),
                generated.volume,
            )
        };
        if args.json {
            println!(
                "{}",
                serde_json::json!({
                    "generated": true,
                    "volume": generated.volume,
                    "cells": generated.cells,
                    "lines": generated.lines,
                })
            );
        } else {
            eprintln!("  written by {where_from}");
            eprintln!("  Every byte is that program's output, so there is no span of any");
            eprintln!("  document to map it to, and no edit here can be carried back.");
            eprintln!("  Change what the cell reads, or the cell itself, and run again.");
        }
        return Ok(ExitCode::SUCCESS);
    }
    let provenance = hickory_cli::output_lineage(&run, &args.output)?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&provenance)?);
    } else {
        eprintln!(
            "{} -> {}: {} provenance span(s)",
            args.doc.display(),
            args.output,
            provenance.len()
        );
        for p in &provenance {
            match &p.origin {
                hickory_lineage::Origin::Synthetic => {
                    println!("{:>8}..{:<8} synthetic", p.start, p.end);
                }
                // Agent bytes are reported by session and turn, with
                // authorship derived from git — printed after the table so a
                // long line never breaks the columns.
                hickory_lineage::Origin::Agent { session, turn, .. } => {
                    println!(
                        "{:>8}..{:<8} {:<12} session {session} turn {turn}",
                        p.start, p.end, "agent"
                    );
                }
                // Ingested bytes name the run they arrived from, not just
                // where they sit: "this arrived on that day from that run" is
                // the difference between an honest blame and one that says
                // you wrote a scaffolder's forty files.
                hickory_lineage::Origin::Ingested {
                    doc_path,
                    span,
                    run,
                } => {
                    let short: String = run.chars().take(12).collect();
                    println!(
                        "{:>8}..{:<8} {:<12} {doc_path} bytes {}..{} · run {short}",
                        p.start, p.end, "ingested", span.0, span.1
                    );
                }
                origin => {
                    let (doc_path, s, e) = origin.location().unwrap_or(("?", 0, 0));
                    let kind = match origin {
                        hickory_lineage::Origin::Literal { .. } => "literal",
                        hickory_lineage::Origin::Paste { .. } => "paste",
                        hickory_lineage::Origin::Exec { .. } => "exec",
                        hickory_lineage::Origin::Variable { .. } => "variable",
                        hickory_lineage::Origin::Substitution { .. } => "substitution",
                        hickory_lineage::Origin::Agent { .. }
                        | hickory_lineage::Origin::Ingested { .. }
                        | hickory_lineage::Origin::Synthetic => unreachable!(),
                    };
                    println!(
                        "{:>8}..{:<8} {kind:<12} {doc_path} bytes {s}..{e}",
                        p.start, p.end
                    );
                }
            }
        }
        // Sessions are per-author and may be private; an unresolvable one is
        // reported, not an error.
        let agent = hickory_cli::agent_lineage_report(&run, &args.output)?;
        if !agent.is_empty() {
            println!();
            for entry in &agent {
                println!("{entry}");
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}
