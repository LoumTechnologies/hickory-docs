<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/landing/demos/DemoEditor.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a18c55c77a46"></a>
## `apps/web/src/landing/demos/DemoEditor.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-a18c55c77a46-export_statement"></a>
### `export_statement`

The demos use the same portable editing surface as external hosts.

<storycode:copy id="f-a18c55c77a46-export_statement">
// The demos use the same portable editing surface as external hosts.
export { EditingSurface as DemoEditor } from "../../embed/EditingSurface";
</storycode:copy>
<a id="f-a18c55c77a46-export_statement-2"></a>
### `export_statement`

<storycode:copy id="f-a18c55c77a46-export_statement-2">
export type { EditingSurfaceProps as DemoEditorProps } from "../../embed/EditingSurface";
</storycode:copy>

### Assemble `apps/web/src/landing/demos/DemoEditor.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/landing/demos/DemoEditor.tsx"><storycode:paste select="#f-a18c55c77a46-export_statement"/><storycode:paste select="#f-a18c55c77a46-export_statement-2"/></storycode:file>

</storycode:doc>
