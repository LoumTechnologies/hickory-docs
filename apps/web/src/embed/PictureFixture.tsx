import { useContext, useEffect, useMemo, useState } from "react";
import { PicturePanel } from "../components/PicturePanel";
import { UiActions } from "../lib/uiActions";
import { UiActionContext, useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
const PNG = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4z8DwHwAFAAH/iZk9HQAAAABJRU5ErkJggg==";
/** Browser acceptance fixture, not a provider call or a launch recording. */
export function PictureFixture() {
  const host = useContext(UiActionContext);
  const fallback = useMemo(() => new UiActions({ workspace: "picture-fixture", window: "picture-fixture" }, () => true, async () => {}), []);
  useEffect(() => () => fallback.dispose(), [fallback]);
  return <UiActionContext.Provider value={host ?? fallback}><PictureFixtureBody registry={host ?? fallback} /></UiActionContext.Provider>;
}
function PictureFixtureBody({ registry }: { registry: UiActions }) {
  const [receipt, setReceipt] = useState<unknown>(null);
  const revision = useUiRevision(() => receipt);
  useUiReading("image capture acceptance receipt", receipt);
  const invoke = useUiCommands([{ key: "picture-fixture:script", command: {
    label: "Read fixture image as agent", parent: "embedding-fixture", subject: "picture-fixture:script", parameters: {}, prerequisites: ["Deterministic fixture-agent action; no provider inference"],
    read: () => ({ revision: revision(), enabled: true }), execute: async () => {
      const action = registry.groups().groups.flatMap(parent => registry.observe(parent, 64).actions).find(action => action.label === "Read displayed image red-pixel.png");
      if (!action) return { status: "unavailable", reason: "Image control unavailable" };
      const result = await registry.act(registry.scope, { id: "fixture-agent", kind: "agent" }, { handle: action.handle, arguments: { max_dimension: 2048 }, request_id: crypto.randomUUID() });
      setReceipt(result);
      if (result.status !== "result") return { status: "unavailable", reason: result.reason };
      if (result.outcome.status === "unavailable" || result.outcome.status === "requires_input") return result.outcome;
      return { status: "completed", value: { scripted: true, provider_inference: false, result: JSON.parse(JSON.stringify(result)) } };
    },
  } }]);
  return <main>
    <h1>Image capture acceptance fixture</h1>
    <p>Real browser raster capture; scripted agent dispatch, no provider inference.</p>
    <PicturePanel src={`data:image/png;base64,${PNG}`} path="red-pixel.png" />
    <button onClick={() => void invoke("picture-fixture:script").catch(reason => setReceipt({ error: String(reason) }))}>Read fixture image as agent</button>
    <pre aria-label="Image capture receipt">{receipt === null ? "" : JSON.stringify(receipt, null, 2)}</pre>
  </main>;
}
