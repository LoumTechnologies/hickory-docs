import { request } from "./client";

export interface AgentCommand { id: string; name: string; command: string; args: string[]; workspace_filesystem?: boolean; available?: boolean; installable?: boolean; cli_available?: boolean }
export interface ConfigValue { value: string; name: string; description?: string }
export interface ConfigOption {
  id: string; name: string; description?: string; category?: string; type: "select" | "boolean";
  currentValue: string | boolean;
  options?: (ConfigValue | { group: string; name: string; options: ConfigValue[] })[];
}
export interface AcpUpdate {
  sessionUpdate?: string; toolCallId?: string; title?: string; status?: string; kind?: string;
  locations?: { path: string; line?: number }[];
  _meta?: { terminal_output_delta?: { data?: string } };
  rawInput?: unknown; rawOutput?: unknown;
  content?: { type: string; content?: { type: string; text?: string }; path?: string; oldText?: string; newText?: string }[];
  entries?: { content: string; priority?: string; status: string }[];
}
export interface AcpPermission {
  id: string; toolCall: AcpUpdate;
  options: { optionId: string; name: string; kind: string }[];
}
export interface AgentChange {
  id: string; name: string; buffer?: string | null; path: string | null; oldText: string; newText: string;
  editor: boolean; status: "pending" | "applying" | "accepted" | "rejected" | "applied" | "failed";
}
export interface AcpState {
  edits?: { mode: "review" | "auto-accept"; changes: AgentChange[] };
  backend: string; ready: boolean; canRewind?: boolean; session?: string; error?: string | null;
  authMethods?: { id: string; name: string; description?: string; type?: string }[];
  configOptions?: ConfigOption[];
  modes?: { currentModeId: string; availableModes: { id: string; name: string }[] };
  commands?: { name: string; description: string }[];
  permissions?: AcpPermission[]; tools?: AcpUpdate[];
}

const path = (doc: string) => `/api/docs/${encodeURIComponent(doc)}/agent/acp`;
export const acpApi = {
  catalogue: () => request<{ agents: AgentCommand[]; workspace_filesystem?: { supported: boolean; bundled: boolean; message: string } }>("GET", "/api/agents"),
  save: (agents: AgentCommand[]) => request<{ agents: AgentCommand[] }>("PUT", "/api/agents", agents),
  install: (agent: string) => request<{ agents: AgentCommand[] }>("POST", `/api/agents/${encodeURIComponent(agent)}/install`),
  connect: (doc: string, backend: string, session?: string) => request<AcpState>("POST", path(doc), { backend, session }),
  state: (doc: string) => request<AcpState>("GET", path(doc)),
  authenticate: (doc: string, method_id: string) => request<AcpState>("POST", `${path(doc)}/authenticate`, { method_id }),
  configure: (doc: string, config_id: string, value: string | boolean) => request<AcpState>("POST", `${path(doc)}/configure`, { config_id, value }),
  edits: (doc: string, body: { mode?: "review" | "auto-accept"; id?: string; accepted?: boolean; error?: string; current_text?: string }) => request<AcpState>("POST", `${path(doc)}/edits`, body),
  permission: (doc: string, request_id: string, option_id: string) => request<{ answered: boolean }>("POST", `${path(doc)}/permission`, { request_id, option_id }),
};
