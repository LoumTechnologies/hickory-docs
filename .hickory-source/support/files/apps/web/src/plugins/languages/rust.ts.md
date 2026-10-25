<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/rust.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `c87a16b36c21e3ac23bbd852dcf0adbdcdce778e612269ad674646f937f62f26`.

<book:entry path="apps/web/src/plugins/languages/rust.ts">
import { rust } from "@codemirror/lang-rust";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "rust", grammar: () => rust(), aliases: ["rs"], tests: { definition: /^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)/, group: 1, attribute: /^\s*#\[\s*(?:tokio::|async_std::)?test\b/ }, comment: ["// ", ""] };
</book:entry>

</book:doc>
