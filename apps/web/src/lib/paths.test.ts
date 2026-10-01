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
});
