import {useState} from 'react';
import type {AdapterSourceReading} from './AdapterSourceReading';
export function AdapterSourcePanel({model}:{model:AdapterSourceReading}){
 const [page,setPage]=useState(0),value=model.value,last=Math.max(0,Math.ceil((value?.content.length??0)/4000)-1),shown=Math.min(page,last);
 let from=shown*4000,to=Math.min(value?.content.length??0,from+4000);
 if(value&&from>0&&/[\uDC00-\uDFFF]/.test(value.content[from]??''))from--;
 if(value&&to<value.content.length&&/[\uDC00-\uDFFF]/.test(value.content[to]??''))to++;
 if(!value&&!model.notice&&!model.busy)return null;
 return <section aria-label="Adapter-owned source"><header><strong>Adapter-owned source</strong><button onClick={()=>model.close()}>Close source reading</button></header><p role="status">{model.notice}</p>{value&&<><p>{model.current()?'Captured current stopped-frame receipt':'Historical stopped-frame receipt'} · reference {value.reference} · frame {value.frame} · SHA-256 {value.source_hash}</p><p>MIME observation: {value.mime_type??'unspecified'}. Literal read-only text; no workspace file was substituted.</p><pre style={{whiteSpace:'pre-wrap'}}>{value.content.slice(from,to)}</pre><button disabled={!shown} onClick={()=>setPage(shown-1)}>Previous source text</button><span>{shown+1}/{last+1}</span><button disabled={shown===last} onClick={()=>setPage(shown+1)}>Next source text</button></>}</section>;
}
