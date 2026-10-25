<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/embed/NativeCoachFixture.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `cd248618c4b9e1ea0da1e8cd3e99f51027812821dad8f819381c462bbdd300a0`.

<book:entry path="apps/web/src/embed/NativeCoachFixture.tsx">
// Focused layout host: actual native coach, with API responses supplied by the browser test.
import { NativeWorkflowTour } from "../components/NativeWorkflowTour";
import "../styles.css";
import "../components/WorkflowTour.css";
export function NativeCoachFixture() {
  return <main style={{ padding: 24 }}><h1>Native coach layout acceptance</h1><NativeWorkflowTour /><p>No native engine, model call or target execution is implied by this fixture.</p></main>;
}
</book:entry>

</book:doc>
