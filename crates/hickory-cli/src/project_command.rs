use crate::project_revision;
use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use hick_lang::{EntryKind, ProjectTree, evaluate_project};
use std::{collections::BTreeMap, path::PathBuf, process::ExitCode};

#[derive(Args)]
pub struct ProjectArgs {
    #[command(subcommand)]
    pub command: ProjectCommand,
}
#[derive(Subcommand)]
pub enum ProjectCommand {
    /// Write generated files, preserving unincorporated external edits.
    Materialize(OutputInput),
    /// Capture external edits without changing their conceptual source owner.
    Capture {
        #[command(flatten)]
        input: OutputInput,
        /// Include newly created output paths not present in the baseline.
        #[arg(long = "path")]
        paths: Vec<String>,
    },
    /// List pending edits, or display the patch of a selected capture.
    Edits {
        #[command(flatten)]
        input: OutputInput,
        #[arg(long)]
        id: Option<String>,
    },
    /// Acknowledge amended/split source only if it reproduces the whole capture.
    Accept {
        #[command(flatten)]
        input: OutputInput,
        #[arg(long)]
        id: String,
    },
    /// Author a new concept patch; leave it pending until explicit acceptance.
    NewPatch {
        #[command(flatten)]
        input: OutputInput,
        #[arg(long)]
        id: String,
        #[arg(long)]
        document: String,
        #[arg(long)]
        patch_id: String,
        #[arg(long)]
        explanation: String,
    },
    /// Construct the complete source tree in memory and print its inventory.
    Generate(ProjectInput),
    /// Compare complete source inventory without executing or writing files.
    Verify(ProjectInput),
    /// Discover and verify every declared project from the exact staged index.
    VerifyIndex {
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Write NUL-separated ordinary Markdown paths not covered by project verification.
        #[arg(long)]
        standalone_list: Option<PathBuf>,
    },
}
#[derive(Args)]
pub struct ProjectInput {
    /// Root-relative entry Markdown, which declares its source closure.
    pub entry: String,
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
    /// Freeze documents and expected outputs from this exact Git revision.
    /// Use :index for the complete stage-zero Git index.
    #[arg(long, conflicts_with = "hg_revision")]
    pub revision: Option<String>,
    #[arg(long, conflicts_with = "revision")]
    pub hg_revision: Option<String>,
}

/// Working-tree synchronization deliberately has no revision flags: captures
/// must refer to the bytes ordinary tools actually edited on disk.
#[derive(Args)]
pub struct OutputInput {
    pub entry: String,
    #[arg(long, default_value = ".")]
    pub root: PathBuf,
}
pub async fn run(args: ProjectArgs) -> Result<ExitCode> {
    use hick_project_env::pending;
    let command = match args.command {
        ProjectCommand::Materialize(input) => {
            let root = input.root.canonicalize()?;
            let sources = pending::sources(&root, &input.entry).map_err(anyhow::Error::msg)?;
            let project = evaluate_project(&input.entry, &sources, &ProjectTree::default())
                .map_err(anyhow::Error::msg)?;
            let paths = pending::materialize(&root, &input.entry, &sources, &project.new)
                .map_err(anyhow::Error::msg)?;
            println!(
                "Materialized {} outputs and recorded their baseline.",
                paths.len()
            );
            return Ok(ExitCode::SUCCESS);
        }
        ProjectCommand::Capture { input, paths } => {
            if let Some(capture) =
                pending::capture(&input.root, &input.entry, &paths).map_err(anyhow::Error::msg)?
            {
                println!("Pending {}: {}", capture.id, capture.paths.join(", "));
            } else {
                println!("No external output edits.");
            }
            return Ok(ExitCode::SUCCESS);
        }
        ProjectCommand::Edits { input, id } => {
            let captures =
                pending::status(&input.root, &input.entry).map_err(anyhow::Error::msg)?;
            if let Some(id) = id {
                let capture = captures
                    .iter()
                    .find(|c| c.id == id)
                    .context("unknown pending capture")?;
                println!(
                    "{}",
                    capture
                        .patch
                        .as_deref()
                        .or(capture.patch_error.as_deref())
                        .unwrap_or("No textual patch")
                );
            } else {
                for capture in &captures {
                    println!("{}: {}", capture.id, capture.paths.join(", "));
                }
                if captures.is_empty() {
                    println!("No pending captures.");
                }
            }
            return Ok(ExitCode::SUCCESS);
        }
        ProjectCommand::Accept { input, id } => {
            let root = input.root.canonicalize()?;
            let source = pending::sources(&root, &input.entry).map_err(anyhow::Error::msg)?;
            let parsed = hick_lang::parse(&source[&input.entry])?;
            if parsed.find_tags("project").is_empty() {
                // Legacy standalone documents still use the same complete
                // pipeline; never infer acceptance from one matching file.
                let woven = crate::run_doc(
                    &root.join(&input.entry),
                    &[],
                    crate::RunMode::Weave,
                    crate::ExecutorChoice::Local,
                )
                .await?;
                let mut generated = ProjectTree::default();
                for (path, content) in &woven.result.files {
                    let bytes = match content {
                        hick_exec::node::FileContent::Text(text) => text.as_bytes().to_vec(),
                        hick_exec::node::FileContent::Binary(binary) => binary.to_bytes()?,
                    };
                    generated
                        .insert(path, hick_lang::ProjectEntry::file(bytes))
                        .map_err(anyhow::Error::msg)?;
                }
                pending::accept_evaluated(&root, &input.entry, &id, &source, &generated)
                    .map_err(anyhow::Error::msg)?;
            } else {
                pending::accept(&root, &input.entry, &id).map_err(anyhow::Error::msg)?;
            }
            println!("Capture incorporated; prior observations archived.");
            return Ok(ExitCode::SUCCESS);
        }
        ProjectCommand::NewPatch {
            input,
            id,
            document,
            patch_id,
            explanation,
        } => {
            pending::new_patch(
                &input.root,
                &input.entry,
                &id,
                &document,
                &patch_id,
                &explanation,
            )
            .map_err(anyhow::Error::msg)?;
            println!("Authored {document}; review it, then accept the pending capture.");
            return Ok(ExitCode::SUCCESS);
        }
        other => other,
    };
    let (input, verify) = match command {
        ProjectCommand::VerifyIndex {
            root,
            standalone_list,
        } => {
            crate::project_index::verify(&root, standalone_list.as_deref())?;
            return Ok(ExitCode::SUCCESS);
        }
        ProjectCommand::Generate(a) => (a, false),
        ProjectCommand::Verify(a) => (a, true),
        _ => unreachable!("working-tree synchronization handled above"),
    };
    hick_lang::validate_project_path(&input.entry).map_err(anyhow::Error::msg)?;
    let snapshot = if let Some(revision) = &input.revision {
        Some(project_revision::git(&input.root, revision)?)
    } else if let Some(revision) = &input.hg_revision {
        Some(project_revision::mercurial(&input.root, revision)?)
    } else {
        None
    };
    let read = |path: &str| -> Result<String> {
        if let Some(tree) = &snapshot {
            let entry = tree
                .get(path)
                .with_context(|| format!("candidate revision lacks Markdown source {path}"))?;
            if entry.kind != EntryKind::File {
                bail!("Markdown source is not a regular file: {path}");
            }
            Ok(std::str::from_utf8(&entry.bytes)?.to_owned())
        } else {
            let full = input.root.join(path);
            if !std::fs::symlink_metadata(&full)?.is_file() {
                bail!("Markdown source is not a regular file: {path}");
            }
            Ok(std::fs::read_to_string(full)?)
        }
    };
    let mut sources = BTreeMap::new();
    load_closure(&input.entry, &read, &mut sources)?;
    let evaluated = evaluate_project(&input.entry, &sources, &ProjectTree::default())
        .map_err(anyhow::Error::msg)?;
    if verify {
        let expected = match snapshot {
            Some(tree) => evaluated
                .inventory
                .select(&tree)
                .map_err(anyhow::Error::msg)?,
            None => project_revision::directory(&input.root, Some(&evaluated.inventory))?,
        };
        let differences = evaluated.new.compare(&expected);
        if differences.is_empty() {
            println!(
                "Verified complete project source ({} entries); no execution or materialization.",
                evaluated.new.entries().len()
            );
            return Ok(ExitCode::SUCCESS);
        }
        for difference in differences {
            eprintln!("{:?}: {}", difference.kind, difference.path);
        }
        Ok(ExitCode::from(1))
    } else {
        for (path, entry) in evaluated.new.entries() {
            println!(
                "{:?} {} {} bytes {}",
                entry.kind,
                if entry.executable {
                    "executable"
                } else {
                    "ordinary"
                },
                entry.bytes.len(),
                path
            );
        }
        Ok(ExitCode::SUCCESS)
    }
}
pub(crate) fn load_closure(
    path: &str,
    read: &impl Fn(&str) -> Result<String>,
    sources: &mut BTreeMap<String, String>,
) -> Result<()> {
    *sources =
        hick_lang::load_project_sources(path, &|name| read(name).map_err(|e| format!("{e:#}")))
            .map_err(anyhow::Error::msg)?;
    Ok(())
}
