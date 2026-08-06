//! `hickory init` — set up a plain local git repository for hickory's
//! local mode.
//!
//! Everything here is idempotent: running `hickory init` twice produces the
//! same files as running it once. Managed content lives between sentinel
//! markers so user edits outside the blocks survive refreshes.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context as _, Result, bail};

/// Start sentinel of the managed block in the pre-commit hook.
pub const HOOK_BLOCK_START: &str = "### HICKORY ###";
/// End sentinel of the managed block in the pre-commit hook.
pub const HOOK_BLOCK_END: &str = "### END HICKORY ###";
/// Start sentinel of the managed section in AGENTS.md.
pub const AGENTS_BLOCK_START: &str = "<!-- HICKORY -->";
/// End sentinel of the managed section in AGENTS.md.
pub const AGENTS_BLOCK_END: &str = "<!-- END HICKORY -->";

/// The managed body installed inside the pre-commit hook sentinels.
///
/// Discovers tracked `*.hick` documents at commit time (so newly added docs
/// are covered without re-running `hickory init`), skips cleanly when there
/// are none, and blocks the commit if `hickory check` reports drift.
const HOOK_BODY: &str = r#"# Managed by `hickory init` — do not edit inside this block.
# Re-run `hickory init` to refresh it.
hick_docs=$(git ls-files -- '*.hick')
if [ -n "$hick_docs" ]; then
    if command -v hickory >/dev/null 2>&1; then
        hick_status=0
        for hick_doc in $hick_docs; do
            hickory check "$hick_doc" || hick_status=1
        done
        if [ "$hick_status" -ne 0 ]; then
            echo "pre-commit: \`hickory check\` failed — documentation drift; commit blocked" >&2
            exit 1
        fi
    else
        echo "pre-commit: hickory not found on PATH; skipping .hick drift check" >&2
    fi
fi"#;

/// The managed body installed inside the AGENTS.md sentinels.
const AGENTS_BODY: &str = r#"## Hickory executable documents

This repository contains `.hick` documents: reproducible, verifiable,
executable documents. Every example in a `.hick` file actually runs, and
drift between the document and reality fails `hickory check` (a pre-commit
hook enforces this).

### Grammar essentials

A `.hick` file is XML-ish with one unusual rule: **only tags carrying the
`hick:` namespace prefix are structure; every other byte is raw text** —
no escaping, no CDATA, no entities. Shell one-liners, generics, and heredocs
paste in verbatim.

- Root: `<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">`
  — prose between tags is Markdown, woven to the `weave` target.
- `<hick:container name="c" image="..." />` declares an execution container.
- `<hick:exec container="c">` holds commands, run in that container.
- `<hick:expect match="exact">…</hick:expect>` (or `match="regex-lines"`)
  inside an exec pins the expected output; mismatch fails `hickory check`.
- `<hick:file path="out/x.py">` blocks are generated files, written on run.
- Execution order is the dependency DAG (containers, volumes, copy/paste
  references) — not source order.

### Golden rules for coding agents

1. Edit `.hick` sources, never the generated outputs (woven `.md`,
   `hick:file` products) — those are overwritten on every run. Use the
   lineage tooling if you must trace an output byte back to its source span.
2. After editing a document, run `hickory run <doc>` to regenerate outputs,
   then `hickory check <doc>` and fix whatever fails before committing.
3. Agent sessions live in `sessions/*.hick` (`hick:session` documents);
   `hickory promote <session>` compacts one into a clean pipeline doc.
"#;

/// Everything `hickory init` did (or verified), for reporting.
#[derive(Debug, Default)]
pub struct InitReport {
    /// Path of the pre-commit hook that now contains the managed block.
    pub hook_path: PathBuf,
    /// True if the hook file was created or its managed block changed.
    pub hook_changed: bool,
    /// True if `.hick-cache/` was appended to .gitignore.
    pub gitignore_changed: bool,
    /// True if AGENTS.md was created or its managed section changed.
    pub agents_md_changed: bool,
    /// True if `@AGENTS.md` was prepended to an existing CLAUDE.md.
    pub claude_md_changed: bool,
    /// Names of commonly needed child language servers missing from PATH.
    pub missing_language_servers: Vec<&'static str>,
}

/// Run `hickory init` against `dir` (any directory inside a git work tree).
pub fn run_init(dir: &Path) -> Result<InitReport> {
    let root = git_toplevel(dir)?;
    let mut report = InitReport::default();

    let hooks_dir = hooks_dir(&root)?;
    report.hook_path = hooks_dir.join("pre-commit");
    report.hook_changed = install_hook_block(&report.hook_path)?;
    report.gitignore_changed = ensure_gitignore_line(&root.join(".gitignore"), ".hick-cache/")?;
    report.agents_md_changed = write_agents_section(&root.join("AGENTS.md"))?;
    report.claude_md_changed = ensure_claude_md_include(&root.join("CLAUDE.md"))?;
    report.missing_language_servers = missing_language_servers();

    Ok(report)
}

/// Resolve the git work-tree root for `dir`.
fn git_toplevel(dir: &Path) -> Result<PathBuf> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("failed to run git (is git installed?)")?;
    if !out.status.success() {
        bail!(
            "{} is not inside a git repository — `hickory init` sets up local git-repo mode. \
             Run `git init` first, or use a cloud workspace instead.",
            dir.display()
        );
    }
    Ok(PathBuf::from(
        String::from_utf8_lossy(&out.stdout).trim().to_string(),
    ))
}

/// The directory where the pre-commit hook belongs, honouring `core.hooksPath`.
fn hooks_dir(root: &Path) -> Result<PathBuf> {
    let cfg = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["config", "--get", "core.hooksPath"])
        .output()
        .context("failed to run git")?;
    if cfg.status.success() {
        let raw = String::from_utf8_lossy(&cfg.stdout).trim().to_string();
        if !raw.is_empty() {
            let path = PathBuf::from(&raw);
            return Ok(if path.is_absolute() {
                path
            } else {
                root.join(path)
            });
        }
    }
    // `--git-path` resolves worktrees and gitdir files correctly.
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--git-path", "hooks"])
        .output()
        .context("failed to run git")?;
    let raw = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let path = PathBuf::from(&raw);
    Ok(if path.is_absolute() {
        path
    } else {
        root.join(path)
    })
}

/// Create or refresh the sentinel-delimited managed block in the pre-commit
/// hook at `hook_path`. Returns true if the file changed.
fn install_hook_block(hook_path: &Path) -> Result<bool> {
    let block = format!("{HOOK_BLOCK_START}\n{HOOK_BODY}\n{HOOK_BLOCK_END}\n");
    let existing = match std::fs::read_to_string(hook_path) {
        Ok(s) => Some(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => {
            return Err(e).with_context(|| format!("failed to read {}", hook_path.display()));
        }
    };

    let new_content = match &existing {
        None => format!("#!/bin/sh\n{block}"),
        Some(content) => {
            match replace_between(content, HOOK_BLOCK_START, HOOK_BLOCK_END, &block)? {
                Some(replaced) => replaced,
                None => {
                    // Existing hook without our block: append it. The block
                    // `exit 1`s on failure but otherwise falls through to the
                    // rest of the hook (there is none after; it is last).
                    let mut s = content.clone();
                    if !s.ends_with('\n') {
                        s.push('\n');
                    }
                    s.push('\n');
                    s.push_str(&block);
                    s
                }
            }
        }
    };

    let changed = existing.as_deref() != Some(new_content.as_str());
    if changed {
        if let Some(parent) = hook_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        std::fs::write(hook_path, &new_content)
            .with_context(|| format!("failed to write {}", hook_path.display()))?;
    }
    // Always ensure the hook is executable, even when content was current.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mut perms = std::fs::metadata(hook_path)?.permissions();
        if perms.mode() & 0o111 != 0o111 {
            perms.set_mode(perms.mode() | 0o755);
            std::fs::set_permissions(hook_path, perms)?;
        }
    }
    Ok(changed)
}

/// Replace the sentinel-delimited region (inclusive) in `content` with
/// `block`. Returns `Ok(None)` if no start sentinel is present, an error if
/// the block is malformed (start without end).
fn replace_between(
    content: &str,
    start: &str,
    end: &str,
    block: &str,
) -> Result<Option<String>> {
    let Some(start_idx) = content.find(start) else {
        return Ok(None);
    };
    let after_start = &content[start_idx..];
    let Some(end_rel) = after_start.find(end) else {
        bail!(
            "found `{start}` without a matching `{end}` — refusing to touch the file; \
             remove the stale block and re-run `hickory init`"
        );
    };
    let mut end_idx = start_idx + end_rel + end.len();
    // Swallow one trailing newline so the replacement block (which ends in
    // one) does not accumulate blank lines run over run.
    if content[end_idx..].starts_with('\n') {
        end_idx += 1;
    }
    let mut out = String::with_capacity(content.len());
    out.push_str(&content[..start_idx]);
    out.push_str(block);
    out.push_str(&content[end_idx..]);
    Ok(Some(out))
}

/// Append `line` to the file if no line equals it yet. Returns true if the
/// file changed.
fn ensure_gitignore_line(path: &Path, line: &str) -> Result<bool> {
    let existing = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e).with_context(|| format!("failed to read {}", path.display())),
    };
    if existing.lines().any(|l| l.trim() == line) {
        return Ok(false);
    }
    let mut out = existing;
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(line);
    out.push('\n');
    std::fs::write(path, out).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(true)
}

/// Create AGENTS.md, or refresh its managed section. Returns true if the
/// file changed.
fn write_agents_section(path: &Path) -> Result<bool> {
    let block = format!("{AGENTS_BLOCK_START}\n{AGENTS_BODY}{AGENTS_BLOCK_END}\n");
    let existing = match std::fs::read_to_string(path) {
        Ok(s) => Some(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(e).with_context(|| format!("failed to read {}", path.display())),
    };
    let new_content = match &existing {
        None => block,
        Some(content) => {
            match replace_between(content, AGENTS_BLOCK_START, AGENTS_BLOCK_END, &block)? {
                Some(replaced) => replaced,
                None => {
                    let mut s = content.clone();
                    if !s.ends_with('\n') {
                        s.push('\n');
                    }
                    s.push('\n');
                    s.push_str(&block);
                    s
                }
            }
        }
    };
    let changed = existing.as_deref() != Some(new_content.as_str());
    if changed {
        std::fs::write(path, &new_content)
            .with_context(|| format!("failed to write {}", path.display()))?;
    }
    Ok(changed)
}

/// If CLAUDE.md exists and does not reference `@AGENTS.md`, prepend the
/// include so Claude Code picks up the managed section. Returns true if the
/// file changed. A missing CLAUDE.md is left missing (Claude Code reads
/// AGENTS.md natively).
fn ensure_claude_md_include(path: &Path) -> Result<bool> {
    let existing = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e).with_context(|| format!("failed to read {}", path.display())),
    };
    if existing.contains("@AGENTS.md") {
        return Ok(false);
    }
    let new_content = format!("@AGENTS.md\n\n{existing}");
    std::fs::write(path, new_content)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(true)
}

/// Child language servers `hick-lsp` commonly spawns for `hick:file` blocks.
/// Missing ones are reported as warnings (non-fatal): docs still run and
/// check fine; you just won't get in-editor diagnostics for that language.
const COMMON_LANGUAGE_SERVERS: &[(&str, &str)] = &[
    ("rust-analyzer", "Rust"),
    ("pyright-langserver", "Python"),
    ("typescript-language-server", "TypeScript/JavaScript"),
    ("gopls", "Go"),
];

/// Which of the common child language servers are not on PATH.
fn missing_language_servers() -> Vec<&'static str> {
    COMMON_LANGUAGE_SERVERS
        .iter()
        .filter(|(bin, _)| !on_path(bin))
        .map(|(bin, _)| *bin)
        .collect()
}

/// True if `bin` resolves on PATH.
fn on_path(bin: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&paths).any(|dir| {
        let candidate = dir.join(bin);
        candidate.is_file()
    })
}

/// Print the human-readable summary for an [`InitReport`].
pub fn print_init_report(report: &InitReport) {
    let describe = |changed: bool| if changed { "updated" } else { "ok" };
    eprintln!(
        "pre-commit hook ({}): {}",
        report.hook_path.display(),
        describe(report.hook_changed)
    );
    eprintln!(".gitignore (.hick-cache/): {}", describe(report.gitignore_changed));
    eprintln!("AGENTS.md managed section: {}", describe(report.agents_md_changed));
    eprintln!("CLAUDE.md @AGENTS.md include: {}", describe(report.claude_md_changed));
    if report.missing_language_servers.is_empty() {
        eprintln!("toolchain: all common child language servers found");
    } else {
        for (bin, lang) in COMMON_LANGUAGE_SERVERS {
            if report.missing_language_servers.contains(bin) {
                eprintln!(
                    "warning: `{bin}` not found on PATH — no in-editor {lang} diagnostics \
                     inside hick:file blocks (non-fatal; install it to enable)"
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn git(dir: &Path, args: &[&str]) {
        let out = Command::new("git").arg("-C").arg(dir).args(args).output().unwrap();
        assert!(out.status.success(), "git {args:?} failed: {out:?}");
    }

    fn init_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        git(dir.path(), &["init", "-q"]);
        dir
    }

    #[test]
    fn init_is_idempotent() {
        let repo = init_repo();
        let first = run_init(repo.path()).unwrap();
        assert!(first.hook_changed);
        assert!(first.gitignore_changed);
        assert!(first.agents_md_changed);
        let hook = std::fs::read_to_string(&first.hook_path).unwrap();
        let gitignore = std::fs::read_to_string(repo.path().join(".gitignore")).unwrap();
        let agents = std::fs::read_to_string(repo.path().join("AGENTS.md")).unwrap();

        let second = run_init(repo.path()).unwrap();
        assert!(!second.hook_changed);
        assert!(!second.gitignore_changed);
        assert!(!second.agents_md_changed);
        assert!(!second.claude_md_changed);
        assert_eq!(hook, std::fs::read_to_string(&second.hook_path).unwrap());
        assert_eq!(
            gitignore,
            std::fs::read_to_string(repo.path().join(".gitignore")).unwrap()
        );
        assert_eq!(
            agents,
            std::fs::read_to_string(repo.path().join("AGENTS.md")).unwrap()
        );
    }

    #[test]
    fn hook_block_appends_to_existing_hook_and_refreshes_in_place() {
        let repo = init_repo();
        let hooks = super::hooks_dir(&repo.path().canonicalize().unwrap()).unwrap();
        std::fs::create_dir_all(&hooks).unwrap();
        let hook_path = hooks.join("pre-commit");
        std::fs::write(&hook_path, "#!/bin/sh\necho user hook\n").unwrap();

        run_init(repo.path()).unwrap();
        let content = std::fs::read_to_string(&hook_path).unwrap();
        assert!(content.starts_with("#!/bin/sh\necho user hook\n"));
        assert_eq!(content.matches(HOOK_BLOCK_START).count(), 1);
        assert_eq!(content.matches(HOOK_BLOCK_END).count(), 1);

        // Tamper inside the block; re-init must restore it without
        // duplicating and without touching the user's line.
        let tampered = content.replace("hickory check", "hickory chekc");
        std::fs::write(&hook_path, tampered).unwrap();
        run_init(repo.path()).unwrap();
        let refreshed = std::fs::read_to_string(&hook_path).unwrap();
        assert_eq!(content, refreshed);
    }

    #[test]
    fn hook_is_executable() {
        let repo = init_repo();
        let report = run_init(repo.path()).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let mode = std::fs::metadata(&report.hook_path).unwrap().permissions().mode();
            assert_eq!(mode & 0o111, 0o111, "hook not executable: {mode:o}");
        }
    }

    #[test]
    fn respects_core_hooks_path() {
        let repo = init_repo();
        git(repo.path(), &["config", "core.hooksPath", ".githooks"]);
        let report = run_init(repo.path()).unwrap();
        assert!(report.hook_path.ends_with(".githooks/pre-commit"));
        assert!(report.hook_path.exists());
    }

    #[test]
    fn gitignore_line_added_once_and_preserved() {
        let repo = init_repo();
        std::fs::write(repo.path().join(".gitignore"), "target/\n").unwrap();
        run_init(repo.path()).unwrap();
        run_init(repo.path()).unwrap();
        let content = std::fs::read_to_string(repo.path().join(".gitignore")).unwrap();
        assert_eq!(content, "target/\n.hick-cache/\n");
    }

    #[test]
    fn claude_md_prepended_only_when_present_and_missing_include() {
        let repo = init_repo();
        // No CLAUDE.md: nothing created.
        let report = run_init(repo.path()).unwrap();
        assert!(!report.claude_md_changed);
        assert!(!repo.path().join("CLAUDE.md").exists());

        // CLAUDE.md without the include: prepended once.
        std::fs::write(repo.path().join("CLAUDE.md"), "# Project notes\n").unwrap();
        let report = run_init(repo.path()).unwrap();
        assert!(report.claude_md_changed);
        let content = std::fs::read_to_string(repo.path().join("CLAUDE.md")).unwrap();
        assert_eq!(content, "@AGENTS.md\n\n# Project notes\n");

        // Already present: untouched.
        let report = run_init(repo.path()).unwrap();
        assert!(!report.claude_md_changed);
    }

    #[test]
    fn agents_md_section_appended_to_existing_and_refreshed() {
        let repo = init_repo();
        std::fs::write(repo.path().join("AGENTS.md"), "# My repo\n\nHand-written.\n").unwrap();
        run_init(repo.path()).unwrap();
        let content = std::fs::read_to_string(repo.path().join("AGENTS.md")).unwrap();
        assert!(content.starts_with("# My repo\n\nHand-written.\n"));
        assert_eq!(content.matches(AGENTS_BLOCK_START).count(), 1);
        run_init(repo.path()).unwrap();
        assert_eq!(
            content,
            std::fs::read_to_string(repo.path().join("AGENTS.md")).unwrap()
        );
    }

    #[test]
    fn init_outside_git_repo_fails() {
        let dir = tempfile::tempdir().unwrap();
        // Guard against the tempdir living under some enclosing repo.
        let err = match run_init(&dir.path().join("definitely-not-a-repo")) {
            Err(e) => e,
            Ok(_) => {
                // tempdir may itself be inside a repo on exotic setups; skip.
                return;
            }
        };
        assert!(err.to_string().contains("not inside a git repository"));
    }
}
