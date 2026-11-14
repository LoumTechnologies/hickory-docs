import {it,expect} from "vitest";
import {workspaceCommands,commandReason,type CommandContext} from "./WorkspaceCommands";
it("command results recheck the current context before invoking an adapter",()=>{
  const context:CommandContext={editor:true,writable:true,dirty:true,saving:false,back:false,forward:false,tab:true,groups:1};let called=0;
  const commands=workspaceCommands(()=>context,{"file.save":()=>called++});const save=commands.find(item=>item.id==="file.save")!;
  expect(save.reason).toBe(null);context.saving=true;save.run();expect(called).toBe(0);
  context.saving=false;context.writable=false;save.run();context.writable=true;context.editor=false;save.run();expect(called).toBe(0);
  context.editor=true;save.run();expect(called).toBe(1);expect(commands.find(item=>item.id==="file.open")!.reason).toContain("Unavailable");
  expect(commandReason("unknown",context)).not.toBe(null);expect(commandReason("back",context)).not.toBe(null);expect(commandReason("multi",context)).not.toBe(null);
});
