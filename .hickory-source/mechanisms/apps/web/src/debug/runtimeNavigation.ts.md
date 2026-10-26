<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/runtimeNavigation.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-60656abb683c"></a>
## `apps/web/src/debug/runtimeNavigation.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-60656abb683c-RUNTIME_REVEAL"></a>
### `RUNTIME_REVEAL`

<storycode:copy id="f-60656abb683c-RUNTIME_REVEAL">
export const RUNTIME_REVEAL = "hickory:runtime-reveal";
</storycode:copy>
<a id="f-60656abb683c-revealRuntimeEvent"></a>
### `revealRuntimeEvent`

<storycode:copy id="f-60656abb683c-revealRuntimeEvent">
export function revealRuntimeEvent(run: string, event: string) {
  window.dispatchEvent(new CustomEvent(RUNTIME_REVEAL, { detail: { run, event } }));
}
</storycode:copy>

### Assemble `apps/web/src/debug/runtimeNavigation.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/runtimeNavigation.ts"><storycode:paste select="#f-60656abb683c-RUNTIME_REVEAL"/><storycode:paste select="#f-60656abb683c-revealRuntimeEvent"/></storycode:file>

</storycode:doc>
