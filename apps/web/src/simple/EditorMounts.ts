import type {EditorView} from '@codemirror/view';
import type {Buffer} from './buffers';
export interface RetainedEditorMount {view:EditorView;interaction:number;viewport?:{top:number;left:number;snapshot?:ReturnType<EditorView["scrollSnapshot"]>;document?:EditorView["state"]["doc"]};mount?:symbol}
/** Capture while attached, before a resource route can detach its styled host.
 * A detached view has no reliable layout; keep a matching attached snapshot. */
export function captureRetainedEditorViewport(record:RetainedEditorMount){const view=record.view,top=view.scrollDOM.scrollTop,left=view.scrollDOM.scrollLeft,document=view.state.doc,previous=record.viewport;record.viewport={top,left,document,snapshot:!view.dom.isConnected&&previous?.top===top&&previous.left===left&&previous.document===document?previous.snapshot:view.scrollSnapshot()};}
export function mountRetainedEditor(buffer:Buffer,record:RetainedEditorMount,parent:HTMLElement,current:()=>boolean,
 schedule:(callback:()=>void)=>number=callback=>requestAnimationFrame(callback),cancel:(ticket:number)=>void=ticket=>cancelAnimationFrame(ticket)):()=>void {
 const view=record.view,lease=Symbol('editor mount'),saved=record.viewport,interaction=record.interaction,text=buffer.text,file=buffer.file,path=buffer.file.path,deleted=buffer.resourceDeleted,doc=view.state.doc,selection=view.state.selection;
 let mounted=true,restored=!saved;record.mount=lease;parent.append(view.dom);
 const ticket=saved?schedule(()=>{
  if(!mounted||!current()||record.mount!==lease||view.dom.parentElement!==parent||record.interaction!==interaction||buffer.file!==file||buffer.file.path!==path||buffer.resourceDeleted!==deleted||buffer.text!==text||view.state.doc!==doc||!view.state.selection.eq(selection))return;
  view.scrollDOM.scrollTop=saved.top;view.scrollDOM.scrollLeft=saved.left;if(saved.snapshot&&saved.document===view.state.doc)view.dispatch({effects:saved.snapshot});restored=true;
 }):null;
 return ()=>{if(!mounted)return;mounted=false;if(ticket!==null)cancel(ticket);if(record.mount!==lease||!current())return;
  if(restored||record.interaction!==interaction||buffer.text!==text||view.state.doc!==doc||!view.state.selection.eq(selection))captureRetainedEditorViewport(record);
  delete record.mount;if(view.dom.parentElement===parent)view.dom.remove();
 };
}
