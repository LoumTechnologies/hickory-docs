<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/api/environments.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f737fd0d315c"></a>
## `apps/web/src/api/environments.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-f737fd0d315c-EnvironmentAction"></a>
### `EnvironmentAction`

<storycode:copy id="f-f737fd0d315c-EnvironmentAction">
import { MOCK, request } from "./client";
import type { TerminalSession } from "./types";

export interface EnvironmentAction {
  id: string;
  label: string;
  argv: string[];
  effects: string;
}
</storycode:copy>
<a id="f-f737fd0d315c-EnvironmentFinding"></a>
### `EnvironmentFinding`

<storycode:copy id="f-f737fd0d315c-EnvironmentFinding">
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
</storycode:copy>
<a id="f-f737fd0d315c-EnvironmentStatus"></a>
### `EnvironmentStatus`

<storycode:copy id="f-f737fd0d315c-EnvironmentStatus">
export interface EnvironmentStatus {
  findings: EnvironmentFinding[];
  running: Record<string, string>;
}
</storycode:copy>
<a id="f-f737fd0d315c-environmentApi"></a>
### `environmentApi`

<storycode:copy id="f-f737fd0d315c-environmentApi">
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
</storycode:copy>

### Assemble `apps/web/src/api/environments.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/api/environments.ts"><storycode:paste select="#f-f737fd0d315c-EnvironmentAction"/><storycode:paste select="#f-f737fd0d315c-EnvironmentFinding"/><storycode:paste select="#f-f737fd0d315c-EnvironmentStatus"/><storycode:paste select="#f-f737fd0d315c-environmentApi"/></storycode:file>

</storycode:doc>
