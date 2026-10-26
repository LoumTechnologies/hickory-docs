<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/workspaceVisibility.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-4350d48f61e3"></a>
## `apps/web/src/lib/workspaceVisibility.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Settings preserves the open workspace](../../../../../guarantees/authoring/settings-preserves-the-open-workspace/contract.md).

<a id="f-4350d48f61e3-workspaceIsVisible"></a>
### `workspaceIsVisible`

A retained workspace must not answer keys while Settings occupies the window.

<storycode:copy id="f-4350d48f61e3-workspaceIsVisible">
/** A retained workspace must not answer keys while Settings occupies the window. */
export function workspaceIsVisible() {
  return !document.querySelector("[data-workspace][hidden]");
}
</storycode:copy>
<a id="f-4350d48f61e3-SETTINGS_CLOSED"></a>
### `SETTINGS_CLOSED`

<storycode:copy id="f-4350d48f61e3-SETTINGS_CLOSED">
export const SETTINGS_CLOSED = "hickory-settings-closed";
</storycode:copy>

### Assemble `apps/web/src/lib/workspaceVisibility.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/workspaceVisibility.ts"><storycode:paste select="#f-4350d48f61e3-workspaceIsVisible"/><storycode:paste select="#f-4350d48f61e3-SETTINGS_CLOSED"/></storycode:file>

</storycode:doc>
