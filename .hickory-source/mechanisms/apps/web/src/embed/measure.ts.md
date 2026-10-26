<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/embed/measure.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-94067f97a135"></a>
## `apps/web/src/embed/measure.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-94067f97a135-measurePortableCore"></a>
### `measurePortableCore`

<storycode:copy id="f-94067f97a135-measurePortableCore">
import { loadHickLang, materializeLiteralFiles, rawStructure } from "../editor/hickLang";
/** Probe only: measured work, not a performance promise. */
export async function measurePortableCore() {
  const begin = performance.now();
  await loadHickLang();
  const ready = performance.now();
  const source = '# 🦀 probe\n<hick:file path="probe.js">\n' + 'var value = "é";\n'.repeat(4096) + '</hick:file>\n';
  rawStructure(source);
  const parsed = performance.now();
  const files = materializeLiteralFiles(source);
  const materialized = performance.now();
  return { sourceBytes: new TextEncoder().encode(source).length, parserLoadMs: ready - begin,
    structureMs: parsed - ready, literalFilesMs: materialized - parsed, outputBytes: new TextEncoder().encode(files[0].content).length };
}
</storycode:copy>

### Assemble `apps/web/src/embed/measure.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/measure.ts"><storycode:paste select="#f-94067f97a135-measurePortableCore"/></storycode:file>

</storycode:doc>
