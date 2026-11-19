import {useEffect,useMemo,useReducer,useRef,useState} from 'react';
import {createPortal} from 'react-dom';
import {request} from '../api/client';
import type {Buffer,Buffers} from './buffers';
import type {WorkspaceChangeHistory} from './ChangePlan';
import {resourceWirePatches,type ResourceEditorRoute} from './ResourceEditors';
import {browserProposalModel,type ProposalSelection} from './AgentProposalAdapter';
import type {VerifiedProposal} from './AgentProposals';
import {BehaviorSourceReading} from './BehaviorSourceReading';
import type {BehaviorReading} from './BehaviorReading';
const object=(value:unknown):value is Record<string,unknown>=>!!value&&typeof value==='object'&&!Array.isArray(value);
const text=(value:unknown,limit=4000)=>typeof value==='string'?value.slice(0,limit):'Unavailable';
function SourceTransition({proposal}:{proposal:VerifiedProposal}){
 const patches=resourceWirePatches(proposal.value.plan),[path,setPath]=useState(patches[0].path),[side,setSide]=useState<'draft_before'|'draft_after'|'saved_before'|'saved_after'>('draft_after'),[offset,setOffset]=useState(0),[previous,setPrevious]=useState<number[]>([]);
 const patch=patches.find(patch=>patch.path===path)??patches[0],saved=side==='saved_before'?proposal.value.inputs.saved_sources[patch.path]:proposal.value.consequences.saved_sources[patch.path],source=side==='draft_before'?patch.before?.text:side==='draft_after'?patch.after?.text:saved?.kind==='file'?saved.text??undefined:undefined;let end=Math.min((source?.length??0),offset+4000);if(source&&end<source.length&&/[\uD800-\uDBFF]/.test(source[end-1])&&/[\uDC00-\uDFFF]/.test(source[end]))end++;
 const reset=()=>{setOffset(0);setPrevious([]);};
 return <section aria-label="Complete proposal source transitions"><label>Source target <select aria-label="Proposal source target" value={patch.path} onChange={event=>{setPath(event.target.value);reset();}}>{patches.map(item=><option key={item.path}>{item.path}</option>)}</select></label><label>Source snapshot <select aria-label="Proposal source snapshot" value={side} onChange={event=>{setSide(event.target.value as typeof side);reset();}}><option value="draft_before">Before draft</option><option value="draft_after">Proposed draft</option><option value="saved_before">Reviewed saved baseline before</option><option value="saved_after">Projected saved baseline after publication</option></select></label><p>Saved before is an observed baseline; saved after is a reviewed projection. Apply alone never claims publication.</p>{side.startsWith('saved_')&&saved?.kind==='file'&&<p>Saved metadata: SHA {text(saved.sha256,128)} · mode {typeof saved.mode==='number'?saved.mode:'unavailable'} · read-only {String(saved.readonly)} · executable {String(saved.executable)}</p>}<p>{patches.length} complete source transitions. {patch.before&&patch.after&&patch.before.identity!==patch.after.identity?'A different retained source identity occupies this address.':''}</p>{source===undefined?<p>{side.startsWith('saved_')&&saved?.kind==='file'?'The saved source is not decoded as text. Inspect its hash and metadata in the retained engine review.':'This address is missing in the selected namespace.'}</p>:<><pre style={{whiteSpace:'pre-wrap'}}>{source.slice(offset,end)||'(empty source)'}</pre><p>Characters {offset}–{end} of {source.length}; source text is paged, never rewritten by this reader.</p><button disabled={!previous.length} onClick={()=>{setOffset(previous.at(-1)!);setPrevious(previous.slice(0,-1));}}>Previous source</button><button disabled={end===source.length} onClick={()=>{setPrevious([...previous,offset]);setOffset(end);}}>Next source</button></>}</section>;
}
function ConnectedReading({raw,current}:{raw:unknown;current:()=>boolean}){
 const [,paint]=useReducer((value:number)=>value+1,0),reader=useMemo(()=>new BehaviorSourceReading((path,body)=>request(body===undefined?'GET':'POST',path,body),paint),[]),[selected,setSelected]=useState(''),[edgePage,setEdgePage]=useState(0);
 useEffect(()=>()=>reader.close(),[reader]);
 if(!object(raw)||!object(raw.graph)||!object(raw.graph.nodes))return <p>Connected reading unavailable in this retained material; impact remains unknown.</p>;
 const reading=raw as unknown as BehaviorReading,nodes=Object.values(reading.graph.nodes).filter(node=>object(node)&&typeof node.id==='string'&&typeof node.label==='string'),node=nodes.find(node=>node.id===selected)??nodes[0],edges=Array.isArray(reading.impact?.via)?reading.impact!.via:[],evidence=object(raw.evidence)?raw.evidence:null,records=Array.isArray(evidence?.records)?evidence!.records.filter(object):[];
 return <section aria-label="Proposal connected reading"><p>Captured relationships; source hashes identify captured bytes, not frozen runtime inputs. Current source authority must be rechecked before any verification.</p><p>Impact: {text(reading.impact?.basis)} · {text(reading.impact?.reason)} {reading.impact?.truncated?'Impact is truncated; unknown relationships remain.':''}</p>{node&&<><label>Connected source <select aria-label="Proposal connected source" value={node.id} onChange={event=>{setSelected(event.target.value);reader.close();}}>{nodes.map(item=><option key={item.id} value={item.id}>{text(item.label,512)} · {item.kind} · {item.status}</option>)}</select></label><p>{node.source_path??node.path} · {node.status} · {node.text_origin??'Source origin unknown'}</p><pre style={{whiteSpace:'pre-wrap'}}>{text(node.preview)}</pre><button disabled={!node.hash||reader.loading} onClick={()=>void reader.read(reading,node,undefined,current)}>Read complete connected source</button></>}
 {reader.notice&&<p role="status">{reader.notice}</p>}{reader.page&&<section aria-label="Complete connected source"><p>{reader.page.state} · {reader.page.target_status} · {reader.page.notice}</p><pre style={{whiteSpace:'pre-wrap'}}>{reader.page.text}</pre><p>{reader.page.offset}–{reader.page.offset+[...reader.page.text].length} of {reader.page.total} Unicode scalars</p><button disabled={reader.page.offset===0||reader.loading} onClick={()=>node&&void reader.read(reading,node,Math.max(0,reader.page!.offset-4000),current)}>Previous connected source</button><button disabled={reader.page.next===null||reader.loading} onClick={()=>node&&void reader.read(reading,node,reader.page!.next!,current)}>Next connected source</button></section>}
 <details><summary>Why these mechanisms and checks are related</summary>{edges.slice(edgePage*50,(edgePage+1)*50).map((edge,index)=><p key={index}>{edge.from} → {edge.to}: {edge.relationship} · {edge.basis} · {edge.authority} · {edge.reason}</p>)}{edges.length>50&&<><button disabled={!edgePage} onClick={()=>setEdgePage(edgePage-1)}>Previous relationships</button><button disabled={(edgePage+1)*50>=edges.length} onClick={()=>setEdgePage(edgePage+1)}>Next relationships</button></>}{!edges.length&&<p>No impact edges were supplied. This is not proof of zero impact.</p>}</details>
 <details><summary>Captured evidence and related checks</summary><p>{text(evidence?.status)} · {text(evidence?.notice)} {evidence?.truncated?'Evidence is truncated.':''}</p>{records.slice(0,100).map((record,index)=><p key={index}>{text(record.id,128)} · {text(record.evidence_kind)} · {text(record.freshness)} · {text(record.origin)}</p>)}{records.length>100&&<p>First 100 evidence records shown; inspect the connected reading for remaining retained observations.</p>}{reading.graph.checks?.slice(0,100).map(check=><p key={check.id}>{check.label} · {check.reason}</p>)}<p>These are declarations and retained evidence, not checks executed by this proposal.</p></details>
 {reading.graph.notices?.map((notice,index)=><p key={index}>{notice}</p>)}{reading.graph.truncated&&<p>Graph truncated; omitted relationships are unknown.</p>}</section>;
}
function ProposalConsequences({proposal}:{proposal:VerifiedProposal}){const [layer,setLayer]=useState<'saved'|'draft'>('saved'),[page,setPage]=useState(0),value=proposal.value.consequences[layer],rows=[...value.entries.map(path=>'Construction entry: '+path),...value.removed_entries.map(path=>'Removed entry: '+path),...value.output_paths.map(path=>'Generated output: '+path)],last=Math.max(0,Math.ceil(rows.length/100)-1),selected=Math.min(page,last);return <section aria-label="Saved and draft generated consequences"><label>Construction layer <select aria-label="Proposal construction layer" value={layer} onChange={event=>{setLayer(event.target.value as typeof layer);setPage(0);}}><option value="saved">Saved construction</option><option value="draft">Draft construction</option></select></label><p>{value.output_paths.length} generated output paths. Compiler-reported relationships; no outputs were materialized and no checks were run.</p><pre style={{whiteSpace:'pre-wrap'}}>{rows.slice(selected*100,(selected+1)*100).join('\n')||'No construction entries or outputs reported.'}</pre>{last>0&&<><button disabled={!selected} onClick={()=>setPage(selected-1)}>Previous consequences</button><span>{selected+1}/{last+1}</span><button disabled={selected===last} onClick={()=>setPage(selected+1)}>Next consequences</button></>}</section>;}
function RetainedMaterial({proposal,current}:{proposal:VerifiedProposal;current:()=>boolean}){
 const value=proposal.value,[side,setSide]=useState<'before'|'after'>('after');
 return <><h3>{value.intention.title}</h3><p>{value.origin_authority} · {value.intention.origin}</p><p>{value.intention.explanation}</p><details open><summary>Agent claims — unverified text</summary>{value.intention.claims.map((claim,index)=><p key={index}>{claim}</p>)}{!value.intention.claims.length&&<p>No agent claims were supplied.</p>}</details><details><summary>Requested check identities — not run</summary>{value.intention.check_ids.map(check=><code key={check}>{check}</code>)}{!value.intention.check_ids.length&&<p>No checks requested.</p>}</details>
 <SourceTransition key={proposal.fingerprint} proposal={proposal}/>
 <ProposalConsequences proposal={proposal}/>
 <label>Connected reading snapshot <select aria-label="Proposal connected snapshot" value={side} onChange={event=>setSide(event.target.value as typeof side)}><option value="before">Before intention</option><option value="after">Proposed intention</option></select></label><ConnectedReading key={proposal.fingerprint+side} raw={value[side]} current={current}/>
 <p>Sealed fingerprint: <code>{proposal.fingerprint}</code>. Apply, Save, Stage and verification are separate outcomes.</p></>;
}
export function RecordedPatchReview({proposal, anchor, close, model, undo, redo}: {
 proposal: VerifiedProposal; anchor: HTMLElement; close: () => void;
 model?: import('./AgentProposals').AgentProposalModel; undo?: () => void; redo?: () => void;
}) {
 const panel=useRef<HTMLElement>(null);
 useEffect(()=>{
  const dialog = panel.current;
  dialog?.focus({preventScroll:true});
  return () => {
   const currentFocus = document.activeElement;
   if (anchor.isConnected && (currentFocus === document.body || dialog?.contains(currentFocus))) {
    anchor.focus({preventScroll:true});
   }
  };
 },[anchor,proposal]);
 return createPortal(<section ref={panel} tabIndex={-1} role="dialog" aria-label="Recorded patch review"
  className="refs-panel" style={{width:"min(38rem, calc(100vw - 2.5rem))",maxHeight:"75dvh"}}
  onKeyDown={event=>{if(event.key==="Escape"){event.preventDefault();close();}}}>
  <header className="refs-head"><strong>Recorded patch review</strong><button onClick={close}>Close</button></header>
  <div style={{padding:"0.7rem"}}><p role="status">{model?.notice || "Read-only review of the captured workspace."}</p>
   {model?.prepared?.proposal === proposal && <button disabled={!model.canApply()} onClick={() => void model.apply()}>Apply complete proposal to drafts</button>}
   {undo && <button disabled={model?.busy} onClick={undo}>Undo workspace draft Apply</button>}
   {redo && <button disabled={model?.busy} onClick={redo}>Redo workspace draft Apply</button>}
   {model?.canReport(proposal.token) && <button onClick={() => void model.report(proposal.token)}>Retry acknowledgment</button>}
   <RetainedMaterial key={proposal.fingerprint} proposal={proposal} current={()=>!!panel.current?.isConnected}/>
  </div>
 </section>,document.body);
}
export function AgentProposalReview({open,close,buffers,history,acquire,adopt,initialSelection}:{open:boolean;close:()=>void;buffers:Buffers;history:WorkspaceChangeHistory;acquire:(path:string)=>Promise<Buffer>;adopt:(routes:ResourceEditorRoute<Buffer>[])=>void;initialSelection?:ProposalSelection|null}){
 const [,paint]=useReducer((value:number)=>value+1,0),lifetime=useRef(open),panel=useRef<HTMLElement>(null);lifetime.current=open;
 const model=useMemo(()=>browserProposalModel(buffers,history,acquire,adopt,()=>lifetime.current),[buffers,history,buffers.workspaceId]);model.changed=paint;
 useEffect(() => {
  if (!open) { model.close(); return; }
  let current = true;
  panel.current?.focus({preventScroll:true});
  void model.refresh().then(() => {
   if (!current || !initialSelection || model.selection) return;
   const row = model.rows.find(item => item.token === initialSelection.token && item.fingerprint === initialSelection.fingerprint);
   if (row) void model.open(row.token);
   else {
    model.notice = 'The selected sealed patch reading is unavailable. Reopen the message and review the fence again.';
    model.changed();
   }
  });
  return () => { current = false; };
 }, [open, model, initialSelection]);
 useEffect(()=>()=>{model.close();model.changed=()=>{};},[model]);
 if(!open)return null;const proposal=model.value,attempt=model.lastAttempt?.proposal.token===proposal?.token?model.lastAttempt:null;
 return createPortal(<section ref={panel} tabIndex={-1} role="dialog" aria-label="Agent proposal review" className="simple-workspace-problems simple-source-tests" onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();close();}}}><header><strong>Agent proposal review</strong><button onClick={close}>Close</button></header><p role="status">{model.notice}</p><button disabled={model.loading||model.busy} onClick={()=>void model.refresh()}>Refresh pending proposals</button><label>Complete intention <select aria-label="Retained agent proposal" value={model.selection} onChange={event=>void model.open(event.target.value)}><option value="">Choose an intention</option>{model.rows.map(row=><option key={row.token} value={row.token}>{row.title} · {row.historical?'historical reading':row.host_application?'host report retained':'pending'}</option>)}</select></label>
 {proposal&&<RetainedMaterial proposal={proposal} current={()=>model.value===proposal}/>}<button disabled={!model.canApply()} onClick={()=>void model.apply()}>{model.prepared?.kind==='namespace'?'Save reviewed resource namespace':'Apply complete proposal to drafts'}</button><p>{model.status}</p>{proposal&&model.reportNotices.has(proposal.token)&&<p>{model.reportNotices.get(proposal.token)}</p>}{proposal&&model.canReport(proposal.token)&&<button onClick={()=>void model.report(proposal.token)}>Retry acknowledgment of recorded transition</button>}{attempt?.publication&&<section aria-label="Retained resource publication response"><p>Received publication response; current editor adoption remains separate. {attempt.publication.adoption_current===false?attempt.publication.adoption_reason:''}</p><code>{attempt.publication.outcome?.recovery??'Recovery identity unavailable'}</code></section>}</section>,document.body);
}
