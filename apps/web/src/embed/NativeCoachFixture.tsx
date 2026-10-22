// Focused layout host: actual native coach, with API responses supplied by the browser test.
import { NativeWorkflowTour } from "../components/NativeWorkflowTour";
import "../styles.css";
import "../components/WorkflowTour.css";
export function NativeCoachFixture() {
  return <main style={{ padding: 24 }}><h1>Native coach layout acceptance</h1><NativeWorkflowTour /><p>No native engine, model call or target execution is implied by this fixture.</p></main>;
}
