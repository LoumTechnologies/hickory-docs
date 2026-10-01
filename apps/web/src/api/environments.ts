import { MOCK, request } from "./client";
import type { TerminalSession } from "./types";

export interface EnvironmentAction {
  id: string;
  label: string;
  argv: string[];
  effects: string;
}
export interface EnvironmentFinding {
  project: string;
  manager: string;
  target: string;
  profile: string;
  revision: string;
  state: "manager-missing" | "environment-missing" | "environment-stale" | "lock-missing" | "lock-stale" | "ambiguous" | "unknown" | "ready";
  message: string;
  details: string;
  actions: EnvironmentAction[];
  install_url: string | null;
  interpreter: string | null;
  manager_choices: string[];
}
export interface EnvironmentStatus {
  findings: EnvironmentFinding[];
  running: Record<string, string>;
}
export const environmentApi = {
  inspect: (refresh = false): Promise<EnvironmentStatus> => MOCK
    ? Promise.resolve({ findings: [], running: {} })
    : request("GET", `/api/environments${refresh ? "?refresh=true" : ""}`),
  act: (finding: EnvironmentFinding, action: string) => request<TerminalSession>("POST", "/api/environments/actions", {
    project: finding.project, manager: finding.manager, revision: finding.revision, action,
  }),
  choose: (finding: EnvironmentFinding, choice: string) => request("PUT", "/api/environments/manager", {
    project: finding.project, manager: finding.manager, revision: finding.revision, choice,
  }),
};
