import {useEffect,useRef,useState,useSyncExternalStore} from 'react';
import {createPortal} from 'react-dom';
import {captureTemplateEditor,readTemplateCatalog,prepareTemplate,applyTemplate,engineTemplateProvider,type TemplateEditor,type TemplateSnapshot,type TemplateCatalog,type TemplatePreview,type TemplateProvider} from './SourceTemplates';
export function SourceTemplatesView({visible,close,editor,provider=engineTemplateProvider}:{visible:boolean;close:()=>void;editor:()=>TemplateEditor|null;provider?:TemplateProvider}){
 const [snapshot,setSnapshot]=useState<TemplateSnapshot|null>(null),[catalog,setCatalog]=useState<Readonly<TemplateCatalog>|null>(null),[preview,setPreview]=useState<TemplatePreview|null>(null),[query,setQuery]=useState(''),[notice,setNotice]=useState(''),[loading,setLoading]=useState(false);
 const ticket=useRef(0),shown=useRef(visible),latest=useRef(editor),providers=useRef(provider),catalogProvider=useRef<TemplateProvider|null>(null),panel=useRef<HTMLElement>(null);shown.current=visible;latest.current=editor;providers.current=provider;
 const [,paint]=useState(0);
 useSyncExternalStore(listener=>snapshot?snapshot.editor.buffer.subscribe(listener):()=>{},()=>snapshot?.editor.buffer.snapshot()??0);
 function sameEditor(held:TemplateSnapshot){const current=latest.current();return !!current&&current.buffer===held.editor.buffer&&current.buffers===held.editor.buffers&&current.view===held.view;}
 function dismiss(){ticket.current++;setLoading(false);close();}
 useEffect(()=>{if(!visible){ticket.current++;setLoading(false);setSnapshot(null);setCatalog(null);setPreview(null);return;}panel.current?.focus();void read();},[visible,provider]);
 useEffect(()=>()=>{ticket.current++;},[]);
 useEffect(()=>{if(!snapshot||!visible)return;const view=snapshot.view,update=()=>paint(value=>value+1);view.dom.addEventListener('keyup',update);view.dom.addEventListener('pointerup',update);view.scrollDOM.addEventListener('scroll',update);return()=>{view.dom.removeEventListener('keyup',update);view.dom.removeEventListener('pointerup',update);view.scrollDOM.removeEventListener('scroll',update);};},[snapshot,visible]);
 async function read(){const id=++ticket.current;setLoading(false);setCatalog(null);setPreview(null);setNotice('');
  try{const current=latest.current();if(!current)throw Error('The source editor is still attaching. Retry after it becomes available.');const held=captureTemplateEditor(current),origin=providers.current;setSnapshot(held);setLoading(true);const reading=await readTemplateCatalog(held,origin,()=>id===ticket.current&&shown.current&&sameEditor(held));const {catalog,snapshot:resolved}=reading;if(id!==ticket.current||!shown.current||!sameEditor(held)||providers.current!==origin)return;catalogProvider.current=origin;setSnapshot(resolved);setCatalog(catalog);if(!catalog.available)setNotice(catalog.reason||'Templates are unavailable for this source.');}
  catch(error){if(id===ticket.current)setNotice(error instanceof Error?error.message:String(error));}finally{if(id===ticket.current)setLoading(false);}
 }
 async function choose(id:string){if(!snapshot||!catalog)return;const held=snapshot,read=++ticket.current;setLoading(true);setPreview(null);setNotice('');
  try{if(!shown.current||!sameEditor(held)||catalogProvider.current!==providers.current)throw Error('The selected editor or provider changed. Refresh templates.');const plan=await prepareTemplate(held,catalog,id,providers.current,()=>read===ticket.current&&shown.current&&sameEditor(held)&&catalogProvider.current===providers.current);if(read!==ticket.current||!shown.current||!sameEditor(held))return;setPreview(plan);}
  catch(error){if(read===ticket.current)setNotice(error instanceof Error?error.message:String(error));}finally{if(read===ticket.current)setLoading(false);}
 }
 async function apply(){if(!preview)return;const held=preview,id=++ticket.current;setLoading(true);setNotice('');
  try{await applyTemplate(held,()=>id===ticket.current&&shown.current&&sameEditor(held.snapshot)&&held.provider===providers.current,message=>{if(id===ticket.current)setNotice(message);});if(id===ticket.current&&shown.current)dismiss();}
  catch(error){if(id===ticket.current)setNotice(error instanceof Error?error.message:String(error));}finally{if(id===ticket.current)setLoading(false);}
 }
 if(!visible)return null;const current=!!snapshot&&sameEditor(snapshot)&&snapshot.current()&&(!catalog||catalogProvider.current===providers.current),writable=current&&snapshot!.current(true),entries=(catalog?.entries??[]).filter(entry=>`${entry.label} ${entry.description} ${entry.language}`.toLowerCase().includes(query.toLowerCase()));
 return createPortal(<section ref={panel} tabIndex={-1} role="dialog" aria-label="Source templates" className="simple-workspace-problems simple-source-templates" onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();dismiss();}}}>
  <button aria-label="Close source templates" onClick={dismiss}>×</button><strong>Source templates</strong><p>{snapshot?.context.path??'No source editor selected'}</p>
  <p>Choose a reusable template, inspect its insertion, then apply it to this draft. Tab moves through linked fields. Files are saved separately.</p>
  <button disabled={loading} onClick={()=>{void read();}}>Read templates</button><label>Find template<input aria-label="Find template" value={query} onChange={event=>{setQuery(event.target.value);}}/></label>
  {!current&&snapshot&&<p role="status">The original source, selection or provider changed. Refresh before applying.</p>}{current&&!writable&&<p role="status">This source is read-only, saving, composing or has several carets. Its templates remain readable.</p>}
  {notice&&<p role="status">{notice}</p>}{loading&&<p role="status">Reading template provider…</p>}{catalog?.notices.map((message,index)=><p key={index}>{message}</p>)}
  {entries.slice(0,100).map(entry=><div key={entry.id}><button disabled={loading||!current||!writable||!catalog?.available||!entry.available} onClick={()=>{void choose(entry.id);}}>Preview {entry.label}</button><span> {entry.language} · {entry.origin}</span><p>{entry.description}{!entry.available?` · ${entry.reason??'Unavailable for this source'}`:''}</p></div>)}{entries.length>100&&<p>Narrow the search to read the remaining templates.</p>}
  {catalog&&entries.length===0&&<p>No templates matched. This does not establish that the language has no generation capabilities.</p>}
  {preview&&<section aria-label="Template preview"><strong>{preview.entry.label}</strong><p>{preview.fields} linked fields · existing mirrors update together</p><pre>{preview.text}</pre>{preview.notices.map((message,index)=><p key={index}>{message}</p>)}<button disabled={loading||!writable} onClick={()=>{void apply();}}>Apply template to draft</button></section>}
 </section>,document.body);
}
