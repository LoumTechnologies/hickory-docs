<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/readLineageStage.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-ab5eee9ca5f7"></a>
## `apps/web/src/views/readLineageStage.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-ab5eee9ca5f7-LINEAGE_READ_TIMEOUT_MS"></a>
### `LINEAGE_READ_TIMEOUT_MS`

<storycode:copy id="f-ab5eee9ca5f7-LINEAGE_READ_TIMEOUT_MS">
import type { api } from "../api/client";
import type { DocSummary } from "../api/types";
import type { SourceDocument } from "../lineage/build";
export const LINEAGE_READ_TIMEOUT_MS = 10_000;
</storycode:copy>
<a id="f-ab5eee9ca5f7-boundedLineageRead"></a>
### `boundedLineageRead`

<storycode:copy id="f-ab5eee9ca5f7-boundedLineageRead">
export async function boundedLineageRead<T>(work: Promise<T>, milliseconds = LINEAGE_READ_TIMEOUT_MS): Promise<T> {
  let timer: ReturnType<typeof setTimeout>;
  try { return await Promise.race([work, new Promise<never>((_, reject) => { timer = setTimeout(() => reject(Error(`Read exceeded ${milliseconds / 1000} seconds`)), milliseconds); })]); }
  finally { clearTimeout(timer!); }
}
</storycode:copy>
<a id="f-ab5eee9ca5f7-readLineageStage"></a>
### `readLineageStage`

A deadline bounds waiting, not execution: these reads never run cells.
A timed-out shared request may settle later; its result cannot be adopted
or start another output read after this stage has retired.

<storycode:copy id="f-ab5eee9ca5f7-readLineageStage">
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
</storycode:copy>

### Assemble `apps/web/src/views/readLineageStage.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/readLineageStage.ts"><storycode:paste select="#f-ab5eee9ca5f7-LINEAGE_READ_TIMEOUT_MS"/><storycode:paste select="#f-ab5eee9ca5f7-boundedLineageRead"/><storycode:paste select="#f-ab5eee9ca5f7-readLineageStage"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-7c84debaead8"></a>
## `apps/web/src/views/readLineageStage.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-7c84debaead8-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-7c84debaead8-expression_statement">
import { afterEach, expect, it, vi } from "vitest";
import { LINEAGE_READ_TIMEOUT_MS, readLineageStage } from "./readLineageStage";
import type { api } from "../api/client";
afterEach(() => vi.useRealTimers());
</storycode:copy>
<a id="f-7c84debaead8-summary"></a>
### `summary`

<storycode:copy id="f-7c84debaead8-summary">
const summary = { id: "session", path: "sessions/large.md", updated_at: "" };
</storycode:copy>
<a id="f-7c84debaead8-reads"></a>
### `reads`

<storycode:copy id="f-7c84debaead8-reads">
function reads() {
 return { doc: vi.fn().mockResolvedValue({ ...summary, source: "# source" }), outputs: vi.fn().mockResolvedValue({files:[]}), outputFile: vi.fn(), docReplay: vi.fn() } as unknown as Pick<typeof api,"doc"|"outputs"|"outputFile"|"docReplay">;
}
</storycode:copy>
<a id="f-7c84debaead8-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-7c84debaead8-expression_statement-2">
it("retains the full large source but refuses late output adoption or follow-on reads after deadline", async () => {
 vi.useFakeTimers(); const host=reads();const source='<hick:context>'+ 'x'.repeat(3_000_000)+'</hick:context>';
 vi.mocked(host.doc).mockResolvedValue({...summary,source});
 let finish!: (value: Awaited<ReturnType<typeof api.outputs>>)=>void;
 vi.mocked(host.outputs).mockImplementation(()=>new Promise(resolve=>{finish=resolve;}));
 const pending=readLineageStage(summary,null,host);await Promise.resolve();await Promise.resolve();
 await vi.advanceTimersByTimeAsync(LINEAGE_READ_TIMEOUT_MS);const result=await pending;
 expect(result.document?.source).toBe(source);expect(result.document?.outputs).toEqual([]);expect(result.error).toMatch(/exceeded 10 seconds/);
 finish({files:[{path:'unread.py',language:'python'}]});await Promise.resolve();await Promise.resolve();
 expect(host.outputFile).not.toHaveBeenCalled();expect(result.document?.outputs).toEqual([]);
});
</storycode:copy>
<a id="f-7c84debaead8-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-7c84debaead8-expression_statement-3">
it("preserves only actually read provenance when a later output stalls", async()=>{
 vi.useFakeTimers();const host=reads();const known={path:'known.py',language:'python',content:'é',provenance:[{start:0,end:2,origin:{kind:'literal' as const,doc_path:summary.path,span:[0,2] as [number,number]}}]};
 vi.mocked(host.outputs).mockResolvedValue({files:[{path:'known.py',language:'python'},{path:'late.py',language:'python'}]});
 let finish!: (value: typeof known)=>void;
 vi.mocked(host.outputFile).mockResolvedValueOnce(known).mockImplementationOnce(()=>new Promise(resolve=>{finish=resolve;}));
 const pending=readLineageStage(summary,null,host);await vi.advanceTimersByTimeAsync(LINEAGE_READ_TIMEOUT_MS);const result=await pending;
 expect(result.document?.outputs).toEqual([known]);expect(result.error).toBeTruthy();
 finish({...known,path:'late.py'});await Promise.resolve();await Promise.resolve();expect(result.document?.outputs).toEqual([known]);
});
</storycode:copy>
<a id="f-7c84debaead8-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-7c84debaead8-expression_statement-4">
it("does not invent a stage for an unreadable source or outputs beyond a grammar boundary", async()=>{
 const host=reads();vi.mocked(host.doc).mockRejectedValue(Error('source unavailable'));
 expect(await readLineageStage(summary,null,host)).toEqual({document:undefined,error:'source unavailable'});
 expect(host.outputs).not.toHaveBeenCalled();
 vi.mocked(host.docReplay).mockResolvedValue({grammar_boundary:true,commit:'old',path:summary.path,message:'Unsupported historical grammar'});
 expect(await readLineageStage(summary,'old',host)).toEqual({boundary:'Unsupported historical grammar'});
 expect(host.outputFile).not.toHaveBeenCalled();
});
</storycode:copy>

### Assemble `apps/web/src/views/readLineageStage.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/readLineageStage.test.ts"><storycode:paste select="#f-7c84debaead8-expression_statement"/><storycode:paste select="#f-7c84debaead8-summary"/><storycode:paste select="#f-7c84debaead8-reads"/><storycode:paste select="#f-7c84debaead8-expression_statement-2"/><storycode:paste select="#f-7c84debaead8-expression_statement-3"/><storycode:paste select="#f-7c84debaead8-expression_statement-4"/></storycode:file>


</storycode:doc>
