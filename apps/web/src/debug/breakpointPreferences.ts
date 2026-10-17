import type {HeldBreakpoint} from "./useDebugger";
// Source preferences survive captured-project identities. Runtime binding state
// remains session-specific; only explicit breakpoint edits update this store.
const held=new Map<string,HeldBreakpoint[]>();
const listeners=new Set<()=>void>();
export const subscribeBreakpointPreferences=(listener:()=>void)=>{listeners.add(listener);return()=>{listeners.delete(listener);};};
export const breakpointPreferences=(key:string|null):HeldBreakpoint[]|undefined=>key?held.get(key):undefined;
export function rememberBreakpointPreferences(key:string|null,breakpoints:HeldBreakpoint[]) {
  if(!key)return;
  held.set(key,breakpoints.map(({line,condition,hit_condition,log_message})=>({line,condition,hit_condition,log_message,state:"pending"})));
  for(const listener of listeners)listener();
}
