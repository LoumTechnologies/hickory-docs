<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/coverage/client.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c4afa50a031d"></a>
## `apps/web/src/coverage/client.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-c4afa50a031d-coverageCall"></a>
### `coverageCall`

<storycode:copy id="f-c4afa50a031d-coverageCall">
import { request } from "../api/client";
export const coverageCall = <T,>(body: Record<string, unknown>) => request<T>("POST", "/api/tests/coverage", body);
</storycode:copy>

### Assemble `apps/web/src/coverage/client.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/coverage/client.ts"><storycode:paste select="#f-c4afa50a031d-coverageCall"/></storycode:file>

</storycode:doc>
