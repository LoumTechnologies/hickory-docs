//! Handler for `<hick:transform>` tags.
//!
//! A transform is a passage an LLM wrote FROM another fragment, under an
//! instruction — the bridge between two representations of one fact that a
//! `hick:paste` cannot cross, because a paste copies bytes verbatim and a
//! sentence is not a JSON literal.
//!
//! Weaving one is deliberately dumb: the pinned body is emitted verbatim and
//! no model is called. The passage lives inline in the document precisely so
//! that it is reviewable in a diff, editable by hand, and free to render in
//! CI. Regeneration is an explicit `hick refresh`; staleness is checked by
//! fingerprint (see `hick_lang::transform_fingerprint`).

use std::sync::Arc;

use anyhow::Result;
use hick_exec::node::StringNode;
use hick_lang::HickTag;

use crate::{ProcessingContext, ProcessingPhase, TagHandler, TagResult, tag_attr};

/// Handler for `<hick:transform>` tags.
///
/// # Attributes
///
/// - `select` — selector for the input fragment(s) the passage was written from
/// - `instruct` — the instruction it was written under
/// - `from` — fingerprint of (input bytes + instruction) at the time it was written
pub struct TransformHandler;

/// `<hick:check claim="#m1" against=".finding,.said" from="…">`: a transform
/// spelled for the question a message asks of its sources — is this sentence
/// backed? Same pinned passage, same fingerprint, same `cites=`; the
/// instruction is built in (`hickory_cli::CHECK_INSTRUCT`) unless overridden.
pub struct CheckHandler;

impl TagHandler for CheckHandler {
    fn tag_name(&self) -> &str {
        "check"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Content
    }

    fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> Result<TagResult> {
        TransformHandler.process(tag, ctx)
    }
}

impl TagHandler for TransformHandler {
    fn tag_name(&self) -> &str {
        "transform"
    }

    fn phase(&self) -> ProcessingPhase {
        ProcessingPhase::Content
    }

    fn process(&self, tag: &HickTag, ctx: &ProcessingContext) -> Result<TagResult> {
        // The pinned passage IS the output. A transform that regenerated here
        // would call a model during `hick test`, making verification cost
        // money and return different bytes every run.
        let body = hick_lang::dedent(&hick_lang::tag_text(tag), ctx.indent);
        let _ = tag_attr(tag, "select");
        let mut out = body.trim_start_matches('\n').to_string();
        // `cites=` is what the author (a model, via refresh, or a person)
        // DECLARES the passage rests on — distinct from `select=`, which is
        // what it was derived from and is fingerprinted. It weaves as an
        // assertion, in words, never as a mark of verification.
        if let Some(cites) = tag_attr(tag, "cites")
            && !cites.trim().is_empty()
        {
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(&format!("\n*cites: {}*\n", cites.trim()));
        }
        Ok(TagResult::Node(Arc::new(StringNode::new(out))))
    }
}
