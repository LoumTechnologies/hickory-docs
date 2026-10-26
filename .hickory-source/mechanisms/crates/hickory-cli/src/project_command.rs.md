<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/project_command.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-3d5623416448"></a>
## `crates/hickory-cli/src/project_command.rs`

Related contracts: [Candidate revision checks never borrow working bytes](../../../../guarantees/literate-projects/candidate-revisions-never-borrow-working-bytes/contract.md).

<a id="f-3d5623416448-ProjectArgs"></a>
### `ProjectArgs`

<storycode:copy id="f-3d5623416448-ProjectArgs">
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
</storycode:copy>
<a id="f-3d5623416448-ProjectCommand"></a>
### `ProjectCommand`

<storycode:copy id="f-3d5623416448-ProjectCommand">
#[derive(Subcommand)]
pub enum ProjectCommand {
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
</storycode:copy>
<a id="f-3d5623416448-ProjectInput"></a>
### `ProjectInput`

<storycode:copy id="f-3d5623416448-ProjectInput">
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
</storycode:copy>
<a id="f-3d5623416448-run"></a>
### `run`

<storycode:copy id="f-3d5623416448-run">
pub fn run(args: ProjectArgs) -> Result<ExitCode> {
    let (input, verify) = match args.command {
        ProjectCommand::VerifyIndex {
            root,
            standalone_list,
        } => {
            crate::project_index::verify(&root, standalone_list.as_deref())?;
            return Ok(ExitCode::SUCCESS);
        }
        ProjectCommand::Generate(a) => (a, false),
        ProjectCommand::Verify(a) => (a, true),
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
</storycode:copy>
<a id="f-3d5623416448-load_closure"></a>
### `load_closure`

<storycode:copy id="f-3d5623416448-load_closure">
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
</storycode:copy>

### Assemble `crates/hickory-cli/src/project_command.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/project_command.rs"><storycode:paste select="#f-3d5623416448-ProjectArgs"/><storycode:paste select="#f-3d5623416448-ProjectCommand"/><storycode:paste select="#f-3d5623416448-ProjectInput"/><storycode:paste select="#f-3d5623416448-run"/><storycode:paste select="#f-3d5623416448-load_closure"/></storycode:file>

</storycode:doc>
