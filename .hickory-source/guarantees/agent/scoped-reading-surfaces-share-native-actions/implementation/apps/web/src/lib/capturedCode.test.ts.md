<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/capturedCode.test.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-2a3bb47f729a"></a>
## `apps/web/src/lib/capturedCode.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Captured reading surfaces share owned native actions](../../../../../contract.md).

<a id="f-2a3bb47f729a-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-2a3bb47f729a-expression_statement">
import { afterEach, expect, it, vi } from "vitest";
import { request } from "../api/client";
import { retainCapturedContext } from "./capturedContexts";
import { openReading, publishReading } from "./readingViews";
import { openCapturedCode } from "./capturedCode";
vi.mock("../api/client", () => ({ request: vi.fn() }));
</storycode:copy>
<a id="f-2a3bb47f729a-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-2a3bb47f729a-expression_statement-2">
vi.mock("./capturedContexts", () => ({ retainCapturedContext: vi.fn() }));
</storycode:copy>
<a id="f-2a3bb47f729a-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-2a3bb47f729a-expression_statement-3">
vi.mock("./readingViews", () => ({ publishReading: vi.fn(), openReading: vi.fn() }));
</storycode:copy>
<a id="f-2a3bb47f729a-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-2a3bb47f729a-expression_statement-4">
afterEach(() => vi.clearAllMocks());
</storycode:copy>
<a id="f-2a3bb47f729a-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-2a3bb47f729a-expression_statement-5">
// Guarantee: docs/guarantees/agent/scoped-reading-surfaces-share-native-actions.md
it("releases late captured code instead of publishing/opening for a retired debugger stop", async () => {
  const release = vi.fn(); vi.mocked(retainCapturedContext).mockReturnValue(release);
  let resolve!: (value: unknown) => void;
  vi.mocked(request).mockImplementationOnce(() => new Promise(done => { resolve = done; }));
  let current = true;
  const target = { uri: "hick-project:///fixture/a.py", range: { start: { line: 1, character: 0 }, end: { line: 1, character: 0 } } };
  const loading = openCapturedCode(target, "owner", () => current);
  current = false; resolve({ path: "a.py", bytes: [120], files: [] });
  await expect(loading).rejects.toThrow("owner changed while loading");
  expect(release).toHaveBeenCalledTimes(1); expect(publishReading).not.toHaveBeenCalled(); expect(openReading).not.toHaveBeenCalled();
  await expect(openCapturedCode(target, "owner", () => false)).rejects.toThrow("before loading");
  expect(request).toHaveBeenCalledTimes(1); expect(retainCapturedContext).toHaveBeenCalledTimes(1);
  current = true; vi.mocked(request).mockResolvedValue({ path: "a.py", bytes: [120], files: [] });
  await openCapturedCode(target, "owner", () => current);
  expect(publishReading).toHaveBeenCalledWith(`captured:${target.uri}`, expect.objectContaining({ before: "x", after: "x", debugOwner: "owner", location: target.range.start, dispose: release }));
  expect(openReading).toHaveBeenCalledWith(`captured:${target.uri}`, "a.py");
});
</storycode:copy>

### Assemble `apps/web/src/lib/capturedCode.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/capturedCode.test.ts"><storycode:paste select="#f-2a3bb47f729a-expression_statement"/><storycode:paste select="#f-2a3bb47f729a-expression_statement-2"/><storycode:paste select="#f-2a3bb47f729a-expression_statement-3"/><storycode:paste select="#f-2a3bb47f729a-expression_statement-4"/><storycode:paste select="#f-2a3bb47f729a-expression_statement-5"/></storycode:file>

</storycode:doc>
