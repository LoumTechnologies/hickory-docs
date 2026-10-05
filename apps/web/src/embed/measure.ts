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
