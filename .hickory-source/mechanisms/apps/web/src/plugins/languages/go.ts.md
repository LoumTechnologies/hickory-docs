<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/go.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-813640cbebf0"></a>
## `apps/web/src/plugins/languages/go.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-813640cbebf0-plugin"></a>
### `plugin`

<storycode:copy id="f-813640cbebf0-plugin">
import { go } from "@codemirror/legacy-modes/mode/go";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "go", grammar: () => StreamLanguage.define(go), aliases: ["golang"], tests: { definition: /^\s*func\s+(Test[A-Za-z0-9_]*)\s*\(\s*\w+\s+\*testing\.T\s*\)/, group: 1 }, comment: ["// ", ""] };
</storycode:copy>

### Assemble `apps/web/src/plugins/languages/go.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/plugins/languages/go.ts"><storycode:paste select="#f-813640cbebf0-plugin"/></storycode:file>

</storycode:doc>
