<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Check that reading links cannot rewrite the program

The tutorial needs live prose links, but identical syntax inside a code example
must stay literal. Mount the read-only editor and inspect its anchors, then prove
the original source text survives exactly. Also check source-relative anchors,
commit-bound fallback links and refusal of executable URL schemes.

<storycode:copy id="public-source-document-links-tests">
import { afterEach, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { sourceHref, sourceLinks } from "./sourceLinks";
const index = { revision: "a".repeat(40), documents: [{ path: "project.md" }, { path: ".hickory-source/01-language.md" }] };
let view: EditorView | undefined;
afterEach(() => { view?.destroy(); view = undefined; document.body.replaceChildren(); });
it("keeps source links and anchors in the tutorial and binds other repository paths to the commit", () => {
  expect(sourceHref("../project.md#start", ".hickory-source/01-language.md", index)).toBe("?path=project.md#start");
  expect(sourceHref("docs/guide.md", "project.md", index)).toContain(`/blob/${index.revision}/docs/guide.md`);
  expect(sourceHref("javascript:alert(1)", "project.md", index)).toBeNull();
  expect(sourceHref("https://example.com/guide", "project.md", index)).toBe("https://example.com/guide");
});
it("renders prose links without interpreting link-looking code or changing the source", () => {
  const source = '<h:doc xmlns:h="http://www.hickorydocs.com/1.0">\n\n[Build the language](.hickory-source/01-language.md)\n\n<a id="part"></a>\n\n```md\n[Code example](private.md)\n<a id="literal-anchor"></a>\n```\n\n<h:copy id="code">\n[Literal file bytes](private.md)\n</h:copy>\n</h:doc>';
  const host = document.body.appendChild(document.createElement("div"));
  view = new EditorView({ parent: host, state: EditorState.create({ doc: source, extensions: [sourceLinks("project.md", index), EditorState.readOnly.of(true)] }) });
  const links = [...view.dom.querySelectorAll("a")];
  expect(links.map(link => link.textContent)).toEqual(["Build the language"]);
  expect(links[0].getAttribute("href")).toBe(`?path=${encodeURIComponent(".hickory-source/01-language.md")}`);
  expect(view.state.doc.toString()).toBe(source);
  expect(view.dom.textContent).not.toContain('<a id="part">');
  expect(view.dom.textContent).toContain('<a id="literal-anchor">');
});
</storycode:copy>

<storycode:file path="apps/web/src/embed/sourceLinks.test.ts"><storycode:paste select="#public-source-document-links-tests"/></storycode:file>
</storycode:doc>
