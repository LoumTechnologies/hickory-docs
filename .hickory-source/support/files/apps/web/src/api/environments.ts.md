<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/api/environments.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `db5b667e69d81be87561b88059855679aa7deade99e355b6753d455873410a8d`.

<book:entry path="apps/web/src/api/environments.ts">
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
</book:entry>

</book:doc>
