<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lsp/documentUri.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `070b19fcf6d3b7730f13ee9973ba1a14adcc09967deb3fb7d5de23b6284a8881`.

<book:entry path="apps/web/src/lsp/documentUri.test.ts">
import { expect, it } from "vitest";
import { documentUri, sameDocumentUri } from "./documentUri";
it("encodes filesystem paths once, retaining literal percent, hash and question mark names", () => {
  expect(documentUri("/Application Support/main.cpp")).toBe("hick:///Application%20Support/main.cpp");
  expect(documentUri("notes/a%20b#c?.cpp", "old")).toBe("hick:///notes/a%2520b%23c%3F.cpp?hick-project=old");
  expect(documentUri("hick-project:///capture/library.cpp")).toBe("hick-project:///capture/library.cpp");
});
it("matches serialized space spellings without collapsing distinct literal filenames or revisions", () => {
  expect(sameDocumentUri("hick:///Application Support/main.cpp", "hick:///Application%20Support/main.cpp")).toBe(true);
  expect(sameDocumentUri(documentUri("a%20b.cpp"), documentUri("a b.cpp"))).toBe(false);
  expect(sameDocumentUri(documentUri("a.cpp", "old"), documentUri("a.cpp"))).toBe(false);
});
</book:entry>

</book:doc>
