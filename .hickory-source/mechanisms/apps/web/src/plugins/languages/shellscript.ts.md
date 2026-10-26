<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/shellscript.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-7077b4a4ffff"></a>
## `apps/web/src/plugins/languages/shellscript.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-7077b4a4ffff-plugin"></a>
### `plugin`

<storycode:copy id="f-7077b4a4ffff-plugin">
import { shell } from "@codemirror/legacy-modes/mode/shell";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "shellscript", grammar: () => StreamLanguage.define(shell), aliases: ["bash", "sh", "shell", "zsh", "console"], comment: ["# ", ""] };
</storycode:copy>

### Assemble `apps/web/src/plugins/languages/shellscript.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/plugins/languages/shellscript.ts"><storycode:paste select="#f-7077b4a4ffff-plugin"/></storycode:file>

</storycode:doc>
