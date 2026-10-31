<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/api/acp.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-ee3a13259742"></a>
## `apps/web/src/api/acp.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-ee3a13259742-AgentCommand"></a>
### `AgentCommand`

<storycode:copy id="f-ee3a13259742-AgentCommand">
import { request } from "./client";

export interface AgentCommand { id: string; name: string; command: string; args: string[]; workspace_filesystem?: boolean; private_prediction_args?: string[] | null; private_prediction_auth_method?: string | null; available?: boolean; installable?: boolean; cli_available?: boolean; kind?: "python"; error?: string }
</storycode:copy>
<a id="f-ee3a13259742-ConfigValue"></a>
### `ConfigValue`

<storycode:copy id="f-ee3a13259742-ConfigValue">
export interface ConfigValue { value: string; name: string; description?: string }
</storycode:copy>
<a id="f-ee3a13259742-ConfigOption"></a>
### `ConfigOption`

<storycode:copy id="f-ee3a13259742-ConfigOption">
export interface ConfigOption {
  id: string; name: string; description?: string; category?: string; type: "select" | "boolean";
  currentValue: string | boolean;
  options?: (ConfigValue | { group: string; name: string; options: ConfigValue[] })[];
}
</storycode:copy>
<a id="f-ee3a13259742-AcpUpdate"></a>
### `AcpUpdate`

<storycode:copy id="f-ee3a13259742-AcpUpdate">
export interface AcpUpdate {
  sessionUpdate?: string; toolCallId?: string; title?: string; status?: string; kind?: string;
  locations?: { path: string; line?: number }[];
  _meta?: { terminal_output_delta?: { data?: string } };
  rawInput?: unknown; rawOutput?: unknown;
  content?: { type: string; content?: { type: string; text?: string }; path?: string; oldText?: string; newText?: string }[];
  entries?: { content: string; priority?: string; status: string }[];
}
</storycode:copy>
<a id="f-ee3a13259742-AcpPermission"></a>
### `AcpPermission`

<storycode:copy id="f-ee3a13259742-AcpPermission">
export interface AcpPermission {
  id: string; toolCall: AcpUpdate;
  options: { optionId: string; name: string; kind: string }[];
}
</storycode:copy>
<a id="f-ee3a13259742-AgentChange"></a>
### `AgentChange`

<storycode:copy id="f-ee3a13259742-AgentChange">
export interface AgentChange {
  id: string; name: string; buffer?: string | null; path: string | null; oldText: string; newText: string;
  editor: boolean; status: "pending" | "applying" | "accepted" | "rejected" | "applied" | "failed";
}
</storycode:copy>
<a id="f-ee3a13259742-AcpState"></a>
### `AcpState`

<storycode:copy id="f-ee3a13259742-AcpState">
export interface AcpState {
  edits?: { captureRequest?: string | null; mode: "review" | "auto-accept"; changes: AgentChange[] };
  backend: string; ready: boolean; canRewind?: boolean; session?: string; error?: string | null;
  authMethods?: { id: string; name: string; description?: string; type?: string }[];
  configOptions?: ConfigOption[];
  models?: { currentModelId: string };
  modes?: { currentModeId: string; availableModes: { id: string; name: string }[] };
  commands?: { name: string; description: string }[];
  permissions?: AcpPermission[]; tools?: AcpUpdate[];
}

</storycode:copy>
<a id="f-ee3a13259742-path"></a>
### `path`

<storycode:copy id="f-ee3a13259742-path">
const path = (doc: string) => `/api/docs/${encodeURIComponent(doc)}/agent/acp`;
</storycode:copy>
<a id="f-ee3a13259742-acpApi"></a>
### `acpApi`

<storycode:copy id="f-ee3a13259742-acpApi">
export const acpApi = {
  pythonSettings: () => request<{ python: string; available: boolean; bundled?: string | null }>("GET", "/api/agents/python"),
  savePython: (python: string) => request<{ python: string; available: boolean; bundled?: string | null }>("PUT", "/api/agents/python", { python }),
  catalogue: () => request<{ agents: AgentCommand[]; workspace_filesystem?: { supported: boolean; bundled: boolean; message: string } }>("GET", "/api/agents"),
  save: (agents: AgentCommand[]) => request<{ agents: AgentCommand[] }>("PUT", "/api/agents", agents),
  install: (agent: string) => request<{ agents: AgentCommand[] }>("POST", `/api/agents/${encodeURIComponent(agent)}/install`),
  connect: (doc: string, backend: string, session?: string) => request<AcpState>("POST", path(doc), { backend, session }),
  state: (doc: string, backend?: string, session?: string) => request<AcpState>("GET", backend === "python" ? `/api/docs/${encodeURIComponent(doc)}/agent/python${session === undefined ? "" : `?session=${encodeURIComponent(session)}`}` : path(doc)),
  authenticate: (doc: string, method_id: string) => request<AcpState>("POST", `${path(doc)}/authenticate`, { method_id }),
  configure: (doc: string, config_id: string, value: string | boolean) => request<AcpState>("POST", `${path(doc)}/configure`, { config_id, value }),
  edits: (doc: string, body: { capture_request?: string; context?: import("./agentTypes").AgentEditorContext; mode?: "review" | "auto-accept"; id?: string; accepted?: boolean; error?: string; current_text?: string; new_text?: string }, backend?: string) => request<AcpState>("POST", backend === "python" ? `/api/docs/${encodeURIComponent(doc)}/agent/python/review` : `${path(doc)}/edits`, body),
  permission: (doc: string, request_id: string, option_id: string) => request<{ answered: boolean }>("POST", `${path(doc)}/permission`, { request_id, option_id }),
};
</storycode:copy>

### Assemble `apps/web/src/api/acp.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/api/acp.ts"><storycode:paste select="#f-ee3a13259742-AgentCommand"/><storycode:paste select="#f-ee3a13259742-ConfigValue"/><storycode:paste select="#f-ee3a13259742-ConfigOption"/><storycode:paste select="#f-ee3a13259742-AcpUpdate"/><storycode:paste select="#f-ee3a13259742-AcpPermission"/><storycode:paste select="#f-ee3a13259742-AgentChange"/><storycode:paste select="#f-ee3a13259742-AcpState"/><storycode:paste select="#f-ee3a13259742-path"/><storycode:paste select="#f-ee3a13259742-acpApi"/></storycode:file>

</storycode:doc>
