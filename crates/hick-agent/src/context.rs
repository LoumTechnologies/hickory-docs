//! Standards-based agent context loading.
//!
//! Loads project context from standard locations:
//! - `AGENTS.md` — agent instructions (emerging standard)
//! - `.agents/*.md` — agent configuration directory
//! - `CLAUDE.md` — project instructions
//!
//! Uses standard paths, not tool-specific directories.

use std::path::Path;

/// Standard file names to look for at the project root.
const CONTEXT_FILES: &[(&str, &str)] = &[
    ("AGENTS.md", "Agent Instructions"),
    ("CLAUDE.md", "Project Instructions"),
    ("CONVENTIONS.md", "Conventions"),
];

/// Standard directories to scan for agent context.
const CONTEXT_DIRS: &[&str] = &[".agents"];

/// Load agent context from a project directory.
///
/// Returns a formatted string suitable for inclusion in the system prompt,
/// or an empty string if no context files are found.
pub fn load_agent_context(project_dir: &Path) -> String {
    let mut parts: Vec<String> = Vec::new();

    // Load standard files from project root
    for (filename, label) in CONTEXT_FILES {
        let path = project_dir.join(filename);
        if path.is_file() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                parts.push(format!("# {label} ({filename})\n\n{content}"));
            }
        }
    }

    // Load files from standard directories
    for dirname in CONTEXT_DIRS {
        let dir = project_dir.join(dirname);
        if dir.is_dir() {
            let mut entries: Vec<_> = std::fs::read_dir(&dir)
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
                .collect();
            entries.sort_by_key(|e| e.file_name());
            for entry in entries {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    let name = entry.file_name().to_string_lossy().to_string();
                    parts.push(format!("# Agent Context: {dirname}/{name}\n\n{content}"));
                }
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

    #[test]
    fn loads_agents_md() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("AGENTS.md"), "# My Agent\nDo things.").unwrap();

        let ctx = load_agent_context(dir.path());
        assert!(ctx.contains("My Agent"));
        assert!(ctx.contains("Do things."));
    }

    #[test]
    fn loads_agents_directory() {
        let dir = tempfile::tempdir().unwrap();
        let agents_dir = dir.path().join(".agents");
        fs::create_dir(&agents_dir).unwrap();
        fs::write(agents_dir.join("skills.md"), "# Skills\nSome skills.").unwrap();

        let ctx = load_agent_context(dir.path());
        assert!(ctx.contains("Skills"));
        assert!(ctx.contains(".agents/skills.md"));
    }

    #[test]
    fn empty_when_no_files() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = load_agent_context(dir.path());
        assert!(ctx.is_empty());
    }
}
