<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Exercise source navigation and revision-bound loading

For engineers maintaining the public source browser.

Drive entry-to-chapter navigation through the static host and assert read-only embedding. Refuse changed bytes and mixed revisions, external fetch targets and duplicate paths. Relative links and includes may navigate only to published source pages; a missing deep link must offer recovery through the entry.

<storycode:copy id="public-source-apps-web-src-embed-SourcePage-test-tsx">
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { sha256 } from "../lib/reviews";
import { decodeSource, decodeSourceIndex, relatedSources, SourcePage } from "./SourcePage";
vi.mock("./DocumentEmbed", () => ({ DocumentEmbed: ({ source, readOnly }: { source: string; readOnly: boolean }) => <pre data-testid="source" data-readonly={String(readOnly)}>{source}</pre> }));
const revision = "a".repeat(40), page = '.hickory-source/01-language.md';
async function fixture() {
  const sources = ['# Build Hickory\n[Language](.hickory-source/01-language.md)', '# Construct the language\n[Start](../project.md)'];
  const documents = await Promise.all(["project.md", page].map(async (path, i) => ({ path, title: path, sha256: await sha256(sources[i]), url: `documents/${String(i).repeat(64)}.json` })));
  const index = decodeSourceIndex({ version: 1, revision, entry: "project.md", documents });
  const payloads = sources.map((source, i) => ({ source, path: documents[i].path, revision }));
  return { index, payloads };
}
afterEach(() => { cleanup(); vi.unstubAllGlobals(); history.replaceState(null, "", "/"); });
describe("public source publication", () => {
  it("opens the entry and navigates to an actual read-only source chapter", async () => {
    const { index, payloads } = await fixture();
    const fetcher = vi.fn(async (url: string) => new Response(JSON.stringify(url.endsWith("index.json") ? index : url.includes("/" + "0".repeat(64)) ? payloads[0] : payloads[1]), { status: 200 }));
    vi.stubGlobal("fetch", fetcher); vi.stubGlobal("scrollTo", vi.fn());
    render(<SourcePage />);
    expect((await screen.findByTestId("source")).textContent).toContain("Build Hickory");
    expect(screen.getByTestId("source").getAttribute("data-readonly")).toBe("true");
    fireEvent.click(screen.getByRole("link", { name: "Start at project.md" }));
    expect(screen.getByTestId("source").textContent).toContain("Build Hickory");
    fireEvent.click(screen.getByText(/Continue reading/));
    fireEvent.click(screen.getByRole("link", { name: page }));
    expect((await screen.findByTestId("source")).textContent).toContain("Construct the language");
    expect(new URL(location.href).searchParams.get("path")).toBe(page);
    expect(fetcher).toHaveBeenCalledTimes(3);
  });
  it("refuses a stale revision or changed bytes rather than rendering a mixed publication", async () => {
    const { index, payloads } = await fixture();
    await expect(decodeSource({ ...payloads[0], revision: "b".repeat(40) }, index, index.documents[0])).rejects.toThrow("revision and hash");
    await expect(decodeSource({ ...payloads[0], source: "tampered" }, index, index.documents[0])).rejects.toThrow("revision and hash");
    expect(() => decodeSourceIndex({ ...index, documents: [{ ...index.documents[0], url: "https://elsewhere.invalid/private" }] })).toThrow("Invalid source document");
    expect(() => decodeSourceIndex({ ...index, documents: [...index.documents, index.documents[0]] })).toThrow("Invalid source document");
    expect(decodeSourceIndex({ ...index, documents: [...index.documents, { ...index.documents[1], path: ".hickory-source/icons/128x128@2x.png.md" }] }).documents).toHaveLength(3);
    expect(() => decodeSourceIndex({ ...index, documents: [{ ...index.documents[0], path: ".hickory-source/../private.md" }] })).toThrow("Invalid source document");
  });
  it("resolves parent links and includes only within the published document inventory", async () => {
    const { index } = await fixture();
    expect(relatedSources('[Start](../project.md)\n[Private](../../sessions/private.md)\n[External](https://example.com/project.md)', page, index).map(doc => doc.path)).toEqual(["project.md"]);
    expect(relatedSources('<book:include file=".hickory-source/01-language.md" />', "project.md", index).map(doc => doc.path)).toEqual([page]);
  });
  it("reports a missing deep link and allows returning to the entry", async () => {
    const { index, payloads } = await fixture();
    history.replaceState(null, "", "/source.html?path=missing.md");
    vi.stubGlobal("fetch", vi.fn(async (url: string) => new Response(JSON.stringify(url.endsWith("index.json") ? index : payloads[0]))));
    vi.stubGlobal("scrollTo", vi.fn());
    render(<SourcePage />);
    expect((await screen.findByRole("alert")).textContent).toContain("not in the published source");
    fireEvent.click(screen.getByRole("link", { name: "Start at project.md" }));
    expect((await screen.findByTestId("source")).textContent).toContain("Build Hickory");
  });
});
</storycode:copy>

<storycode:file path="apps/web/src/embed/SourcePage.test.tsx"><storycode:paste select="#public-source-apps-web-src-embed-SourcePage-test-tsx"/></storycode:file>
</storycode:doc>
