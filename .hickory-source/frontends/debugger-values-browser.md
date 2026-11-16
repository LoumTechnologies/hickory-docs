<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Expand only values reported in the selected debugger observation

For maintainers changing the browser's existing floating debugger inspector. Numeric adapter references are reusable handles, not identities. Each displayed node belongs to a particular reported root, selected frame and pause; each expansion adds a fresh request lifetime. Replacing a root, changing scope or collapsing a branch retires all descendants and in-flight requests before another reference can reuse their numbers.

## Make roots, visibility and expansion receipts explicit

The model has no transport or React dependency. The controller sends only requests returned by `expand`, and routes replies back through exact session/stop/thread/frame/request/reference admission. Empty, pending and refused expansions are distinct. Depth and total-node budgets fail with readable notices rather than recursively rendering unbounded provider data.

<ui:copy id="debug-value-tree-browser">
import type {Variable} from './client';
export interface DebugValueScope {session:string;stop:number;thread:number;frame:number}
export interface DebugValueNode {id:string;slot:string;name:string;value:string;type:string|null;reference:number;scope:DebugValueScope;parent:string|null;depth:number;expanded:boolean;pending:string|null;error:string|null;children:string[]|null}
export interface DebugChildrenRequest extends DebugValueScope {reference:number;request_id:string;node:string}
const equalScope=(a:DebugValueScope|null,b:DebugValueScope)=>!!a&&a.session===b.session&&a.stop===b.stop&&a.thread===b.thread&&a.frame===b.frame;
const validReference=(value:unknown):value is number=>typeof value==='number'&&Number.isSafeInteger(value)&&value>=0;
export class DebugValueTree {
 private scope:DebugValueScope|null=null;private nodes=new Map<string,DebugValueNode>();private slots=new Map<string,string[]>();private pending=new Map<string,DebugChildrenRequest>();private serial=0;
 historical:string[]=[];notice:string|null=null;
 get currentScope():DebugValueScope|null{return this.scope?{...this.scope}:null;}
 roots():DebugValueNode[]{return [...this.slots.values()].flat().flatMap(id=>{const node=this.nodes.get(id);return node?[this.copy(node)]:[];});}
 node(id:string):DebugValueNode|null {const value=this.nodes.get(id);return value?this.copy(value):null;}
 private copy(node:DebugValueNode):DebugValueNode{return {...node,scope:{...node.scope},children:node.children?[...node.children]:null};}
 private visible(id:string):boolean {let node=this.nodes.get(id);if(!node)return false;for(let depth=0;node.parent!==null;depth++){if(depth>12)return false;const parent=this.nodes.get(node.parent);if(!parent?.expanded||parent.pending!==null||!parent.children?.includes(node.id))return false;node=parent;}return this.slots.get(node.slot)?.includes(node.id)??false;}
 setScope(scope:DebugValueScope){if(!equalScope(this.scope,scope)){this.retire();this.scope={...scope};}}
 retire(){if(this.nodes.size)this.historical=[...this.nodes.values()].filter(node=>this.visible(node.id)).slice(0,512).map(node=>`${'  '.repeat(node.depth)}${node.name} = ${node.value}`).map(text=>text.slice(0,4096));this.scope=null;this.nodes.clear();this.slots.clear();this.pending.clear();this.notice=null;}
 closeInspection():boolean {const scope=this.scope;if(!scope)return false;const roots=this.roots(),slots=new Map<string,Variable[]>();for(const node of roots){const rows=slots.get(node.slot)??[];rows.push({name:node.name,value:node.value,type_name:node.type,variables_reference:node.reference});slots.set(node.slot,rows);}this.retire();this.setScope(scope);for(const [slot,rows] of slots)this.replace(slot,scope,rows);return true;}
 private drop(id:string){const node=this.nodes.get(id);if(!node)return;for(const child of node.children??[])this.drop(child);for(const [request,value] of this.pending)if(value.node===id)this.pending.delete(request);this.nodes.delete(id);}
 remove(slot:string){for(const id of this.slots.get(slot)??[])this.drop(id);this.slots.delete(slot);}
 replace(slot:string,scope:DebugValueScope,variables:readonly Variable[]):boolean {
  if(!equalScope(this.scope,scope))return false;if(variables.length>4096||variables.some(variable=>!variable||typeof variable.name!=='string'||typeof variable.value!=='string'||!validReference(variable.variables_reference))){this.notice='The provider value roots are malformed or too large.';return false;}
  const previous=[...this.nodes.values()].filter(node=>node.slot===slot).length;if(this.nodes.size-previous+variables.length>16384){this.notice='Reported value roots exceed the inspection budget.';return false;}this.remove(slot);
  const roots=variables.map(variable=>this.make(slot,scope,variable,null,0));this.slots.set(slot,roots);this.notice=null;return true;
 }
 private make(slot:string,scope:DebugValueScope,variable:Variable,parent:string|null,depth:number):string {const id=`value-${++this.serial}-${crypto.randomUUID()}`;this.nodes.set(id,{id,slot,name:variable.name,value:variable.value,type:variable.type_name??variable.type??null,reference:variable.variables_reference,scope:{...scope},parent,depth,expanded:false,pending:null,error:null,children:null});return id;}
 expand(id:string):DebugChildrenRequest|null {
  const node=this.nodes.get(id);if(!node||!this.scope||!equalScope(this.scope,node.scope)||!this.visible(id)||node.reference<=0||node.pending)return null;
  if(node.depth>=12){node.error='Maximum value inspection depth reached.';return null;}
  const descendants=new Set<string>();const collect=(id:string)=>{const child=this.nodes.get(id);if(!child)return;descendants.add(id);for(const next of child.children??[])collect(next);};for(const child of node.children??[])collect(child);for(const [id,pending] of this.pending)if(descendants.has(pending.node)){this.pending.delete(id);const child=this.nodes.get(pending.node);if(child)child.pending=null;}
  const request={...node.scope,reference:node.reference,request_id:`children-${crypto.randomUUID()}`,node:id};node.expanded=true;node.pending=request.request_id;node.error=null;this.pending.set(request.request_id,request);return {...request};
 }
 collapse(id:string){const node=this.nodes.get(id);if(!node)return;for(const child of node.children??[])this.drop(child);node.children=null;node.expanded=false;node.pending=null;node.error=null;for(const [request,value] of this.pending)if(value.node===id)this.pending.delete(request);}
 admit(event:{session:string;stop?:number;thread?:number|null;frame?:number|null;request_id?:string|null;reference:number;variables:Variable[]}):boolean {
  const request=event.request_id?this.pending.get(event.request_id):undefined,node=request?this.nodes.get(request.node):undefined;
  if(!request||!node||!this.scope||!equalScope(this.scope,node.scope)||!this.visible(node.id)||event.session!==request.session||event.stop!==request.stop||event.thread!==request.thread||event.frame!==request.frame||event.reference!==request.reference||node.pending!==event.request_id)return false;
  if(!Array.isArray(event.variables)||event.variables.length>4096||event.variables.some(variable=>!variable||typeof variable.name!=='string'||typeof variable.value!=='string'||!validReference(variable.variables_reference))){this.fail(event.request_id,'Provider children are malformed or exceed the inspection budget.');return false;}
  const prior=node.children??[],removed=new Set<string>();const collect=(id:string)=>{const value=this.nodes.get(id);if(!value)return;removed.add(id);for(const child of value.children??[])collect(child);};for(const child of prior)collect(child);
  if(this.nodes.size-removed.size+event.variables.length>16384){this.fail(event.request_id,'Provider children exceed the total inspection budget.');return false;}
  for(const child of prior)this.drop(child);node.children=event.variables.map(variable=>this.make(node.slot,node.scope,variable,node.id,node.depth+1));node.pending=null;node.error=null;this.pending.delete(event.request_id!);return true;
 }
 fail(request:unknown,message:string):boolean {if(typeof request!=='string')return false;const pending=this.pending.get(request),node=pending?this.nodes.get(pending.node):undefined;if(!pending||!node||!this.visible(node.id)||node.pending!==request||!equalScope(this.scope,node.scope))return false;node.pending=null;node.error=message;this.pending.delete(request);return true;}
}
</ui:copy>
<ui:file path="apps/web/src/debug/DebugValueTree.ts"><ui:paste select="#debug-value-tree-browser"/></ui:file>

## Reuse a reference number without reusing its old authority

These witnesses hold real expansion packets across root replacement, collapse and pause changes. They also cover empty and failed reads, nested provenance, duplicate names and total-root refusal. The actual adapter/browser witness must independently confirm a usable recursive path.

<ui:copy id="debug-value-tree-browser-tests">
import {expect,it} from 'vitest';
import {DebugValueTree,type DebugValueScope} from './DebugValueTree';
const scope:DebugValueScope={session:'s',stop:1,thread:0,frame:-1};const variable=(name='root',reference=7)=>({name,value:'reported',type_name:'dict',variables_reference:reference});
function tree(){const value=new DebugValueTree();value.setScope(scope);value.replace('locals',scope,[variable()]);return value;}
it('binds nested replies to node/request/frame even when references are reused',()=>{const value=tree(),root=value.roots()[0],first=value.expand(root.id)!;expect(first).toMatchObject({...scope,reference:7});const response={...scope,request_id:first.request_id,reference:7,variables:[variable('nested',8),variable('nested',8)]};expect(value.admit({...response,frame:9})).toBe(false);expect(value.admit(response)).toBe(true);const children=value.node(root.id)!.children!;expect(children[0]).not.toBe(children[1]);const nested=value.expand(children[0])!;value.replace('locals',scope,[variable('new root',7)]);expect(value.admit({...scope,request_id:nested.request_id,reference:8,variables:[variable('old leaf',0)]})).toBe(false);expect(value.admit(response)).toBe(false);expect(value.roots()[0].name).toBe('new root');});
it('retires descendants on collapse and distinguishes refusal, retry and empty',()=>{const value=tree(),root=value.roots()[0],one=value.expand(root.id)!;expect(value.fail(one.request_id,'provider refused')).toBe(true);expect(value.node(root.id)?.error).toContain('refused');const two=value.expand(root.id)!;expect(two.request_id).not.toBe(one.request_id);expect(value.admit({...scope,request_id:two.request_id,reference:7,variables:[]})).toBe(true);expect(value.node(root.id)?.children).toEqual([]);const held=value.expand(root.id)!;value.collapse(root.id);expect(value.admit({...scope,request_id:held.request_id,reference:7,variables:[variable()]})).toBe(false);expect(value.node(root.id)?.expanded).toBe(false);});
it('keeps old readings historical while denying old-stop and removed-expression expansion',()=>{const value=tree(),old=value.roots()[0];value.retire();expect(value.historical.join('\n')).toContain('root = reported');expect(value.expand(old.id)).toBeNull();value.setScope({...scope,stop:2});value.replace('watch:x',{...scope,stop:2},[variable('x')]);const current=value.roots()[0],pending=value.expand(current.id)!;value.remove('watch:x');expect(value.admit({...scope,stop:2,request_id:pending.request_id,reference:7,variables:[]})).toBe(false);});
it('retires the inspection lease on close even when root handles and frame stay the same',()=>{const value=tree(),root=value.roots()[0],pending=value.expand(root.id)!;value.closeInspection();expect(value.roots()[0].id).not.toBe(root.id);expect(value.expand(root.id)).toBeNull();expect(value.admit({...scope,request_id:pending.request_id,reference:7,variables:[variable('late')]})).toBe(false);});
it('refuses oversized roots without throwing or installing partial nodes',()=>{const value=tree();expect(value.replace('huge',scope,Array.from({length:4097},()=>variable()))).toBe(false);expect(value.roots()).toHaveLength(1);expect(value.notice).toContain('too large');});
</ui:copy>
<ui:file path="apps/web/src/debug/DebugValueTree.test.ts"><ui:paste select="#debug-value-tree-browser-tests"/></ui:file>
</ui:doc>
