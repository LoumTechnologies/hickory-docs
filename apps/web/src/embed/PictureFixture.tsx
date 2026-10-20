import { useMemo, useState } from "react";
import { PicturePanel } from "../components/PicturePanel";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
const PNG = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4z8DwHwAFAAH/iZk9HQAAAABJRU5ErkJggg==";
/** Browser acceptance fixture, not a provider call or a launch recording. */
export function PictureFixture() {
  const registry = useMemo(() => new UiActions({ workspace: "picture-fixture", window: "picture-fixture" }, () => true, async () => {}), []);
  const [receipt, setReceipt] = useState<unknown>(null);
  return <UiActionContext.Provider value={registry}><main>
    <h1>Image capture acceptance fixture</h1>
    <p>Real browser raster capture; scripted agent dispatch, no provider inference.</p>
    <PicturePanel src={`data:image/png;base64,${PNG}`} path="red-pixel.png" />
    <button onClick={() => {
      const action = registry.groups().groups.flatMap(parent => registry.observe(parent).actions).find(action => action.label === "Read displayed image red-pixel.png");
      if (!action) { setReceipt({ error: "Image control unavailable" }); return; }
      void registry.act(registry.scope, { id: "fixture-agent", kind: "agent" }, { handle: action.handle, arguments: { max_dimension: 2048 }, request_id: crypto.randomUUID() }).then(setReceipt);
    }}>Read fixture image as agent</button>
    <pre aria-label="Image capture receipt">{receipt === null ? "" : JSON.stringify(receipt, null, 2)}</pre>
  </main></UiActionContext.Provider>;
}
