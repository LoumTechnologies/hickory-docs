<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/fsharp.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c1a9d00fbe80"></a>
## `apps/web/src/plugins/languages/fsharp.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-c1a9d00fbe80-plugin"></a>
### `plugin`

<storycode:copy id="f-c1a9d00fbe80-plugin">
import { fSharp } from "@codemirror/legacy-modes/mode/mllike";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "fsharp", grammar: () => StreamLanguage.define(fSharp), aliases: ["f#"], comment: ["// ", ""] };
</storycode:copy>

### Assemble `apps/web/src/plugins/languages/fsharp.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/plugins/languages/fsharp.ts"><storycode:paste select="#f-c1a9d00fbe80-plugin"/></storycode:file>

</storycode:doc>
