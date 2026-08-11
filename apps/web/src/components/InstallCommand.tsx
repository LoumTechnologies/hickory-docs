import { useState } from "react";
import { emit } from "../analytics/events";
import { config } from "../config";

/**
 * The primary call to action for a product you install.
 *
 * Shown as a command rather than a download button on purpose: the audience is
 * people who live in a terminal, the command is the same on every platform, and
 * a copyable line is honest about what it does — nobody has to trust a binary
 * they did not watch arrive.
 *
 * Clicking copies and records `cta_clicked` with `cta_id: "install"`, which is
 * the funnel's new conversion event now that there is no signup to count.
 */
export function InstallCommand() {
  const [copied, setCopied] = useState(false);

  const copy = async () => {
    emit({ name: "cta_clicked", cta_id: "install" });
    try {
      await navigator.clipboard.writeText(config.installCommand);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 2000);
    } catch {
      // Clipboard access can be refused (permissions, insecure context, an
      // older browser). The command is visible and selectable either way, so
      // the page still works — it just cannot congratulate itself.
    }
  };

  return (
    <div className="install-command">
      <code>{config.installCommand}</code>
      <button className="btn btn-sm" onClick={() => void copy()} aria-label="Copy install command">
        {copied ? "Copied" : "Copy"}
      </button>
    </div>
  );
}
