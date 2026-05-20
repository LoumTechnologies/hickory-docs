//! Standards-based agent context loading.
//!
//! Loads project context from standard locations:
//! - `AGENTS.md` — agent instructions (emerging standard)
//! - `.agents/*.md` — agent configuration directory
//! - `CLAUDE.md` — project instructions
//! - `CONVENTIONS.md` — project conventions
//! - `.agents/memory/*.md` — persistent agent memory bank
//!
//! Paths matching `.agentignore` patterns are silently excluded.

use std::path::{Path, PathBuf};

use crate::agentignore::AgentIgnoreFilter;

const CONTEXT_FILES: &[(&str, &str)] = &[
    ("AGENTS.md", "Agent Instructions"),
    ("CLAUDE.md", "Project Instructions"),
    ("CONVENTIONS.md", "Conventions"),
];

const CONTEXT_DIRS: &[&str] = &[".agents"];

/// Loads agent context from standard project locations.
///
/// Scans `AGENTS.md`, `CLAUDE.md`, `CONVENTIONS.md`, `.agents/*.md`, and
/// `.agents/memory/*.md` in a project directory and returns a formatted string
/// suitable for inclusion in an LLM system prompt. Paths matching
/// `.agentignore` are silently excluded.
pub struct ProjectContextLoader {
    project_dir: PathBuf,
    ignore_filter: AgentIgnoreFilter,
}

impl ProjectContextLoader {
    /// Create a loader for the given project directory.
    ///
    /// Loads `.agentignore` from the project root if present.
    pub fn new(project_dir: impl Into<PathBuf>) -> Self {
        let project_dir = project_dir.into();
        let ignore_filter = AgentIgnoreFilter::load(&project_dir);
        Self {
            project_dir,
            ignore_filter,
        }
    }

    /// Load and return formatted context, or an empty string if none found.
    pub fn load(&self) -> String {
        load_agent_context(&self.project_dir, &self.ignore_filter)
    }

    /// Project directory this loader is bound to.
    pub fn project_dir(&self) -> &Path {
        &self.project_dir
    }

    /// The ignore filter derived from `.agentignore` (if present).
    pub fn ignore_filter(&self) -> &AgentIgnoreFilter {
        &self.ignore_filter
    }
}

/// Load agent context from a project directory applying the given ignore filter.
///
/// Returns a formatted string suitable for inclusion in the system prompt,
/// or an empty string if no context files are found.
pub fn load_agent_context(project_dir: &Path, filter: &AgentIgnoreFilter) -> String {
    let mut parts: Vec<String> = Vec::new();

    for (filename, label) in CONTEXT_FILES {
        let path = project_dir.join(filename);
        if filter.is_ignored(&path) {
            continue;
        }
        if path.is_file()
            && let Ok(content) = std::fs::read_to_string(&path)
        {
            parts.push(format!("# {label} ({filename})\n\n{content}"));
        }
    }

    for dirname in CONTEXT_DIRS {
        let dir = project_dir.join(dirname);
        if dir.is_dir() {
            let mut entries: Vec<_> = std::fs::read_dir(&dir)
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path().extension().is_some_and(|ext| ext == "md")
                        && e.path().is_file()
                })
                .collect();
            entries.sort_by_key(|e| e.file_name());
            for entry in entries {
                let path = entry.path();
                if filter.is_ignored(&path) {
                    continue;
                }
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    parts.push(format!("# Agent Context: {dirname}/{name}\n\n{content}"));
                }
            }
        }
    }

    // Load .agents/memory/*.md — persistent agent notes.
    let memory_dir = project_dir.join(".agents").join("memory");
    if memory_dir.is_dir() && !filter.is_ignored(&memory_dir) {
        let mut memory_entries: Vec<_> = std::fs::read_dir(&memory_dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().is_some_and(|ext| ext == "md")
                    && e.path().is_file()
            })
            .collect();
        memory_entries.sort_by_key(|e| e.file_name());
        for entry in memory_entries {
            let path = entry.path();
            if filter.is_ignored(&path) {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&path) {
                let name = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                parts.push(format!("# Memory: {name}\n\n{content}"));
            }
        }
    }

    if parts.is_empty() {
        String::new()
    } else {
        parts.join("\n\n---\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn empty_filter() -> AgentIgnoreFilter {
        AgentIgnoreFilter::from_str_content("")
    }

    #[test]
    fn loads_agents_md() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("AGENTS.md"), "# My Agent\nDo things.").unwrap();

        let loader = ProjectContextLoader::new(dir.path());
        let ctx = loader.load();
        assert!(ctx.contains("My Agent"));
        assert!(ctx.contains("Do things."));
    }

    #[test]
    fn loads_agents_directory() {
        let dir = tempfile::tempdir().unwrap();
        let agents_dir = dir.path().join(".agents");
        fs::create_dir(&agents_dir).unwrap();
        fs::write(agents_dir.join("skills.md"), "# Skills\nSome skills.").unwrap();

        let loader = ProjectContextLoader::new(dir.path());
        let ctx = loader.load();
        assert!(ctx.contains("Skills"));
        assert!(ctx.contains(".agents/skills.md"));
    }

    #[test]
    fn empty_when_no_files() {
        let dir = tempfile::tempdir().unwrap();
        let loader = ProjectContextLoader::new(dir.path());
        let ctx = loader.load();
        assert!(ctx.is_empty());
    }

    #[test]
    fn project_dir_accessor() {
        let dir = tempfile::tempdir().unwrap();
        let loader = ProjectContextLoader::new(dir.path());
        assert_eq!(loader.project_dir(), dir.path());
    }

    #[test]
    fn ignored_context_file_excluded() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".agentignore"), "CLAUDE.md\n").unwrap();
        fs::write(dir.path().join("CLAUDE.md"), "secret instructions").unwrap();
        fs::write(dir.path().join("AGENTS.md"), "public instructions").unwrap();

        let loader = ProjectContextLoader::new(dir.path());
        let ctx = loader.load();
        assert!(!ctx.contains("secret instructions"));
        assert!(ctx.contains("public instructions"));
    }

    #[test]
    fn ignored_agents_dir_file_excluded() {
        let dir = tempfile::tempdir().unwrap();
        let agents_dir = dir.path().join(".agents");
        fs::create_dir(&agents_dir).unwrap();
        fs::write(agents_dir.join("secret.md"), "secret content").unwrap();
        fs::write(agents_dir.join("public.md"), "public content").unwrap();
        fs::write(dir.path().join(".agentignore"), "secret.md\n").unwrap();

        let loader = ProjectContextLoader::new(dir.path());
        let ctx = loader.load();
        assert!(!ctx.contains("secret content"));
        assert!(ctx.contains("public content"));
    }

    #[test]
    fn memory_files_included_in_context() {
        let dir = tempfile::tempdir().unwrap();
        let memory_dir = dir.path().join(".agents").join("memory");
        fs::create_dir_all(&memory_dir).unwrap();
        fs::write(memory_dir.join("project.md"), "Tech stack: Rust.\n").unwrap();

        let ctx = load_agent_context(dir.path(), &empty_filter());
        assert!(ctx.contains("Memory: project"));
        assert!(ctx.contains("Tech stack: Rust."));
    }

    #[test]
    fn ignored_memory_file_excluded() {
        let dir = tempfile::tempdir().unwrap();
        let memory_dir = dir.path().join(".agents").join("memory");
        fs::create_dir_all(&memory_dir).unwrap();
        fs::write(memory_dir.join("sensitive.md"), "sensitive memory").unwrap();
        fs::write(dir.path().join(".agentignore"), "sensitive.md\n").unwrap();

        let loader = ProjectContextLoader::new(dir.path());
        let ctx = loader.load();
        assert!(!ctx.contains("sensitive memory"));
    }
}
