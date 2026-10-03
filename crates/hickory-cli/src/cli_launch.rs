//! Preserve clap's command errors while accepting editor-style path arguments.
use anyhow::Result;
use clap::CommandFactory;
use std::{ffi::OsString, path::Path, process::ExitCode};

pub fn arguments() -> Vec<OsString> {
    normalize(std::env::args_os().collect())
}

fn normalize(mut args: Vec<OsString>) -> Vec<OsString> {
    if args.len() == 1 {
        args.extend(["open".into(), "--blank-window".into()]);
    } else {
        let first = &args[1];
        let is_command = super::Cli::command()
            .get_subcommands()
            .any(|c| first == c.get_name() || c.get_all_aliases().any(|alias| first == alias));
        let text = first.to_string_lossy();
        // A typo like `hick tes` still gets clap's suggestion. Ambiguous folder
        // names can always be written `./test` or `hick open test`.
        if !is_command
            && !text.starts_with('-')
            && (Path::new(first).exists()
                || text.contains('/')
                || text.contains('\\')
                || text.ends_with(".md"))
        {
            args.insert(1, "open".into());
        }
    }
    args
}

pub fn open(target: Option<&Path>) -> Result<ExitCode> {
    if let Some(path) = target
        && !path.exists()
    {
        anyhow::bail!(
            "{} does not exist, so there is nothing to open.\n  \
            `hick open` takes a folder or a `.md` file and defaults to the working directory.",
            path.display()
        );
    }
    let app = hickory_cli::open_app::find(|name| std::env::var(name).ok(), |p| p.exists())
        .ok_or_else(|| anyhow::anyhow!(hickory_cli::open_app::not_installed()))?;
    if let Some(path) = target {
        hickory_cli::open_app::open(&app, path)?;
    } else {
        hickory_cli::open_app::open_blank(&app)?;
    }
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;
    // Guarantee: docs/guarantees/authoring/hick-open-hands-a-folder-to-the-app.md
    #[test]
    fn launch_forms_keep_commands_and_errors() {
        for (input, expected) in [
            (vec!["hick"], vec!["hick", "open", "--blank-window"]),
            (vec!["hick", "."], vec!["hick", "open", "."]),
            (
                vec!["hick", "missing.md"],
                vec!["hick", "open", "missing.md"],
            ),
            (vec!["hick", "test"], vec!["hick", "test"]),
            (vec!["hick", "tes"], vec!["hick", "tes"]),
            (vec!["hick", "--help"], vec!["hick", "--help"]),
            (vec!["hick", "--version"], vec!["hick", "--version"]),
        ] {
            assert_eq!(
                normalize(input.into_iter().map(OsString::from).collect()),
                expected.into_iter().map(OsString::from).collect::<Vec<_>>()
            );
        }
    }
}
