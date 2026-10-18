import { DatasetPanel } from "../../components/DatasetPanel";
import type { ElementView } from "../types";
export const datasetView: ElementView = {
  kind: "dataset",
  draws: block => block.name === "dataset",
  render: (slot, cx) => <DatasetPanel src={slot.dataset?.src ?? ""} format={slot.dataset?.format ?? "parquet"} path={cx.path} />,
};
