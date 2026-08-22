//! Assets: the image you dropped into a note, written into the notes folder.
//!
//! A note that references `assets/chart.png` is only a note as long as that
//! file exists beside it. So an image dropped or pasted into the editor is
//! written to disk here, into an `assets/` directory next to the document,
//! and the editor writes an ordinary markdown reference to it. Nothing is
//! embedded, nothing is uploaded, and nothing is stored anywhere this app
//! invented: the folder is the user's, the file is a file, and a `git add`
//! picks up the note and the picture together.
//!
//! Base64 in a JSON body rather than a multipart upload because the caller is
//! a `FileReader` in the same process's own window and the payload is
//! megabytes at most — a second body format on this server would earn nothing.
//!
//! What this deliberately is NOT:
//!
//! - **Not a general file-write route.** Only the image types a note can
//!   actually display are accepted; `PUT /api/file` is where text goes, and
//!   it stays the only way to overwrite something that already exists.
//! - **Not a store.** The name is derived from what was dropped, kept unique
//!   against what is already there, and never recorded anywhere else.

use std::path::{Path as FsPath, PathBuf};

use axum::Json;
use axum::extract::{Query, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use base64::Engine as _;
use serde::Deserialize;
use serde_json::{Value, json};

use super::LocalState;
use super::api::{ApiError, ApiResult};

/// Past this, an "image in a note" is really a file somebody should keep
/// beside the repository rather than inside it.
const MAX_ASSET_BYTES: usize = 25 * 1024 * 1024;

/// The directory an image lands in, relative to the document that references
/// it. One shared name so a folder of notes has one place to look, and the
/// same name a static-site generator would expect.
const ASSET_DIR: &str = "assets";

/// The image types a woven markdown file can show in a browser, GitHub, or a
/// preview pane. Anything else is refused by name rather than written and
/// then silently not rendered.
const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "svg", "avif", "bmp", "ico", "tif", "tiff",
];

#[derive(Deserialize)]
pub struct SaveAsset {
    /// The document the image was dropped into, root-relative. Absent (an
    /// untitled buffer) puts the image in the folder's top-level `assets/`.
    #[serde(default)]
    pub doc_path: Option<String>,
    /// What the file was called where it came from. A pasted screenshot has
    /// no name and the caller sends the one it invented.
    pub name: String,
    /// The bytes, standard base64, no data-URL prefix.
    pub content_base64: String,
}

/// `POST /api/asset` — write an image into the notes folder.
///
/// Answers both paths the caller needs and neither can be derived from the
/// other without knowing the folder's shape: `path` is root-relative (the
/// tree, a later read), and `relative` is what goes inside the markdown
/// parentheses, resolved from the document's own directory so the note keeps
/// working when the folder moves.
pub async fn post_asset(
    State(state): State<LocalState>,
    Json(body): Json<SaveAsset>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let bytes = decode(&body.content_base64)?;
    let dir = asset_dir(body.doc_path.as_deref())?;
    let name = unique_name(&root, &dir, &clean_name(&body.name)?, &bytes)?;
    let rel = join_rel(&dir, &name);
    write_asset(&root, &dir, &name, &bytes)?;
    Ok(Json(json!({
        "path": rel,
        "relative": relative_to_doc(body.doc_path.as_deref(), &rel),
        "bytes": bytes.len(),
    })))
}

#[derive(Deserialize)]
pub struct AssetQuery {
    pub path: String,
}

/// `GET /api/asset?path=…` — the image's bytes, so the editor can show it.
///
/// A route rather than a `data:` URL in the decoration: the picture is a file
/// on disk that the note points at, and reading it through the same path the
/// woven markdown would is what makes "the editor shows what the markdown
/// says" true rather than approximately true. Images only — this is a
/// viewer's read, not a way to exfiltrate arbitrary bytes over a route with
/// no other checks on it.
pub async fn get_asset(
    State(state): State<LocalState>,
    Query(q): Query<AssetQuery>,
) -> ApiResult<Response> {
    let root = state.index.root().to_path_buf();
    let rel = q.path.trim();
    let ext = rel
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default();
    if !IMAGE_EXTENSIONS.contains(&ext.as_str()) {
        return Err(ApiError::unprocessable(format!(
            "{rel} is not an image. This route serves only {} — use \
             GET /api/file for text.",
            IMAGE_EXTENSIONS.join(", ")
        )));
    }
    let path = safe_existing(&root, rel)?;
    let bytes = std::fs::read(&path)
        .map_err(|e| ApiError::internal(format!("could not read {rel}: {e}")))?;
    Ok((
        [
            (header::CONTENT_TYPE, mime_of(&ext)),
            // The bytes at a path can change when the file is replaced, and
            // the window is the only client — a revalidation each time costs
            // nothing on loopback and is never wrong.
            (header::CACHE_CONTROL, "no-cache"),
        ],
        bytes,
    )
        .into_response())
}

/// The content type for an extension this route has already accepted.
fn mime_of(ext: &str) -> &'static str {
    match ext {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        // Served as an image, never as a document: an SVG opened as
        // `image/svg+xml` in an <img> cannot run script, and this route has
        // no reason to hand one back any other way.
        "svg" => "image/svg+xml",
        "avif" => "image/avif",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        "tif" | "tiff" => "image/tiff",
        _ => "application/octet-stream",
    }
}

/// `rel` inside `root`, refusing anything that names its way out — including
/// through a symlink, which only a canonicalized path can catch.
fn safe_existing(root: &FsPath, rel: &str) -> Result<PathBuf, ApiError> {
    let as_path = FsPath::new(rel);
    if rel.is_empty()
        || as_path.is_absolute()
        || as_path
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
    {
        return Err(ApiError::bad_request(format!(
            "path must be relative to the open folder, with no `..` — got {rel:?}"
        )));
    }
    let joined = root.join(rel);
    if !joined.is_file() {
        return Err(ApiError::not_found(format!(
            "no image named {rel} in the open folder. If the note was moved \
             without its assets folder, the picture moved with the folder."
        )));
    }
    let canonical_root = root
        .canonicalize()
        .map_err(|e| ApiError::internal(format!("cannot resolve the served folder: {e}")))?;
    let canonical = joined
        .canonicalize()
        .map_err(|e| ApiError::internal(format!("cannot resolve {rel}: {e}")))?;
    if !canonical.starts_with(&canonical_root) {
        return Err(ApiError::forbidden(format!(
            "{rel} resolves outside the open folder (a symlink?), so this \
             session will not read it"
        )));
    }
    Ok(canonical)
}

/// The bytes, or the refusal that says which half was wrong.
fn decode(encoded: &str) -> Result<Vec<u8>, ApiError> {
    // A `data:` URL is what a clipboard image most naturally arrives as, and
    // stripping it here costs one line and saves every caller from getting it
    // wrong in its own way.
    let payload = match encoded.find(";base64,") {
        Some(at) if encoded.starts_with("data:") => &encoded[at + ";base64,".len()..],
        _ => encoded,
    };
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(payload.trim())
        .map_err(|e| {
            ApiError::bad_request(format!(
                "the image did not decode as base64 ({e}). Send the raw bytes \
                 base64-encoded, with or without a `data:…;base64,` prefix."
            ))
        })?;
    if bytes.is_empty() {
        return Err(ApiError::bad_request(
            "the image was empty — nothing was written.",
        ));
    }
    if bytes.len() > MAX_ASSET_BYTES {
        return Err(ApiError::unprocessable(format!(
            "that image is {} bytes, over the {} MB this route writes. Save it \
             into the folder yourself and link to it, or shrink it first.",
            bytes.len(),
            MAX_ASSET_BYTES / (1024 * 1024),
        )));
    }
    Ok(bytes)
}

/// A file name that is safe on every filesystem this ships to, keeping the
/// extension because it is what decides whether the image renders.
fn clean_name(raw: &str) -> Result<String, ApiError> {
    let base = raw.rsplit(['/', '\\']).next().unwrap_or(raw).trim();
    let (stem, ext) = match base.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => (stem, ext.to_ascii_lowercase()),
        _ => {
            return Err(ApiError::bad_request(format!(
                "{base:?} has no file extension, so nothing can tell what kind \
                 of image it is. Name it with one — `.png`, `.jpg`, `.svg`."
            )));
        }
    };
    if !IMAGE_EXTENSIONS.contains(&ext.as_str()) {
        return Err(ApiError::unprocessable(format!(
            "`.{ext}` is not an image type a note can display. Accepted: {}. \
             Anything else belongs in the folder beside the note, linked \
             rather than embedded.",
            IMAGE_EXTENSIONS.join(", ")
        )));
    }
    // Windows forbids most of this set outright; the rest would need quoting
    // inside a markdown destination, which is worse than a hyphen.
    let slug: String = stem
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '_' | '-' => c,
            _ => '-',
        })
        .collect();
    let slug = slug.trim_matches(['-', '.']).to_string();
    let slug = if slug.is_empty() {
        "image".to_string()
    } else {
        slug
    };
    // A long name is a name a `ls` cannot read; the extension is the part
    // that matters.
    let slug: String = slug.chars().take(64).collect();
    Ok(format!("{slug}.{ext}"))
}

/// Where images for the document at `doc_path` live, root-relative.
fn asset_dir(doc_path: Option<&str>) -> Result<String, ApiError> {
    let Some(doc) = doc_path.map(str::trim).filter(|d| !d.is_empty()) else {
        return Ok(ASSET_DIR.to_string());
    };
    let as_path = FsPath::new(doc);
    if as_path.is_absolute()
        || as_path
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
    {
        return Err(ApiError::bad_request(format!(
            "doc_path must be relative to the open folder, with no `..` — got {doc:?}"
        )));
    }
    let parent = doc.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("");
    Ok(join_rel(parent, ASSET_DIR))
}

/// `a` and `b` as one root-relative path, with no leading or doubled slash.
fn join_rel(a: &str, b: &str) -> String {
    if a.is_empty() {
        b.to_string()
    } else {
        format!("{a}/{b}")
    }
}

/// A name nothing else in `dir` is using — unless the bytes there are already
/// these bytes, which makes the same screenshot pasted twice one file.
fn unique_name(root: &FsPath, dir: &str, name: &str, bytes: &[u8]) -> Result<String, ApiError> {
    let (stem, ext) = name.rsplit_once('.').unwrap_or((name, ""));
    for attempt in 0..1000 {
        let candidate = if attempt == 0 {
            name.to_string()
        } else {
            format!("{stem}-{}.{ext}", attempt + 1)
        };
        let path = root.join(dir).join(&candidate);
        match std::fs::read(&path) {
            Err(_) => return Ok(candidate),
            Ok(existing) if existing == bytes => return Ok(candidate),
            Ok(_) => continue,
        }
    }
    Err(ApiError::conflict(format!(
        "a thousand files in {dir} are already called something like {name}. \
         Rename what is there, or drop the image with a different name."
    )))
}

/// Write the bytes, creating the directory the first time.
fn write_asset(root: &FsPath, dir: &str, name: &str, bytes: &[u8]) -> Result<PathBuf, ApiError> {
    let directory = root.join(dir);
    std::fs::create_dir_all(&directory).map_err(|e| {
        ApiError::internal(format!(
            "could not create {dir}: {e}. Check that the open folder is writable."
        ))
    })?;
    let path = directory.join(name);
    std::fs::write(&path, bytes).map_err(|e| {
        ApiError::internal(format!(
            "could not write {dir}/{name}: {e}. Check that the folder is \
             writable and the disk is not full."
        ))
    })?;
    Ok(path)
}

/// `rel` seen from the document's own directory — what belongs inside the
/// markdown parentheses. A link relative to the note travels with the folder;
/// one relative to the root breaks the moment the note is read from anywhere
/// else.
fn relative_to_doc(doc_path: Option<&str>, rel: &str) -> String {
    let Some(doc) = doc_path.map(str::trim).filter(|d| !d.is_empty()) else {
        return rel.to_string();
    };
    let base = doc.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("");
    if base.is_empty() {
        return rel.to_string();
    }
    rel.strip_prefix(&format!("{base}/"))
        .map(str::to_string)
        .unwrap_or_else(|| rel.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_images_are_served_and_only_from_inside_the_folder() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("assets")).unwrap();
        std::fs::write(dir.path().join("assets/a.png"), b"bytes").unwrap();
        assert!(safe_existing(dir.path(), "assets/a.png").is_ok());
        assert!(safe_existing(dir.path(), "../outside.png").is_err());
        assert!(safe_existing(dir.path(), "assets/missing.png").is_err());
        assert_eq!(mime_of("svg"), "image/svg+xml");
    }

    #[test]
    fn a_data_url_prefix_is_tolerated() {
        let plain = decode("aGk=").unwrap();
        let prefixed = decode("data:image/png;base64,aGk=").unwrap();
        assert_eq!(plain, b"hi");
        assert_eq!(prefixed, b"hi");
    }

    #[test]
    fn an_empty_or_undecodable_image_is_refused_by_name() {
        assert!(decode("").unwrap_err().message().contains("empty"));
        assert!(
            decode("not base64!!")
                .unwrap_err()
                .message()
                .contains("base64")
        );
    }

    #[test]
    fn a_name_keeps_its_extension_and_loses_everything_unsafe() {
        assert_eq!(
            clean_name("Screen Shot 2026.png").unwrap(),
            "Screen-Shot-2026.png"
        );
        assert_eq!(clean_name("/tmp/a/b/chart.PNG").unwrap(), "chart.png");
        assert_eq!(clean_name("..:*?.jpg").unwrap(), "image.jpg");
    }

    #[test]
    fn a_non_image_is_refused_with_the_list_of_what_is_not() {
        let refusal = clean_name("notes.pdf").unwrap_err();
        let err = refusal.message();
        assert!(err.contains("pdf"), "{err}");
        assert!(err.contains("png"), "{err}");
        assert!(
            clean_name("noextension")
                .unwrap_err()
                .message()
                .contains("extension")
        );
    }

    #[test]
    fn images_land_beside_the_document_that_references_them() {
        assert_eq!(
            asset_dir(Some("notes/weekly/mon.hick")).unwrap(),
            "notes/weekly/assets"
        );
        assert_eq!(asset_dir(Some("top.hick")).unwrap(), "assets");
        assert_eq!(asset_dir(None).unwrap(), "assets");
        assert!(asset_dir(Some("../escape.hick")).is_err());
    }

    #[test]
    fn the_markdown_destination_is_relative_to_the_note() {
        assert_eq!(
            relative_to_doc(Some("notes/weekly/mon.hick"), "notes/weekly/assets/a.png"),
            "assets/a.png"
        );
        assert_eq!(relative_to_doc(None, "assets/a.png"), "assets/a.png");
    }

    #[test]
    fn the_same_image_twice_is_one_file_and_a_different_one_gets_a_number() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("assets")).unwrap();
        std::fs::write(dir.path().join("assets/a.png"), b"first").unwrap();
        assert_eq!(
            unique_name(dir.path(), "assets", "a.png", b"first").unwrap(),
            "a.png"
        );
        assert_eq!(
            unique_name(dir.path(), "assets", "a.png", b"second").unwrap(),
            "a-2.png"
        );
    }
}
