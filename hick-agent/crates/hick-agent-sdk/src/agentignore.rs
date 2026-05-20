//! `.agentignore` — gitignore-style path exclusion for agent file access.

use std::path::{Path, PathBuf};

use globset::{Glob, GlobMatcher};

struct Pattern {
    negation: bool,
    /// True if the original pattern contained a `/` (excluding a lone trailing `/`),
    /// meaning it must match against the full path relative to the project root.
    anchored: bool,
    /// True when the original pattern ended with `/`, meaning it matches a directory
    /// prefix rather than a file name.
    directory_only: bool,
    matcher: GlobMatcher,
    /// Kept for directory-only prefix matching.
    raw: String,
}

/// Gitignore-style path filter loaded from `.agentignore`.
///
/// When no `.agentignore` file is present the filter is empty and never
/// ignores any path.
pub struct AgentIgnoreFilter {
    patterns: Vec<Pattern>,
    project_dir: PathBuf,
}

impl AgentIgnoreFilter {
    /// Load from `<project_dir>/.agentignore`. Returns an empty filter if absent.
    pub fn load(project_dir: &Path) -> Self {
        let path = project_dir.join(".agentignore");
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Self::empty(project_dir),
        };
        Self::from_str_content_with_dir(&content, project_dir)
    }

    /// Parse patterns from a string (primarily for tests).
    pub fn from_str_content(content: &str) -> Self {
        Self::from_str_content_with_dir(content, Path::new("."))
    }

    fn from_str_content_with_dir(content: &str, project_dir: &Path) -> Self {
        let mut patterns = Vec::new();
        for line in content.lines() {
            let line = line.trim_end();
            // Blank lines and comments are ignored.
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let (negation, line) = if let Some(rest) = line.strip_prefix('!') {
                (true, rest)
            } else {
                (false, line)
            };

            let directory_only = line.ends_with('/');
            let trimmed = line.trim_end_matches('/');

            // A pattern is anchored when it contains a `/` outside of the
            // leading position that globset would interpret as "match from root".
            // We treat any `/` in the pattern (before stripping trailing /) as
            // anchoring — i.e. match against the full relative path.
            let anchored = trimmed.contains('/');

            // Build the glob. For non-anchored patterns we want to match just
            // the file name component, so we strip any leading `/`.
            let glob_pat = trimmed.trim_start_matches('/');
            let Ok(glob) = Glob::new(glob_pat) else {
                log::warn!("agentignore: invalid pattern '{}' — skipped", line);
                continue;
            };

            patterns.push(Pattern {
                negation,
                anchored,
                directory_only,
                matcher: glob.compile_matcher(),
                raw: trimmed.to_string(),
            });
        }
        Self {
            patterns,
            project_dir: project_dir.to_path_buf(),
        }
    }

    fn empty(project_dir: &Path) -> Self {
        Self {
            patterns: Vec::new(),
            project_dir: project_dir.to_path_buf(),
        }
    }

    /// Returns `true` if `path` should be excluded based on the loaded patterns.
    ///
    /// Applies gitignore semantics: patterns are processed in order; the last
    /// matching pattern wins. Negation patterns (prefixed with `!`) un-ignore
    /// a previously ignored path.
    pub fn is_ignored(&self, path: &Path) -> bool {
        if self.patterns.is_empty() {
            return false;
        }

        // Compute the path relative to the project root for anchored patterns.
        let rel = path
            .strip_prefix(&self.project_dir)
            .unwrap_or(path);

        let file_name = path
            .file_name()
            .map(|n| Path::new(n))
            .unwrap_or(path);

        let mut ignored = false;
        for pat in &self.patterns {
            let matches = if pat.directory_only {
                // Directory-only: path starts with the raw pattern component.
                rel.starts_with(&pat.raw)
            } else if pat.anchored {
                // Anchored: match against the full relative path.
                pat.matcher.is_match(rel)
            } else {
                // Non-anchored: match against just the file name.
                pat.matcher.is_match(file_name)
            };

            if matches {
                ignored = !pat.negation;
            }
        }
        ignored
    }

    /// True when no patterns are configured (the filter never ignores anything).
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn absence_of_file_means_no_restrictions() {
        let dir = tempfile::tempdir().unwrap();
        let filter = AgentIgnoreFilter::load(dir.path());
        assert!(filter.is_empty());
        assert!(!filter.is_ignored(Path::new(".env")));
    }

    #[test]
    fn secret_env_file_is_ignored() {
        let filter = AgentIgnoreFilter::from_str_content(".env\n");
        assert!(filter.is_ignored(Path::new(".env")));
        assert!(!filter.is_ignored(Path::new("src/main.rs")));
    }

    #[test]
    fn generated_dir_is_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let filter = AgentIgnoreFilter::load(dir.path());
        // Write an agentignore with a directory pattern.
        fs::write(dir.path().join(".agentignore"), "target/\n").unwrap();
        let filter = AgentIgnoreFilter::load(dir.path());

        let inside = dir.path().join("target").join("foo.rs");
        assert!(filter.is_ignored(&inside));

        let outside = dir.path().join("src").join("main.rs");
        assert!(!filter.is_ignored(&outside));
    }

    #[test]
    fn negation_unignores_file() {
        let filter = AgentIgnoreFilter::from_str_content("*.log\n!important.log\n");
        assert!(filter.is_ignored(Path::new("app.log")));
        assert!(!filter.is_ignored(Path::new("important.log")));
    }

    #[test]
    fn wildcard_star_matches_filenames() {
        let filter = AgentIgnoreFilter::from_str_content("*.secret\n");
        assert!(filter.is_ignored(Path::new("credentials.secret")));
        assert!(!filter.is_ignored(Path::new("credentials.toml")));
    }

    #[test]
    fn empty_content_never_ignores() {
        let filter = AgentIgnoreFilter::from_str_content("");
        assert!(!filter.is_ignored(Path::new("anything.rs")));
    }

    #[test]
    fn comment_lines_skipped() {
        let filter = AgentIgnoreFilter::from_str_content("# this is a comment\n.env\n");
        assert!(filter.is_ignored(Path::new(".env")));
    }

    #[test]
    fn anchored_pattern_matches_relative_path() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".agentignore"), "dist/bundle.js\n").unwrap();
        let filter = AgentIgnoreFilter::load(dir.path());

        let target = dir.path().join("dist").join("bundle.js");
        assert!(filter.is_ignored(&target));

        let other = dir.path().join("src").join("bundle.js");
        assert!(!filter.is_ignored(&other));
    }
}
