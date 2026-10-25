<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/runCellFresh.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `0e5ccef137c3b88cedb2fdb86b132a46a2abecb0507c01cd188090552058e352`.

<book:entry path="apps/web/src/views/runCellFresh.test.ts">
import { afterEach, expect, it, vi } from "vitest";
// @ts-expect-error Node test runtime only.
import { webcrypto } from "node:crypto";
import { api, installMockHandler } from "../api/client";
import { runCellFresh } from "./runCellFresh";

const source = '# Nonzero exit QA\n<hick:exec id="cell">exit 7</hick:exec>\n';
async function receipt(overrides = {}) {
  const digest = await webcrypto.subtle.digest("SHA-256", new TextEncoder().encode(source));
  return { document: "doc", source_hash: `sha256:${[...new Uint8Array(digest)].map(byte => byte.toString(16).padStart(2, "0")).join("")}`, blocks: [{ kind: "exec", id: "cell", span: [71, 99] }], ...overrides };
}
afterEach(() => { installMockHandler(null as never); vi.restoreAllMocks(); vi.unstubAllGlobals(); });

it("bypasses an older in-flight render and runs the current cell byte with exact source identity", async () => {
  vi.stubGlobal("crypto", webcrypto);
  const fresh = await receipt();
  let release!: (value: unknown) => void;
  let renders = 0;
  const handler = vi.fn(async (method: string, path: string, body?: unknown) => {
    if (method === "GET") {
      if (++renders === 1) return new Promise(resolve => { release = resolve; });
      return fresh;
    }
    expect(path).toBe("/api/docs/doc/blocks/71/run");
    expect(body).toEqual({ expected_source_hash: fresh.source_hash });
    return { outcome: "run", run_id: "new-run", cells: ["cell"] };
  });
  installMockHandler(handler);
  const older = api.render("doc");
  await expect(runCellFresh("doc", "cell", () => source)).resolves.toMatchObject({ run_id: "new-run" });
  expect(renders).toBe(2);
  release({ blocks: [{ kind: "exec", id: "cell", span: [73, 101] }] });
  await older;
  expect(handler.mock.calls.filter(([method]) => method === "POST")).toHaveLength(1);
});

it.each(["wrong document", "wrong source", "missing receipt", "removed cell", "editor changed"])("refuses %s before any action is dispatched", async reason => {
  vi.stubGlobal("crypto", webcrypto);
  let current = source;
  const fresh = await receipt(reason === "wrong document" ? { document: "other" }
    : reason === "wrong source" ? { source_hash: "sha256:older" }
    : reason === "missing receipt" ? { source_hash: undefined }
    : reason === "removed cell" ? { blocks: [] } : {});
  const handler = vi.fn(async () => {
    if (reason === "editor changed") current = source + "new edit";
    return fresh;
  });
  installMockHandler(handler);
  await expect(runCellFresh("doc", "cell", () => current)).rejects.toThrow(reason === "removed cell" ? "no longer" : "Save it");
  expect(handler).toHaveBeenCalledTimes(1);
});

it("never retries the action after an uncertain dispatched failure", async () => {
  vi.stubGlobal("crypto", webcrypto);
  const fresh = await receipt();
  const handler = vi.fn(async (method: string) => {
    if (method === "GET") return fresh;
    throw new Error("connection lost after dispatch");
  });
  installMockHandler(handler);
  await expect(runCellFresh("doc", "cell", () => source)).rejects.toThrow("connection lost after dispatch");
  expect(handler.mock.calls.filter(([method]) => method === "POST")).toHaveLength(1);
});
</book:entry>

</book:doc>
