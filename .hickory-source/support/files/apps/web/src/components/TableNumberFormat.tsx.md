<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/TableNumberFormat.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `5edcc81388b329072b6db181e5a2840624ac590c2932acc1b75132440a2f0c84`.

<book:entry path="apps/web/src/components/TableNumberFormat.tsx">
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
</book:entry>

</book:doc>
