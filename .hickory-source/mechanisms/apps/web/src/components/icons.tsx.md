<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/icons.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-fcdfd36508a0"></a>
## `apps/web/src/components/icons.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A running agent can be stopped](../../../../../guarantees/agent/a-running-agent-can-be-stopped/contract.md).

<a id="f-fcdfd36508a0-StopMark"></a>
### `StopMark`

Minimal inline glyphs. No icon library and no image assets exist in this
app — everything themes via `currentColor`, matching that convention.
The stop square — the universal "halt what is running" glyph.

<storycode:copy id="f-fcdfd36508a0-StopMark">
// Minimal inline glyphs. No icon library and no image assets exist in this
// app — everything themes via `currentColor`, matching that convention.

/** The stop square — the universal "halt what is running" glyph. */
export function StopMark({ size = 14 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="currentColor"
      stroke="none"
      aria-hidden="true"
    >
      <rect x="5" y="5" width="14" height="14" rx="2" />
    </svg>
  );
}

</storycode:copy>
<a id="f-fcdfd36508a0-TreeMark"></a>
### `TreeMark`

<storycode:copy id="f-fcdfd36508a0-TreeMark">
export function TreeMark({ size = 18 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.6"
      strokeLinecap="round"
      aria-hidden="true"
    >
      <path d="M12 21v-7" />
      <path d="M12 14l-4-4M12 14l4-4M12 10l-3-3M12 10l3-3M12 7V3" />
    </svg>
  );
}
</storycode:copy>

### Assemble `apps/web/src/components/icons.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/icons.tsx"><storycode:paste select="#f-fcdfd36508a0-StopMark"/><storycode:paste select="#f-fcdfd36508a0-TreeMark"/></storycode:file>

</storycode:doc>
