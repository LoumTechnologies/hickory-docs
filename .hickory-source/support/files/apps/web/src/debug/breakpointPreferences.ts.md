<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/breakpointPreferences.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Launch profiles choose targets within the viewed project](../../../../../../guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project/contract.md)

Captured SHA-256: `e2fdd09eef78474c6646a7882c7cc2d4d17a52e9b00561db19ab3c61719afe1f`.

<book:entry path="apps/web/src/debug/breakpointPreferences.ts">
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
</book:entry>

</book:doc>
