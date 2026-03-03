use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

/// Resolve the Remill semantics directory.
///
/// Search order:
///   1. Explicit override (from CLI `--semantics-dir`)
///   2. Standard Remill install locations
///   3. `REMILL_PREFIX` env var (with versioned subdirectory scanning)
pub fn resolve_semantics_dir(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(dir) = explicit {
        return Ok(dir.to_path_buf());
    }

    // Check standard Remill install locations.
    let candidates = [
        "/usr/local/share/remill/semantics",
        "/usr/share/remill/semantics",
    ];
    for candidate in &candidates {
        let p = Path::new(candidate);
        if p.exists() {
            return Ok(p.to_path_buf());
        }
    }

    // Check REMILL_PREFIX env var.
    if let Ok(prefix) = std::env::var("REMILL_PREFIX") {
        let base = Path::new(&prefix).join("share/remill");
        // Try direct path first, then versioned subdirs (e.g. share/remill/21/semantics).
        let direct = base.join("semantics");
        if direct.exists() {
            return Ok(direct);
        }
        // Scan for versioned subdirectories (numerically named).
        if let Ok(entries) = std::fs::read_dir(&base) {
            let mut versioned: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_name()
                        .to_str()
                        .is_some_and(|n| n.chars().all(|c| c.is_ascii_digit()))
                })
                .collect();
            // Sort descending so we pick the highest version.
            versioned.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
            for entry in versioned {
                let sem = entry.path().join("semantics");
                if sem.exists() {
                    log::info!("Found Remill semantics at {}", sem.display());
                    return Ok(sem);
                }
            }
        }
    }

    bail!(
        "Remill semantics directory not found. Set --semantics-dir or REMILL_SEMANTICS_DIR.\n\
         Expected locations: /usr/local/share/remill/semantics or $REMILL_PREFIX/share/remill/<version>/semantics"
    );
}
