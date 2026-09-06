//! An element's body, resolved before it is woven.
//!
//! Two renderers must READ their body rather than stream it: a graph scene
//! must be parsed to be downgraded to mermaid, and a table's CSV must be
//! parsed to be laid out as markdown. Both go through here, so a
//! `<hick:paste>` inside either resolves — which is what lets a picture or a
//! table be DERIVED from the cell that produced its numbers.

use std::sync::Arc;

use hick_exec::state::MultiDocumentState;
use hick_lang::{HickNode, dedent};

use crate::tag_attr;

/// An element's body, resolved synchronously: text children dedented,
/// `<hick:paste>` children answered from the run state's fragments — which is
/// what lets a derived scene's topology, or a table's rows, arrive from the
/// cell that proved them.
///
/// The streaming path the other renderers use cannot be READ before it is
/// woven, and both callers must read: a scene must be parsed to be
/// downgraded to mermaid, and CSV must be parsed to be laid out as a
/// markdown table.
pub(crate) fn resolved_body(tag: &hick_lang::HickTag, state: &Arc<MultiDocumentState>) -> String {
    let mut body = String::new();
    for (position, child) in tag.children.iter().enumerate() {
        match child {
            HickNode::Text(text, span) => {
                // The line break that ends the open tag's line is the tag's
                // formatting, not the body's — dropped here exactly as the
                // file-output pass drops it (`strip_opening_break`). Without
                // this a `<hick:table path="…">` wrote `region,units` as the
                // CSV's first line and wove a table with an EMPTY header and
                // `region,units` as its first DATA row: the two halves of one
                // block disagreeing about their own first line.
                let text = match position {
                    0 => crate::strip_opening_break(text, span.as_ref()).0,
                    _ => text.as_str(),
                };
                body.push_str(&dedent(text, tag.source_column));
            }
            HickNode::Tag(child_tag) if child_tag.name == "paste" => {
                if let Some(select) = tag_attr(child_tag, "select")
                    && let Some(resolved) = state.resolve_paste(&select, None, false)
                {
                    body.push_str(&resolved);
                }
            }
            HickNode::Tag(_) => {}
        }
    }
    body
}
