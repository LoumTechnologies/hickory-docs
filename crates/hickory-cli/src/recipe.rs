//! Recipe commits: a command, run somewhere clean, whose output becomes a
//! commit that carries the command in its trailers.
//!
//! `docs/specs/freeform/lenses.md`, steps 4 and 5. Two verbs share one
//! routine here:
//!
//! * **Replay** takes a recipe commit S1, runs its `Hick-Recipe` again, and
//!   makes S2 — a *new* commit, never a rewrite of S1
//!   (`expression-and-log.md`: nothing may re-produce a commit that exists).
//!   Above the publication floor S2 is S1's sibling and the commits after S1
//!   are rebased onto it; below the floor S2 is S1's child and is merged.
//! * **The tail** runs a command a person typed and commits its output as a
//!   child of HEAD, with the recipe — `hick emit` with a place to type it.
//!
//! ## Where the command runs
//!
//! In a **detached worktree** checked out at a commit, never in the person's
//! own working tree. That is what makes the output commit honest about what
//! the command wrote: the person's uncommitted work is not in the directory
//! the command sees, and nothing the command writes lands beside their
//! files. The repository's own `.gitignore` applies, because a worktree is a
//! real checkout of it. The worktree is removed with the run.
//!
//! ## What the commit holds
//!
//! The parent's tree with the output folder replaced by what the command
//! wrote — composed through a temporary index (`read-tree`, `rm --cached`,
//! `read-tree --prefix`, `write-tree`, `commit-tree`), so the person's real
//! index is untouched. `Hick-Output` is the tree hash of that folder, which
//! is what lets the history lens compare a replay to the commit it replayed
//! with one `rev-parse` (`Hick-Replay-Same`).

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

/// What one recipe run produced.
#[derive(Debug, Clone)]
pub struct RecipeRun {
    /// The new commit. NOT yet on any ref: the caller decides where it goes.
    pub sha: String,
    pub short: String,
    /// The tree hash of the output folder — the `Hick-Output` value.
    pub output_tree: String,
    /// The command's own output, for a terminal or a failure message.
    pub said: Vec<String>,
}

/// A recipe as a commit records it.
#[derive(Debug, Clone)]
pub struct Recipe {
    pub command: String,
    pub image: Option<String>,
    /// The folder the command writes, relative to the repository.
    pub output_path: String,
}

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
pub fn git_ok(root: &Path, args: &[&str], index: Option<&Path>) -> Result<String> {
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

/// Whether the working tree and index hold nothing uncommitted.
///
/// Replay and the tail both refuse a dirty tree, the way the Git pane's
/// verbs do: a rebase or a merge over uncommitted work is a decision with
/// conflicts in it that nobody asked for.
pub fn is_clean(root: &Path) -> Result<bool> {
    Ok(git_ok(root, &["status", "--porcelain"], None)?
        .trim()
        .is_empty())
}

/// The whole message of a commit.
pub fn message_of(root: &Path, sha: &str) -> Result<String> {
    git_ok(root, &["log", "-1", "--format=%B", sha], None)
}

/// The parents of a commit.
pub fn parents_of(root: &Path, sha: &str) -> Result<Vec<String>> {
    Ok(git_ok(root, &["log", "-1", "--format=%P", sha], None)?
        .split_whitespace()
        .map(str::to_string)
        .collect())
}

/// Run the recipe in a detached worktree at `run_at` and write the output
/// folder's tree object. Returns `(tree, said)`.
fn run_in_worktree(root: &Path, recipe: &Recipe, run_at: &str) -> Result<(String, Vec<String>)> {
    let scratch = tempfile::tempdir().context("making a scratch directory for the run")?;
    let worktree: PathBuf = scratch.path().join("wt");
    let worktree_arg = worktree.to_string_lossy().into_owned();
    git_ok(
        root,
        &[
            "worktree",
            "add",
            "--detach",
            "--quiet",
            &worktree_arg,
            run_at,
        ],
        None,
    )?;
    // Whatever failed, the worktree must not be left registered.
    let result = (|| -> Result<(String, Vec<String>)> {
        // A scaffolder refuses a folder that is not empty, and a replay's
        // checkout holds the old scaffold. The command writes it afresh.
        let target = worktree.join(&recipe.output_path);
        if target.exists() {
            std::fs::remove_dir_all(&target)
                .with_context(|| format!("clearing {}", target.display()))?;
        }
        let said = run_command(&recipe.command, &worktree)?;
        // The output folder's tree, from an index of its own in the worktree.
        let index = scratch.path().join("index");
        git_ok(&worktree, &["read-tree", "--empty"], Some(&index))?;
        git_ok(&worktree, &["add", "--", &recipe.output_path], Some(&index))?;
        let whole = git_ok(&worktree, &["write-tree"], Some(&index))?;
        let tree = git(
            &worktree,
            &[
                "rev-parse",
                "--verify",
                "-q",
                &format!("{whole}:{}", recipe.output_path),
            ],
            None,
        )?;
        if !tree.status.success() {
            bail!(
                "the command wrote nothing under `{}/` that git would keep. Either it wrote \
                 nothing there, or everything it wrote is ignored by this repository's \
                 .gitignore.\n{}",
                recipe.output_path,
                said.join("\n")
            );
        }
        Ok((
            String::from_utf8_lossy(&tree.stdout).trim().to_string(),
            said,
        ))
    })();
    let _ = git(
        root,
        &["worktree", "remove", "--force", &worktree_arg],
        None,
    );
    let _ = git(root, &["worktree", "prune"], None);
    result
}

/// Run one command line through the platform shell, in `cwd`.
///
/// The recipe is a line a person would type, spelled the way `Hick-Recipe`
/// records it; the shell is what a person's own typing goes through. Nothing
/// is sandboxed here, and that is not new: the scaffold that wrote the
/// recipe ran the same way.
fn run_command(command: &str, cwd: &Path) -> Result<Vec<String>> {
    let output = if cfg!(windows) {
        Command::new("cmd")
            .args(["/C", command])
            .current_dir(cwd)
            .output()
    } else {
        Command::new("sh")
            .args(["-c", command])
            .current_dir(cwd)
            .output()
    }
    .with_context(|| format!("could not run `{command}`"))?;
    let mut said: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect();
    said.extend(
        String::from_utf8_lossy(&output.stderr)
            .lines()
            .map(str::to_string),
    );
    if !output.status.success() {
        bail!(
            "`{command}` failed (exit {}). Its own output says why:\n{}",
            output.status.code().unwrap_or(-1),
            said.join("\n")
        );
    }
    Ok(said)
}

/// `parent`'s tree with `path` replaced by `tree`, as a tree object.
fn compose_tree(root: &Path, parent: Option<&str>, path: &str, tree: &str) -> Result<String> {
    let scratch = tempfile::tempdir().context("making a scratch index")?;
    let index = scratch.path().join("index");
    match parent {
        Some(parent) => git_ok(root, &["read-tree", parent], Some(&index))?,
        None => git_ok(root, &["read-tree", "--empty"], Some(&index))?,
    };
    git_ok(
        root,
        &["rm", "-r", "-q", "--cached", "--ignore-unmatch", "--", path],
        Some(&index),
    )?;
    let prefix = format!("--prefix={}/", path.trim_end_matches('/'));
    git_ok(root, &["read-tree", &prefix, tree], Some(&index))?;
    git_ok(root, &["write-tree"], Some(&index))
}

/// Make the commit object: `parent`'s tree with the output replaced, and
/// `message`. Not on any ref yet.
fn commit_object(root: &Path, parent: Option<&str>, tree: &str, message: &str) -> Result<String> {
    let scratch = tempfile::tempdir().context("making a scratch directory")?;
    let file = scratch.path().join("message");
    std::fs::write(&file, message).context("writing the commit message")?;
    let file_arg = file.to_string_lossy().into_owned();
    let mut args: Vec<&str> = vec!["commit-tree", tree, "-F", &file_arg];
    if let Some(parent) = parent {
        args.push("-p");
        args.push(parent);
    }
    git_ok(root, &args, None).with_context(|| {
        "committing. Common cause: git has no identity here — set `user.name` and \
         `user.email` with `git config`."
            .to_string()
    })
}

/// The command has run and its output has been composed into a tree; the
/// commit itself is still to be made, so the message can say what the tree
/// turned out to be.
struct Composed {
    tree: String,
    output_tree: String,
    said: Vec<String>,
}

fn run_and_compose(
    root: &Path,
    recipe: &Recipe,
    run_at: &str,
    parent: Option<&str>,
) -> Result<Composed> {
    let (output_tree, said) = run_in_worktree(root, recipe, run_at)?;
    let tree = compose_tree(root, parent, &recipe.output_path, &output_tree)?;
    Ok(Composed {
        tree,
        output_tree,
        said,
    })
}

fn commit_composed(
    root: &Path,
    recipe: &Recipe,
    parent: Option<&str>,
    composed: Composed,
    subject: &str,
    prose: &str,
    extra: &[(&str, &str)],
) -> Result<RecipeRun> {
    let mut message = format!("{subject}\n\n{prose}\n\nHick-Recipe: {}\n", recipe.command);
    if let Some(image) = &recipe.image {
        message.push_str(&format!("Hick-Image: {image}\n"));
    }
    message.push_str(&format!(
        "Hick-Output: {} {}\n",
        composed.output_tree, recipe.output_path
    ));
    for (key, value) in extra {
        message.push_str(&format!("{key}: {value}\n"));
    }
    let sha = commit_object(root, parent, &composed.tree, &message)?;
    let short = git_ok(root, &["rev-parse", "--short", &sha], None)?;
    Ok(RecipeRun {
        sha,
        short,
        output_tree: composed.output_tree,
        said: composed.said,
    })
}

/// Run `recipe` in a worktree at `run_at` and commit its output as a child
/// of `parent`, with `subject`, `prose` and the recipe trailers (plus any
/// `extra` trailers). The commit is made but put on no ref.
pub fn record(
    root: &Path,
    recipe: &Recipe,
    run_at: &str,
    parent: Option<&str>,
    subject: &str,
    prose: &str,
    extra: &[(&str, &str)],
) -> Result<RecipeRun> {
    let composed = run_and_compose(root, recipe, run_at, parent)?;
    commit_composed(root, recipe, parent, composed, subject, prose, extra)
}

/// How a replay was joined to the history.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Moved {
    /// Above the floor: the commits after S1 were rebased onto S2.
    Rebase,
    /// Below the floor: S2 (a child of S1) was merged into HEAD.
    Merge,
}

/// What a replay did.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Replay {
    /// The commit replayed.
    pub of: String,
    /// The new recipe commit.
    pub sha: String,
    pub short: String,
    /// Whether the new output tree is the one S1 recorded.
    pub same: bool,
    pub moved: Moved,
    /// HEAD afterwards.
    pub head: String,
    pub said: Vec<String>,
}

/// The join failed in git's own words: a rebase or merge stopped on a
/// conflict and is left where git left it, for the person to finish.
#[derive(Debug)]
pub struct JoinStopped {
    pub moved: Moved,
    pub replayed: String,
    pub words: String,
}

impl std::fmt::Display for JoinStopped {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "the replay was committed as {}, but joining it stopped: {}\n  The repository is \
             where git left it (a {} in progress); finish or abort it from a terminal, the \
             way git says.",
            &self.replayed[..self.replayed.len().min(12)],
            self.words,
            match self.moved {
                Moved::Rebase => "rebase",
                Moved::Merge => "merge",
            }
        )
    }
}

impl std::error::Error for JoinStopped {}

/// Replay recipe commit `sha`: run its recipe again, commit the result as
/// S2, and join S2 to the history — rebase above the floor, merge below.
pub fn replay(root: &Path, sha: &str) -> Result<Replay> {
    if !is_clean(root)? {
        bail!(
            "the working tree has uncommitted changes. A replay rebases or merges, and doing \
             that over uncommitted work is a decision with conflicts in it that nobody asked \
             for.\n  Next step: commit or stash what is uncommitted, then replay."
        );
    }
    let sha = git_ok(
        root,
        &["rev-parse", "--verify", &format!("{sha}^{{commit}}")],
        None,
    )?;
    let message = message_of(root, &sha)?;
    let Some(recorded) = crate::serve::git::recipe_of(&message) else {
        bail!(
            "commit {sha} carries no recipe (no `Hick-Recipe` trailer), so there is nothing to replay."
        );
    };
    let Some(output_path) = recorded.output_path.clone() else {
        bail!(
            "commit {sha}'s recipe records no output folder (`Hick-Output: <tree> <folder>`), \
             so where its output goes cannot be told."
        );
    };
    let recipe = Recipe {
        command: recorded.command.clone(),
        image: recorded.image.clone(),
        output_path,
    };
    let parents = parents_of(root, &sha)?;
    if parents.len() > 1 {
        bail!("commit {sha} is a merge, and a merge is not a recipe that can be replayed.");
    }
    let floor = crate::floor::compute(root);
    let is_draft = floor.as_ref().map(|f| f.is_draft(&sha)).unwrap_or(true);
    let subject = git_ok(root, &["log", "-1", "--format=%s", &sha], None)?;
    let short_of = git_ok(root, &["rev-parse", "--short", &sha], None)?;
    let today = crate::ingest::today().unwrap_or_else(|| "an unknown date".to_string());

    // Above the floor S1 is a draft: S2 replaces it, as a sibling. Below it
    // S1 is a record someone else may hold: S2 is its child and is merged.
    let (run_at, parent, moved) = if is_draft {
        let Some(parent) = parents.first() else {
            bail!(
                "commit {sha} is the first commit of this repository, and the first commit \
                 cannot be replaced by a replay. Merge is not possible either, since there \
                 is nothing to merge into."
            );
        };
        (parent.clone(), Some(parent.clone()), Moved::Rebase)
    } else {
        (sha.clone(), Some(sha.clone()), Moved::Merge)
    };
    let composed = run_and_compose(root, &recipe, &run_at, parent.as_deref())?;
    let same = recorded.output.as_deref() == Some(composed.output_tree.as_str());
    // The prose says what a reader of `git log` needs: what was replayed,
    // and whether anything changed — with the same/differs fact also in a
    // trailer for the lens.
    let prose = format!(
        "Replayed the recipe of {short_of} on {today}. {}",
        if same {
            "The output is the same tree, so nothing about the scaffold changed."
        } else {
            "The output differs from what that commit recorded, so the scaffolder changed \
             something; the diff of this commit is what."
        }
    );
    let probe = commit_composed(
        root,
        &recipe,
        parent.as_deref(),
        composed,
        &format!("Replay: {subject}"),
        &prose,
        &[
            ("Hick-Replay-Of", sha.as_str()),
            ("Hick-Replay-Same", if same { "yes" } else { "no" }),
        ],
    )?;

    let join = match moved {
        Moved::Rebase => git_ok(root, &["rebase", "--onto", &probe.sha, &sha], None),
        Moved::Merge => git_ok(
            root,
            &[
                "merge",
                "--no-ff",
                "--no-edit",
                "-m",
                &format!("Merge replay {} of {short_of}", probe.short),
                &probe.sha,
            ],
            None,
        ),
    };
    if let Err(error) = join {
        bail!(JoinStopped {
            moved,
            replayed: probe.sha.clone(),
            words: format!("{error:#}"),
        });
    }
    let head = git_ok(root, &["rev-parse", "HEAD"], None)?;
    Ok(Replay {
        of: sha,
        sha: probe.sha,
        short: probe.short,
        same,
        moved,
        head,
        said: probe.said,
    })
}

/// The tail: run `command` in a worktree at HEAD and commit its output as
/// HEAD's child, with the recipe; then put the files in the working tree.
pub fn emit(root: &Path, command: &str, output_path: &str) -> Result<RecipeRun> {
    let output_path = crate::scaffold_commit::checked_output(output_path)?;
    crate::scaffold_commit::refuse_occupied(root, &output_path)?;
    let head = git(root, &["rev-parse", "--verify", "-q", "HEAD"], None)?;
    if !head.status.success() {
        bail!(
            "this repository has no commits yet, so there is nothing to run the command from. \
             Make a first commit, then try again."
        );
    }
    let head = String::from_utf8_lossy(&head.stdout).trim().to_string();
    let recipe = Recipe {
        command: command.trim().to_string(),
        image: None,
        output_path: output_path.clone(),
    };
    let today = crate::ingest::today().unwrap_or_else(|| "an unknown date".to_string());
    let run = record(
        root,
        &recipe,
        &head,
        Some(&head),
        &format!("Run `{}`", first_words(&recipe.command)),
        &format!(
            "Ran the command on {today} and committed what it wrote under `{output_path}/`. \
             Every byte here is the command's; nothing was edited before it was committed."
        ),
        &[],
    )?;
    git_ok(
        root,
        &["update-ref", "-m", "recipe", "HEAD", &run.sha, &head],
        None,
    )?;
    // The commit exists; now the working tree and index catch up to it for
    // the output folder alone.
    git_ok(root, &["checkout", "HEAD", "--", &output_path], None)?;
    Ok(run)
}

/// A subject-length prefix of a command line.
fn first_words(command: &str) -> String {
    let words: Vec<&str> = command.split_whitespace().take(4).collect();
    let mut out = words.join(" ");
    if command.split_whitespace().count() > 4 {
        out.push('…');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sh(root: &Path, args: &[&str]) -> String {
        let out = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
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

    /// A scaffolder whose output is whatever `version` file says, so a
    /// replay can be made to differ without changing the repository.
    fn scaffolder(version_file: &Path) -> String {
        format!(
            "mkdir -p app app/obj && cp {} app/gen.txt && echo cache > app/obj/x",
            version_file.display()
        )
    }

    #[test]
    fn a_recipe_commit_holds_only_what_the_command_wrote() {
        // Protects docs/guarantees/lenses/a-recipe-commit-can-be-replayed.md
        let dir = repo();
        let root = dir.path();
        let version = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(version.path(), "v1\n").unwrap();
        let recipe = Recipe {
            command: scaffolder(version.path()),
            image: None,
            output_path: "app".into(),
        };
        let head = sh(root, &["rev-parse", "HEAD"]);
        let run = record(root, &recipe, &head, Some(&head), "Scaffold", "prose", &[]).unwrap();
        // Not on any ref, and the working tree untouched.
        assert_eq!(sh(root, &["rev-parse", "HEAD"]), head);
        assert!(!root.join("app").exists());
        assert_eq!(sh(root, &["status", "--porcelain"]), "");
        // The commit: parent's tree plus app/, ignoring obj/ as the repo does.
        let files = sh(root, &["ls-tree", "-r", "--name-only", &run.sha]);
        assert_eq!(files, ".gitignore\napp/gen.txt\nnotes.md");
        assert_eq!(
            sh(root, &["rev-parse", &format!("{}:app", run.sha)]),
            run.output_tree
        );
        let message = message_of(root, &run.sha).unwrap();
        assert!(
            message.contains(&format!("Hick-Output: {} app", run.output_tree)),
            "{message}"
        );
        assert!(message.contains("Hick-Recipe: mkdir -p app"), "{message}");
    }

    #[test]
    fn a_replay_above_the_floor_replaces_the_scaffold_and_keeps_the_edits() {
        // Protects docs/guarantees/lenses/a-recipe-commit-can-be-replayed.md
        let dir = repo();
        let root = dir.path();
        let version = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(version.path(), "v1\n").unwrap();
        let recipe = Recipe {
            command: scaffolder(version.path()),
            image: None,
            output_path: "app".into(),
        };
        // S1, on HEAD, then the person's edit in the next commit.
        let head = sh(root, &["rev-parse", "HEAD"]);
        let s1 = record(root, &recipe, &head, Some(&head), "Scaffold", "", &[]).unwrap();
        sh(root, &["update-ref", "HEAD", &s1.sha]);
        sh(root, &["checkout", "HEAD", "--", "app"]);
        std::fs::write(root.join("app/mine.txt"), "mine\n").unwrap();
        sh(root, &["add", "-A"]);
        sh(root, &["commit", "-qm", "Add mine"]);
        // The scaffolder "upgrades".
        std::fs::write(version.path(), "v2\n").unwrap();

        let replay = replay(root, &s1.sha).unwrap();
        assert_eq!(replay.moved, Moved::Rebase);
        assert!(!replay.same, "v2 differs from v1");
        // History: start → S2 → Add mine; S1 is gone from the branch.
        let log = sh(root, &["log", "--format=%s", "HEAD"]);
        assert_eq!(log, "Add mine\nReplay: Scaffold\nstart");
        assert_eq!(sh(root, &["rev-parse", "HEAD~1"]), replay.sha);
        assert_eq!(
            std::fs::read_to_string(root.join("app/gen.txt")).unwrap(),
            "v2\n"
        );
        assert_eq!(
            std::fs::read_to_string(root.join("app/mine.txt")).unwrap(),
            "mine\n"
        );
        let message = message_of(root, &replay.sha).unwrap();
        assert!(
            message.contains(&format!("Hick-Replay-Of: {}", s1.sha)),
            "{message}"
        );
        assert!(message.contains("Hick-Replay-Same: no"), "{message}");
        assert_eq!(sh(root, &["status", "--porcelain"]), "");
        assert_eq!(
            sh(root, &["worktree", "list"]).lines().count(),
            1,
            "the scratch worktree is gone"
        );

        // Replaying again, with nothing changed, says so.
        let again = super::replay(root, &replay.sha).unwrap();
        assert!(again.same);
        assert!(
            message_of(root, &again.sha)
                .unwrap()
                .contains("Hick-Replay-Same: yes")
        );
    }

    #[test]
    fn a_replay_below_the_floor_is_a_merge_and_a_dirty_tree_is_refused() {
        // Protects docs/guarantees/lenses/a-recipe-commit-can-be-replayed.md
        let dir = repo();
        let root = dir.path();
        let version = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(version.path(), "v1\n").unwrap();
        let recipe = Recipe {
            command: scaffolder(version.path()),
            image: None,
            output_path: "app".into(),
        };
        let head = sh(root, &["rev-parse", "HEAD"]);
        let s1 = record(root, &recipe, &head, Some(&head), "Scaffold", "", &[]).unwrap();
        sh(root, &["update-ref", "HEAD", &s1.sha]);
        sh(root, &["checkout", "HEAD", "--", "app"]);
        std::fs::write(root.join("app/mine.txt"), "mine\n").unwrap();
        sh(root, &["add", "-A"]);
        sh(root, &["commit", "-qm", "Add mine"]);
        // Publish everything: a bare remote, pushed, so the floor is HEAD.
        let remote = tempfile::tempdir().unwrap();
        sh(remote.path(), &["init", "-q", "--bare"]);
        sh(
            root,
            &["remote", "add", "origin", &remote.path().to_string_lossy()],
        );
        sh(root, &["push", "-q", "-u", "origin", "master"]);

        std::fs::write(root.join("notes.md"), "# Notes\ndirty\n").unwrap();
        let refused = replay(root, &s1.sha).unwrap_err();
        assert!(
            format!("{refused:#}").contains("uncommitted"),
            "{refused:#}"
        );
        sh(root, &["checkout", "--", "notes.md"]);

        std::fs::write(version.path(), "v2\n").unwrap();
        let replay = replay(root, &s1.sha).unwrap();
        assert_eq!(replay.moved, Moved::Merge);
        // S1 is still in history — it is a record — and S2 is its child,
        // merged in.
        let log = sh(root, &["log", "--format=%s", "--first-parent", "HEAD"]);
        assert!(log.starts_with("Merge replay"), "{log}");
        assert!(log.contains("Add mine\nScaffold\nstart"), "{log}");
        assert_eq!(
            sh(root, &["rev-parse", &format!("{}^", replay.sha)]),
            s1.sha
        );
        assert_eq!(
            std::fs::read_to_string(root.join("app/gen.txt")).unwrap(),
            "v2\n"
        );
        assert_eq!(
            std::fs::read_to_string(root.join("app/mine.txt")).unwrap(),
            "mine\n"
        );
    }

    #[test]
    fn the_tail_commits_a_commands_output_as_a_recipe_child_of_head() {
        // Protects docs/guarantees/lenses/the-tail-of-the-story-is-the-next-commit.md
        let dir = repo();
        let root = dir.path();
        let before = sh(root, &["rev-parse", "HEAD"]);
        let run = emit(root, "mkdir -p out && echo hi > out/a.txt", "out").unwrap();
        assert_eq!(sh(root, &["rev-parse", "HEAD"]), run.sha);
        assert_eq!(sh(root, &["rev-parse", "HEAD^"]), before);
        assert_eq!(
            std::fs::read_to_string(root.join("out/a.txt")).unwrap(),
            "hi\n"
        );
        assert_eq!(sh(root, &["status", "--porcelain"]), "");
        let message = message_of(root, &run.sha).unwrap();
        assert!(message.starts_with("Run `mkdir -p out &&…`"), "{message}");
        assert!(
            message.contains("Hick-Recipe: mkdir -p out && echo hi > out/a.txt"),
            "{message}"
        );
        // An occupied folder is refused before anything runs.
        let refused = emit(root, "echo x > out/b.txt", "out").unwrap_err();
        assert!(format!("{refused:#}").contains("already exists"));
    }
}
