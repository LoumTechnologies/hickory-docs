<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/paths.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `85d3c814c77187bb181a5a9c590c36a14fe97e8658a5a0c7294fc2d967a819c8`.

<book:entry path="apps/web/src/lib/paths.test.ts">
import { describe, expect, it } from "vitest";
import { samePath } from "./paths";

describe("is this provenance about this document", () => {
  it("matches an absolute engine path against a relative app path", () => {
    // The bug this exists for: the engine answers with the path it opened,
    // the app knows the path it listed, and `===` says no forever — silently,
    // because the only symptom is ribbons that never draw.
    expect(samePath("/home/me/project/stats.md", "stats.md")).toBe(true);
    expect(samePath("/home/me/project/docs/stats.md", "docs/stats.md")).toBe(true);
    expect(samePath("stats.md", "stats.md")).toBe(true);
  });

  it("matches on whole segments, never on characters", () => {
    expect(samePath("/home/me/my-stats.md", "stats.md")).toBe(false);
    expect(samePath("/home/me/za/b.md", "a/b.md")).toBe(false);
  });

  it("does not match a different document", () => {
    expect(samePath("/home/me/project/other.md", "stats.md")).toBe(false);
  });

  it("tolerates windows separators and a leading ./", () => {
    expect(samePath("C:\\projects\\demo\\stats.md", "stats.md")).toBe(true);
    expect(samePath("./stats.md", "stats.md")).toBe(true);
  });
  it("matches editor buffer identities exactly, without filesystem suffix matching", () => {
    expect(samePath("hickory-buffer://u1", "hickory-buffer://u1")).toBe(true);
    expect(samePath("hickory-buffer://u1", "u1")).toBe(false);
    expect(samePath("hickory-buffer://u1", "hickory-buffer://u2")).toBe(false);
  });

});
</book:entry>

</book:doc>
