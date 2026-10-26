<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/swift.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f53f9145d6b1"></a>
## `apps/web/src/plugins/languages/swift.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-f53f9145d6b1-plugin"></a>
### `plugin`

<storycode:copy id="f-f53f9145d6b1-plugin">
import { swift } from "@codemirror/legacy-modes/mode/swift";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "swift", grammar: () => StreamLanguage.define(swift), aliases: [], comment: ["// ", ""] };
</storycode:copy>

### Assemble `apps/web/src/plugins/languages/swift.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/plugins/languages/swift.ts"><storycode:paste select="#f-f53f9145d6b1-plugin"/></storycode:file>

</storycode:doc>
