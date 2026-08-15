import { describe, expect, it } from "vitest";
import { samePath } from "./paths";

describe("is this provenance about this document", () => {
  it("matches an absolute engine path against a relative app path", () => {
    // The bug this exists for: the engine answers with the path it opened,
    // the app knows the path it listed, and `===` says no forever — silently,
    // because the only symptom is ribbons that never draw.
    expect(samePath("/home/me/project/stats.hick", "stats.hick")).toBe(true);
    expect(samePath("/home/me/project/docs/stats.hick", "docs/stats.hick")).toBe(true);
    expect(samePath("stats.hick", "stats.hick")).toBe(true);
  });

  it("matches on whole segments, never on characters", () => {
    expect(samePath("/home/me/my-stats.hick", "stats.hick")).toBe(false);
    expect(samePath("/home/me/za/b.hick", "a/b.hick")).toBe(false);
  });

  it("does not match a different document", () => {
    expect(samePath("/home/me/project/other.hick", "stats.hick")).toBe(false);
  });

  it("tolerates windows separators and a leading ./", () => {
    expect(samePath("C:\\projects\\demo\\stats.hick", "stats.hick")).toBe(true);
    expect(samePath("./stats.hick", "stats.hick")).toBe(true);
  });
});
