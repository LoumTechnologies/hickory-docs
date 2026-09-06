// The conversation's elements, drawn as the cards the agent pane has always
// drawn — now in place of their source in the session document, with line
// numbers beside them. See docs/guarantees/agent/a-session-is-the-conversation.md
// and docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md.
import type { Block } from "../../api/types";
import { matchBlock } from "../../lib/blockMatch";
import type { RenderedSlot } from "../../editor/rendered";
import type { ElementView, SlotContext, SlotKind } from "../types";

type Of<K extends Block["kind"]> = Extract<Block, { kind: K }>;

/** The server's block for this slot, when the session view has arrived. */
function serverBlock<K extends Block["kind"]>(
  slot: RenderedSlot,
  cx: SlotContext,
  kind: K,
): Of<K> | undefined {
  const same = (cx.sessionBlocks ?? []).filter((b): b is Of<K> => b.kind === kind);
  return matchBlock({ span: slot.span, index: slot.index }, same);
}

function view(kind: SlotKind, name: string, render: ElementView["render"]): ElementView {
  return { kind, draws: (block) => block.name === name, render };
}

export const sessionUserView = view("session-user", "user", (slot) => (
  <div className="chat-msg chat-user">
    <span className="chat-role">you</span>
    <div className="chat-bubble">
      <p>{slot.text.trim()}</p>
    </div>
  </div>
));

export const sessionAssistantView = view("session-assistant", "assistant", (slot, cx) => {
  // The prose alone: the tools nested in the answer are blocks of their own.
  const body =
    serverBlock(slot, cx, "session-assistant")?.body ?? slot.text.replace(/<hick:[\s\S]*$/, "").trim();
  return (
    <div className="chat-msg chat-agent">
      <span className="chat-role">agent</span>
      <div className="chat-bubble">
        {body ? <p className="chat-answer">{body}</p> : <p className="muted">no answer recorded</p>}
      </div>
    </div>
  );
});

export const sessionToolView = view("session-tool", "tool", (slot, cx) => {
  const block = serverBlock(slot, cx, "session-tool");
  return (
    <details className="chat-step chat-step--tool">
      <summary>
        tool <code>{block?.name ?? slot.text.slice(0, 40)}</code>
        {(block?.args ?? []).map(([k, v]) => (
          <span key={k} className="chat-step__arg">
            {" "}
            {k}=<code>{v.length > 60 ? `${v.slice(0, 60)}…` : v}</code>
          </span>
        ))}
      </summary>
    </details>
  );
});

export const sessionToolResultView = view("session-tool-result", "tool-result", (slot, cx) => {
  const block = serverBlock(slot, cx, "session-tool-result");
  return (
    <details className="chat-step chat-step--tool">
      <summary>
        result
        {block?.name ? (
          <>
            {" "}
            of <code>{block.name}</code>
          </>
        ) : null}
        {block?.ok === false ? <span className="chat-error"> failed</span> : null}
      </summary>
      <pre>{block?.body ?? slot.text.trim()}</pre>
    </details>
  );
});

export const sessionReadView = view("session-read", "read", (slot, cx) => {
  const block = serverBlock(slot, cx, "session-read");
  return (
    <p
      className="chat-step chat-step--read muted"
      data-tip={
        block?.sha256 ? `sha256 ${block.sha256}${block.commit ? ` · commit ${block.commit}` : ""}` : undefined
      }
    >
      read <code>{block?.file ?? "a file"}</code>
      {block?.lines ? <> lines {block.lines}</> : null}
    </p>
  );
});

export const sessionWroteView = view("session-wrote", "wrote", (slot, cx) => {
  const block = serverBlock(slot, cx, "session-wrote");
  return (
    <p className="chat-step chat-step--wrote muted">
      wrote <code>{block?.file ?? "a file"}</code>
      {block?.lines ? <> lines {block.lines}</> : null}
    </p>
  );
});

export const sessionContextView = view("session-context", "context", (slot, cx) => {
  const block = serverBlock(slot, cx, "session-context");
  return (
    <details className="chat-step chat-step--context">
      <summary>context{block?.context_kind ? <> · {block.context_kind}</> : null}</summary>
      <pre>{block?.body ?? slot.text.trim()}</pre>
    </details>
  );
});

export const sessionObservationView = view("session-observation", "observation", (slot, cx) => {
  const block = serverBlock(slot, cx, "session-observation");
  return (
    <details className="chat-step chat-step--tool">
      <summary>observation{block?.exit ? <> · exit {block.exit}</> : null}</summary>
      <pre>{block?.body ?? slot.text.trim()}</pre>
    </details>
  );
});

export const sessionActionView = view("session-action", "action", (slot, cx) => {
  const block = serverBlock(slot, cx, "session-action");
  return (
    <details className="chat-step chat-step--action" open>
      <summary>action{block?.lang ? <> · {block.lang}</> : null}</summary>
      <pre>{block?.body ?? slot.text.trim()}</pre>
    </details>
  );
});

export const sessionViews: ElementView[] = [
  sessionUserView,
  sessionAssistantView,
  sessionToolView,
  sessionToolResultView,
  sessionReadView,
  sessionWroteView,
  sessionContextView,
  sessionObservationView,
  sessionActionView,
];
