import { useMemo, useState } from "react";
import { GraphEditorPanel } from "../components/graph/GraphEditorPanel";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";

/** Real canvas acceptance surface; its agent dispatch is scripted and labelled. */
export function GraphFixture() {
  const registry = useMemo(() => new UiActions({ workspace: "graph-fixture", window: "graph-fixture" }, () => true, async () => {}), []);
  const [source, setSource] = useState(JSON.stringify({ nodes: [{ id: "api", label: "API" }, { id: "db", label: "Store" }],
    edges: [{ from: "api", to: "db" }], layout: { api: { x: 0, y: 0 }, db: { x: 0, y: 160 } } }));
  const [receipt, setReceipt] = useState<unknown>(null);
  return <UiActionContext.Provider value={registry}><main style={{ width: 900, margin: 30 }}>
    <h1>Graph controller acceptance fixture</h1>
    <p>Real browser canvas and native gestures; scripted agent dispatch, no provider inference.</p>
    <GraphEditorPanel source={source} onCommit={setSource} />
    <button onClick={() => {
      const action = registry.groups().groups.flatMap(parent => registry.observe(parent, 64).actions)
        .find(action => action.label === "Move graph node api");
      if (!action) { setReceipt({ error: "Current graph action unavailable" }); return; }
      void registry.act(registry.scope, { id: "fixture-agent", kind: "agent" }, {
        handle: action.handle, arguments: { x: 300, y: 96 }, request_id: crypto.randomUUID(),
      }).then(setReceipt);
    }}>Agent move API</button>
    <pre aria-label="Graph action receipt">{receipt === null ? "" : JSON.stringify(receipt)}</pre>
    <pre aria-label="Graph document source">{source}</pre>
  </main></UiActionContext.Provider>;
}
