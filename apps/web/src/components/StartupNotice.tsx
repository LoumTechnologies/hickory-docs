import { useSyncExternalStore } from "react";
import { startupNotice, subscribeStartup } from "../api/workspaceStartup";

export function StartupNotice() {
  const notice = useSyncExternalStore(subscribeStartup, startupNotice);
  return notice ? <div className="menu-notice" role="status">{notice}</div> : null;
}
