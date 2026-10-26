<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/terraform.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-afa0268a79c6"></a>
## `apps/web/src/plugins/languages/terraform.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-afa0268a79c6-plugin"></a>
### `plugin`

<storycode:copy id="f-afa0268a79c6-plugin">
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "terraform", aliases: [], comment: ["// ", ""] };
</storycode:copy>

### Assemble `apps/web/src/plugins/languages/terraform.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/plugins/languages/terraform.ts"><storycode:paste select="#f-afa0268a79c6-plugin"/></storycode:file>

</storycode:doc>
