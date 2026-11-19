import {act} from '@testing-library/react';
import {afterEach,expect,it,vi} from 'vitest';
import type {CoverageSurfaceClient} from './CoverageSurface';
const captured=vi.hoisted(()=>({client:null as CoverageSurfaceClient|null}));
vi.mock('./CoverageTools',()=>({CoveragePanel:({client}:{client:CoverageSurfaceClient})=>{captured.client=client;return <p>Coverage surface</p>;}}));
import {mountNativeCoverage} from './NativeCoverage';
afterEach(()=>{document.body.replaceChildren();captured.client=null;vi.useRealTimers();});
function fixture(){const host=document.createElement('div');document.body.append(host);const post=vi.fn();const surface=mountNativeCoverage(host,post);act(()=>surface.update({workspace:'one',path:'app.py',language:'python',selectedId:null}));return {surface,post,client:captured.client!};}
it('routes service and navigation replies by request identity',async()=>{
 const {surface,post,client}=fixture();const response=client.call({action:'list'});const message=post.mock.calls.at(-1)![0];expect(message).toMatchObject({kind:'api',workspace:'one',body:{action:'list'}});
 surface.reply(message.id+1,{wrong:true},null);surface.reply(message.id,{runs:[]},null);await expect(response).resolves.toEqual({runs:[]});
 const location={path:'app.py',line:1,sourceHash:'a'.repeat(64),sourceState:'current' as const,runId:null};
 const navigation=client.navigate(location,()=>true);const next=post.mock.calls.at(-1)![0];expect(next.kind).toBe('navigate');surface.reply(next.id,null,'Source changed');await expect(navigation).rejects.toThrow('Source changed');
 const stale=client.navigate(location,()=>true);const previous=post.mock.calls.at(-1)![0];client.retireNavigation?.();
 expect(post.mock.calls.at(-1)![0]).toEqual({kind:'retire-navigation',workspace:'one'});
 surface.reply(previous.id,false,null);await expect(stale).resolves.toBe(false);act(()=>surface.dispose());
});
it('context replacement rejects pending requests and cannot reuse old replies',async()=>{
 const {surface,post,client}=fixture();const old=client.call({action:'get',id:'old'});const rejected=expect(old).rejects.toThrow('context changed');const first=post.mock.calls.at(-1)![0];
 act(()=>surface.update({workspace:'two',path:null,language:'rust',selectedId:null}));await rejected;surface.reply(first.id,{old:true},null);
 const fresh=captured.client!.call({action:'list'});const second=post.mock.calls.at(-1)![0];expect(second.workspace).toBe('two');expect(second.id).toBeGreaterThan(first.id);surface.reply(second.id,{runs:[]},null);await expect(fresh).resolves.toEqual({runs:[]});act(()=>surface.dispose());
});
it('bounds pending requests and retires them on timeout and disposal',async()=>{
 vi.useFakeTimers();const {surface,client}=fixture();const pending=Array.from({length:64},()=>client.call({action:'list'}).catch(error=>String(error)));
 await expect(client.call({action:'list'})).rejects.toThrow('Too many');await act(async()=>{vi.advanceTimersByTime(45000);});expect((await Promise.all(pending)).every(value=>typeof value==='string'&&value.includes('timed out'))).toBe(true);
 const held=client.call({action:'list'});const rejected=expect(held).rejects.toThrow('context changed');act(()=>surface.dispose());await rejected;await expect(client.call({action:'list'})).rejects.toThrow('closed');
});
