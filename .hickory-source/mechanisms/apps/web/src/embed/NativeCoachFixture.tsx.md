<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/embed/NativeCoachFixture.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-770a8ed91c58"></a>
## `apps/web/src/embed/NativeCoachFixture.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-770a8ed91c58-NativeCoachFixture"></a>
### `NativeCoachFixture`

Focused layout host: actual native coach, with API responses supplied by the browser test.

<storycode:copy id="f-770a8ed91c58-NativeCoachFixture">
// Focused layout host: actual native coach, with API responses supplied by the browser test.
import { NativeWorkflowTour } from "../components/NativeWorkflowTour";
import "../styles.css";
import "../components/WorkflowTour.css";
export function NativeCoachFixture() {
  return <main style={{ padding: 24 }}><h1>Native coach layout acceptance</h1><NativeWorkflowTour /><p>No native engine, model call or target execution is implied by this fixture.</p></main>;
}
</storycode:copy>

### Assemble `apps/web/src/embed/NativeCoachFixture.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/NativeCoachFixture.tsx"><storycode:paste select="#f-770a8ed91c58-NativeCoachFixture"/></storycode:file>

</storycode:doc>
