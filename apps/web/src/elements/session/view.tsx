// The conversation's elements, drawn as the cards the agent pane has always
// drawn — now in place of their source in the session document, with line
// numbers beside them. See docs/guarantees/agent/a-session-is-the-conversation.md
// and docs/guarantees/agent/an-answer-in-the-agent-pane-has-ribbons.md.
//
// What a reader sees first is the conversation: your words, the agent's
// answer, and under each answer the files it rests on. Everything the
// agent did on the way — its reasoning, the tools it called and what they
// returned — is there, folded, one click away; and the record's own
// bookkeeping (usage, the protocol marker) is drawn as nothing at all.
import type { Block, SessionLink } from "../../api/types";
import { matchBlock } from "../../lib/blockMatch";
import { openLocation } from "../../lib/revealLine";
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

/** Open a file the conversation named, at its first line when it has
 * lines — the same road a definition or a stack frame takes. */
function openTarget(link: SessionLink) {
  openLocation(link.to.path, link.to.lines?.[0] ?? 1);
}

const VERB: Record<SessionLink["family"], string> = {
  context: "read",
  lineage: "wrote",
  declared: "cites",
};

/** The files an answer rests on, as chips: what the turn read (context),
 * wrote (lineage) and pointed at (declared) — the same three families the
 * ribbons draw, in words, under the answer they belong to. */
export function SourceChips({ links }: { links: readonly SessionLink[] }) {
  if (links.length === 0) return null;
  const seen = new Set<string>();
  const shown = links.filter((link) => {
    const key = `${link.family}:${link.to.path}:${link.to.lines?.join("-") ?? ""}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
  return (
    <div className="chat-sources" aria-label="What this answer rests on">
      {shown.map((link, i) => (
        <button
          key={i}
          type="button"
          className={`chat-source chat-source--${link.family}`}
          data-tip={link.title}
          onClick={() => openTarget(link)}
        >
          <span className="chat-source__verb">{VERB[link.family]}</span>{" "}
          <span className="mono">{link.to.path.split("/").pop() ?? link.to.path}</span>
          {link.to.lines ? (
            <span className="chat-source__lines">
              {" "}
              {link.to.lines[0] === link.to.lines[1]
                ? link.to.lines[0]
                : `${link.to.lines[0]}–${link.to.lines[1]}`}
            </span>
          ) : null}
        </button>
      ))}
    </div>
  );
}

export const sessionUserView = view("session-user", "user", (slot, cx) => (
  <div className="chat-msg chat-user">
    <span className="chat-role">you</span>
    <div className="chat-bubble">
      <p className="chat-answer">{serverBlock(slot, cx, "session-user")?.body ?? slot.text.trim()}</p>
    </div>
  </div>
));

/** The blocks nested inside this one — an answer's reasoning and tool
 * calls — in order, with a slot each so their own views draw them. */
function nested(slot: RenderedSlot, cx: SlotContext): { block: Block; slot: RenderedSlot }[] {
  // The slot's own span, in the editor's units; the lens hands the server's
  // blocks over in the same units.
  const [from, to] = slot.span;
  return (cx.sessionBlocks ?? [])
    .filter((b) => b.kind !== "session-assistant" && b.span[0] > from && b.span[1] <= to)
    .map((block) => ({
      block,
      slot: { ...slot, span: block.span, at: block.span[0], text: "", key: `${slot.key}:${block.span[0]}` },
    }));
}

export const sessionAssistantView = view("session-assistant", "assistant", (slot, cx) => {
  // The prose is the answer; the reasoning and the tool calls nested in it
  // are its work, drawn inside the card, folded.
  const body =
    serverBlock(slot, cx, "session-assistant")?.body ?? slot.text.replace(/<hick:[\s\S]*$/, "").trim();
  const links = cx.sessionLinksAt?.get(slot.at) ?? [];
  const work = nested(slot, cx).filter(({ block }) => block.kind !== "session-meta");
  // A step with nothing to show — the agent only emitted a protocol marker
  // — is not an answer, and "no answer" under it reads as failure.
  if (!body && links.length === 0 && work.length === 0) {
    return <div className="chat-msg chat-agent chat-agent--silent" />;
  }
  // The work — reasoning, tool calls — is not speech: it sits above the
  // bubble as plain folds with no tail. The bubble, with its tail, holds
  // only what the agent said and what it rests on.
  return (
    <div className="chat-msg chat-agent">
      {work.length > 0 && (
        <div className="chat-work">
          {work.map(({ block, slot: inner }) => {
            const drawer = INNER[block.kind];
            return drawer ? <div key={inner.key}>{drawer(inner, cx)}</div> : null;
          })}
        </div>
      )}
      {(body || links.length > 0) && (
        <>
          <span className="chat-role">agent</span>
          <div className="chat-bubble">
            {body ? <p className="chat-answer">{body}</p> : null}
            <SourceChips links={links} />
          </div>
        </>
      )}
    </div>
  );
});

/** Which views draw an answer's nested work, by kind — filled in below,
 * once those views exist. */
const INNER: Partial<Record<Block["kind"], ElementView["render"]>> = {};

export const sessionReasoningView = view("session-reasoning", "reasoning", (slot, cx) => (
  <details className="chat-reasoning">
    <summary>reasoning</summary>
    <pre className="chat-reasoning__text">{serverBlock(slot, cx, "session-reasoning")?.body ?? slot.text.trim()}</pre>
  </details>
));

export const sessionToolView = view("session-tool", "tool", (slot, cx) => {
  const block = serverBlock(slot, cx, "session-tool");
  return (
    <details className="chat-step chat-step--tool">
      <summary>
        <code>{block?.name ?? slot.text.slice(0, 40)}</code>
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

export const sessionInputView = view("session-input", "input", (slot, cx) => {
  const block = serverBlock(slot, cx, "session-input");
  return (
    <details className="chat-step chat-step--tool">
      <summary>input{block?.name ? <> · {block.name}</> : null}</summary>
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
    <details className="chat-step chat-step--action">
      <summary>ran{block?.lang ? <> · {block.lang}</> : null}</summary>
      <pre>{block?.body ?? slot.text.trim()}</pre>
    </details>
  );
});

INNER["session-reasoning"] = sessionReasoningView.render;
INNER["session-tool"] = sessionToolView.render;
INNER["session-action"] = sessionActionView.render;
INNER["session-input"] = sessionInputView.render;

export const sessionViews: ElementView[] = [
  sessionUserView,
  sessionAssistantView,
  sessionReasoningView,
  sessionToolView,
  sessionToolResultView,
  sessionInputView,
  sessionReadView,
  sessionWroteView,
  sessionContextView,
  sessionObservationView,
  sessionActionView,
  {
    kind: "session-meta",
    draws: (block) => block.name === "usage" || block.name === "next",
    render: () => <span className="session-meta" aria-hidden />,
  },
];
