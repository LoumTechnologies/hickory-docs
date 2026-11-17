export type ChangeCommand = "rename" | "fixAll" | "sourceActions" | "actions" | "format" | "cleanup" | "undo" | "redo";
export const workspaceChangeEventName = "hickory:simple:workspace-change";
export function requestWorkspaceChange(command:ChangeCommand){window.dispatchEvent(new CustomEvent(workspaceChangeEventName,{detail:command}));}
export function requestWorkspaceProposal(title:string,edit:unknown,expected:ReadonlyMap<string,string>):void {
 const proposal={kind:'proposal' as const,title,edit,expected,accepted:false,error:''};
 window.dispatchEvent(new CustomEvent(workspaceChangeEventName,{detail:proposal}));
 if(!proposal.accepted)throw new Error(proposal.error||'The workspace change preview is unavailable. Retry after opening the workspace.');
}
