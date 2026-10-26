<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/InstallCommand.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9ae4608fde8d"></a>
## `apps/web/src/components/InstallCommand.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-9ae4608fde8d-InstallCommand"></a>
### `InstallCommand`

<storycode:copy id="f-9ae4608fde8d-InstallCommand">
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
</storycode:copy>

### Assemble `apps/web/src/components/InstallCommand.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/InstallCommand.tsx"><storycode:paste select="#f-9ae4608fde8d-InstallCommand"/></storycode:file>

</storycode:doc>
