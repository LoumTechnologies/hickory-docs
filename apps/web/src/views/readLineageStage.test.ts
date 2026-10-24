import { afterEach, expect, it, vi } from "vitest";
import { LINEAGE_READ_TIMEOUT_MS, readLineageStage } from "./readLineageStage";
import type { api } from "../api/client";
afterEach(() => vi.useRealTimers());
const summary = { id: "session", path: "sessions/large.md", updated_at: "" };
function reads() {
 return { doc: vi.fn().mockResolvedValue({ ...summary, source: "# source" }), outputs: vi.fn().mockResolvedValue({files:[]}), outputFile: vi.fn(), docReplay: vi.fn() } as unknown as Pick<typeof api,"doc"|"outputs"|"outputFile"|"docReplay">;
}
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
it("preserves only actually read provenance when a later output stalls", async()=>{
 vi.useFakeTimers();const host=reads();const known={path:'known.py',language:'python',content:'é',provenance:[{start:0,end:2,origin:{kind:'literal' as const,doc_path:summary.path,span:[0,2] as [number,number]}}]};
 vi.mocked(host.outputs).mockResolvedValue({files:[{path:'known.py',language:'python'},{path:'late.py',language:'python'}]});
 let finish!: (value: typeof known)=>void;
 vi.mocked(host.outputFile).mockResolvedValueOnce(known).mockImplementationOnce(()=>new Promise(resolve=>{finish=resolve;}));
 const pending=readLineageStage(summary,null,host);await vi.advanceTimersByTimeAsync(LINEAGE_READ_TIMEOUT_MS);const result=await pending;
 expect(result.document?.outputs).toEqual([known]);expect(result.error).toBeTruthy();
 finish({...known,path:'late.py'});await Promise.resolve();await Promise.resolve();expect(result.document?.outputs).toEqual([known]);
});
it("does not invent a stage for an unreadable source or outputs beyond a grammar boundary", async()=>{
 const host=reads();vi.mocked(host.doc).mockRejectedValue(Error('source unavailable'));
 expect(await readLineageStage(summary,null,host)).toEqual({document:undefined,error:'source unavailable'});
 expect(host.outputs).not.toHaveBeenCalled();
 vi.mocked(host.docReplay).mockResolvedValue({grammar_boundary:true,commit:'old',path:summary.path,message:'Unsupported historical grammar'});
 expect(await readLineageStage(summary,'old',host)).toEqual({boundary:'Unsupported historical grammar'});
 expect(host.outputFile).not.toHaveBeenCalled();
});
