import { useId } from "react";
import type { LspLocation } from "../lsp/client";
import { openCapturedCode } from "../lib/capturedCode";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand } from "../lib/uiActions";

export function CapturedReferences({ targets, scope, owner, current, onClose, onError }: {
  targets: LspLocation[]; scope: string; owner: string; current: () => boolean;
  onClose: () => void; onError: (error: string) => void;
}) {
  const parent = `captured-references:${useId()}`;
  const revision = useUiRevision(() => [targets, scope, owner, current()]);
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, label: string, parameters: UiCommand["parameters"], execute: UiCommand["execute"]) => commands.push({ key: `${parent}:${key}`, command: { parent, subject: parent, label, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled: current(), reason: current() ? undefined : "Captured reference source changed or closed" }), execute } });
  add("read", "Read captured references bytes", { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, async args => {
    const bytes = new TextEncoder().encode(JSON.stringify({ owner, targets, scope: "Captured LSP references; navigation opens the exact captured source, not a working-file edit baseline" }));
    const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
    return offset > bytes.length ? { status: "unavailable", reason: "Offset outside current references" } : { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
  });
  add("pick", "Open captured reference", { index: { type: "number", minimum: 0, maximum: 1000000, integer: true } }, async args => {
    const target = targets[args.index as number];
    if (!target || !current()) return { status: "unavailable", reason: "Observe a reference in the current captured source first" };
    try { await openCapturedCode(target, owner); if (current()) onClose(); return { status: "completed", value: { captured_navigation_requested: true, uri: target.uri, line: target.range.start.line, source_unit: "zero-based LSP line; captured source" } }; }
    catch (e) { const reason = String(e); onError(reason); return { status: "unavailable", reason }; }
  });
  add("close", "Close captured references", {}, async () => { onClose(); return { status: "completed", value: { closed: true } }; });
  const invoke = useUiCommands(commands);
  const human = (key: string, args: Parameters<typeof invoke>[1] = {}) => void invoke(`${parent}:${key}`, args).catch(() => undefined);
  return <div className="code-references" role="dialog" aria-label="References" onKeyDown={event => { if (event.key === "Escape") human("close"); }}>
    <button type="button" aria-label="Close references" onClick={() => human("close")}>×</button>
    {targets.length ? targets.map((target, index) => <button type="button" key={index} onClick={() => human("pick", { index })}>{decodeURIComponent(target.uri.split("/").slice(4).join("/"))}:{target.range.start.line + 1}</button>) : <p>No references found.</p>}
  </div>;
}
