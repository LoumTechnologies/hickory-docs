import { cellLabel, parseCellLabel } from "./cellRef";

export const NUMBER_FORMATS = {
  general: "General", number: "Number (1,234.50)", percent: "Percent",
  USD: "Money ($)", EUR: "Money (€)", GBP: "Money (£)", JPY: "Money (¥)",
} as const;
export type NumberFormat = keyof typeof NUMBER_FORMATS;
export type CellFormats = Record<string, NumberFormat>;
const CURRENCY_SYMBOLS = { "$": "USD", "€": "EUR", "£": "GBP", "¥": "JPY" } as const;

/** Compact, diffable metadata: B2:USD B3:percent. No quotes or XML escaping. */
export function readFormats(text = ""): CellFormats {
  const formats: CellFormats = {};
  for (const entry of text.split(/\s+/)) {
    const [label, format] = entry.split(":");
    if (parseCellLabel(label ?? "") && Object.hasOwn(NUMBER_FORMATS, format)) {
      formats[label] = format as NumberFormat;
    }
  }
  return formats;
}
export function writeFormats(formats: CellFormats = {}): string {
  return Object.entries(formats).sort(([a], [b]) => a.localeCompare(b))
    .map(([label, format]) => `${label}:${format}`).join(" ");
}

/** Explicit symbols only. Ambiguous decimal-comma currency remains text. */
export function currencyInput(raw: string): { value: string; format: NumberFormat } | null {
  let text = raw.trim();
  const accounting = text.startsWith("(") && text.endsWith(")");
  if (accounting) text = text.slice(1, -1).trim();
  const number = "(\\d+(?:\\.\\d+)?|\\d{1,3}(?:,\\d{3})+(?:\\.\\d+)?)";
  const prefix = new RegExp(`^([+-]?)\\s*([$€£¥])\\s*([+-]?)\\s*${number}$`).exec(text);
  const suffix = new RegExp(`^([+-]?)\\s*${number}\\s*([$€£¥])$`).exec(text);
  const match = prefix ?? (suffix ? [suffix[0], suffix[1], suffix[3], "", suffix[2]] : null);
  if (!match) return null;
  const symbol = match[2];
  const digits = match[4];
  if (!digits || (match[1] && match[3]) || (accounting && (match[1] || match[3]))) return null;
  const value = `${accounting ? "-" : match[1] || match[3]}${digits.replace(/,/g, "")}`;
  if (!Number.isFinite(Number(value))) return null;
  return { value, format: CURRENCY_SYMBOLS[symbol as keyof typeof CURRENCY_SYMBOLS] };
}

export function displayNumber(raw: string, format?: NumberFormat): string {
  if (!format || format === "general" || !/^[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?$/i.test(raw.trim())) return raw;
  const value = Number(raw);
  if (!Number.isFinite(value)) return raw;
  const options: Intl.NumberFormatOptions = format === "number"
    ? { minimumFractionDigits: 2, maximumFractionDigits: 2 }
    : format === "percent" ? { style: "percent", maximumFractionDigits: 2 }
    : { style: "currency", currency: format };
  return new Intl.NumberFormat("en-US", options).format(value);
}

/** Formats follow cells through structural edits; removed cells lose theirs. */
export function moveFormats(formats: CellFormats | undefined, move: (row: number, column: number) => { row: number; column: number } | null): CellFormats {
  const next: CellFormats = {};
  for (const [label, format] of Object.entries(formats ?? {})) {
    const at = parseCellLabel(label);
    const target = at && move(at.row, at.column);
    if (target) next[cellLabel(target.column, target.row)] = format;
  }
  return next;
}

export function splitFormatComment(source: string): { body: string; formats: string } {
  const match = /^(?:<!-- hickory-table-formats: ([^\r\n]*?) -->\r?\n)/.exec(source);
  return match ? { body: source.slice(match[0].length), formats: match[1] } : { body: source, formats: "" };
}
export function withFormatComment(body: string, formats: string): string {
  return formats ? `<!-- hickory-table-formats: ${formats} -->${body.includes("\r\n") ? "\r\n" : "\n"}${body}` : body;
}
