//! A scaffold becomes a commit, as one act.
//!
//! `docs/specs/freeform/lenses.md`, step 3. A scaffold is an act that
//! happened once, so it is recorded where acts go — git — as a commit whose
//! trailers carry its recipe. The moment between "the scaffolder wrote its
//! files" and "they are committed" is the moment that breaks upgradeability
//! (an edit in there fuses into the recipe commit and cannot be separated
//! from it later), so this module does not let that moment exist: run, then
//! commit, with nothing a person can do in between.
//!
//! ## Why a temporary index
//!
//! The person may have work in flight — staged hunks, an unstaged edit — and
//! a `git add -A && git commit` would sweep it into the recipe commit, which
//! would then be a lie about what the scaffolder wrote. So the commit is
//! built through git's plumbing on an index of its own: `read-tree HEAD`,
//! add exactly the scaffold's files, `write-tree`, `commit-tree`,
//! `update-ref`. The real index and working tree are not read; afterwards the
//! scaffold's paths are added to the real index so `git status` agrees with
//! HEAD, and nothing else in it moves.
//!
//! ## What the trailer proves
//!
//! `Hick-Output` is the git tree hash of the scaffolded directory as the
//! scaffolder wrote it. Because the commit is made from that same index, the
//! commit's own tree at that path has that hash — and the history lens checks
//! it with one `git rev-parse`, no replay needed. A commit whose tree does
//! not match its trailer was edited before it was committed (or had its
//! trailer written by hand), and the lens says so instead of offering to
//! upgrade something it cannot separate.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::scaffold::ScaffoldSpec;

/// The folder the app has open is not a git repository.
///
/// A **type**, like `NoDotnetSdk`: the dialog draws its own sentence for
/// this, and a reworded message must not be able to take it away. A folder
/// of notes with no repository is normal; it is only that a scaffold has
/// nowhere to be recorded in one.
#[derive(Debug, Clone, Copy)]
pub struct NotARepository;

impl std::fmt::Display for NotARepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            "this folder is not a git repository, so a scaffold has nowhere to be recorded: a \
             new project is a commit that carries the command that made it.\n  \
             Next step: open a terminal on the folder and run `git init`, then try again.",
        )
    }
}

impl std::error::Error for NotARepository {}

/// What the commit carried.
#[derive(Debug, Clone)]
pub struct ScaffoldCommit {
    pub sha: String,
    pub short: String,
    /// The paths committed, relative to the repository.
    pub files: Vec<String>,
    /// The message, trailers included.
    pub message: String,
    /// The tree hash of the scaffolded directory — the `Hick-Output` value.
    pub output_tree: String,
}

/// Run git in `root`, with an index of our own when `index` is given.
fn git(root: &Path, args: &[&str], index: Option<&Path>) -> Result<std::process::Output> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(root)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0");
    if let Some(index) = index {
        command.env("GIT_INDEX_FILE", index);
    }
    command
        .output()
        .with_context(|| format!("could not run `git {}`", args.join(" ")))
}

/// Run git and turn a refusal into git's own words.
fn git_ok(root: &Path, args: &[&str], index: Option<&Path>) -> Result<String> {
    let out = git(root, args, index)?;
    if out.status.success() {
        return Ok(String::from_utf8_lossy(&out.stdout).trim_end().to_string());
    }
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let said = if stderr.is_empty() { stdout } else { stderr };
    bail!(
        "git {} refused: {}",
        args.first().copied().unwrap_or(""),
        if said.is_empty() {
            "no reason given".to_string()
        } else {
            said
        }
    )
}

pub fn is_repository(root: &Path) -> bool {
    git(root, &["rev-parse", "--is-inside-work-tree"], None)
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// The output directory, checked: relative, inside the repository, and not
/// the repository itself.
///
/// The root is refused rather than allowed: `-o .` would mix the scaffold's
/// tree with everything already there, and `Hick-Output` would then name the
/// whole repository's tree, which is not what the scaffolder wrote.
pub fn checked_output(output: &str) -> Result<String> {
    let trimmed = output.trim().trim_end_matches('/');
    if trimmed.is_empty() || trimmed == "." {
        bail!(
            "name a folder for the project's files. A scaffold is committed as its own tree, \
             so it needs a folder of its own rather than the root of the repository."
        );
    }
    let path = Path::new(trimmed);
    if path.is_absolute()
        || path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        bail!(
            "`{trimmed}` leaves the folder you have open. Name a folder inside it, like \
             `apps/greeter`."
        );
    }
    Ok(trimmed.replace('\\', "/"))
}

/// Refuse a target that already holds anything.
///
/// A scaffold is committed as it was written, so its folder must be its own.
/// Overwriting is not offered: whether the files there are yours, tracked,
/// or half of a previous attempt, they are not the scaffolder's, and a
/// recipe commit over them would claim they were.
pub fn refuse_occupied(root: &Path, output: &str) -> Result<()> {
    let target = root.join(output);
    if target.exists() {
        let occupied = std::fs::read_dir(&target)
            .map(|mut entries| entries.next().is_some())
            .unwrap_or(true);
        if occupied {
            bail!(
                "`{output}/` already exists and is not empty. A scaffold is committed exactly as \
                 the scaffolder wrote it, so it needs an empty folder.\n  \
                 Next step: choose another folder name, or move what is there first."
            );
        }
    }
    Ok(())
}

/// Copy the scaffolder's output into the repository.
fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    std::fs::create_dir_all(to).with_context(|| format!("creating {}", to.display()))?;
    for entry in std::fs::read_dir(from).with_context(|| format!("reading {}", from.display()))? {
        let entry = entry?;
        let source = entry.path();
        let target = to.join(entry.file_name());
        if source.is_dir() {
            copy_tree(&source, &target)?;
        } else {
            std::fs::copy(&source, &target)
                .with_context(|| format!("copying {} to {}", source.display(), target.display()))?;
        }
    }
    Ok(())
}

/// Commit the scaffolder's output at `from_dir` into `root/<spec.output>`,
/// through a temporary index, with the recipe in the message.
///
/// Returns what was committed. On any failure before `update-ref`, nothing
/// has been committed; the copied files may be on disk, and the caller says
/// so.
pub fn commit_scaffold(
    root: &Path,
    spec: &ScaffoldSpec,
    from_dir: &Path,
) -> Result<ScaffoldCommit> {
    if !is_repository(root) {
        bail!(NotARepository);
    }
    let output = checked_output(&spec.output)?;
    refuse_occupied(root, &output)?;

    copy_tree(from_dir, &root.join(&output))?;

    // An index of our own. The person's real index — whatever they have
    // staged — is neither read nor written by the commit.
    let scratch = tempfile::tempdir().context("making a scratch directory for the index")?;
    let index: PathBuf = scratch.path().join("index");
    let head = git(root, &["rev-parse", "--verify", "-q", "HEAD"], None)?;
    let head = head
        .status
        .success()
        .then(|| String::from_utf8_lossy(&head.stdout).trim().to_string())
        .filter(|h| !h.is_empty());
    match &head {
        Some(head) => {
            git_ok(root, &["read-tree", head], Some(&index))?;
        }
        None => {
            git_ok(root, &["read-tree", "--empty"], Some(&index))?;
        }
    }
    // `git add` honours the repository's own `.gitignore`, the same filter
    // the ingest used: `obj/` from a restore stays out.
    git_ok(root, &["add", "--", &output], Some(&index))?;
    let files: Vec<String> = git_ok(root, &["ls-files", "--", &output], Some(&index))?
        .lines()
        .map(str::to_string)
        .filter(|l| !l.is_empty())
        .collect();
    if files.is_empty() {
        bail!(
            "the scaffolder wrote nothing under `{output}/` that git would keep. Either it wrote \
             nothing, or everything it wrote is ignored by this repository's .gitignore."
        );
    }
    let tree = git_ok(root, &["write-tree"], Some(&index))?;
    let output_tree = git_ok(root, &["rev-parse", &format!("{tree}:{output}")], None)?;

    let message = crate::scaffold::commit_message(spec, &output_tree, &output);
    let message_file = scratch.path().join("message");
    std::fs::write(&message_file, &message).context("writing the commit message")?;
    let message_arg = message_file.to_string_lossy().into_owned();
    let mut args: Vec<&str> = vec!["commit-tree", &tree, "-F", &message_arg];
    if let Some(head) = &head {
        args.push("-p");
        args.push(head);
    }
    let sha = git_ok(root, &args, None).with_context(|| {
        "committing the scaffold. Common cause: git has no identity here — set `user.name` \
         and `user.email` with `git config`."
            .to_string()
    })?;

    let reflog = format!("scaffold: {}", spec.name);
    let mut update: Vec<&str> = vec!["update-ref", "-m", &reflog, "HEAD", &sha];
    if let Some(head) = &head {
        update.push(head);
    }
    git_ok(root, &update, None)?;
    // Now the real index: the scaffold's paths, and nothing else, so `git
    // status` agrees with HEAD about them.
    git_ok(root, &["add", "--", &output], None)?;
    let short = git_ok(root, &["rev-parse", "--short", &sha], None)?;

    Ok(ScaffoldCommit {
        sha,
        short,
        files,
        message,
        output_tree,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(output: &str) -> ScaffoldSpec {
        ScaffoldSpec {
            template: "console".into(),
            title: "Console App".into(),
            language: Some("C#".into()),
            name: "Greeter".into(),
            output: output.into(),
            image: "mcr.microsoft.com/dotnet/sdk:10.0".into(),
            options: vec![],
        }
    }

    fn sh(root: &Path, args: &[&str]) -> String {
        let out = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .env("GIT_AUTHOR_NAME", "T")
            .env("GIT_AUTHOR_EMAIL", "t@example.com")
            .env("GIT_COMMITTER_NAME", "T")
            .env("GIT_COMMITTER_EMAIL", "t@example.com")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    fn repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        sh(dir.path(), &["init", "-q", "-b", "master"]);
        sh(dir.path(), &["config", "user.email", "t@example.com"]);
        sh(dir.path(), &["config", "user.name", "T"]);
        std::fs::write(dir.path().join("notes.md"), "# Notes\n").unwrap();
        std::fs::write(dir.path().join(".gitignore"), "obj/\n").unwrap();
        sh(dir.path(), &["add", "-A"]);
        sh(dir.path(), &["commit", "-qm", "start"]);
        dir
    }

    fn scaffolder_output() -> tempfile::TempDir {
        let out = tempfile::tempdir().unwrap();
        std::fs::write(out.path().join("Program.cs"), "// scaffolded\n").unwrap();
        std::fs::write(out.path().join("Greeter.csproj"), "<Project />\n").unwrap();
        std::fs::create_dir_all(out.path().join("obj")).unwrap();
        std::fs::write(out.path().join("obj/project.assets.json"), "{}").unwrap();
        out
    }

    #[test]
    fn the_output_folder_is_its_own_and_inside_the_repository() {
        // Protects docs/guarantees/authoring/a-new-project-is-a-recipe-commit.md
        assert_eq!(checked_output("greeter/").unwrap(), "greeter");
        assert_eq!(checked_output("apps/greeter").unwrap(), "apps/greeter");
        assert!(checked_output(".").is_err());
        assert!(checked_output("  ").is_err());
        assert!(checked_output("../out").is_err());
        assert!(checked_output("/tmp/out").is_err());
    }

    #[test]
    fn a_scaffold_is_committed_through_its_own_index_and_leaves_staged_work_alone() {
        // Protects docs/guarantees/authoring/a-new-project-is-a-recipe-commit.md
        let dir = repo();
        let root = dir.path();
        // Work in flight: a staged edit and an unstaged one. Neither may end
        // up in the recipe commit, and neither may be lost.
        std::fs::write(root.join("notes.md"), "# Notes\nstaged\n").unwrap();
        sh(root, &["add", "notes.md"]);
        std::fs::write(root.join("notes.md"), "# Notes\nstaged\nunstaged\n").unwrap();
        let out = scaffolder_output();

        let commit = commit_scaffold(root, &spec("greeter"), out.path()).unwrap();

        // The commit holds the scaffold's files, filtered by .gitignore, and
        // nothing of the person's.
        let changed = sh(
            root,
            &[
                "diff-tree",
                "--no-commit-id",
                "--name-only",
                "-r",
                &commit.sha,
            ],
        );
        assert_eq!(changed, "greeter/Greeter.csproj\ngreeter/Program.cs");
        assert_eq!(
            commit.files,
            vec!["greeter/Greeter.csproj", "greeter/Program.cs"]
        );
        assert_eq!(sh(root, &["rev-parse", "HEAD"]), commit.sha);
        // The trailer names the tree git holds at that path — checkable by
        // rev-parse alone.
        assert_eq!(sh(root, &["rev-parse", "HEAD:greeter"]), commit.output_tree);
        assert!(
            commit
                .message
                .contains(&format!("Hick-Output: {} greeter", commit.output_tree))
        );
        assert!(
            commit
                .message
                .contains("Hick-Recipe: dotnet new console -o greeter -n Greeter --language 'C#'")
        );
        assert!(
            commit
                .message
                .contains("Hick-Image: mcr.microsoft.com/dotnet/sdk:10.0")
        );
        // The staged edit is still staged, the unstaged one still unstaged,
        // and the scaffold's own paths are clean.
        let status = sh(root, &["status", "--porcelain"]);
        assert_eq!(status, "MM notes.md", "{status}");
        assert!(
            root.join("greeter/obj/project.assets.json").exists(),
            "ignored files stay on disk"
        );
    }

    #[test]
    fn the_first_commit_of_an_empty_repository_is_a_root_commit() {
        let dir = tempfile::tempdir().unwrap();
        sh(dir.path(), &["init", "-q", "-b", "master"]);
        sh(dir.path(), &["config", "user.email", "t@example.com"]);
        sh(dir.path(), &["config", "user.name", "T"]);
        let out = scaffolder_output();
        let commit = commit_scaffold(dir.path(), &spec("greeter"), out.path()).unwrap();
        assert_eq!(sh(dir.path(), &["rev-parse", "HEAD"]), commit.sha);
        assert_eq!(sh(dir.path(), &["rev-list", "--count", "HEAD"]), "1");
    }

    #[test]
    fn an_occupied_folder_and_a_folder_without_a_repository_are_refused_by_name() {
        let dir = repo();
        std::fs::create_dir_all(dir.path().join("taken")).unwrap();
        std::fs::write(dir.path().join("taken/x.txt"), "").unwrap();
        let out = scaffolder_output();
        let error = commit_scaffold(dir.path(), &spec("taken"), out.path()).unwrap_err();
        assert!(
            format!("{error:#}").contains("already exists and is not empty"),
            "{error:#}"
        );

        let plain = tempfile::tempdir().unwrap();
        let error = commit_scaffold(plain.path(), &spec("greeter"), out.path()).unwrap_err();
        assert!(
            error.downcast_ref::<NotARepository>().is_some(),
            "{error:#}"
        );
    }
}
