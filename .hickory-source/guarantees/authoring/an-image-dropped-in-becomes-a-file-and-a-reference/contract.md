<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Image Dropped In Becomes A File And A Reference

For: Authors editing executable documents and the files those documents produce.

Given an image dragged into an open document, or pasted from the clipboard,
when it lands then the bytes are written into an `assets/` directory beside
that document and the buffer gains an ordinary markdown reference to it —
`![a chart](assets/a-chart.png)` — at the point of the drop. The picture is
drawn under the line that references it. Nothing is embedded in the document,
nothing is uploaded anywhere, and the woven markdown needs no special support
for either, because there is nothing special in the document to support.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/an-image-dropped-in-becomes-a-file-and-a-reference.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-a83dd2c67bbf"></a>
## `crates/hickory-cli/src/serve/asset.rs`

Related contracts: [An Image Dropped In Becomes A File And A Reference](contract.md).

<a id="f-a83dd2c67bbf-MAX_ASSET_BYTES"></a>
### `MAX_ASSET_BYTES`

Assets: the image you dropped into a note, written into the notes folder.
A note that references `assets/chart.png` is only a note as long as that
file exists beside it. So an image dropped or pasted into the editor is
written to disk here, into an `assets/` directory next to the document,
and the editor writes an ordinary markdown reference to it. Nothing is
embedded, nothing is uploaded, and nothing is stored anywhere this app
invented: the folder is the user's, the file is a file, and a `git add`
picks up the note and the picture together.
Base64 in a JSON body rather than a multipart upload because the caller is
a `FileReader` in the same process's own window and the payload is
megabytes at most — a second body format on this server would earn nothing.
What this deliberately is NOT:
- **Not a general file-write route.** Only the image types a note can
actually display are accepted; `PUT /api/file` is where text goes, and
it stays the only way to overwrite something that already exists.
- **Not a store.** The name is derived from what was dropped, kept unique
against what is already there, and never recorded anywhere else.

<storycode:copy id="f-a83dd2c67bbf-MAX_ASSET_BYTES">
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

</storycode:copy>
<a id="f-a83dd2c67bbf-ASSET_DIR"></a>
### `ASSET_DIR`

The directory an image lands in, relative to the document that references
it. One shared name so a folder of notes has one place to look, and the
same name a static-site generator would expect.

<storycode:copy id="f-a83dd2c67bbf-ASSET_DIR">
/// The directory an image lands in, relative to the document that references
/// it. One shared name so a folder of notes has one place to look, and the
/// same name a static-site generator would expect.
const ASSET_DIR: &str = "assets";

</storycode:copy>
<a id="f-a83dd2c67bbf-IMAGE_EXTENSIONS"></a>
### `IMAGE_EXTENSIONS`

The image types a woven markdown file can show in a browser, GitHub, or a
preview pane. Anything else is refused by name rather than written and
then silently not rendered.

<storycode:copy id="f-a83dd2c67bbf-IMAGE_EXTENSIONS">
/// The image types a woven markdown file can show in a browser, GitHub, or a
/// preview pane. Anything else is refused by name rather than written and
/// then silently not rendered.
const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "svg", "avif", "bmp", "ico", "tif", "tiff",
];

</storycode:copy>
<a id="f-a83dd2c67bbf-SaveAsset"></a>
### `SaveAsset`

<storycode:copy id="f-a83dd2c67bbf-SaveAsset">
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

</storycode:copy>
<a id="f-a83dd2c67bbf-post_asset"></a>
### `post_asset`

`POST /api/asset` — write an image into the notes folder.
Answers both paths the caller needs and neither can be derived from the
other without knowing the folder's shape: `path` is root-relative (the
tree, a later read), and `relative` is what goes inside the markdown
parentheses, resolved from the document's own directory so the note keeps
working when the folder moves.

<storycode:copy id="f-a83dd2c67bbf-post_asset">
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

</storycode:copy>
<a id="f-a83dd2c67bbf-AssetQuery"></a>
### `AssetQuery`

<storycode:copy id="f-a83dd2c67bbf-AssetQuery">
#[derive(Deserialize)]
pub struct AssetQuery {
    pub path: String,
}

</storycode:copy>
<a id="f-a83dd2c67bbf-get_asset"></a>
### `get_asset`

`GET /api/asset?path=…` — the image's bytes, so the editor can show it.
A route rather than a `data:` URL in the decoration: the picture is a file
on disk that the note points at, and reading it through the same path the
woven markdown would is what makes "the editor shows what the markdown
says" true rather than approximately true. Images only — this is a
viewer's read, not a way to exfiltrate arbitrary bytes over a route with
no other checks on it.

<storycode:copy id="f-a83dd2c67bbf-get_asset">
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

</storycode:copy>
<a id="f-a83dd2c67bbf-mime_of"></a>
### `mime_of`

The content type for an extension this route has already accepted.

<storycode:copy id="f-a83dd2c67bbf-mime_of">
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

</storycode:copy>
<a id="f-a83dd2c67bbf-safe_existing"></a>
### `safe_existing`

`rel` inside `root`, refusing anything that names its way out — including
through a symlink, which only a canonicalized path can catch.

<storycode:copy id="f-a83dd2c67bbf-safe_existing">
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

</storycode:copy>
<a id="f-a83dd2c67bbf-decode"></a>
### `decode`

The bytes, or the refusal that says which half was wrong.

<storycode:copy id="f-a83dd2c67bbf-decode">
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

</storycode:copy>
<a id="f-a83dd2c67bbf-clean_name"></a>
### `clean_name`

A file name that is safe on every filesystem this ships to, keeping the
extension because it is what decides whether the image renders.

<storycode:copy id="f-a83dd2c67bbf-clean_name">
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

</storycode:copy>
<a id="f-a83dd2c67bbf-asset_dir"></a>
### `asset_dir`

Where images for the document at `doc_path` live, root-relative.

<storycode:copy id="f-a83dd2c67bbf-asset_dir">
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

</storycode:copy>
<a id="f-a83dd2c67bbf-join_rel"></a>
### `join_rel`

`a` and `b` as one root-relative path, with no leading or doubled slash.

<storycode:copy id="f-a83dd2c67bbf-join_rel">
/// `a` and `b` as one root-relative path, with no leading or doubled slash.
fn join_rel(a: &str, b: &str) -> String {
    if a.is_empty() {
        b.to_string()
    } else {
        format!("{a}/{b}")
    }
}

</storycode:copy>
<a id="f-a83dd2c67bbf-unique_name"></a>
### `unique_name`

A name nothing else in `dir` is using — unless the bytes there are already
these bytes, which makes the same screenshot pasted twice one file.

<storycode:copy id="f-a83dd2c67bbf-unique_name">
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

</storycode:copy>
<a id="f-a83dd2c67bbf-write_asset"></a>
### `write_asset`

Write the bytes, creating the directory the first time.

<storycode:copy id="f-a83dd2c67bbf-write_asset">
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

</storycode:copy>
<a id="f-a83dd2c67bbf-relative_to_doc"></a>
### `relative_to_doc`

`rel` seen from the document's own directory — what belongs inside the
markdown parentheses. A link relative to the note travels with the folder;
one relative to the root breaks the moment the note is read from anywhere
else.

<storycode:copy id="f-a83dd2c67bbf-relative_to_doc">
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

</storycode:copy>
<a id="f-a83dd2c67bbf-tests"></a>
### `tests`

<storycode:copy id="f-a83dd2c67bbf-tests">
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
            asset_dir(Some("notes/weekly/mon.md")).unwrap(),
            "notes/weekly/assets"
        );
        assert_eq!(asset_dir(Some("top.md")).unwrap(), "assets");
        assert_eq!(asset_dir(None).unwrap(), "assets");
        assert!(asset_dir(Some("../escape.md")).is_err());
    }

    #[test]
    fn the_markdown_destination_is_relative_to_the_note() {
        assert_eq!(
            relative_to_doc(Some("notes/weekly/mon.md"), "notes/weekly/assets/a.png"),
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
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/asset.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/asset.rs"><storycode:paste select="#f-a83dd2c67bbf-MAX_ASSET_BYTES"/><storycode:paste select="#f-a83dd2c67bbf-ASSET_DIR"/><storycode:paste select="#f-a83dd2c67bbf-IMAGE_EXTENSIONS"/><storycode:paste select="#f-a83dd2c67bbf-SaveAsset"/><storycode:paste select="#f-a83dd2c67bbf-post_asset"/><storycode:paste select="#f-a83dd2c67bbf-AssetQuery"/><storycode:paste select="#f-a83dd2c67bbf-get_asset"/><storycode:paste select="#f-a83dd2c67bbf-mime_of"/><storycode:paste select="#f-a83dd2c67bbf-safe_existing"/><storycode:paste select="#f-a83dd2c67bbf-decode"/><storycode:paste select="#f-a83dd2c67bbf-clean_name"/><storycode:paste select="#f-a83dd2c67bbf-asset_dir"/><storycode:paste select="#f-a83dd2c67bbf-join_rel"/><storycode:paste select="#f-a83dd2c67bbf-unique_name"/><storycode:paste select="#f-a83dd2c67bbf-write_asset"/><storycode:paste select="#f-a83dd2c67bbf-relative_to_doc"/><storycode:paste select="#f-a83dd2c67bbf-tests"/></storycode:file>

## Implementation review

### Implementation

- [apps/desktop/src-tauri/src/lib.rs](../../../mechanisms/apps/desktop/src-tauri/src/lib.rs.md#output-42caba993099)
- [apps/web/src/editor/DocumentEditor.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.tsx.md#output-aab82870e517)
- [apps/web/src/editor/mdLinks.ts](../../../mechanisms/apps/web/src/editor/mdLinks.ts.md#output-1645f056db0c)
- [apps/web/src/editor/mdPaste.ts](../../../mechanisms/apps/web/src/editor/mdPaste.ts.md#output-722af83950b4)
- [crates/hickory-cli/src/serve/asset.rs](contract.md#output-a83dd2c67bbf)
- [crates/hickory-cli/src/serve/mod.rs](../../../mechanisms/crates/hickory-cli/src/serve/mod.rs.md#output-09ea558208cd)

### Tests and fixtures

- [apps/web/src/editor/mdLinks.test.ts](../../../mechanisms/apps/web/src/editor/mdLinks.ts.md#output-419d07f37b80)
- [apps/web/src/editor/mdPaste.test.ts](../../../mechanisms/apps/web/src/editor/mdPaste.ts.md#output-c4094026435c)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
