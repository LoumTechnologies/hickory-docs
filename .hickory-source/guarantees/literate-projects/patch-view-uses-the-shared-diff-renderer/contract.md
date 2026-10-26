<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Patch elements use the shared diff renderer

For: Maintainers reorganizing source while preserving the complete generated program.

Given a saved executable patch element, when the document view renders its
block, then the element registry selects the patch view and draws its raw unified
diff through the same DiffView used for existing changes. The context labels
identify New as the completed project and Old as the scope input. Rendering
does not apply a patch, execute a cell, build code or launch a native tool.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/literate-projects/patch-view-uses-the-shared-diff-renderer.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Implementation

<a id="output-cb810da6b8ce"></a>
## `crates/hick-literate/src/patch_element.rs`

Related contracts: [Patch elements use the shared diff renderer](contract.md).

<a id="f-cb810da6b8ce-PatchElement"></a>
### `PatchElement`

<storycode:copy id="f-cb810da6b8ce-PatchElement">
use crate::render::BlockModelInput;
use hick_blocks::{AttrSpec, Block, Descend, Element, span_of};
use hick_lang::HickTag;

pub(crate) struct PatchElement;
</storycode:copy>
<a id="f-cb810da6b8ce-impl-Element-BlockModelInput-a-for-PatchElement"></a>
### `impl-Element<BlockModelInput<'a>> for PatchElement`

<storycode:copy id="f-cb810da6b8ce-impl-Element-BlockModelInput-a-for-PatchElement">
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

</storycode:copy>
<a id="f-cb810da6b8ce-ScopeElement"></a>
### `ScopeElement`

<storycode:copy id="f-cb810da6b8ce-ScopeElement">
pub(crate) struct ScopeElement;
</storycode:copy>
<a id="f-cb810da6b8ce-impl-Element-BlockModelInput-a-for-ScopeElement"></a>
### `impl-Element<BlockModelInput<'a>> for ScopeElement`

<storycode:copy id="f-cb810da6b8ce-impl-Element-BlockModelInput-a-for-ScopeElement">
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
</storycode:copy>

### Assemble `crates/hick-literate/src/patch_element.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/src/patch_element.rs"><storycode:paste select="#f-cb810da6b8ce-PatchElement"/><storycode:paste select="#f-cb810da6b8ce-impl-Element-BlockModelInput-a-for-PatchElement"/><storycode:paste select="#f-cb810da6b8ce-ScopeElement"/><storycode:paste select="#f-cb810da6b8ce-impl-Element-BlockModelInput-a-for-ScopeElement"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/DiffView.tsx](../../collaboration/the-git-pane-does-the-daily-loop/contract.md#output-2d0324b58c4d)
- [apps/web/src/elements/index.ts](../../../mechanisms/apps/web/src/elements/index.ts.md#output-9b7b2fa9b3c3)
- [apps/web/src/elements/patch/view.tsx](../../../mechanisms/apps/web/src/elements/patch/view.tsx.md#output-d6e75baad108)
- [crates/hick-literate/src/patch_element.rs](contract.md#output-cb810da6b8ce)
- [crates/hick-literate/src/render.rs](../../../mechanisms/crates/hick-literate/src/render.rs.md#output-4a7f39f68eb8)

### Tests and fixtures

- [apps/web/src/elements/patch/view.test.tsx](../../../mechanisms/apps/web/src/elements/patch/view.tsx.md#output-059795295090)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
