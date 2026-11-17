import {act,cleanup,fireEvent,render,screen} from '@testing-library/react';
import {afterEach,expect,it,vi} from 'vitest';
import {EditorState} from '@codemirror/state';
import {EditorView} from '@codemirror/view';
import {Buffer,Buffers} from './buffers';
import {WorkspaceChanges} from './WorkspaceChanges';
import {WorkspaceChangeHistory} from './ChangePlan';
import {requestWorkspaceChange} from './WorkspaceChangeEvents';
const mocks=vi.hoisted(()=>({editor:vi.fn(),lookup:vi.fn(),outline:vi.fn()}));
vi.mock('./Document',()=>({editorForBuffer:mocks.editor}));
vi.mock('./WorkspaceLanguages',()=>({useWorkspaceLanguageLookup:()=>mocks.lookup,useWorkspaceOutlineReader:()=>mocks.outline}));
let view:EditorView|undefined;
afterEach(()=>{cleanup();view?.destroy();view=undefined;vi.clearAllMocks();});
const replacement={range:{start:{line:0,character:0},end:{line:0,character:3}},newText:'new'};
function action(kind='source.fixAll.provider'){return {title:'Complete file fix',kind,edit:{changes:{'hick:///a.py':[replacement]}}};}
function fixture(result:unknown){
 const buffers=new Buffers(()=>{}),file={path:'a.py',name:'a.py',dir:false};const buffer=new Buffer(file,'old\n','base',()=>{});buffers.values.set(file.path,buffer);
 view=new EditorView({state:EditorState.create({doc:buffer.text,selection:{anchor:2},extensions:[EditorView.updateListener.of(update=>{if(update.docChanged)buffer.edit(update.state.doc.toString());})]}),parent:document.body});mocks.editor.mockImplementation(target=>target===buffer?{view}:undefined);
 const request=vi.fn(async(_method:string,_params?:unknown)=>result),client={request};mocks.lookup.mockReturnValue({uri:'hick:///a.py',client,ready:true});
 render(<WorkspaceChanges buffers={buffers} files={[file]} current={()=>buffer} focus={()=>{}} adoptResources={()=>{throw new Error('No resource publication expected');}} history={new WorkspaceChangeHistory()}/>);
 act(()=>requestWorkspaceChange('fixAll'));return {buffer,request};
}
async function prepare(){fireEvent.click(screen.getByRole('button',{name:'Prepare file-wide diagnostic fixes'}));}
it('requests the entire source family, reviews one complete action and coordinates draft Undo/Redo',async()=>{
 const {buffer,request}=fixture([action(),{title:'Unrelated quick fix',kind:'quickfix'}, {title:'Wrong prefix',kind:'source.fixAllWrong'}]);await prepare();await screen.findByRole('button',{name:'Complete file fix'});expect(screen.queryByRole('button',{name:'Unrelated quick fix'})).toBeNull();expect(screen.queryByRole('button',{name:'Wrong prefix'})).toBeNull();expect(request).toHaveBeenCalledWith('textDocument/codeAction',expect.objectContaining({range:{start:{line:0,character:0},end:{line:1,character:0}},context:{diagnostics:[],only:['source.fixAll']}}));expect(buffer.text).toBe('old\n');expect(view!.state.selection.main.head).toBe(2);
 fireEvent.click(screen.getByRole('button',{name:'Complete file fix'}));await screen.findByRole('button',{name:'Apply to source drafts'});expect(buffer.text).toBe('old\n');fireEvent.click(screen.getByRole('button',{name:'Apply to source drafts'}));expect(buffer.text).toBe('new\n');act(()=>requestWorkspaceChange('undo'));expect(buffer.text).toBe('old\n');act(()=>requestWorkspaceChange('redo'));expect(buffer.text).toBe('new\n');expect(request).toHaveBeenCalledTimes(1);
});
it('reports unavailable when the provider returns no complete-file family',async()=>{fixture([action('quickfix')]);await prepare();await screen.findByText(/offers no file-wide diagnostic fixes/);expect(screen.queryByRole('button',{name:'Complete file fix'})).toBeNull();expect(screen.queryByRole('button',{name:'Apply to source drafts'})).toBeNull();});
it('retires an equal-byte edit/Undo before a held bulk list arrives',async()=>{const {buffer,request}=fixture([]);let release!:(value:unknown)=>void;request.mockImplementationOnce(()=>new Promise(resolve=>{release=resolve;}));await prepare();act(()=>{buffer.edit('later');buffer.edit('old\n');});await act(async()=>release([action()]));await screen.findByText(/Source, selection or provider changed/);expect(screen.queryByRole('button',{name:'Complete file fix'})).toBeNull();expect(buffer.text).toBe('old\n');});
it('refuses a resolved action that changes from complete-file fixing to a quick fix',async()=>{const original={...action(),edit:undefined,data:{hickory_action:'held'}};const {buffer,request}=fixture([original]);request.mockImplementation(async(method:string)=>method==='codeAction/resolve'?action('quickfix'):[original]);await prepare();fireEvent.click(await screen.findByRole('button',{name:'Complete file fix'}));await screen.findByText(/changed the complete file-wide action kind/);expect(screen.queryByRole('button',{name:'Apply to source drafts'})).toBeNull();expect(buffer.text).toBe('old\n');});
