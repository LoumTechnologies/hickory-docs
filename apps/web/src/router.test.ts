import { describe, expect, it } from "vitest";
import { parseRoute } from "./router";

describe("email-link routes", () => {
  it("parses a verification link, token intact", () => {
    const r = parseRoute("#/verify?token=abc123_-XY");
    expect(r).toEqual({ name: "verify", token: "abc123_-XY" });
  });

  it("parses a reset link", () => {
    expect(parseRoute("#/reset?token=zzz")).toEqual({
      name: "reset",
      token: "zzz",
    });
  });

  it("keeps a percent-encoded token byte-exact", () => {
    // Tokens are URL-safe base64, but mail clients rewrite links. A token
    // that survives the trip mangled is a token that will not redeem, and
    // the failure would look like an expired link.
    expect(parseRoute("#/verify?token=a%2Bb%3Dc")).toEqual({
      name: "verify",
      token: "a+b=c",
    });
  });

  it("routes the forgot-password page", () => {
    expect(parseRoute("#/forgot")).toEqual({ name: "forgot" });
  });

  it("does not mistake a tokenless path for a link", () => {
    expect(parseRoute("#/verify")).toEqual({ name: "projects" });
  });

  // Protects docs/guarantees/landing/discovery-page-is-interest-organized.md
  it("routes the bare domain to the landing page, not straight to a login form", () => {
    expect(parseRoute("")).toEqual({ name: "landing" });
    expect(parseRoute("#")).toEqual({ name: "landing" });
    expect(parseRoute("#/")).toEqual({ name: "landing" });
  });

  it("still parses the routes that existed before", () => {
    expect(parseRoute("#/login")).toEqual({ name: "login" });
    expect(parseRoute("#/docs/abc")).toEqual({ name: "doc", id: "abc" });
  });
});
