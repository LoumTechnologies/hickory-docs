<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/NotebookOutputs.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `cb3d08357a13225dec407884ca800e7b7133e73add801b6b7802a2e83fa56cd2`.

<book:entry path="apps/web/src/components/NotebookOutputs.tsx">
import type { NotebookOutput } from "../lib/notebookOutput";
import { ReadOnlyTable } from "./ReadOnlyTable";
export function NotebookOutputs({ outputs }: { outputs: NotebookOutput[] }) {
  return <div className="notebook-outputs">{outputs.map((output, i) => output.kind === "table" ? <ReadOnlyTable key={i} table={output} /> : output.kind === "image" ? <figure key={i}><img alt="Python cell output" src={`data:${output.mime};base64,${output.data}`} /></figure> : <pre key={i}>{output.text}</pre>)}</div>;
}
</book:entry>

</book:doc>
