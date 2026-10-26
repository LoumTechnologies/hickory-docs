<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/embed/AssetPreviewImage.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d6a4681e6566"></a>
## `apps/web/src/embed/AssetPreviewImage.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-d6a4681e6566-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-d6a4681e6566-expression_statement">
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { AssetPreviewImage } from "./AssetPreviewImage";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
</storycode:copy>
<a id="f-d6a4681e6566-fixtureBlob"></a>
### `fixtureBlob`

<storycode:copy id="f-d6a4681e6566-fixtureBlob">
const fixtureBlob = () => ({ size: 3, arrayBuffer: async () => new Uint8Array([1, 2, 3]).buffer }) as Blob;
</storycode:copy>
<a id="f-d6a4681e6566-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-d6a4681e6566-expression_statement-2">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("captures explicit host image pixels and refuses a late replacement owner without attributing its error", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const url = "https://example.test/image.png";
  const ui = (path: string) => <UiActionContext.Provider value={registry}><AssetPreviewImage path={path} url={url} /></UiActionContext.Provider>;
  const rendered = render(ui("first.png"));
  const img = screen.getByRole("img") as HTMLImageElement;
  for (const [key, value] of Object.entries({ complete: true, currentSrc: url, naturalWidth: 40, naturalHeight: 20 })) Object.defineProperty(img, key, { configurable: true, value });
  fireEvent.load(img);
  const draw = vi.fn(); vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue({ drawImage: draw } as unknown as CanvasRenderingContext2D);
  let encode!: BlobCallback;
  vi.spyOn(HTMLCanvasElement.prototype, "toBlob").mockImplementation(callback => { encode = callback; });
  const action = () => registry.groups().groups.flatMap(group => registry.observe(group).actions).find(action => ["Read embedded image first.png", "Read embedded image second.png", "Read embedded image protected.png"].includes(action.label))!;
  const actor = { id: "agent", kind: "agent" as const };
  let pending!: ReturnType<typeof registry.act>;
  await act(async () => { pending = registry.act(registry.scope, actor, { handle: action().handle, arguments: { max_dimension: 256 }, request_id: "old" }); });
  expect(draw).toHaveBeenCalledWith(img, 0, 0, 40, 20);
  rendered.rerender(ui("second.png"));
  await act(async () => { encode(fixtureBlob()); expect(await pending).toMatchObject({ outcome: { status: "unavailable" } }); });
  expect(screen.queryByRole("alert")).toBeNull();
  vi.spyOn(HTMLCanvasElement.prototype, "toBlob").mockImplementation(callback => callback(fixtureBlob()));
  await act(async () => expect(await registry.act(registry.scope, actor, { handle: action().handle, arguments: { max_dimension: 256 }, request_id: "new" })).toMatchObject({ outcome: { value: { path: "second.png", width: 40, height: 20, ui_image: { media_type: "image/png", byte_length: 3 } } } }));
});
</storycode:copy>
<a id="f-d6a4681e6566-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-d6a4681e6566-expression_statement-3">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("returns an explicit browser capture request when displayed pixels are CORS-protected", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const url = "https://example.test/protected.png";
  render(<UiActionContext.Provider value={registry}><AssetPreviewImage path="protected.png" url={url} /></UiActionContext.Provider>);
  const img = screen.getByRole("img") as HTMLImageElement;
  for (const [key, value] of Object.entries({ complete: true, currentSrc: url, naturalWidth: 40, naturalHeight: 20 })) Object.defineProperty(img, key, { configurable: true, value });
  fireEvent.load(img);
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockImplementation(() => { throw new DOMException("Tainted canvas", "SecurityError"); });
  const action = registry.groups().groups.flatMap(group => registry.observe(group).actions).find(action => ["Read embedded image first.png", "Read embedded image second.png", "Read embedded image protected.png"].includes(action.label))!;
  await act(async () => expect(await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: action.handle, arguments: { max_dimension: 256 }, request_id: "protected" })).toMatchObject({ outcome: { status: "requires_input", request: expect.stringContaining("capture-access") } }));
  expect(screen.getByRole("alert").textContent).toContain("Tainted canvas");
});
</storycode:copy>

### Assemble `apps/web/src/embed/AssetPreviewImage.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/AssetPreviewImage.test.tsx"><storycode:paste select="#f-d6a4681e6566-expression_statement"/><storycode:paste select="#f-d6a4681e6566-fixtureBlob"/><storycode:paste select="#f-d6a4681e6566-expression_statement-2"/><storycode:paste select="#f-d6a4681e6566-expression_statement-3"/></storycode:file>

</storycode:doc>
