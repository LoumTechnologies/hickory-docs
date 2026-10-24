import type { api } from "../api/client";
import type { DocSummary } from "../api/types";
import type { SourceDocument } from "../lineage/build";
export const LINEAGE_READ_TIMEOUT_MS = 10_000;
export async function boundedLineageRead<T>(work: Promise<T>, milliseconds = LINEAGE_READ_TIMEOUT_MS): Promise<T> {
  let timer: ReturnType<typeof setTimeout>;
  try { return await Promise.race([work, new Promise<never>((_, reject) => { timer = setTimeout(() => reject(Error(`Read exceeded ${milliseconds / 1000} seconds`)), milliseconds); })]); }
  finally { clearTimeout(timer!); }
}
/** A deadline bounds waiting, not execution: these reads never run cells.
 * A timed-out shared request may settle later; its result cannot be adopted
 * or start another output read after this stage has retired. */
export async function readLineageStage(summary: DocSummary, at: string | null,
  reads: Pick<typeof api, "doc" | "outputs" | "outputFile" | "docReplay">,
  current: () => boolean = () => true,
): Promise<{ document?: SourceDocument; error?: string; boundary?: string }> {
  let active = true;
  let captured: SourceDocument | undefined;
  async function read<T>(promise: Promise<T>): Promise<T> {
    const value = await promise;
    if (!active || !current()) throw Error("Lineage read retired");
    return value;
  }
  const work = async () => {
    if (at !== null) {
      const replayed = await read(reads.docReplay(summary.id, at));
      if (replayed.grammar_boundary) return { boundary: replayed.message };
      captured = { id: summary.id, path: summary.path, source: replayed.source, outputs: [] };
      for (const path of replayed.outputs) {
        const one = await read(reads.docReplay(summary.id, at, path));
        if (!one.grammar_boundary && one.output) captured.outputs.push({ ...one.output, language: "" });
      }
    } else {
      const doc = await read(reads.doc(summary.id));
      captured = { id: summary.id, path: doc.path, source: doc.source, outputs: [] };
      const listed = await read(reads.outputs(summary.id));
      for (const meta of listed.files) captured.outputs.push(await read(reads.outputFile(summary.id, meta.path)));
    }
    return { document: captured };
  };
  try { return await boundedLineageRead(work()); }
  catch (error) { return { document: captured ? { ...captured, outputs: [...captured.outputs] } : undefined, error: String((error as Error).message ?? error) }; }
  finally { active = false; }
}
