<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/csharp.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-4a4375f04b64"></a>
## `apps/web/src/plugins/languages/csharp.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-4a4375f04b64-plugin"></a>
### `plugin`

<storycode:copy id="f-4a4375f04b64-plugin">
import { csharp } from "@codemirror/legacy-modes/mode/clike";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "csharp", grammar: () => StreamLanguage.define(csharp), aliases: ["c#", "cs", "dotnet"], tests: { definition: /^\s*(?:public\s+|private\s+|internal\s+|static\s+|async\s+|virtual\s+|override\s+)*[A-Za-z_<>\[\],. ]+?\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/, group: 1, attribute: /^\s*\[\s*(?:Fact|Theory|Test|TestMethod|TestCase)\b/ }, comment: ["// ", ""] };
</storycode:copy>

### Assemble `apps/web/src/plugins/languages/csharp.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/plugins/languages/csharp.ts"><storycode:paste select="#f-4a4375f04b64-plugin"/></storycode:file>

</storycode:doc>
