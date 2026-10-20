use super::{civil_date, free_path, slug};
use anyhow::{Context, Result, bail};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Build a note from text somebody typed here, rather than from a file.
///
/// This is deliberately **not** a transcript. A `hick:transcript` means "bytes
/// another tool produced", which is exactly right for a downloaded export and
/// exactly wrong for a sentence the user just typed: on the attributable axis
/// in `provenance-and-standing.md`, scratchpad text is the strongest material
/// there is — a named human wrote it, here, and git will say so. Wrapping it as
/// machine-conveyed material would throw that away.
///
/// So the text becomes the note's own prose, and the only thing recorded about
/// its origin is that it came from the scratchpad.
pub fn note_from_scratchpad(text: &str, date: Option<&str>) -> Result<String> {
    note_from_scratchpad_with_attribution(text, date, false)
}

/// Agent-assisted text stays prose, with explicit origin rather than human-only attribution.
pub fn note_from_scratchpad_with_attribution(
    text: &str,
    date: Option<&str>,
    agent_assisted: bool,
) -> Result<String> {
    if text.trim().is_empty() {
        bail!(
            "there is nothing in the scratchpad to save.\n  \
             Next step: type something first — a note with no words is not a note."
        );
    }
    // Prose becomes document content, so hick markup in it would be parsed as
    // structure. There is no escaping in this language by design, so this is a
    // refusal rather than a transformation.
    if text.contains("<hick:") {
        bail!(
            "this text contains `<hick:`, which the document parser reads as \
             structure rather than as words.\n  \
             hick has no escaping by design, so prose cannot contain a hick tag \
             it does not mean.\n  \
             Next step: remove or reword that part, or write it into a document \
             by hand where the tag is meant to be a tag."
        );
    }

    let mut body = text.trim_end().to_string();
    let title = match body.lines().next().map(str::trim) {
        // A leading heading IS the title; keeping it would print it twice.
        Some(first) if first.starts_with("# ") => {
            let title = first[2..].trim().to_string();
            body = body
                .split_once('\n')
                .map(|(_, rest)| rest.trim_start_matches('\n').to_string())
                .unwrap_or_default();
            title
        }
        Some(first) if !first.is_empty() => {
            let mut title: String = first.chars().take(60).collect();
            if first.chars().count() > 60 {
                title.push('…');
            }
            title
        }
        _ => "Note".to_string(),
    };

    let mut note = String::from("---\n");
    if let Some(date) = date {
        note.push_str(&format!("date: {date}\n"));
    }
    note.push_str("source: scratchpad\n");
    if agent_assisted {
        note.push_str("authorship: agent-assisted\n");
    }
    note.push_str("---\n\n");
    note.push_str(&format!("# {title}\n"));
    if !body.trim().is_empty() {
        note.push('\n');
        note.push_str(body.trim_start_matches('\n'));
        note.push('\n');
    }
    Ok(note)
}

/// Today, as `YYYY-MM-DD`.
///
/// The clock is correct here and nowhere else in this module: a scratchpad note
/// is *created* now, so "now" is a fact about it. File ingest derives its date
/// from the source instead, because re-reading the same file must not depend on
/// the day.
pub fn today() -> Option<String> {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    Some(civil_date(secs / 86_400))
}

/// Save scratchpad text as a note in `notes_dir`, returning its path.
pub fn save_scratchpad(text: &str, notes_dir: &Path) -> Result<PathBuf> {
    save_scratchpad_with_attribution(text, notes_dir, false)
}

pub fn save_scratchpad_with_attribution(
    text: &str,
    notes_dir: &Path,
    agent_assisted: bool,
) -> Result<PathBuf> {
    let date = today();
    let note = note_from_scratchpad_with_attribution(text, date.as_deref(), agent_assisted)?;
    let title = note
        .lines()
        .find_map(|line| line.strip_prefix("# "))
        .unwrap_or("note")
        .to_string();
    let slug = slug(&title);
    let slug = if slug.is_empty() {
        "note".to_string()
    } else {
        slug
    };
    let stem = match &date {
        Some(date) => format!("{date}-{slug}"),
        None => slug,
    };
    std::fs::create_dir_all(notes_dir)
        .with_context(|| format!("failed to create {}", notes_dir.display()))?;
    for _ in 0..1000 {
        let path = free_path(notes_dir, &stem, "md");
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                file.write_all(note.as_bytes()).with_context(|| {
                    format!(
                        "failed to finish {}; inspect this partial note before retrying",
                        path.display()
                    )
                })?;
                return Ok(path);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(error).with_context(|| format!("failed to create {}", path.display()));
            }
        }
    }
    bail!("Could not claim a new scratchpad filename; existing notes were not overwritten")
}
