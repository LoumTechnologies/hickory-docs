export type SavedHistorySource={kind:'file';text:string;sha256:string;readonly:boolean}|{kind:'missing'};
export interface SourceHistoryEntry {id:string;at:number;outcome:'prepared'|'published'|'failed'|'unknown';fingerprint:string;before:SavedHistorySource;after:SavedHistorySource;available:boolean;reason?:string}
export interface SourceHistoryReading {version:number;history_protocol:number;workspace:string;namespace:string;path:string;current:SavedHistorySource;current_identity:string;entries:SourceHistoryEntry[];notices:string[]}
export interface SourceHistoryDraft {version:number;history_protocol:number;workspace:string;namespace:string;path:string;id:string;fingerprint:string;current_identity:string;current_hash:string;draft_hash:string;side:'before'|'after';source:string;consequences:string;notice:string}
export interface SourceHistoryContext {namespace:string;path:string;source:string;readCurrent?:()=>boolean;current:()=>boolean;apply:(source:string)=>void}
export type SourceHistoryTransport=<T>(method:'GET'|'POST',path:string,body?:unknown)=>Promise<T>;
export async function historySourceHash(source:string):Promise<string>{return [...new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(source)))].map(byte=>byte.toString(16).padStart(2,'0')).join('');}
function hex(value:unknown,count:number):value is string{return typeof value==='string'&&new RegExp('^[0-9a-f]{'+count+'}$').test(value);}
export async function validateSourceHistory(value:SourceHistoryReading,path:string,namespace:string):Promise<void>{
 if(value.version!==1||value.history_protocol!==1||value.path!==path||!namespace||value.namespace!==namespace||!value.workspace||!value.current_identity||!Array.isArray(value.entries)||value.entries.length>1000||!Array.isArray(value.notices)||value.current.kind!=='file')throw Error('Unsupported or changed local history identity.');
 const ids=new Set<string>();
 for(const entry of value.entries){if(!hex(entry.id,32)||ids.has(entry.id)||!hex(entry.fingerprint,64)||!Number.isSafeInteger(entry.at)||entry.at<0||entry.at>8640000000000000||!['prepared','published','failed','unknown'].includes(entry.outcome)||typeof entry.available!=='boolean'||entry.available&&entry.outcome!=='published')throw Error('Local history contains ambiguous outcome evidence.');ids.add(entry.id);}
 for(const saved of [value.current,...value.entries.flatMap(entry=>[entry.before,entry.after])])if(saved.kind==='file'){if(typeof saved.text!=='string'||!hex(saved.sha256,64)||await historySourceHash(saved.text)!==saved.sha256)throw Error('Local history source fingerprint changed.');}else if(saved.kind!=='missing')throw Error('Unsupported history source state.');
}
export class SourceHistoryModel {
 reading:SourceHistoryReading|null=null;selected:SourceHistoryEntry|null=null;side:'before'|'after'='before';notice='';busy=false;private ticket=0;private context:SourceHistoryContext|null=null;
 constructor(private load:SourceHistoryTransport,private changed:()=>void=()=>{}){}
 close(){this.ticket++;this.context=null;this.selected=null;this.busy=false;}
 async open(context:SourceHistoryContext){this.close();this.context=context;this.reading=null;const ticket=++this.ticket;this.busy=true;this.notice='Reading retained local source history…';this.changed();
  try{const value=structuredClone(await this.load<SourceHistoryReading>('GET','/api/source-history?path='+encodeURIComponent(context.path)));await validateSourceHistory(value,context.path,context.namespace);if(ticket!==this.ticket)return;if(!(context.readCurrent?.()??context.current()))throw Error('Source editor changed while reading history. Open it again.');this.reading=value;this.notice='Compare retained text before restoring to this draft. Saved files are unchanged.';}catch(error){if(ticket===this.ticket)this.notice=String(error);}finally{if(ticket===this.ticket){this.busy=false;this.changed();}}
 }
 choose(id:string,side:'before'|'after'){if(this.busy)return;const entry=this.reading?.entries.find(entry=>entry.id===id);if(!entry)return;this.selected=entry;this.side=side;this.notice=entry.reason??'Preview only; current unsaved text is retained.';this.changed();}
 get preview(){const saved=this.selected?.[this.side];return saved?.kind==='file'?saved.text:null;}
 current(){return this.context?.current()===true&&this.reading?.current.kind==='file'&&this.reading.current.readonly===false;}
 async restore():Promise<boolean>{const context=this.context,reading=this.reading,entry=this.selected,side=this.side,source=this.preview;if(this.busy||!context?.current()||!reading||!entry?.available||source===null||reading.current.kind!=='file'||reading.current.readonly!==false){this.notice='This history or editor is stale or unavailable. Current draft is retained.';this.changed();return false;}
  const ticket=++this.ticket;this.busy=true;this.selected=null;this.changed();
  try{const draftHash=await historySourceHash(context.source);if(ticket!==this.ticket||!context.current())return false;const body={version:1,path:context.path,id:entry.id,fingerprint:entry.fingerprint,workspace:reading.workspace,namespace:context.namespace,current_identity:reading.current_identity,current_hash:reading.current.sha256,draft_hash:draftHash,side};const value=await this.load<SourceHistoryDraft>('POST','/api/source-history/restore-draft',body);if(ticket!==this.ticket)return false;
   if(!context.current()||value.version!==1||value.history_protocol!==1||value.workspace!==body.workspace||value.namespace!==body.namespace||value.path!==body.path||value.id!==body.id||value.fingerprint!==body.fingerprint||value.current_identity!==body.current_identity||value.current_hash!==body.current_hash||value.draft_hash!==body.draft_hash||value.side!==side||value.source!==source||value.consequences!=='unknown')throw Error('History authority or editor changed. No draft was replaced.');context.apply(source);this.context=null;this.notice=value.notice;return true;
  }catch(error){if(ticket===this.ticket)this.notice=String(error);return false;}finally{if(ticket===this.ticket){this.busy=false;this.changed();}}
 }
}
