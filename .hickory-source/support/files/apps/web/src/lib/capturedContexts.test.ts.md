<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/capturedContexts.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `bcd82416d086f5c7f514f9506318e3dc4b80169febfb5aeebae580652725e098`.

<book:entry path="apps/web/src/lib/capturedContexts.test.ts">
import { expect, it, vi } from "vitest";
import { request } from "../api/client";
import { acquireCapturedContext, retainCapturedContext } from "./capturedContexts";
vi.mock("../api/client", () => ({request:vi.fn()}));
it("shares a complete endpoint across patch panes and retains it throughout debugging", async () => {
  vi.mocked(request).mockResolvedValue({id:"same",uri:"hick-project:///same/"});
  const entries={"main.py":{bytes:[1]},"library.py":{bytes:[2]}};
  const [caller,callee]=await Promise.all([acquireCapturedContext(entries),acquireCapturedContext({...entries})]);
  expect(request).toHaveBeenCalledTimes(1);
  expect(caller.uri).toBe(callee.uri);
  const debug=retainCapturedContext(caller.uri);
  caller.release();callee.release();
  expect(request).not.toHaveBeenCalledWith("DELETE",expect.anything());
  const reopened=await acquireCapturedContext(entries);
  expect(reopened.uri).toBe(caller.uri);
  expect(request).toHaveBeenCalledTimes(1);
  reopened.release();debug();
  expect(request).toHaveBeenLastCalledWith("DELETE","/api/project-contexts/same");
});
</book:entry>

</book:doc>
