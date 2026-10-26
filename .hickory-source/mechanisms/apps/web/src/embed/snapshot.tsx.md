<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/embed/snapshot.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-e9d2d7727ec4"></a>
## `apps/web/src/embed/snapshot.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A viewer snapshot keeps recorded results historical](../../../../../guarantees/embedding/a-viewer-snapshot-keeps-recorded-results-historical/contract.md), [Published executable results identify reviewed inputs](../../../../../guarantees/embedding/publication-is-bound-to-reviewed-inputs/contract.md).

<a id="f-e9d2d7727ec4-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-e9d2d7727ec4-expression_statement">
import { createRoot } from "react-dom/client";
import { SnapshotPage } from "./SnapshotPage";
import { StaticUiSurface } from "./StaticUiSurface";
import "../styles.css";
import "./snapshot.css";
createRoot(document.getElementById("root")!).render(<StaticUiSurface><SnapshotPage /></StaticUiSurface>);
</storycode:copy>

### Assemble `apps/web/src/embed/snapshot.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/snapshot.tsx"><storycode:paste select="#f-e9d2d7727ec4-expression_statement"/></storycode:file>

</storycode:doc>
