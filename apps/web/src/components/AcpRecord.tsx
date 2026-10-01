import { ToolDetails } from "./AcpControls";
import type { AcpUpdate } from "../api/acp";

/** The same recorded ACP evidence is readable in the dock and session editor. */
export function AcpRecord({ kind, body }: { kind: string; body: string }) {
  let value: Record<string, unknown>;
  try { value = JSON.parse(body); } catch { return <details><summary>Agent activity</summary><pre>{body}</pre></details>; }
  if (kind === "acp-stream") return <details className="chat-step"><summary>Recorded {value.kind === "agent_thought_chunk" ? "reasoning" : "message"} fragment</summary><pre>{String(value.text ?? "")}</pre></details>;
  if (kind === "acp-activity") {
    const tool = value as AcpUpdate;
    if (["usage_update", "session_info_update", "available_commands_update", "config_option_update", "current_mode_update"].includes(tool.sessionUpdate ?? "")) return null;
    if (tool.sessionUpdate === "agent_message_chunk" || tool.sessionUpdate === "agent_thought_chunk") return <details className="chat-step"><summary>Recorded {tool.sessionUpdate === "agent_thought_chunk" ? "reasoning" : "message"} fragment</summary><pre>{JSON.stringify(value.content)}</pre></details>;
    if (tool.sessionUpdate === "plan") return <details className="chat-step"><summary>Agent plan</summary>{tool.entries?.map((e, i) => <p key={i}>{e.content} <span className="muted">{e.status}</span></p>)}</details>;
    return <details className="chat-step chat-step--tool"><summary>{tool.title ?? (tool.toolCallId ? "Tool update" : "Agent activity")} {tool.status && <span className="muted">· {tool.status}</span>}</summary><ToolDetails tool={tool} /><details><summary>Recorded details</summary><pre>{JSON.stringify(value, null, 2)}</pre></details></details>;
  }
  if (kind === "acp-turn-status") return <p className={`chat-step ${value.status === "error" ? "error" : "muted"}`}>{value.status === "ok" ? "Turn completed" : value.status === "stopped" ? "Stopped by you" : "Turn failed"}{typeof value.error === "string" && value.status !== "stopped" ? ` — ${value.error}` : ""}</p>;
  const caption: Record<string, string> = { "filesystem-access": "Workspace bytes accessed", "filesystem-write": "Workspace edit saved", "filesystem-refusal": "Workspace edit refused", "acp-session": "Agent session", "acp-permission-request": "Permission requested", "acp-permission-result": "Permission decision", "acp-file-write": "File written by agent" };
  return <details className="chat-step"><summary>{caption[kind] ?? "Agent evidence"}</summary><pre>{JSON.stringify(value, null, 2)}</pre></details>;
}
