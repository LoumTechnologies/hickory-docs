<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/workspaceVisibility.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Settings preserves the open workspace](../../../../../../guarantees/authoring/settings-preserves-the-open-workspace/contract.md)

Captured SHA-256: `2c5c456f9359d2020432fbc453e72836a7219232af78eaafef4aa93046720b33`.

<book:entry path="apps/web/src/lib/workspaceVisibility.ts">
/** A retained workspace must not answer keys while Settings occupies the window. */
export function workspaceIsVisible() {
  return !document.querySelector("[data-workspace][hidden]");
}
export const SETTINGS_CLOSED = "hickory-settings-closed";
</book:entry>

</book:doc>
