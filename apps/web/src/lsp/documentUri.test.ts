import { expect, it } from "vitest";
import { documentUri, capturedDocumentUri, sameDocumentUri } from "./documentUri";
it("encodes filesystem paths once, retaining literal percent, hash and question mark names", () => {
  expect(capturedDocumentUri("hick-project:///capture/","%2F λ?#.py")).toBe("hick-project:///capture/%252F%20%CE%BB%3F%23.py");
  expect(capturedDocumentUri("hick-project:///capture/","hick-project:///other/a%20b.py")).toBe("hick-project:///other/a%20b.py");
  expect(documentUri("/Application Support/main.cpp")).toBe("hick:///Application%20Support/main.cpp");
  expect(documentUri("notes/a%20b#c?.cpp", "old")).toBe("hick:///notes/a%2520b%23c%3F.cpp?hick-project=old");
  expect(documentUri("hick-project:///capture/library.cpp")).toBe("hick-project:///capture/library.cpp");
});
it("matches serialized space spellings without collapsing distinct literal filenames or revisions", () => {
  expect(sameDocumentUri("hick:///Application Support/main.cpp", "hick:///Application%20Support/main.cpp")).toBe(true);
  expect(sameDocumentUri(documentUri("a%20b.cpp"), documentUri("a b.cpp"))).toBe(false);
  expect(sameDocumentUri(documentUri("a.cpp", "old"), documentUri("a.cpp"))).toBe(false);
});
