<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/formatOnSave.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Save Can Format First, With The File's Own Formatter](../../../../../contract.md)

Captured SHA-256: `d4fc71528239d8acc05eb879a393847611a90e7ef65396bb7f2d0de5d6da2fb8`.

<book:entry path="apps/web/src/lib/formatOnSave.ts">
// Whether Save runs the formatter first.
//
// A per-user setting the server persists (ui.json, beside the window title)
// so the desktop app and `hick up` agree. Cached here because the moment it
// is read — a Save keystroke — is not a moment to wait on a request; the
// workspace loads it once at mount and the Settings page updates it on
// toggle.

import { api } from "../api/client";

let enabled = false;

/** Whether Save formats first. */
export function formatOnSave(): boolean {
  return enabled;
}

export function setFormatOnSave(on: boolean): void {
  enabled = on;
}

/** Read the setting from the server. A failure leaves it off, which is the
 * default, rather than surfacing as an error a Save could hit. */
export async function loadFormatOnSave(): Promise<void> {
  try {
    const ui = await api.settingsUi();
    enabled = ui.format_on_save === true;
  } catch {
    // Off: the honest default when nothing can be read.
  }
}
</book:entry>

</book:doc>
