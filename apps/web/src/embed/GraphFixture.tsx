import { useContext, useEffect, useMemo, useState } from "react";
import { GraphEditorPanel } from "../components/graph/GraphEditorPanel";
import { UiActions } from "../lib/uiActions";
import { UiActionContext, useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";

/** Real canvas acceptance surface; its scripted dispatch is not provider inference. */
export function GraphFixture() {
  const host = useContext(UiActionContext);
  const fallback = useMemo(() => new UiActions({ workspace: "graph-fixture", window: "graph-fixture" }, () => true, async () => {}), []);
  useEffect(() => () => fallback.dispose(), [fallback]);
  return <UiActionContext.Provider value={host ?? fallback}><GraphFixtureBody registry={host ?? fallback} /></UiActionContext.Provider>;
}
function GraphFixtureBody({ registry }: { registry: UiActions }) {
  const [source, setSource] = useState(JSON.stringify({ nodes: [{ id: "api", label: "API" }, { id: "db", label: "Store" }],
    edges: [{ from: "api", to: "db" }], layout: { api: { x: 0, y: 0 }, db: { x: 0, y: 160 } } }));
  const [receipt, setReceipt] = useState<unknown>(null);
  const revision = useUiRevision(() => [source, receipt]);
  useUiReading("graph acceptance source and receipt", { source, receipt });
  const invoke = useUiCommands([{ key: "graph-fixture:script", command: {
    label: "Agent move API", parent: "embedding-fixture", subject: "graph-fixture:script", parameters: {}, prerequisites: ["Deterministic fixture-agent action; no provider inference or human review"],
    read: () => ({ revision: revision(), enabled: true }), execute: async () => {
      const action = registry.groups().groups.flatMap(parent => registry.observe(parent, 64).actions).find(action => action.label === "Move graph node api");
      if (!action) return { status: "unavailable", reason: "Current graph action unavailable" };
      const result = await registry.act(registry.scope, { id: "fixture-agent", kind: "agent" }, { handle: action.handle, arguments: { x: 300, y: 96 }, request_id: crypto.randomUUID() });
      setReceipt(result);
      if (result.status !== "result") return { status: "unavailable", reason: result.reason };
      if (result.outcome.status === "unavailable" || result.outcome.status === "requires_input") return result.outcome;
      return { status: "completed", value: { scripted: true, provider_inference: false, result: JSON.parse(JSON.stringify(result)) } };
    },
  } }]);
  return <main style={{ width: 900, margin: 30 }}>
    <h1>Graph controller acceptance fixture</h1>
    <p>Real browser canvas and native gestures; scripted agent dispatch, no provider inference.</p>
    <GraphEditorPanel source={source} onCommit={setSource} />
    <button onClick={() => void invoke("graph-fixture:script").catch(reason => setReceipt({ error: String(reason) }))}>Agent move API</button>
    <pre aria-label="Graph action receipt">{receipt === null ? "" : JSON.stringify(receipt)}</pre>
    <pre aria-label="Graph document source">{source}</pre>
  </main>;
}
