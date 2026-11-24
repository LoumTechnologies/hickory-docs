import "./TableNumberFormat.css";
import { NUMBER_FORMATS, type NumberFormat } from "../lib/tableFormats";

export function TableNumberFormat({ value, disabled, onChange }: {
  value: string; disabled: boolean; onChange: (format: NumberFormat) => void;
}) {
  return <label className="table-number-format">Format <select className="btn btn-small" aria-label="Cell number format" value={value} disabled={disabled} title={disabled ? "Select an editable table cell in the current source." : undefined}
    onChange={event => onChange(event.target.value as NumberFormat)}>
    <option value="mixed" disabled title={"The selected cells use different number formats. Choose one format to change them."}>Mixed</option>
    {Object.entries(NUMBER_FORMATS).map(([format, label]) => <option key={format} value={format}>{label}</option>)}
  </select></label>;
}
