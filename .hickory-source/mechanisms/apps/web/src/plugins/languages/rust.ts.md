<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/plugins/languages/rust.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-51911212f033"></a>
## `apps/web/src/plugins/languages/rust.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-51911212f033-plugin"></a>
### `plugin`

<storycode:copy id="f-51911212f033-plugin">
import { rust } from "@codemirror/lang-rust";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "rust", grammar: () => rust(), aliases: ["rs"], tests: { definition: /^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)/, group: 1, attribute: /^\s*#\[\s*(?:tokio::|async_std::)?test\b/ }, comment: ["// ", ""] };
</storycode:copy>

### Assemble `apps/web/src/plugins/languages/rust.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/plugins/languages/rust.ts"><storycode:paste select="#f-51911212f033-plugin"/></storycode:file>

</storycode:doc>
