<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/elements/dataset/view.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f51c4124c547"></a>
## `apps/web/src/elements/dataset/view.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-f51c4124c547-datasetView"></a>
### `datasetView`

<storycode:copy id="f-f51c4124c547-datasetView">
import { DatasetPanel } from "../../components/DatasetPanel";
import type { ElementView } from "../types";
export const datasetView: ElementView = {
  kind: "dataset",
  draws: block => block.name === "dataset",
  render: (slot, cx) => <DatasetPanel src={slot.dataset?.src ?? ""} format={slot.dataset?.format ?? "parquet"} path={cx.path} />,
};
</storycode:copy>

### Assemble `apps/web/src/elements/dataset/view.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/elements/dataset/view.tsx"><storycode:paste select="#f-f51c4124c547-datasetView"/></storycode:file>

</storycode:doc>
