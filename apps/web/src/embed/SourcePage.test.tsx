import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { sha256 } from "../lib/reviews";
import { decodeSource, decodeSourceIndex, SourcePage } from "./SourcePage";
vi.mock("./DocumentEmbed", () => ({ DocumentEmbed: ({ source, readOnly }: { source: string; readOnly: boolean }) => <pre data-testid="source" data-readonly={String(readOnly)}>{source}</pre> }));
async function gzip(text: string): Promise<ArrayBuffer> {
  return new Response(new Response(text).body!.pipeThrough(new CompressionStream("gzip"))).arrayBuffer();
}
const revision = "a".repeat(40), page = '.hickory-source/01-language.md';
async function fixture() {
  const sources = ['# Build Hickory\n[Language](.hickory-source/01-language.md)', '# Construct the language\n[Start](../project.md)'];
  const documents = await Promise.all(["project.md", page].map(async (path, i) => ({ path, title: path, sha256: await sha256(sources[i]), url: `documents/${String(i).repeat(64)}.json` })));
  const index = decodeSourceIndex({ version: 1, revision, entry: "project.md", documents });
  const payloads = sources.map((source, i) => ({ source, path: documents[i].path, revision }));
  return { index, payloads };
}
afterEach(() => { cleanup(); sessionStorage.clear(); vi.unstubAllGlobals(); history.replaceState(null, "", "/"); });
describe("public tutorial reader", () => {
  it("renders the document with files and breadcrumbs instead of a second introduction", async () => {
    const { index, payloads } = await fixture();
    const fetcher = vi.fn(async (url: string) => new Response(JSON.stringify(url.endsWith("index.json") ? index : payloads[0]), { status: 200 }));
    vi.stubGlobal("fetch", fetcher);
    render(<SourcePage />);
    expect((await screen.findByTestId("source")).textContent).toContain("Build Hickory");
    expect(screen.getByTestId("source").getAttribute("data-readonly")).toBe("true");
    const files = screen.getByRole("navigation", { name: "Source files" });
    expect(within(files).getByRole("link", { name: "project.md" }).getAttribute("aria-current")).toBe("page");
    fireEvent.click(within(files).getByText(".hickory-source/"));
    expect(within(files).getByRole("link", { name: "01-language.md" }).getAttribute("href")).toBe(`?path=${encodeURIComponent(page)}`);
    expect(screen.getByRole("navigation", { name: "Breadcrumbs" }).textContent).toContain("project.md");
    expect(screen.queryByRole("combobox")).toBeNull();
    expect(screen.queryByRole("navigation", { name: "Related source pages" })).toBeNull();
    expect(fetcher).toHaveBeenCalledTimes(2);
  });
  it("opens a generic entry and reads its compressed source payload", async () => {
    const { index, payloads } = await fixture();
    const document = { ...index.documents[0], path: "book.md", url: index.documents[0].url + ".gz" };
    const generic = { ...index, entry: "book.md", documents: [document] };
    const payload = { ...payloads[0], path: "book.md" };
    vi.stubGlobal("fetch", vi.fn(async (url: string) => new Response(url.endsWith("index.json") ? JSON.stringify(generic) : await gzip(JSON.stringify(payload)))));
    render(<SourcePage />);
    expect((await screen.findByTestId("source")).textContent).toContain("Build Hickory");
    expect(screen.getByRole("link", { name: "book.md" }).getAttribute("aria-current")).toBe("page");
    expect(() => decodeSourceIndex({ ...generic, source_url: "https://secret:token@example.invalid/" })).toThrow("Invalid public source URL");
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
  it("keeps export metadata out of the reading view while loading language facts", async () => {
    const { index, payloads } = await fixture();
    const digest = "c".repeat(64);
    const withModels = { ...index, models: [{ language: "typescript", sha256: digest, url: `models/${digest}.json.gz` }] };
    const intelligence = { version: 1, source_hash: index.documents[0].sha256, state: "captured", spans: [],
      language: { capabilities: {}, queries: {}, diagnostics: [], occurrences: [] } };
    const document = { ...index.documents[0], intelligence_sha256: await sha256(JSON.stringify(intelligence)) };
    const published = { ...withModels, documents: [document, index.documents[1]] };
    const fetcher = vi.fn(async (url: string) => new Response(JSON.stringify(url.endsWith("index.json") ? published : { ...payloads[0], intelligence })));
    vi.stubGlobal("fetch", fetcher);
    render(<SourcePage />);
    await screen.findByTestId("source");
    expect(screen.queryByText("Captured code models")).toBeNull();
    expect(screen.queryByText("Published language information")).toBeNull();
    expect(screen.getByRole("main").querySelector("details")).toBeNull();
    expect(fetcher).toHaveBeenCalledTimes(2);
    expect(() => decodeSourceIndex({ ...index, models: [{ ...withModels.models[0], url: "https://unselected.invalid/model.json" }] })).toThrow("code model artifact");
  });
  it("opens a deep-linked chapter and expands its ancestor folders", async () => {
    const { index, payloads } = await fixture();
    history.replaceState(null, "", `/source.html?path=${encodeURIComponent(page)}`);
    vi.stubGlobal("fetch", vi.fn(async (url: string) => new Response(JSON.stringify(url.endsWith("index.json") ? index : payloads[1]))));
    render(<SourcePage />);
    expect((await screen.findByTestId("source")).textContent).toContain("Construct the language");
    const files = screen.getByRole("navigation", { name: "Source files" });
    expect(within(files).getByRole("link", { name: "01-language.md" }).getAttribute("aria-current")).toBe("page");
    expect(screen.getByRole("navigation", { name: "Breadcrumbs" }).textContent).toContain("01-language.md");
  });
  it("reports a missing deep link while retaining the files pane for recovery", async () => {
    const { index } = await fixture();
    history.replaceState(null, "", "/source.html?path=missing.md");
    vi.stubGlobal("fetch", vi.fn(async () => new Response(JSON.stringify(index))));
    render(<SourcePage />);
    expect((await screen.findByRole("alert")).textContent).toContain("not published");
    expect(within(screen.getByRole("navigation", { name: "Source files" })).getByRole("link", { name: "project.md" }).getAttribute("href")).toBe("?path=project.md");
    expect(screen.queryByTestId("source")).toBeNull();
  });
});
