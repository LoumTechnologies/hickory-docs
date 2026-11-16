import {useState} from 'react';
import type {AdapterSourceReading} from './AdapterSourceReading';
export function AdapterSourcePanel({model}:{model:AdapterSourceReading}){
 const [page,setPage]=useState(0),value=model.value,length=value?.content.length??0;
 const crosses=(at:number)=>!!value&&at>0&&/[\uD800-\uDBFF]/.test(value.content[at-1]??'')&&/[\uDC00-\uDFFF]/.test(value.content[at]??'');
 const last=Math.max(0,Math.ceil(length/4000)-1-(length>4000&&length%4000===1&&crosses(length-1)?1:0)),shown=Math.min(page,last);
 const start=shown*4000,end=(shown+1)*4000,from=start+(crosses(start)?1:0),to=Math.min(length,end+(crosses(end)?1:0));
 if(!value&&!model.notice&&!model.busy)return null;
 return <section aria-label="Adapter-owned source"><header><strong>Adapter-owned source</strong><button onClick={()=>model.close()}>Close source reading</button></header><p role="status">{model.notice}</p>{value&&<><p>{model.current()?'Captured current stopped-frame receipt':'Historical stopped-frame receipt'} · reference {value.reference} · frame {value.frame} · SHA-256 {value.source_hash}</p><p>MIME observation: {value.mime_type??'unspecified'}. Literal read-only text; no workspace file was substituted.</p><pre style={{whiteSpace:'pre-wrap'}}>{value.content.slice(from,to)}</pre><button disabled={!shown} onClick={()=>setPage(shown-1)}>Previous source text</button><span>{shown+1}/{last+1}</span><button disabled={shown===last} onClick={()=>setPage(shown+1)}>Next source text</button></>}</section>;
}
