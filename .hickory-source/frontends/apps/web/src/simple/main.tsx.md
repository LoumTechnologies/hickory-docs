<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Start the small workspace

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Mount only the new shell. Its CSS is independent of the old editor themes and toolbars.

<ui:copy id="simple-aff233fffa-0">
import { createRoot } from "react-dom/client";
import { App } from "./App";
import "./simple.css";
createRoot(document.getElementById("root")!).render(<App />);
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/web/src/simple/main.tsx"><ui:paste select="#simple-aff233fffa-0"/></ui:file>

</ui:doc>
