<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/AcpRecord.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1861640bb163"></a>
## `apps/web/src/components/AcpRecord.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [ACP Agents Work In The Agent Pane And Leave A Hickory Record](../../../../../guarantees/agent/acp-agents-are-first-class/contract.md).

<a id="f-1861640bb163-AcpRecord"></a>
### `AcpRecord`

<storycode:copy id="f-1861640bb163-AcpRecord">
import { ToolDetails } from "./AcpControls";
import { EvidenceDisclosure } from "./EvidenceDisclosure";
import type { AcpUpdate } from "../api/acp";

/** The same recorded ACP evidence is readable in the dock and session editor. */
export function AcpRecord({ kind, body }: { kind: string; body: string }) {
  let value: Record<string, unknown>;
  try {
    const parsed: unknown = JSON.parse(body);
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) throw Error("Not a record");
    value = parsed as Record<string, unknown>;
  } catch { return <EvidenceDisclosure label="Agent activity" value={body}><pre>{body}</pre></EvidenceDisclosure>; }
  if (kind === "acp-stream") return <EvidenceDisclosure className="chat-step" label={`Recorded ${value.kind === "agent_thought_chunk" ? "reasoning" : "message"} fragment`} value={value}><pre>{String(value.text ?? "")}</pre></EvidenceDisclosure>;
  if (kind === "acp-activity") {
    const tool = value as AcpUpdate;
    if (["usage_update", "session_info_update", "available_commands_update", "config_option_update", "current_mode_update"].includes(tool.sessionUpdate ?? "")) return null;
    if (tool.sessionUpdate === "agent_message_chunk" || tool.sessionUpdate === "agent_thought_chunk") return <EvidenceDisclosure className="chat-step" label={`Recorded ${tool.sessionUpdate === "agent_thought_chunk" ? "reasoning" : "message"} fragment`} value={value}><pre>{JSON.stringify(value.content)}</pre></EvidenceDisclosure>;
    if (tool.sessionUpdate === "plan") return <EvidenceDisclosure className="chat-step" label="Agent plan" value={value}>{tool.entries?.map((e, i) => <p key={i}>{e.content} <span className="muted">{e.status}</span></p>)}</EvidenceDisclosure>;
    return <EvidenceDisclosure className="chat-step chat-step--tool" label={`Adapter report · ${tool.title ?? (tool.toolCallId ? "Tool update" : "Agent activity")}${tool.status ? ` · ${tool.status}` : ""}`} value={tool}>
      <p className="muted">Reported by the adapter; this is not an independent byte or execution check.</p><ToolDetails tool={tool} />
      <EvidenceDisclosure label="Recorded details" value={value}><pre>{JSON.stringify(value, null, 2)}</pre></EvidenceDisclosure>
    </EvidenceDisclosure>;
  }
  if (kind === "acp-turn-status") return <RecordedTurnStatus value={value} />;
  const caption: Record<string, string> = { "filesystem-access": "Workspace bytes accessed", "filesystem-write": "Workspace edit saved", "filesystem-refusal": "Workspace edit refused", "acp-session": "Agent session", "acp-permission-request": "Permission requested", "acp-permission-result": "Permission decision", "acp-file-write": "File written by agent" };
  return <EvidenceDisclosure className="chat-step" label={caption[kind] ?? "Agent evidence"} value={value}><pre>{JSON.stringify(value, null, 2)}</pre></EvidenceDisclosure>;
}

</storycode:copy>
<a id="f-1861640bb163-RecordedTurnStatus"></a>
### `RecordedTurnStatus`

<storycode:copy id="f-1861640bb163-RecordedTurnStatus">
function RecordedTurnStatus({ value }: { value: Record<string, unknown> }) {
  const text = value.status === "ok" ? "Turn completed" : value.status === "stopped" ? "Turn stopped" : "Turn failed";
  return <EvidenceDisclosure className={`chat-step ${value.status === "error" ? "error" : "muted"}`} label={text} value={value}>{typeof value.error === "string" && value.status !== "stopped" && <p>{value.error}</p>}<p className="muted">Recorded adapter status; inspect its retained origin and scope.</p></EvidenceDisclosure>;
}
</storycode:copy>

### Assemble `apps/web/src/components/AcpRecord.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/AcpRecord.tsx"><storycode:paste select="#f-1861640bb163-AcpRecord"/><storycode:paste select="#f-1861640bb163-RecordedTurnStatus"/></storycode:file>

</storycode:doc>
