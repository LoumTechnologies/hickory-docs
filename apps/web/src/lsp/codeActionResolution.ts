import type {CodeAction} from './client';
export interface CodeActionResolver {request:(method:string,params?:unknown)=>Promise<unknown>}
export function actionNeedsResolution(action:CodeAction):boolean {
 const data=action.data,held=!!data&&typeof data==='object'&&typeof (data as {hickory_action?:unknown}).hickory_action==='string';
 return held||(!action.edit&&data!==undefined&&data!==null);
}
/** Resolve retained provider data without granting command or edit authority. */
export async function resolveCodeAction(client:CodeActionResolver,action:CodeAction,uri:string):Promise<CodeAction>{
 if(!actionNeedsResolution(action))return action;
 const value=await client.request('codeAction/resolve',{...action,textDocument:{uri}});
 if(!value||typeof value!=='object'||Array.isArray(value)||typeof (value as CodeAction).title!=='string')throw new Error('Provider did not resolve this action. Prepare actions again.');
 return value as CodeAction;
}
