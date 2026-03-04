//! Backing file escape hatch.
//!
//! When a generated file is edited outside the hick pipeline (e.g. direct edits
//! that can't be mapped back via provenance), we create a "backing" .hick file
//! that wraps the content so our edit format can still work.
//!
//! The backing file uses tree-sitter to derive structural elements and wrap them
//! in hick: tags. For now we create a minimal wrapper; full tree-sitter integration
//! can be added later.

use std::path::Path;

/// Create a backing .hick file for content that has lost provenance.
///
/// Wraps the raw content in a minimal hick:file. When tree-sitter support is
/// added, this will insert hick:fn, hick:struct, etc. elements for better
/// structural editing.
pub fn create_backing_file(_output_path: &str, content: &str, _project_dir: &Path) -> String {
    // Minimal wrapper: raw content in hick:file.
    // Hick parser treats only hick: tags as structure; other text is literal.
    // Avoid </hick:file> in content (would close tag early).
    let safe = content.replace("</hick:file>", "</ hick:file>");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:file path="{}">
{}
</hick:file>
</hick:doc>"#,
        _output_path, safe
    )
}
