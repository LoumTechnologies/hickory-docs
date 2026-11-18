<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Offer changes without importing an editor renderer

For maintainers changing workspace services independently of the editor renderer.

Capabilities request a command or offer a source-bound proposal through one small event boundary. This module owns no editor, preview layout or history. Synchronous acceptance means that the existing controller prepared a reviewable preview; absent or refusing controllers return an explicit error. Source changes still require the preview’s guarded apply action.

<ui:copy id="workspace-change-browser-events">
export type ChangeCommand = "rename" | "fixAll" | "sourceActions" | "actions" | "format" | "cleanup" | "entryMove" | "undo" | "redo";
export const workspaceChangeEventName = "hickory:simple:workspace-change";
export function requestWorkspaceChange(command:ChangeCommand){window.dispatchEvent(new CustomEvent(workspaceChangeEventName,{detail:command}));}
export function requestWorkspaceProposal(title:string,edit:unknown,expected:ReadonlyMap<string,string>):void {
 const proposal={kind:'proposal' as const,title,edit,expected,accepted:false,error:''};
 window.dispatchEvent(new CustomEvent(workspaceChangeEventName,{detail:proposal}));
 if(!proposal.accepted)throw new Error(proposal.error||'The workspace change preview is unavailable. Retry after opening the workspace.');
}
</ui:copy>

<ui:file path="apps/web/src/simple/WorkspaceChangeEvents.ts"><ui:paste select="#workspace-change-browser-events"/></ui:file>
</ui:doc>
