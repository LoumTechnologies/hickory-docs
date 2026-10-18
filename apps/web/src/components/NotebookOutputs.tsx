import type { NotebookOutput } from "../lib/notebookOutput";
import { ReadOnlyTable } from "./ReadOnlyTable";
export function NotebookOutputs({ outputs }: { outputs: NotebookOutput[] }) {
  return <div className="notebook-outputs">{outputs.map((output, i) => output.kind === "table" ? <ReadOnlyTable key={i} table={output} /> : output.kind === "image" ? <figure key={i}><img alt="Python cell output" src={`data:${output.mime};base64,${output.data}`} /></figure> : <pre key={i}>{output.text}</pre>)}</div>;
}
