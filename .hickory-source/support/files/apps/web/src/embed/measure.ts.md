<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/embed/measure.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `95a90b05bc85306068bfb458121e76c39b827d02681853997c5e28fef5a1dae4`.

<book:entry path="apps/web/src/embed/measure.ts">
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
</book:entry>

</book:doc>
