<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/breakpointPreferences.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9ef53203ebd0"></a>
## `apps/web/src/debug/breakpointPreferences.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Launch profiles choose targets within the viewed project](../../../../../guarantees/debugging/launch-profiles-choose-targets-within-the-viewed-project/contract.md).

<a id="f-9ef53203ebd0-held"></a>
### `held`

<storycode:copy id="f-9ef53203ebd0-held">
import type {HeldBreakpoint} from "./useDebugger";
// Source preferences survive captured-project identities. Runtime binding state
// remains session-specific; only explicit breakpoint edits update this store.
const held=new Map<string,HeldBreakpoint[]>();
</storycode:copy>
<a id="f-9ef53203ebd0-listeners"></a>
### `listeners`

<storycode:copy id="f-9ef53203ebd0-listeners">
const listeners=new Set<()=>void>();
</storycode:copy>
<a id="f-9ef53203ebd0-subscribeBreakpointPreferences"></a>
### `subscribeBreakpointPreferences`

<storycode:copy id="f-9ef53203ebd0-subscribeBreakpointPreferences">
export const subscribeBreakpointPreferences=(listener:()=>void)=>{listeners.add(listener);return()=>{listeners.delete(listener);};};
</storycode:copy>
<a id="f-9ef53203ebd0-breakpointPreferences"></a>
### `breakpointPreferences`

<storycode:copy id="f-9ef53203ebd0-breakpointPreferences">
export const breakpointPreferences=(key:string|null):HeldBreakpoint[]|undefined=>key?held.get(key):undefined;
</storycode:copy>
<a id="f-9ef53203ebd0-rememberBreakpointPreferences"></a>
### `rememberBreakpointPreferences`

<storycode:copy id="f-9ef53203ebd0-rememberBreakpointPreferences">
export function rememberBreakpointPreferences(key:string|null,breakpoints:HeldBreakpoint[]) {
  if(!key)return;
  held.set(key,breakpoints.map(({line,condition,hit_condition,log_message})=>({line,condition,hit_condition,log_message,state:"pending"})));
  for(const listener of listeners)listener();
}
</storycode:copy>

### Assemble `apps/web/src/debug/breakpointPreferences.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/breakpointPreferences.ts"><storycode:paste select="#f-9ef53203ebd0-held"/><storycode:paste select="#f-9ef53203ebd0-listeners"/><storycode:paste select="#f-9ef53203ebd0-subscribeBreakpointPreferences"/><storycode:paste select="#f-9ef53203ebd0-breakpointPreferences"/><storycode:paste select="#f-9ef53203ebd0-rememberBreakpointPreferences"/></storycode:file>

</storycode:doc>
