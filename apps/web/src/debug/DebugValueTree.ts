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
