<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/languages.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Compiled Language Launches What A Build Produced](../../../../../contract.md)

Captured SHA-256: `212774325193eccf82d44bb863a4a3439238530388b02e7a0504112e61d91905`.

<book:entry path="apps/web/src/debug/languages.ts">
// Debug availability comes from the same language plugins as the native host.
import { DEBUGGABLE_LANGUAGE_IDS } from "../editor/generated/languages";

export const DEBUGGABLE_LANGUAGES = new Set(DEBUGGABLE_LANGUAGE_IDS);

export function isDebuggable(language: string | null | undefined): boolean {
  return !!language && DEBUGGABLE_LANGUAGES.has(language);
}
</book:entry>

</book:doc>
