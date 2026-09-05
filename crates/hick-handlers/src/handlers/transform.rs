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
/// - `wrote` — fingerprint of the passage AS THE MODEL WROTE IT
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
        // A passage whose bytes are no longer the ones the model produced
        // says so, here, in the weave.
        //
        // Editing one is allowed — it is the author's document — and this is
        // not a failure. What is not allowed is going on claiming the model
        // wrote words a person typed. `from=` pins the INPUTS, so an edit to
        // the prose alone leaves it matching and the passage looked untouched;
        // `wrote=` pins the output, which is the other half.
        //
        // A passage with no `wrote=` at all was written before this existed
        // and claims nothing either way: silence, not a mark.
        if let Some(recorded) = tag_attr(tag, "wrote")
            && !recorded.trim().is_empty()
            && hick_lang::passage_fingerprint(&body) != recorded.trim()
        {
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str("\n*written by a model, and edited by hand since*\n");
        }
        Ok(TagResult::Node(Arc::new(StringNode::new(out))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hick_exec::state::MultiDocumentState;
    use std::collections::HashMap;

    /// Weave one transform holding `passage`, stamped with `wrote`.
    fn woven(passage: &str, wrote: Option<&str>) -> String {
        let mut attributes = vec![
            ("select".to_string(), ".x".to_string()),
            ("instruct".to_string(), "Summarize.".to_string()),
        ];
        if let Some(wrote) = wrote {
            attributes.push(("wrote".to_string(), wrote.to_string()));
        }
        let tag = HickTag {
            name: "transform".to_string(),
            attributes,
            children: vec![hick_lang::HickNode::Text(passage.to_string(), None)],
            self_closing: false,
            source_line: 1,
            source_column: 0,
            source_span: None,
            close_span: None,
        };
        let state = Arc::new(MultiDocumentState::default());
        let transcripts = HashMap::new();
        let ctx = ProcessingContext {
            state: &state,
            transcripts: &transcripts,
            indent: 0,
            registry: None,
            context: None,
            source_file: None,
            span_files: &[],
        };
        match TransformHandler.process(&tag, &ctx).expect("weaves") {
            TagResult::Node(node) => node.as_string_value().unwrap_or_default().to_string(),
            _ => panic!("a transform weaves to a node"),
        }
    }

    /// Protects docs/guarantees/lineage/an-edited-ai-passage-stops-claiming-the-model-wrote-it.md
    #[test]
    fn an_edited_passage_says_so_in_the_weave() {
        let passage = "The release was red for two days.";
        let wrote = hick_lang::passage_fingerprint(passage);
        let same = woven(passage, Some(&wrote));
        assert!(!same.contains("edited by hand"), "{same}");

        let changed = woven("Something else entirely.", Some(&wrote));
        assert!(
            changed.contains("written by a model, and edited by hand since"),
            "{changed}"
        );
    }

    /// A passage from before the attribute existed claims nothing either way.
    #[test]
    fn no_fingerprint_means_no_claim() {
        let woven = woven("anything at all", None);
        assert!(!woven.contains("edited by hand"), "{woven}");
    }
}
