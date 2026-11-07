<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Build the small browser frontend independently

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Reuse the existing development proxy and test configuration; change only the production entry and output directory.

<ui:copy id="simple-de070d2bb6-0">
import { defineConfig, mergeConfig } from "vitest/config";
import base from "./vite.config";
export default mergeConfig(base, defineConfig({
  build: { outDir: "dist-simple", rollupOptions: { input: "simple.html" } },
}));
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/web/vite.simple.config.ts"><ui:paste select="#simple-de070d2bb6-0"/></ui:file>

</ui:doc>
