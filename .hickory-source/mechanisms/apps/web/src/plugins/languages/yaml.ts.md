<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/yaml.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-e1cd2e757893"></a>
## `apps/web/src/plugins/languages/yaml.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-e1cd2e757893-plugin"></a>
### `plugin`

<storycode:copy id="f-e1cd2e757893-plugin">
import { yaml } from "@codemirror/legacy-modes/mode/yaml";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "yaml", grammar: () => StreamLanguage.define(yaml), aliases: ["yml"], comment: ["# ", ""] };
</storycode:copy>

### Assemble `apps/web/src/plugins/languages/yaml.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/plugins/languages/yaml.ts"><storycode:paste select="#f-e1cd2e757893-plugin"/></storycode:file>

</storycode:doc>
