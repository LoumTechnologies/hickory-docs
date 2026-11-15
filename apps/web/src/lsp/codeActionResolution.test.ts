import {expect,it,vi} from 'vitest';
import {resolveCodeAction} from './codeActionResolution';
it('eager legacy actions remain usable without an unsupported resolver',async()=>{
 const action={title:'Eager',edit:{changes:{}}},client={request:vi.fn()};expect(await resolveCodeAction(client,action,'hick:///a.py')).toBe(action);expect(client.request).not.toHaveBeenCalled();
});
it('held actions resolve even with an edit and retain the complete returned command boundary',async()=>{
 const action={title:'Held',data:{hickory_action:'token'},edit:{changes:{}}},resolved={title:'Resolved',edit:{changes:{}},command:{title:'Provider command',command:'provider.effect',arguments:['opaque']}};
 const client={request:vi.fn(async()=>resolved)};expect(await resolveCodeAction(client,action,'hick:///a.py')).toBe(resolved);
 expect(client.request).toHaveBeenCalledWith('codeAction/resolve',{...action,textDocument:{uri:'hick:///a.py'}});
});
it('unresolved opaque actions propagate refusal and never substitute an eager partial value',async()=>{
 const action={title:'Lazy',data:{opaque:[1,2]}};await expect(resolveCodeAction({request:async()=>null},action,'hick:///a.py')).rejects.toThrow('did not resolve');
 await expect(resolveCodeAction({request:async()=>{throw new Error('source changed');}},action,'hick:///a.py')).rejects.toThrow('source changed');
});
