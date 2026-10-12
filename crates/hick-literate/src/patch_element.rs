use crate::render::BlockModelInput;
use hick_blocks::{AttrSpec, Block, Descend, Element, span_of};
use hick_lang::HickTag;

pub(crate) struct PatchElement;
impl<'a> Element<BlockModelInput<'a>> for PatchElement {
    fn name(&self) -> &'static str {
        "patch"
    }
    fn attributes(&self) -> &'static [AttrSpec] {
        const ATTRS: &[AttrSpec] = &[
            AttrSpec::optional("id", "construction identity"),
            AttrSpec::optional("after", "explicit predecessor construction identities"),
        ];
        ATTRS
    }
    fn render(&self, tag: &HickTag, _: &BlockModelInput<'a>) -> Option<Block> {
        Some(
            Block::new("patch", span_of(tag))
                .with("body", tag.text_content())
                .with("project_context", "New · completed project")
                .with("old_context", "Old · scope input"),
        )
    }
}

pub(crate) struct ScopeElement;
impl<'a> Element<BlockModelInput<'a>> for ScopeElement {
    fn name(&self) -> &'static str {
        "scope"
    }
    fn attributes(&self) -> &'static [AttrSpec] {
        const ATTRS: &[AttrSpec] = &[
            AttrSpec::required("id", "explicit atomic project boundary"),
            AttrSpec::optional("after", "scope input constructions"),
        ];
        ATTRS
    }
    fn render(&self, _tag: &HickTag, _cx: &BlockModelInput<'a>) -> Option<Block> {
        None
    }
    fn descend(&self) -> Descend {
        Descend::All
    }
}
