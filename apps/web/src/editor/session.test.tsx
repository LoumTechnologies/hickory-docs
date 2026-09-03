// @vitest-environment jsdom
import { cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { DocumentEditor } from "./DocumentEditor";
import { LocalRealtime } from "../api/realtime";
import { parseHickDoc, languageForBlock } from "./hickDoc";

afterEach(cleanup);

// A hick:session document: the conversation, as the file the agent wrote.
const SESSION = `<hick:session xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:user>Make the greeting friendlier.</hick:user>
<hick:assistant>Writing the file now.
<hick:action lang="bash">
printf 'hello' > out.txt
</hick:action>
</hick:assistant>
<hick:observation source="action-0" exit="0">done
</hick:observation>
</hick:session>
`;

describe("a session document renders as a conversation", () => {
  it("frames each turn and labels who is speaking", async () => {
    const realtime = new LocalRealtime();
    const { container } = render(
      <DocumentEditor
        docId="s1"
        initialSource={SESSION}
        realtime={realtime}
        execBlocks={[]}
        runningCells={new Set()}
        onRunCell={() => undefined}
      />,
    );
    await waitFor(() =>
      expect(container.querySelector(".cm-content")?.textContent).toContain(
        "Make the greeting friendlier.",
      ),
    );

    const chips = [...container.querySelectorAll(".cm-hick-banner-label")].map(
      (e) => e.textContent,
    );
    expect(chips).toEqual(expect.arrayContaining(["you", "agent", "ran", "output"]));

    // The turn bodies are framed, so the conversation reads as one.
    expect(container.querySelector(".cm-turn-user")).toBeTruthy();
    expect(container.querySelector(".cm-turn-assistant")).toBeTruthy();
    expect(container.querySelector(".cm-turn-action")).toBeTruthy();
    expect(container.querySelector(".cm-turn-observation")).toBeTruthy();

    // The exit status is on the chip: a failed action must be visible without
    // reading the raw attribute.
    const details = [...container.querySelectorAll(".cm-hick-banner-detail")].map(
      (e) => e.textContent,
    );
    expect(details).toContain("exit 0");

    // Every byte is still the document's own text — nothing was replaced.
    expect(container.querySelector(".cm-content")?.textContent).toContain("<hick:action");

    // Top-level turns are bubbles — yours on the right, the agent's on the
    // left (Apple Messages' sides) — and a nested action is framed inside
    // its bubble, not bubbled.
    // docs/guarantees/agent/a-session-is-the-conversation.md
    const you = container.querySelector(".cm-turn-user.cm-bubble-you");
    expect(you?.classList.contains("cm-bubble-first")).toBe(true);
    expect(container.querySelector(".cm-turn-assistant.cm-bubble-agent")).toBeTruthy();
    expect(container.querySelector(".cm-turn-observation.cm-bubble-agent")).toBeTruthy();
    expect(container.querySelector(".cm-turn-action.cm-bubble-agent")).toBeTruthy();
    expect(container.querySelector(".cm-turn-action.cm-bubble-you")).toBeNull();
    // The opening tag line of a bubble is its first line; the closing tag
    // line its last — the tags are inside the bubble, not around it.
    const lines = [...container.querySelectorAll(".cm-line")];
    // (The chip leads the line now, so the tag is inside the text, not at
    // its start.)
    const userOpen = lines.find((l) => l.textContent?.includes("<hick:user>"));
    expect(userOpen?.classList.contains("cm-bubble-first")).toBe(true);
    expect(userOpen?.classList.contains("cm-bubble-last")).toBe(true);
    const obsClose = lines.find((l) => l.textContent?.includes("</hick:observation>"));
    expect(obsClose?.classList.contains("cm-bubble-last")).toBe(true);
    // The chip is the bubble's caption: it leads the opening line.
    expect(userOpen?.textContent?.indexOf("you")).toBeLessThan(
      userOpen?.textContent?.indexOf("<hick:user>") ?? -1,
    );

    // The work opens folded: the action's script is hidden behind a
    // placeholder, its opening and closing tag lines still on screen.
    await waitFor(() =>
      expect(container.querySelector(".cm-hick-fold-placeholder")).toBeTruthy(),
    );
    const shown = container.querySelector(".cm-content")?.textContent ?? "";
    expect(shown).toContain("<hick:action");
    expect(shown).toContain("</hick:action>");
    expect(shown).not.toContain("printf 'hello'");
    // What was SAID is not folded.
    expect(shown).toContain("Writing the file now.");
    realtime.close();
  });

  it("treats an agent's action as code, not transcript", () => {
    const structure = parseHickDoc(SESSION);
    const action = structure.blocks.find((b) => b.name === "action")!;
    expect(languageForBlock(structure, action)).toBe("shellscript");
  });

  it("shows a failed observation as failed", () => {
    const failed = SESSION.replace('exit="0"', 'exit="1"');
    const structure = parseHickDoc(failed);
    const obs = structure.blocks.find((b) => b.name === "observation")!;
    expect(obs.attrs.exit).toBe("1");
  });
});
