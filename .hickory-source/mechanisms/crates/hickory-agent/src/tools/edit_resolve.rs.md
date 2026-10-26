<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-agent/src/tools/edit_resolve.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-01a94e79b0a3"></a>
## `crates/hickory-agent/src/tools/edit_resolve.rs`

<a id="f-01a94e79b0a3-ResolvedEdit"></a>
### `ResolvedEdit`

<storycode:copy id="f-01a94e79b0a3-ResolvedEdit">
use super::{
    ToolInvocation,
    hashline::{Anchor, LineIndex, ResolvedAnchor, parse_anchor, resolve_anchor},
};

/// A resolved hashline edit: byte range, replacement text, and the line the
/// edited region starts on (for rendering the result).
pub(super) struct ResolvedEdit {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) text: String,
    pub(super) first_line: usize,
}

</storycode:copy>
<a id="f-01a94e79b0a3-resolve_edit"></a>
### `resolve_edit`

Resolve run/after/occurrence + `<hick:input>` into a byte edit.

<storycode:copy id="f-01a94e79b0a3-resolve_edit">
/// Resolve run/after/occurrence + `<hick:input>` into a byte edit.
pub(super) fn resolve_edit(
    index: &LineIndex,
    inv: &ToolInvocation,
    resynced: bool,
) -> Result<ResolvedEdit, String> {
    if resynced {
        return Err("The document or an upstream changed since the previous surface. Read it again before editing; nothing was changed.".into());
    }
    if let Some(snapshot) = inv.arg("snapshot")
        && snapshot != index.snapshot
    {
        return Err("The supplied snapshot no longer matches the whole surface, including the range interior. Read it again; nothing was changed.".into());
    }
    if let Some(edit) = crate::review::whole_edit(index, inv) {
        let (end, text) = edit?;
        return Ok(ResolvedEdit {
            start: 0,
            end,
            text,
            first_line: 0,
        });
    }
    let anchor: Anchor = parse_anchor(inv.arg("run"), inv.arg("after")).map_err(|e| e.0)?;
    let occurrence = match inv.arg("occurrence") {
        Some(v) => Some(
            v.parse::<usize>()
                .map_err(|_| format!("occurrence '{v}' is not a positive integer"))?,
        ),
        None => None,
    };
    let resolved = resolve_anchor(index, &anchor, occurrence).map_err(|e| {
        if resynced {
            format!(
                "the document changed on disk since you last read it; it was re-woven \
                 automatically, but this edit's anchors no longer resolve: {}",
                e.0
            )
        } else {
            e.0
        }
    })?;

    let payload = inv.input.clone();
    match resolved {
        ResolvedAnchor::Run { first, last } => match payload {
            Some(text) => Ok(ResolvedEdit {
                start: index.starts[first],
                end: index.content_end(last),
                text,
                first_line: first,
            }),
            // No payload: delete the whole run including its newlines.
            None => Ok(ResolvedEdit {
                start: index.starts[first],
                end: index.line_end(last),
                text: String::new(),
                first_line: first,
            }),
        },
        ResolvedAnchor::After(at) => {
            let text = payload
                .ok_or_else(|| "insertion (after=...) needs an <hick:input> payload".to_string())?;
            match at {
                None => Ok(ResolvedEdit {
                    start: 0,
                    end: 0,
                    text: format!("{text}\n"),
                    first_line: 0,
                }),
                Some(i) => {
                    let last_line = i + 1 == index.lines.len();
                    if last_line && !index.trailing_newline {
                        Ok(ResolvedEdit {
                            start: index.content_end(i),
                            end: index.content_end(i),
                            text: format!("\n{text}"),
                            first_line: i + 1,
                        })
                    } else {
                        Ok(ResolvedEdit {
                            start: index.line_end(i),
                            end: index.line_end(i),
                            text: format!("{text}\n"),
                            first_line: i + 1,
                        })
                    }
                }
            }
        }
    }
}
</storycode:copy>

### Assemble `crates/hickory-agent/src/tools/edit_resolve.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-agent/src/tools/edit_resolve.rs"><storycode:paste select="#f-01a94e79b0a3-ResolvedEdit"/><storycode:paste select="#f-01a94e79b0a3-resolve_edit"/></storycode:file>

</storycode:doc>
