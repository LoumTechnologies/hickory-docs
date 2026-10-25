<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/embed/snapshot.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A viewer snapshot keeps recorded results historical](../../../../../../guarantees/embedding/a-viewer-snapshot-keeps-recorded-results-historical/contract.md)
- [Published executable results identify reviewed inputs](../../../../../../guarantees/embedding/publication-is-bound-to-reviewed-inputs/contract.md)

Captured SHA-256: `8e7a0a0063d1b8317509cb7a446dac791077766328513b6d3246da849b028674`.

<book:entry path="apps/web/src/embed/snapshot.tsx">
import { createRoot } from "react-dom/client";
import { SnapshotPage } from "./SnapshotPage";
import { StaticUiSurface } from "./StaticUiSurface";
import "../styles.css";
import "./snapshot.css";
createRoot(document.getElementById("root")!).render(<StaticUiSurface><SnapshotPage /></StaticUiSurface>);
</book:entry>

</book:doc>
