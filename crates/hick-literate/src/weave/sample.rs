//! Weaving one `<hick:sample>`.
//!
//! Lives beside `weave.rs` rather than in it because that file is at the
//! thousand-line limit the ratchet enforces (`scripts/check-file-length.sh`)
//! and a sample is a self-contained rendering: one tag in, one markdown
//! block out, reading either this run's produced bytes or the file on disk.

use std::sync::Arc;

use hick_exec::node::{InsertionPoint, StringNode};
use hick_exec::state::MultiDocumentState;

use super::extension_to_language;
use crate::tag_attr;
use crate::text::interpolate_path;

/// Weave one `<hick:sample>`: a window onto a few lines of what a cell
/// generated.
pub(crate) fn weave_sample(
    tag: &hick_lang::HickTag,
    weave_ip: &Arc<InsertionPoint>,
    state: &Arc<MultiDocumentState>,
    doc_path: &str,
) {
    let raw_path = tag_attr(tag, "path").unwrap_or_default();
    let path = interpolate_path(&raw_path, state);
    let caption = tag_attr(tag, "caption").unwrap_or_default();
    let from: usize = tag_attr(tag, "from")
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    let to: usize = tag_attr(tag, "to")
        .and_then(|v| v.parse().ok())
        .unwrap_or(usize::MAX);

    let base = std::path::Path::new(doc_path)
        .parent()
        .unwrap_or(std::path::Path::new("."));
    let full = base.join(&path);
    let language = extension_to_language(&path);

    // This run's own bytes first: an output volume is not on disk
    // until after the weave, so reading the file would show the
    // PREVIOUS run's output — and, on a first run, nothing at all.
    let produced = state.produced_file(&path);

    let mut block = String::new();
    if !caption.is_empty() {
        block.push_str(&format!("\n*{caption}*\n"));
    }
    match produced
        .ok_or(())
        .or_else(|()| std::fs::read_to_string(&full))
    {
        Ok(text) => {
            let lines: Vec<&str> = text.lines().collect();
            let last = to.min(lines.len());
            let first = from.max(1);
            if first > last {
                // Named rather than shown empty: a sample whose range
                // has slid off the end of a file it no longer matches
                // is exactly the stale illustration this element
                // exists to make impossible.
                block.push_str(&format!(
                    "\n> `{path}` has {} line(s), so lines {from}–{to} are not there \
                             any more. Re-pick the sample.\n\n",
                    lines.len()
                ));
            } else {
                block.push_str(&format!("\n```{language}\n"));
                block.push_str(&lines[first - 1..last].join("\n"));
                block.push_str(&format!(
                    "\n```\n\n<sub>{path} lines {first}–{last}, generated — \
                             shown here, not stored here.</sub>\n\n"
                ));
            }
        }
        Err(_) => block.push_str(&format!(
            "\n> `{path}` has not been generated yet, so there is nothing to show. \
                     Run the document.\n\n"
        )),
    }
    weave_ip.add(Arc::new(StringNode::new(block)));
}
