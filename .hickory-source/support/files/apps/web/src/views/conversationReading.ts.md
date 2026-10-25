<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/conversationReading.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `09d0ad907736b34eaa006dc5268db26dc0069469f0d52e9baea2e46ac63f90e1`.

<book:entry path="apps/web/src/views/conversationReading.ts">
import type { AgentTurn, SessionLink, SessionViewResponse } from "../api/types";
import { bytesToChars } from "../lib/offsets";

/** Preserve the recorded bytes of the selected branch; remap receipts, never invent evidence. */
export function conversationReading(data: SessionViewResponse | null, branch: AgentTurn[], running: string | null, stream: string, reasoning: string) {
  let source = '<hick:session xmlns:hick="http://www.hickorydocs.com/1.0">\n';
  const links: SessionLink[] = [];
  const lines = data?.source.split(/(?<=\n)/) ?? [];
  const recorded = data?.view.turns ?? [];
  const offsets = data ? bytesToChars(data.source, data.links.flatMap(link => link.span)) : [];
  for (const turn of branch) {
    const index = recorded.findIndex(t => t.id === turn.id);
    // A running or interrupted turn may have durable work but not its final speech yet.
    if (data && index >= 0) {
      const startLine = recorded[index].session_line;
      const endLine = recorded[index + 1]?.session_line ?? lines.length + 1;
      const from = lines.slice(0, startLine - 1).join("").length;
      const chunk = lines.slice(startLine - 1, endLine - 1).join("").replace(/<\/hick:session>\s*$/, "");
      const to = from + chunk.length;
      const offset = source.length;
      const lineOffset = source.split("\n").length - startLine;
      for (const [index, link] of data.links.entries()) {
        const start = offsets[index * 2], end = offsets[index * 2 + 1];
        if (start < from || end > to) continue;
        links.push({ ...link, span: [offset + start - from, offset + end - from],
          lines: [link.lines[0] + lineOffset, link.lines[1] + lineOffset] });
      }
      source += chunk;
    } else {
      source += `<hick:user>\n${turn.prompt}\n</hick:user>\n`;
      if (turn.id !== running && turn.answer) source += `<hick:assistant>\n${turn.answer}\n</hick:assistant>\n`;
    }
    if (turn.id === running) {
      if (reasoning) source += `<hick:reasoning>\n${reasoning}\n</hick:reasoning>\n`;
      source += `<hick:assistant>\n${stream || "Working…"}\n</hick:assistant>\n`;
    } else if (turn.status === "stopped") {
      source += "\nStopped by you. Work already recorded is kept.\n";
    } else if (turn.status === "error") {
      source += `\nTurn failed: ${turn.error ?? "session failed"}\n`;
    }
  }
  source += "</hick:session>\n\n## Your response\n\n";
  return { source, links };
}
</book:entry>

</book:doc>
