<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/TableNumberFormat.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-77b3b46354cf"></a>
## `apps/web/src/components/TableNumberFormat.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-77b3b46354cf-TableNumberFormat"></a>
### `TableNumberFormat`

<storycode:copy id="f-77b3b46354cf-TableNumberFormat">
import "./TableNumberFormat.css";
import { NUMBER_FORMATS, type NumberFormat } from "../lib/tableFormats";

export function TableNumberFormat({ value, disabled, onChange }: {
  value: string; disabled: boolean; onChange: (format: NumberFormat) => void;
}) {
  return <label className="table-number-format">Format <select className="btn btn-small" aria-label="Cell number format" value={value} disabled={disabled}
    onChange={event => onChange(event.target.value as NumberFormat)}>
    <option value="mixed" disabled>Mixed</option>
    {Object.entries(NUMBER_FORMATS).map(([format, label]) => <option key={format} value={format}>{label}</option>)}
  </select></label>;
}
</storycode:copy>

### Assemble `apps/web/src/components/TableNumberFormat.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/TableNumberFormat.tsx"><storycode:paste select="#f-77b3b46354cf-TableNumberFormat"/></storycode:file>

</storycode:doc>
