import {Buffer,Buffers} from './buffers';
import {editorForBuffer} from './Document';
import {groups,isGroup,sourceTab,tabIdentity,type Layout} from './layout';
import {validateEditorRecoverySnapshot,type EditorRecoverySnapshot,type EditorRecoveryBuffer} from './EditorRecovery';
export function captureEditorRecovery(buffers:Buffers,tree:Layout,focused:string,extra:Iterable<Buffer>=[],resolve?:(tab:string)=>Buffer|undefined):EditorRecoverySnapshot {
 const rows=[...new Set([...buffers.values.values(),...buffers.retainedResources.values(),...extra])];
 const captured:EditorRecoveryBuffer[]=rows.map(b=>{const view=editorForBuffer(b)?.view;if(view?.composing||view&&view.state.doc.toString()!==b.text)throw Error('Finish source attachment and composition before the next private editor checkpoint.');const named=buffers.values.get(b.file.path)===b&&!b.resourceDeleted&&!b.isUntitledDraft&&!b.transient;
  const address:EditorRecoveryBuffer['address']=b.readonly?{kind:'generated',path:b.file.path,owner:b.file.generated_by!}:named?{kind:'source',path:b.file.path}:b.isUntitledDraft?{kind:'unnamed',name:b.file.name}:{kind:'detached',name:b.file.name,original_path:b.file.path};
  const selection=view?.state.selection.main??b.recoveryView?.selection??{anchor:0,head:0};
  return {id:b.sourceId,address,text:b.text,baseline:(b.isUntitledDraft||b.transient||b.resourceDeleted)?b.recoveryBaseline??{text:b.base,save_guard:b.saveGuard??null}:{text:b.base,save_guard:b.saveGuard??null},view:{selection:{anchor:selection.anchor,head:selection.head},scroll_x:view?.scrollDOM.scrollLeft??b.recoveryView?.scroll_x??0,scroll_y:view?.scrollDOM.scrollTop??b.recoveryView?.scroll_y??0}};
 });
 const tabs:Record<string,string>={};for(const group of groups(tree))for(const tab of group.tabs){const id=tabIdentity(tab);if(id.kind==='source'){const b=resolve?.(tab)??buffers.values.get(id.key)??rows.find(b=>b.sourceId===id.key);if(b)tabs[tab]=b.sourceId;}}
 return {host:'browser',buffers:captured,layout:{tree,focused,tabs}};
}
export function editorRecoveryLease(buffers:Buffers,tree:()=>Layout,focused:()=>string,extra:()=>Iterable<Buffer>=()=>[]):()=>boolean {
 const workspace=buffers.workspace,runtime=buffers.workspaceId,layout=tree(),focus=focused(),rows=[...new Set([...buffers.values.values(),...buffers.retainedResources.values(),...extra()])];
 const held=rows.map(b=>{const editor=editorForBuffer(b);return {b,file:b.file,text:b.text,base:b.base,guard:b.saveGuard,revision:b.sourceRevision,view:editor?.view,selection:editor?.view.state.selection.toJSON(),interaction:editor?.interaction};});const addresses=[...buffers.values];
 return ()=>buffers.workspace===workspace&&buffers.workspaceId===runtime&&tree()===layout&&focused()===focus&&buffers.values.size===addresses.length&&addresses.every(([path,b])=>buffers.values.get(path)===b)&&new Set([...buffers.values.values(),...buffers.retainedResources.values(),...extra()]).size===rows.length&&rows.every(b=>[...buffers.values.values(),...buffers.retainedResources.values(),...extra()].includes(b))&&held.every(h=>{const editor=editorForBuffer(h.b);return h.b.file===h.file&&h.b.text===h.text&&h.b.base===h.base&&h.b.saveGuard===h.guard&&h.b.sourceRevision===h.revision&&!h.b.saving&&editor?.view===h.view&&editor?.interaction===h.interaction&&!editor?.view.composing&&JSON.stringify(editor?.view.state.selection.toJSON())===JSON.stringify(h.selection);});
}
export function restoreEditorRecovery(buffers:Buffers,snapshot:EditorRecoverySnapshot,existing:Layout):{tree:Layout;focused:string} {
 validateEditorRecoverySnapshot(snapshot);const prefix='recovered-'+crypto.randomUUID(),added=new Map<string,Buffer>(),ids=new Map<string,string>();
 for(const row of snapshot.buffers){const path=prefix+'/'+row.id,name='Recovered '+(row.address.kind==='source'||row.address.kind==='generated'?row.address.path:row.address.name);if(buffers.values.has(path))throw Error('Recovery destination identity was reused. Preview again.');const generated=row.address.kind==='generated';const b=new Buffer({path,name,dir:false,...(row.address.kind==='generated'?{generated_by:row.address.owner}:{})},row.text,'',buffers.changed,generated,buffers.workspace);b.isUntitledDraft=!generated;b.base='';b.recoveryView=structuredClone(row.view);b.recoveryBaseline=row.baseline?structuredClone(row.baseline):null;b.saveGuardReason='Recovered private session: choose guarded Save As; no source save authority was restored.';added.set(path,b);ids.set(row.id,path);}
 let tree:Layout={id:prefix+'-sources',tabs:[...added.keys()].map(sourceTab),active:added.size?sourceTab([...added.keys()][0]):null},focused=tree.id;
 if(snapshot.host==='browser'){
  const data=snapshot.layout as {tree?:Layout;focused?:string;tabs?:Record<string,string>};let count=0;const seen=new Set<string>();
  function project(node:Layout,depth=0):Layout {if(!node||typeof node!=='object'||depth>32||++count>512||typeof node.id!=='string'||seen.has(node.id))throw Error('Malformed captured pane tree. Open drafts were preserved.');seen.add(node.id);const id=prefix+'-'+node.id;
   if(isGroup(node)){if(!Array.isArray(node.tabs)||node.tabs.length>512||node.tabs.some(t=>typeof t!=='string'))throw Error('Malformed captured pane tabs.');const tabs=node.tabs.flatMap(t=>{const path=ids.get(data.tabs?.[t]??'');return path?[sourceTab(path)]:[]});return {id,tabs:[...new Set(tabs)],active:node.active&&data.tabs?.[node.active]&&ids.has(data.tabs[node.active])?sourceTab(ids.get(data.tabs[node.active])!):tabs[0]??null};}
   if(!['horizontal','vertical'].includes(node.axis)||!Number.isFinite(node.ratio)||node.ratio<.1||node.ratio>.9)throw Error('Malformed captured pane proportions.');return {id,axis:node.axis,ratio:node.ratio,first:project(node.first,depth+1),second:project(node.second,depth+1)};
  }
  if(data?.tree){tree=project(data.tree);const placed=new Set(groups(tree).flatMap(g=>g.tabs)),missing=[...added.keys()].map(sourceTab).filter(tab=>!placed.has(tab));if(missing.length){tree={id:prefix+'-unplaced-split',axis:'horizontal',ratio:.75,first:tree,second:{id:prefix+'-unplaced',tabs:missing,active:missing[0]}};}focused=data.focused&&seen.has(data.focused)?prefix+'-'+data.focused:groups(tree).find(g=>g.tabs.length)?.id??groups(tree)[0].id;}
 }
 // Commit only after every buffer and layout node has been prepared successfully.
 for(const [path,b] of added)buffers.values.set(path,b);buffers.changed();return {tree:{id:prefix+'-session',axis:'horizontal',ratio:.5,first:existing,second:tree},focused};
}
