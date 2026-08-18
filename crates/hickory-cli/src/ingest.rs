//! Ingest: a file somebody else's tool produced becomes a note.
//!
//! See `docs/specs/freeform/ingest.md`. Three rules shape everything here:
//!
//! 1. **Ingest never deletes the user's file.** The bytes came from outside and
//!    we do not own them. The source is *moved* into an `ingested/` directory
//!    beside the inbox, which anyone can undo by looking at it.
//! 2. **Ingest never calls a model.** Parsing is offline, first-party, and
//!    deterministic. The summary passages are written empty and stale, and stay
//!    that way until `hick refresh` runs with the user's own key — so a meeting
//!    recorded on a plane becomes a note on the plane.
//! 3. **Ingesting the same transcript twice produces one note.** Identity is the
//!    content hash of the source bytes, recorded in the note's frontmatter,
//!    because the reflex when something looks like it failed is to try again.
//!
//! There is no clock in here. Dates come from the source file, so ingesting the
//! same file twice on different days produces the same bytes.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};

/// Environment variable naming the inbox directory.
pub const INBOX_VAR: &str = "HICKORY_INBOX";

/// The default inbox, relative to the notes folder.
pub const DEFAULT_INBOX: &str = "inbox";

/// Where ingest looks for material, and where it puts it afterwards.
///
/// Strongly typed and validated at construction, per
/// `.instructions/config-and-environments.md`: a bad value here fails on
/// somebody else's laptop, where nobody can debug it for them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboxConfig {
    /// Directory name, relative to the notes folder.
    dir: String,
}

impl Default for InboxConfig {
    fn default() -> Self {
        Self {
            dir: DEFAULT_INBOX.to_string(),
        }
    }
}

impl InboxConfig {
    /// Read the configuration from a lookup, validating it.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self> {
        let Some(raw) = lookup(INBOX_VAR) else {
            return Ok(Self::default());
        };
        let value = raw.trim();
        if value.is_empty() {
            bail!(
                "{INBOX_VAR} is set but empty, so there is no directory to watch \
                 for transcripts.\n  \
                 A valid value is a directory name relative to your notes folder, \
                 like `{DEFAULT_INBOX}` or `meetings/inbox`.\n  \
                 Next step: unset {INBOX_VAR} to use the default (`{DEFAULT_INBOX}`), \
                 or give it a name."
            );
        }
        let path = Path::new(value);
        if path.is_absolute() {
            bail!(
                "{INBOX_VAR} is `{value}`, which is an absolute path.\n  \
                 The inbox lives inside the notes folder so that a notes \
                 repository is self-contained and can be moved or cloned \
                 anywhere.\n  \
                 Next step: use a relative name like `{DEFAULT_INBOX}`."
            );
        }
        if path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            bail!(
                "{INBOX_VAR} is `{value}`, which climbs out of the notes folder \
                 with `..`.\n  \
                 Ingest moves files after reading them, so a path that escapes \
                 the folder would move somebody's file somewhere they did not \
                 ask for.\n  \
                 Next step: use a name inside the folder, like `{DEFAULT_INBOX}`."
            );
        }
        Ok(Self {
            dir: value.to_string(),
        })
    }

    /// Read the configuration from the process environment.
    pub fn from_env() -> Result<Self> {
        Self::from_lookup(|key| std::env::var(key).ok())
    }

    /// The inbox directory inside `root`.
    pub fn inbox(&self, root: &Path) -> PathBuf {
        root.join(&self.dir)
    }

    /// Where a source file goes once it has become a note.
    pub fn ingested(&self, root: &Path) -> PathBuf {
        self.inbox(root).join("ingested")
    }
}

/// What happened to one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// A note was written and the source moved out of the inbox.
    Ingested {
        source: PathBuf,
        note: PathBuf,
        moved_to: PathBuf,
    },
    /// A note for these exact bytes already exists.
    AlreadyIngested { source: PathBuf, note: PathBuf },
    /// Nothing could be made of it, with the reason. Never an error: a file
    /// that cannot be read must not stop the ones that can.
    Skipped { source: PathBuf, reason: String },
    /// Still arriving. Left alone, and looked at again next time.
    Waiting { source: PathBuf },
}

impl std::fmt::Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Outcome::Ingested {
                source,
                note,
                moved_to,
            } => write!(
                f,
                "ingested {} -> {} (source moved to {})",
                source.display(),
                note.display(),
                moved_to.display()
            ),
            Outcome::AlreadyIngested { source, note } => write!(
                f,
                "skipped {}: already ingested as {}",
                source.display(),
                note.display()
            ),
            Outcome::Skipped { source, reason } => {
                write!(f, "skipped {}: {reason}", source.display())
            }
            Outcome::Waiting { source } => write!(
                f,
                "waiting for {} to finish arriving — it is still growing",
                source.display()
            ),
        }
    }
}

/// Suffixes browsers and file managers give a file that is still arriving.
///
/// These are not skipped as a courtesy — a half-written download is a
/// truncated transcript, and ingesting one would produce a note that looks
/// complete and is not. They are ignored **silently**: the file gets its real
/// name the moment the transfer finishes, and reporting it every cycle would be
/// noise about something that is working correctly.
const PARTIAL_SUFFIXES: [&str; 7] = [
    ".crdownload",
    ".part",
    ".partial",
    ".download",
    ".opdownload",
    ".filepart",
    ".tmp",
];

/// Google Drive shortcut files, which hold a URL rather than a document.
const DRIVE_STUB_SUFFIXES: [&str; 4] = [".gdoc", ".gsheet", ".gslides", ".gdraw"];

/// Should this file be passed over without a word?
fn is_in_flight(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    if name.starts_with('.') || name.starts_with("~$") || name.ends_with('~') {
        return true;
    }
    PARTIAL_SUFFIXES
        .iter()
        .any(|suffix| lower.ends_with(suffix))
}

/// A Drive shortcut names a document that lives in the cloud, not on disk.
fn drive_stub_reason(name: &str) -> Option<String> {
    let lower = name.to_ascii_lowercase();
    DRIVE_STUB_SUFFIXES
        .iter()
        .any(|suffix| lower.ends_with(suffix))
        .then(|| {
            format!(
                "`{name}` is a Google Drive shortcut, not a document — the file on \
                 disk holds a URL, and the words are still in the cloud.\n  \
                 Next step: open it in Drive and use File > Download > Markdown \
                 (or Plain text), then put THAT file in the inbox. A Google Meet \
                 transcript and a Gemini notes doc both export this way."
            )
        })
}

/// How long to wait for a file to stop growing before reading it.
///
/// A copy or a download arrives in pieces. Reading one mid-flight yields a
/// truncated transcript that would then be fingerprinted, ingested, and moved
/// out of the inbox — a silently incomplete note, which is the worst outcome
/// available. A slower transfer keeps generating filesystem events, so the next
/// cycle picks it up rather than this one blocking for it.
const SETTLE_BUDGET: std::time::Duration = std::time::Duration::from_millis(2_000);
const SETTLE_POLL: std::time::Duration = std::time::Duration::from_millis(200);

/// Has this file stopped changing?
fn has_settled(path: &Path) -> bool {
    let size_of = |path: &Path| std::fs::metadata(path).map(|m| m.len()).ok();
    let mut last = size_of(path);
    let deadline = std::time::Instant::now() + SETTLE_BUDGET;
    while std::time::Instant::now() < deadline {
        std::thread::sleep(SETTLE_POLL);
        let current = size_of(path);
        if current.is_none() {
            // It went away mid-transfer, which is the file manager's business.
            return false;
        }
        if current == last {
            return true;
        }
        last = current;
    }
    false
}

/// Where a downloaded file came from, if the operating system recorded it.
///
/// This is **derived evidence, not an assertion** — the same category as
/// `git blame` in `provenance-and-standing.md`. Nobody types it, and a note
/// that carries it can say where its material came from without anyone
/// vouching for it.
///
/// Support is uneven and that is the honest summary: Windows records it
/// reliably (browsers write a `Zone.Identifier` stream), macOS records it
/// reliably (`kMDItemWhereFroms`), and on Linux only some browsers and file
/// managers set `user.xdg.origin.url`. `None` means "nothing was recorded",
/// never "this was not downloaded".
fn download_origin(path: &Path) -> Option<String> {
    let raw = read_origin_attribute(path)?;
    Some(redact_url(&raw))
}

#[cfg(windows)]
fn read_origin_attribute(path: &Path) -> Option<String> {
    // NTFS alternate data stream, written by the Attachment Execution Service.
    let stream = format!("{}:Zone.Identifier", path.display());
    let contents = std::fs::read_to_string(stream).ok()?;
    for key in ["HostUrl=", "ReferrerUrl="] {
        if let Some(line) = contents.lines().find(|line| line.starts_with(key)) {
            let value = line[key.len()..].trim();
            if value.starts_with("http") {
                return Some(value.to_string());
            }
        }
    }
    None
}

#[cfg(unix)]
fn read_origin_attribute(path: &Path) -> Option<String> {
    // Linux: a plain URL string. macOS: `kMDItemWhereFroms` is a binary plist
    // holding the download URL and the page that linked it. Parsing bplist to
    // read one string is not worth a dependency, so the URL is scanned out of
    // the bytes — a heuristic, and one that fails closed by returning `None`.
    for attribute in [
        "user.xdg.origin.url",
        "user.xdg.referrer.url",
        "com.apple.metadata:kMDItemWhereFroms",
    ] {
        let Ok(Some(bytes)) = xattr::get(path, attribute) else {
            continue;
        };
        if let Some(url) = first_url(&bytes) {
            return Some(url);
        }
    }
    None
}

/// The first `http(s)` URL in a blob of bytes.
fn first_url(bytes: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(bytes);
    let start = text.find("http")?;
    let url: String = text[start..]
        .chars()
        .take_while(|c| !c.is_whitespace() && !c.is_control() && *c != '"')
        .collect();
    (url.starts_with("http://") || url.starts_with("https://")).then_some(url)
}

/// A URL reduced to where it points, with the query string removed.
///
/// The query string is dropped deliberately and is not a rounding error: an
/// export link from Drive, S3, or any signing service carries the credential
/// **in the query**, and this value is about to be written into a file that
/// goes into a git repository and may be pushed to a remote somebody else can
/// read. `config-and-environments` says never write a secret into a file we
/// create, and a signed URL is a secret. What survives — scheme, host, path —
/// is the part that answers "where did this come from".
fn redact_url(url: &str) -> String {
    let cut = url.find(['?', '#']).unwrap_or(url.len());
    url[..cut].to_string()
}

/// Content identity of a source file.
fn fingerprint(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// The date a source file carries, as `YYYY-MM-DD`.
///
/// From the file's own modification time, never the clock: ingesting the same
/// file twice must produce the same bytes, and "when did I get round to it" is
/// not a fact about the meeting.
fn source_date(path: &Path) -> Option<String> {
    let modified = std::fs::metadata(path).ok()?.modified().ok()?;
    let secs = modified
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    Some(civil_date(secs / 86_400))
}

/// Days since the Unix epoch to `YYYY-MM-DD` (Howard Hinnant's civil_from_days).
fn civil_date(days: u64) -> String {
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

/// A file name reduced to something usable as a note name.
fn slug(value: &str) -> String {
    let mut out = String::new();
    let mut dash = true;
    for ch in value.chars() {
        if ch.is_alphanumeric() {
            for lower in ch.to_lowercase() {
                out.push(lower);
            }
            dash = false;
        } else if !dash {
            out.push('-');
            dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

/// A readable title for the note.
fn title_of(source_text: &str, source_path: &Path) -> String {
    // An exporter's markdown usually opens with the meeting's name.
    for line in source_text.lines().take(20) {
        if let Some(heading) = line.trim().strip_prefix("# ") {
            let heading = heading.trim();
            if !heading.is_empty() {
                return heading.to_string();
            }
        }
    }
    let stem = source_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Untitled".to_string());
    let words: Vec<String> = stem
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect();
    if words.is_empty() {
        "Untitled".to_string()
    } else {
        words.join(" ")
    }
}

/// Build the note for `source_text`, without touching the filesystem.
pub fn note_for(
    source_text: &str,
    source_path: &Path,
    date: Option<&str>,
    origin: Option<&str>,
) -> Result<String> {
    // The raw block is the source of truth, and there is no escaping in this
    // language by design — so a transcript that contains its own closing tag
    // cannot be represented, and saying so is the only honest option.
    if source_text.contains("</hick:transcript>") {
        bail!(
            "this file contains the text `</hick:transcript>`, which would end \
             the block that is supposed to hold it.\n  \
             hick has no escaping by design — the raw bytes of a transcript are \
             the raw bytes — so this file cannot be stored verbatim.\n  \
             Next step: ingest it by hand into a document that wraps it \
             differently, or remove that line if it is an artefact of the export."
        );
    }

    let format = hick_transcript::detect(source_text);
    let utterances = format
        .map(|f| hick_transcript::parse(source_text, f))
        .unwrap_or_default();

    // Attendees are DERIVED from who actually spoke, not asserted by whoever
    // ran the command.
    let mut attendees: Vec<String> = Vec::new();
    for utterance in &utterances {
        if let Some(speaker) = &utterance.speaker
            && !attendees.iter().any(|a| a == speaker)
        {
            attendees.push(speaker.clone());
        }
    }

    let title = title_of(source_text, source_path);
    let source_name = source_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let mut note = String::from("---\n");
    if let Some(date) = date {
        note.push_str(&format!("date: {date}\n"));
    }
    if !attendees.is_empty() {
        note.push_str(&format!("attendees: [{}]\n", attendees.join(", ")));
    }
    note.push_str(&format!("source: {source_name}\n"));
    // Derived evidence, never an assertion: the operating system recorded this
    // when the file arrived, and nobody typed it.
    if let Some(origin) = origin {
        note.push_str(&format!("source-url: {origin}\n"));
    }
    note.push_str(&format!(
        "source-format: {}\n",
        format.map(|f| f.name()).unwrap_or("unrecognised")
    ));
    note.push_str(&format!(
        "source-sha256: {}\n",
        fingerprint(source_text.as_bytes())
    ));
    note.push_str("---\n\n");

    note.push_str(&format!("# {title}\n\n"));

    note.push_str("<hick:transcript id=\"transcript\"");
    if let Some(format) = format {
        note.push_str(&format!(" format=\"{}\"", format.name()));
    }
    note.push_str(">\n");
    note.push_str(source_text);
    if !source_text.ends_with('\n') {
        note.push('\n');
    }
    note.push_str("</hick:transcript>\n\n");

    note.push_str("## Summary\n\n");
    note.push_str(
        "<hick:transform select=\"#transcript\" instruct=\"Summarize this meeting for \
         someone who missed it, in at most five sentences: what was decided, what is \
         blocked, and on whom. Plain prose, no bullet points.\" from=\"\">\n</hick:transform>\n\n",
    );
    note.push_str("## Action items\n\n");
    note.push_str(
        "<hick:transform select=\"#transcript\" instruct=\"Extract every action item as \
         a markdown checklist, one line each, naming who owns it. Output only the list.\" \
         from=\"\">\n</hick:transform>\n",
    );
    Ok(note)
}

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
    let date = today();
    let note = note_from_scratchpad(text, date.as_deref())?;
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
    let path = free_path(notes_dir, &stem, "hick");
    std::fs::write(&path, note).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(path)
}

/// Is there already a note built from these exact bytes?
fn existing_note(notes_dir: &Path, fingerprint: &str) -> Option<PathBuf> {
    let needle = format!("source-sha256: {fingerprint}");
    let entries = std::fs::read_dir(notes_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "hick")
            && let Ok(text) = std::fs::read_to_string(&path)
            && text.contains(&needle)
        {
            return Some(path);
        }
    }
    None
}

/// A path in `dir` that nothing occupies yet.
fn free_path(dir: &Path, stem: &str, extension: &str) -> PathBuf {
    let first = dir.join(format!("{stem}.{extension}"));
    if !first.exists() {
        return first;
    }
    for n in 2..1000 {
        let candidate = dir.join(format!("{stem}-{n}.{extension}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    dir.join(format!("{stem}-{}.{extension}", std::process::id()))
}

/// Turn one file into a note.
pub fn ingest_one(source: &Path, notes_dir: &Path, config: &InboxConfig) -> Result<Outcome> {
    let name = source
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    if let Some(reason) = drive_stub_reason(&name) {
        return Ok(Outcome::Skipped {
            source: source.to_path_buf(),
            reason,
        });
    }
    if !has_settled(source) {
        return Ok(Outcome::Waiting {
            source: source.to_path_buf(),
        });
    }

    let bytes =
        std::fs::read(source).with_context(|| format!("failed to read {}", source.display()))?;
    let Ok(text) = String::from_utf8(bytes) else {
        return Ok(Outcome::Skipped {
            source: source.to_path_buf(),
            reason: format!(
                "not text — {} is not valid UTF-8, so there is no transcript in it to read. \
                 Audio and video files are not transcripts; export or transcribe them first.",
                source.display()
            ),
        });
    };

    let id = fingerprint(text.as_bytes());
    if let Some(note) = existing_note(notes_dir, &id) {
        return Ok(Outcome::AlreadyIngested {
            source: source.to_path_buf(),
            note,
        });
    }

    let date = source_date(source);
    // Read BEFORE the move: extended attributes are a property of the file
    // where it currently is.
    let origin = download_origin(source);
    let note_text = match note_for(&text, source, date.as_deref(), origin.as_deref()) {
        Ok(note) => note,
        Err(e) => {
            return Ok(Outcome::Skipped {
                source: source.to_path_buf(),
                reason: format!("{e:#}"),
            });
        }
    };

    let title = title_of(&text, source);
    let stem = {
        let slug = slug(&title);
        let slug = if slug.is_empty() {
            "note".to_string()
        } else {
            slug
        };
        match &date {
            Some(date) => format!("{date}-{slug}"),
            None => slug,
        }
    };
    let note_path = free_path(notes_dir, &stem, "hick");
    std::fs::write(&note_path, note_text)
        .with_context(|| format!("failed to write {}", note_path.display()))?;

    // The source is MOVED, never deleted: we did not create these bytes.
    let ingested_dir = config.ingested(notes_dir);
    std::fs::create_dir_all(&ingested_dir).with_context(|| {
        format!(
            "failed to create {} to move the ingested file into",
            ingested_dir.display()
        )
    })?;
    let name = source
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "transcript".to_string());
    let (stem, ext) = match name.rsplit_once('.') {
        Some((stem, ext)) => (stem.to_string(), ext.to_string()),
        None => (name.clone(), "txt".to_string()),
    };
    let moved_to = free_path(&ingested_dir, &stem, &ext);
    if std::fs::rename(source, &moved_to).is_err() {
        // Across filesystems a rename fails; copy-then-remove is the fallback,
        // and the copy happens first so a failure cannot lose the file.
        std::fs::copy(source, &moved_to)
            .with_context(|| format!("failed to move {} out of the inbox", source.display()))?;
        std::fs::remove_file(source).ok();
    }

    Ok(Outcome::Ingested {
        source: source.to_path_buf(),
        note: note_path,
        moved_to,
    })
}

/// How many files are in the inbox but still arriving.
///
/// In-flight files are skipped in silence, which is right for a watching loop
/// and wrong for a person who just asked: "nothing to ingest" when a download
/// is visibly in progress reads as a broken tool.
pub fn in_flight_count(root: &Path, config: &InboxConfig) -> usize {
    let inbox = config.inbox(root);
    let Ok(entries) = std::fs::read_dir(&inbox) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.file_name()
                .map(|n| is_in_flight(&n.to_string_lossy()))
                .unwrap_or(false)
        })
        .count()
}

/// Ingest everything sitting in `root`'s inbox.
///
/// A missing inbox is not an error — most notes folders will never have one,
/// and `hick up` calls this on every cycle.
pub fn ingest_inbox(root: &Path, config: &InboxConfig) -> Result<Vec<Outcome>> {
    let inbox = config.inbox(root);
    if !inbox.is_dir() {
        return Ok(Vec::new());
    }
    let ingested_dir = config.ingested(root);

    let mut sources: Vec<PathBuf> = std::fs::read_dir(&inbox)
        .with_context(|| format!("failed to read {}", inbox.display()))?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && *path != ingested_dir)
        // A file that is still arriving has a name that says so, and will get
        // its real one when the transfer finishes.
        .filter(|path| {
            !path
                .file_name()
                .map(|n| is_in_flight(&n.to_string_lossy()))
                .unwrap_or(true)
        })
        .collect();
    // Stable order so a folder of files ingests the same way twice.
    sources.sort();

    let mut out = Vec::new();
    for source in sources {
        // One unreadable file must not stop the rest.
        match ingest_one(&source, root, config) {
            Ok(outcome) => out.push(outcome),
            Err(e) => out.push(Outcome::Skipped {
                source,
                reason: format!("{e:#}"),
            }),
        }
    }
    Ok(out)
}
