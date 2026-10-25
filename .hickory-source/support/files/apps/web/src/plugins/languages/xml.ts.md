<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/plugins/languages/xml.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `1ee81fb1300d65837f01cc06f123702f142d663435c273e604bb1d399af45e1a`.

<book:entry path="apps/web/src/plugins/languages/xml.ts">
import { xml } from "@codemirror/legacy-modes/mode/xml";
import { StreamLanguage } from "@codemirror/language";
import type { LanguageUiPlugin } from "./types";
export const plugin: LanguageUiPlugin = { id: "xml", grammar: () => StreamLanguage.define(xml), aliases: [], comment: ["<!-- ", " -->"] };
</book:entry>

</book:doc>
