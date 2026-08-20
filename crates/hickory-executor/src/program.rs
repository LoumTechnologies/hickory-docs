//! Finding a program the way the thing that will run it would.
//!
//! `<hick:needs bin="python" />` asks a filesystem question — is there a
//! program by this name a cell could run? — and it used to be answered with a
//! shell string, `command -v '<bin>' > /dev/null 2>&1`, handed across the
//! executor boundary. That is wrong twice over.
//!
//! It does not work on Windows, where a cell runs through `cmd.exe /C` and
//! `command -v` does not exist, so every declared tool was reported missing
//! and a document declaring `bin="python"` refused to run on a machine with
//! python installed.
//!
//! And the shape is wrong even where it works: the dialect depends on which
//! executor and which platform will run the cell, and `needs` cannot know
//! either. Answering it here, in Rust, removes the question rather than
//! doubling the dialect — no subprocess per declared tool, and nothing to
//! quote.
//!
//! # `PATHEXT` matters in both directions
//!
//! On Windows `python` on PATH is really `python.exe`, `python.cmd` or
//! `python.bat`, so looking only for an extensionless file finds nothing. The
//! converse matters too: an extensionless file must NOT count as found,
//! because `cmd` cannot run one. Accepting it would trade a clear preflight
//! message for `'python' is not recognized` in the middle of a document.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Where `bin` would be found from `cwd`, or `None`.
pub fn find_program(bin: &str, cwd: &Path) -> Option<PathBuf> {
    find_program_in(
        bin,
        cwd,
        std::env::var_os("PATH"),
        std::env::var_os("PATHEXT"),
    )
}

/// [`find_program`] with the environment passed in.
///
/// Separate so the rules can be tested without mutating a process-wide
/// variable, which is unsound across parallel tests — the same shape
/// `policy::launcher_from` uses.
pub fn find_program_in(
    bin: &str,
    cwd: &Path,
    path_var: Option<OsString>,
    pathext: Option<OsString>,
) -> Option<PathBuf> {
    // A name with a separator in it is a path, not something to look up.
    if bin.contains('/') || (cfg!(windows) && bin.contains('\\')) {
        return runnable(&cwd.join(bin), &extensions(pathext));
    }

    let exts = extensions(pathext);
    let mut dirs: Vec<PathBuf> = Vec::new();
    // `cmd` resolves a bare name against the current directory before PATH;
    // `sh` never does. The cwd here is the cell's own workdir, so on Windows
    // it is genuinely part of the answer.
    if cfg!(windows) {
        dirs.push(cwd.to_path_buf());
    }
    if let Some(path) = path_var.as_ref() {
        dirs.extend(std::env::split_paths(path));
    }

    dirs.iter().find_map(|dir| runnable(&dir.join(bin), &exts))
}

/// The spellings Windows will try, lowercased and dot-prefixed.
fn extensions(pathext: Option<OsString>) -> Vec<String> {
    if !cfg!(windows) {
        return Vec::new();
    }
    let raw = pathext
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| ".COM;.EXE;.BAT;.CMD".to_string());
    raw.split(';')
        .map(|ext| ext.trim().to_ascii_lowercase())
        .filter(|ext| ext.starts_with('.') && ext.len() > 1)
        .collect()
}

/// `candidate` itself, or a `PATHEXT` spelling of it, if one can be run.
fn runnable(candidate: &Path, exts: &[String]) -> Option<PathBuf> {
    if !cfg!(windows) {
        return is_executable(candidate).then(|| candidate.to_path_buf());
    }
    // An extension already in PATHEXT means the name is complete.
    let named = candidate
        .extension()
        .map(|ext| format!(".{}", ext.to_string_lossy().to_ascii_lowercase()));
    if let Some(ext) = named
        && exts.contains(&ext)
        && candidate.is_file()
    {
        return Some(candidate.to_path_buf());
    }
    exts.iter().find_map(|ext| {
        let mut spelling = candidate.as_os_str().to_os_string();
        spelling.push(ext);
        let path = PathBuf::from(spelling);
        path.is_file().then_some(path)
    })
}

/// A file this platform would actually execute.
fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        // A shell requires the bit; so does anything else that would run it.
        std::fs::metadata(path)
            .map(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir_with(name: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("a temp dir");
        let path = dir.path().join(name);
        std::fs::write(&path, b"").expect("a file");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
                .expect("executable");
        }
        dir
    }

    fn path_var(dirs: &[&Path]) -> Option<OsString> {
        Some(std::env::join_paths(dirs).expect("a PATH"))
    }

    fn exe(name: &str) -> String {
        if cfg!(windows) {
            format!("{name}.exe")
        } else {
            name.to_string()
        }
    }

    #[test]
    fn a_program_on_the_path_is_found() {
        let dir = dir_with(&exe("tool"));
        let empty = tempfile::tempdir().expect("a temp dir");
        assert!(
            find_program_in("tool", empty.path(), path_var(&[dir.path()]), None).is_some(),
            "tool was not found on a PATH containing it"
        );
    }

    #[test]
    fn a_program_that_is_not_there_is_not_found() {
        let dir = tempfile::tempdir().expect("a temp dir");
        assert!(find_program_in("absent", dir.path(), path_var(&[dir.path()]), None).is_none());
    }

    #[test]
    fn a_directory_is_not_a_program() {
        // `command -v` would not report a directory either, and a cell that
        // tried to run one would fail in a way nobody could read.
        let dir = tempfile::tempdir().expect("a temp dir");
        std::fs::create_dir(dir.path().join(exe("tool"))).expect("a directory");
        assert!(find_program_in("tool", dir.path(), path_var(&[dir.path()]), None).is_none());
    }

    #[test]
    fn the_first_directory_on_the_path_wins() {
        let first = dir_with(&exe("tool"));
        let second = dir_with(&exe("tool"));
        let empty = tempfile::tempdir().expect("a temp dir");
        let found = find_program_in(
            "tool",
            empty.path(),
            path_var(&[first.path(), second.path()]),
            None,
        )
        .expect("found");
        assert!(found.starts_with(first.path()), "{}", found.display());
    }

    #[test]
    fn a_name_with_a_separator_is_a_path_not_a_lookup() {
        let dir = tempfile::tempdir().expect("a temp dir");
        std::fs::create_dir(dir.path().join("bin")).expect("a bin dir");
        let path = dir.path().join("bin").join(exe("tool"));
        std::fs::write(&path, b"").expect("a file");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        // Nothing on PATH at all: it must be resolved against the workdir.
        assert!(find_program_in(&format!("bin/{}", exe("tool")), dir.path(), None, None).is_some());
    }

    /// The end-to-end one: the shell that runs every cell must be findable.
    ///
    /// This is what the old probe got wrong. `command -v` is POSIX text, and
    /// on Windows it reached `cmd.exe`, which has no such builtin — so every
    /// declared tool came back missing on a machine that had them all.
    #[test]
    fn the_shell_that_runs_cells_is_itself_found() {
        let cwd = tempfile::tempdir().expect("a temp dir");
        let shell = if cfg!(windows) { "cmd" } else { "sh" };
        assert!(
            find_program(shell, cwd.path()).is_some(),
            "{shell} was not found, so no cell could run either"
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_file_without_the_executable_bit_is_not_a_program() {
        let dir = tempfile::tempdir().expect("a temp dir");
        std::fs::write(dir.path().join("tool"), b"").expect("a file");
        assert!(find_program_in("tool", dir.path(), path_var(&[dir.path()]), None).is_none());
    }

    #[cfg(unix)]
    #[test]
    fn the_working_directory_is_not_searched() {
        // `sh` does not resolve a bare name against the cwd, so neither does
        // this. A `./tool` beside a document must not shadow the real one.
        let dir = dir_with("tool");
        assert!(find_program_in("tool", dir.path(), None, None).is_none());
    }

    #[cfg(windows)]
    #[test]
    fn a_pathext_spelling_is_found_and_a_bare_file_is_not() {
        // `python` on PATH is really `python.exe`; and an extensionless file
        // must NOT count, because cmd cannot run one — reporting it present
        // would trade a clear preflight message for "'python' is not
        // recognized" in the middle of a document.
        let dir = dir_with("tool.exe");
        std::fs::write(dir.path().join("bare"), b"").expect("a file");
        assert!(find_program_in("tool", dir.path(), path_var(&[dir.path()]), None).is_some());
        assert!(find_program_in("bare", dir.path(), path_var(&[dir.path()]), None).is_none());
    }

    #[cfg(windows)]
    #[test]
    fn the_working_directory_is_searched_first() {
        // cmd resolves a bare name against the current directory before PATH,
        // and the cwd here is the cell's own workdir.
        let cwd = dir_with("tool.exe");
        let elsewhere = dir_with("tool.exe");
        let found = find_program_in("tool", cwd.path(), path_var(&[elsewhere.path()]), None)
            .expect("found");
        assert!(found.starts_with(cwd.path()), "{}", found.display());
    }
}
