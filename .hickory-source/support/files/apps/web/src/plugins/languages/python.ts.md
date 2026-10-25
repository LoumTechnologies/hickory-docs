<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/python.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `bfa974e005cbead67539c62d89d47711f601cec0a57d8e8502df971a2bb55117`.

<book:entry path="apps/web/src/plugins/languages/python.ts">
import { python } from "@codemirror/lang-python";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "python", grammar: () => python(), aliases: ["py"], tests: { definition: /^\s*(?:async\s+)?def\s+(test_[A-Za-z0-9_]*)\s*\(/, group: 1 }, comment: ["# ", ""] };
</book:entry>

</book:doc>
