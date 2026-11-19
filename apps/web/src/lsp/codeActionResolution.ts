import type {CodeAction, LspClient} from './client';
import type {LspRange} from './positions';

export type SourceActionKind = "source" | "source.fixAll";

export function codeActionMatchesKind(action: CodeAction, kind: SourceActionKind): boolean {
  return action?.kind === kind || (typeof action?.kind === "string" && action.kind.startsWith(kind + "."));
}

export async function queryCodeActions(client: Pick<LspClient, "codeActions">, uri: string, range: LspRange, scope?: SourceActionKind): Promise<CodeAction[]> {
  if (!scope) return client.codeActions(uri, range);
  const actions = await client.codeActions(uri, range, [], [scope]);
  return actions.filter(action => codeActionMatchesKind(action, scope));
}

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
