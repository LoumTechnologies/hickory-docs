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

/// The folder a project would land in is not inside a git repository.
///
/// A **type**, like `NoDotnetSdk`: the dialog draws its own screen for this,
/// and a reworded message must not be able to take it away. It carries the
/// folder, because a project may be made anywhere on this machine and the
/// answer — offering to make a repository there — has to name *which* folder
/// it would make one in.
#[derive(Debug, Clone)]
pub struct NotARepository {
    /// The folder that would hold the project, and so the folder a `git
    /// init` would be run in.
    pub path: PathBuf,
}

impl std::fmt::Display for NotARepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} is not inside a git repository, so a scaffold has nowhere to be recorded: a \
             new project is a commit that carries the command that made it.\n  \
             Next step: make a repository there, or choose a folder inside one.",
            self.path.display()
        )
    }
}

impl std::error::Error for NotARepository {}

/// Where a project will land, and which repository will record it.
///
/// The dialog asks two things — a **location** (any folder on this machine)
/// and a **folder name** — and everything downstream needs a third: the
/// repository the recipe commit is made in, which is whichever one contains
/// the location. That is the one this resolves, so `output` below is always
/// what `-o` must say when the recipe is replayed *from the repository root*,
/// however far from that root the person was pointing.
#[derive(Debug, Clone)]
pub struct Target {
    /// The repository the recipe commit is made in.
    pub repo_root: PathBuf,
    /// The project's folder, relative to `repo_root`, `/`-separated. What
    /// the recipe's `-o` records and what `Hick-Output` names.
    pub output: String,
    /// That folder, absolute.
    pub dir: PathBuf,
    /// The location the person named, absolute — the parent of `dir`.
    pub location: PathBuf,
}

/// Expand a leading `~`, then make the path absolute against `base`.
///
/// A relative location is read against the folder the app has open, which is
/// what makes the field's default (`.`) and a typed `../elsewhere` both mean
/// the obvious thing.
pub fn absolute_folder(base: &Path, location: &str) -> PathBuf {
    let trimmed = location.trim();
    if trimmed.is_empty() {
        return base.to_path_buf();
    }
    let expanded: PathBuf = if trimmed == "~" {
        home()
    } else if let Some(rest) = trimmed
        .strip_prefix("~/")
        .or_else(|| trimmed.strip_prefix("~\\"))
    {
        home().join(rest)
    } else {
        PathBuf::from(trimmed)
    };
    if expanded.is_absolute() {
        normalize(&expanded)
    } else {
        normalize(&base.join(expanded))
    }
}

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

/// Resolve `.` and `..` lexically, and canonicalize as far as the filesystem
/// already goes.
///
/// Both halves are needed. Lexical resolution is what lets a person type
/// `../scratch` for a folder that does not exist yet; canonicalizing the part
/// that *does* exist is what makes `strip_prefix` against git's own answer
/// work, since `git rev-parse --show-toplevel` reports a real path and
/// `/tmp` is a symlink on macOS.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    // The longest existing prefix, canonicalized, with the rest put back.
    let mut existing = out.clone();
    let mut rest: Vec<std::ffi::OsString> = Vec::new();
    while !existing.exists() {
        match existing.file_name() {
            Some(name) => {
                rest.push(name.to_os_string());
                if !existing.pop() {
                    return out;
                }
            }
            None => return out,
        }
    }
    let mut resolved = existing.canonicalize().unwrap_or(existing);
    for name in rest.into_iter().rev() {
        resolved.push(name);
    }
    resolved
}

/// The repository containing `dir` — which need not exist yet, so the walk
/// starts at its nearest existing ancestor.
pub fn repository_of(dir: &Path) -> Option<PathBuf> {
    let mut probe = dir.to_path_buf();
    while !probe.is_dir() {
        if !probe.pop() {
            return None;
        }
    }
    let top = git(&probe, &["rev-parse", "--show-toplevel"], None).ok()?;
    if !top.status.success() {
        return None;
    }
    let root = String::from_utf8_lossy(&top.stdout).trim().to_string();
    if root.is_empty() {
        return None;
    }
    Some(normalize(Path::new(&root)))
}

/// Make sure `dir` is inside a git repository, making one there if it is not.
///
/// The New Project checkbox, in one function. Its wording is "create a git
/// repository", and its meaning is "see to it that there is one" — so a
/// location already inside a repository is answered with **that** repository
/// and nothing is created. That is not laxity, it is the whole safety
/// property: a `git init` in a subfolder of a repository makes a nested one,
/// which is a mess neither git nor a person recovers from quickly, and this
/// is the only path in the app that runs `git init` at all.
///
/// Saying it this way rather than refusing also removes a race the form would
/// otherwise have: the checkbox is ticked from a **debounced** preview, so a
/// location typed quickly can be submitted while the box still holds the
/// previous location's answer. Under a refusal that is an error message about
/// nesting for an action that was perfectly reasonable. Under this, it is
/// nothing at all.
///
/// The folder is created when it is not there: a location typed for a project
/// that does not exist yet is the ordinary case, not a mistake.
pub fn ensure_repository(dir: &Path) -> Result<PathBuf> {
    let dir = normalize(dir);
    if let Some(existing) = repository_of(&dir) {
        return Ok(existing);
    }
    std::fs::create_dir_all(&dir).with_context(|| {
        format!(
            "could not make {}. Check that the path is spelled right and that you can write \
             there.",
            dir.display()
        )
    })?;
    git_ok(&dir, &["init"], None)?;
    Ok(dir)
}

/// Where the project goes, from what the dialog asked for.
///
/// `open_folder` is the folder the app has open, which a relative location is
/// read against and which an empty one means. `folder` is the project's own
/// directory name, checked by [`checked_output`] — the same check whether the
/// project lands next to your notes or three directories away, because the
/// thing being checked is the shape of the name, not where it is.
pub fn resolve_target(open_folder: &Path, location: &str, folder: &str) -> Result<Target> {
    let folder = checked_output(folder)?;
    let location = absolute_folder(open_folder, location);
    let dir = normalize(&location.join(&folder));
    let repo_root = repository_of(&location).ok_or(NotARepository {
        path: location.clone(),
    })?;
    let output = dir
        .strip_prefix(&repo_root)
        .map(|rel| rel.to_string_lossy().replace('\\', "/"))
        .map_err(|_| {
            anyhow::anyhow!(
                "{} is not inside {}, which is the repository that would record the scaffold. \
                 Choose a folder inside that repository.",
                dir.display(),
                repo_root.display()
            )
        })?;
    if output.is_empty() || output == "." {
        bail!(
            "{} is the root of its repository. A scaffold is committed as its own tree, so it \
             needs a folder of its own — name one inside the repository instead.",
            dir.display()
        );
    }
    Ok(Target {
        repo_root,
        output,
        dir,
        location,
    })
}

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
    toolchain: crate::toolchain::Toolchain,
    spec: &ScaffoldSpec,
    from_dir: &Path,
) -> Result<ScaffoldCommit> {
    if !is_repository(root) {
        bail!(NotARepository {
            path: root.to_path_buf()
        });
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

    // The toolchain's own wording and its own recipe line — `uv init` is not
    // `dotnet new`, and a replay runs whatever `Hick-Recipe` says.
    let message = toolchain.commit_message(spec, &output_tree, &output);
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

        let commit = commit_scaffold(
            root,
            crate::toolchain::Toolchain::Dotnet,
            &spec("greeter"),
            out.path(),
        )
        .unwrap();

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
        let commit = commit_scaffold(
            dir.path(),
            crate::toolchain::Toolchain::Dotnet,
            &spec("greeter"),
            out.path(),
        )
        .unwrap();
        assert_eq!(sh(dir.path(), &["rev-parse", "HEAD"]), commit.sha);
        assert_eq!(sh(dir.path(), &["rev-list", "--count", "HEAD"]), "1");
    }

    #[test]
    fn a_location_anywhere_is_recorded_by_the_repository_that_holds_it() {
        // Protects docs/guarantees/authoring/a-new-project-is-a-recipe-commit.md:
        // the repository is whichever one contains the location, and `-o` is
        // spelled from *its* root — never from the folder the app has open.
        let open = repo();
        let elsewhere = repo();
        std::fs::create_dir_all(elsewhere.path().join("apps")).unwrap();

        let here = resolve_target(open.path(), "", "greeter").unwrap();
        assert_eq!(here.output, "greeter");
        assert_eq!(here.repo_root, normalize(open.path()));

        let there = resolve_target(
            open.path(),
            &elsewhere.path().join("apps").to_string_lossy(),
            "greeter",
        )
        .unwrap();
        assert_eq!(there.repo_root, normalize(elsewhere.path()));
        assert_eq!(there.output, "apps/greeter");
        assert_eq!(there.dir, normalize(&elsewhere.path().join("apps/greeter")));

        // A location relative to the open folder, and one that does not exist
        // yet — the scaffolder makes it.
        let nested = resolve_target(open.path(), "apps/new", "greeter").unwrap();
        assert_eq!(nested.output, "apps/new/greeter");
        assert!(!nested.dir.exists());
    }

    #[test]
    fn a_location_in_no_repository_names_the_folder_it_would_make_one_in() {
        let plain = tempfile::tempdir().unwrap();
        let bare = plain.path().join("fresh");
        let error = resolve_target(plain.path(), &bare.to_string_lossy(), "greeter").unwrap_err();
        let missing = error
            .downcast_ref::<NotARepository>()
            .expect("the typed refusal, so a reworded sentence cannot take the button away");
        assert_eq!(missing.path, normalize(&bare));
    }

    #[test]
    fn the_repository_root_is_still_refused_however_it_is_reached() {
        // `-o .` would mix the scaffold's tree with everything already there,
        // and `Hick-Output` would name the whole repository. Pointing the
        // location at the root and naming the folder `.` is the same thing
        // said a longer way, and is refused the same way.
        let dir = repo();
        assert!(resolve_target(dir.path(), &dir.path().to_string_lossy(), ".").is_err());
        assert!(resolve_target(dir.path(), "", "../escape").is_err());
        assert!(resolve_target(dir.path(), "", "/tmp/escape").is_err());
    }

    #[test]
    fn an_occupied_folder_and_a_folder_without_a_repository_are_refused_by_name() {
        let dir = repo();
        std::fs::create_dir_all(dir.path().join("taken")).unwrap();
        std::fs::write(dir.path().join("taken/x.txt"), "").unwrap();
        let out = scaffolder_output();
        let error = commit_scaffold(
            dir.path(),
            crate::toolchain::Toolchain::Dotnet,
            &spec("taken"),
            out.path(),
        )
        .unwrap_err();
        assert!(
            format!("{error:#}").contains("already exists and is not empty"),
            "{error:#}"
        );

        let plain = tempfile::tempdir().unwrap();
        let error = commit_scaffold(
            plain.path(),
            crate::toolchain::Toolchain::Dotnet,
            &spec("greeter"),
            out.path(),
        )
        .unwrap_err();
        assert!(
            error.downcast_ref::<NotARepository>().is_some(),
            "{error:#}"
        );
    }
}
