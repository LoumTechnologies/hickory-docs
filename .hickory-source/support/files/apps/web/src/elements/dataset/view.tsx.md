<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/elements/dataset/view.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `3d384a0b67f40ba1de35b8d229617c359676219b2db888901fa80e8c6fe2e361`.

<book:entry path="apps/web/src/elements/dataset/view.tsx">
import { DatasetPanel } from "../../components/DatasetPanel";
import type { ElementView } from "../types";
export const datasetView: ElementView = {
  kind: "dataset",
  draws: block => block.name === "dataset",
  render: (slot, cx) => <DatasetPanel src={slot.dataset?.src ?? ""} format={slot.dataset?.format ?? "parquet"} path={cx.path} />,
};
</book:entry>

</book:doc>
